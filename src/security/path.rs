//! Filesystem path validation for host and sync inputs.

use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

/// Errors returned when untrusted paths or mount specifications are unsafe.
#[derive(Debug, Error)]
pub enum SecurityError {
    /// A required path value was empty.
    #[error("path must not be empty")]
    EmptyPath,
    /// A path contains a byte that is unsafe to pass to filesystem or runtime APIs.
    #[error("path contains a NUL or control character")]
    InvalidCharacter,
    /// A path could not be resolved without ambiguity.
    #[error("failed to resolve path {path}: {source}")]
    Canonicalize {
        /// The path that failed resolution.
        path: PathBuf,
        /// The filesystem error returned by canonicalization.
        #[source]
        source: std::io::Error,
    },
    /// A resolved path has an unexpected filesystem type.
    #[error("expected {expected} path, got {path}")]
    WrongFileType {
        /// The expected filesystem type.
        expected: &'static str,
        /// The resolved path.
        path: PathBuf,
    },
    /// A sync path was not a safe relative path below the project root.
    #[error("sync path must remain relative to the project root: {0}")]
    UnsafeRelativePath(String),
    /// A user-specified mount reaches a protected runtime or configuration path.
    #[error("user mount targets protected host path {0}")]
    BlockedMount(PathBuf),
    /// A runtime mount flag was incomplete or malformed.
    #[error("invalid runtime mount specification: {0}")]
    InvalidMount(String),
}

/// Host-specific policy inputs used to build the protected path set.
#[derive(Clone, Debug)]
pub(crate) struct PathPolicy {
    home_dir: Option<PathBuf>,
    xdg_runtime_dir: Option<PathBuf>,
}

impl Default for PathPolicy {
    fn default() -> Self {
        Self {
            home_dir: dirs::home_dir(),
            xdg_runtime_dir: env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from),
        }
    }
}

impl PathPolicy {
    /// Builds a policy from explicit environment paths.
    ///
    /// This constructor also makes policy tests independent of process-global environment state.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(home_dir: Option<PathBuf>, xdg_runtime_dir: Option<PathBuf>) -> Self {
        Self {
            home_dir,
            xdg_runtime_dir,
        }
    }

    /// Resolves a user path and rejects container-runtime sockets and configuration directories.
    pub(crate) fn validate_user_mount_path(&self, input: &str) -> Result<PathBuf, SecurityError> {
        let canonical = canonicalize_path(input)?;
        if self
            .blocked_roots()
            .iter()
            .any(|blocked| paths_overlap(&canonical, blocked))
        {
            return Err(SecurityError::BlockedMount(canonical));
        }
        Ok(canonical)
    }

    fn blocked_roots(&self) -> Vec<PathBuf> {
        let mut roots = [
            "/var/run/docker.sock",
            "/run/docker.sock",
            "/var/run/docker",
            "/run/docker",
            "/var/run/containerd",
            "/run/containerd",
            "/var/run/podman",
            "/run/podman",
        ]
        .into_iter()
        .map(canonicalize_if_present)
        .collect::<Vec<_>>();

        if let Some(home) = &self.home_dir {
            roots.push(canonicalize_if_present(home.join(".docker")));
            roots.push(canonicalize_if_present(home.join(".config/containers")));
        }
        if let Some(runtime) = &self.xdg_runtime_dir {
            roots.push(canonicalize_if_present(runtime.join("docker.sock")));
            roots.push(canonicalize_if_present(runtime.join("docker")));
            roots.push(canonicalize_if_present(runtime.join("podman")));
            roots.push(canonicalize_if_present(runtime.join("containerd")));
            roots.push(canonicalize_if_present(runtime.join("containerd-rootless")));
        }
        roots
    }
}

/// Resolves an existing directory after rejecting ambiguous input.
pub fn validate_dir_path(input: &str) -> Result<PathBuf, SecurityError> {
    let path = canonicalize_path(input)?;
    if !path.is_dir() {
        return Err(SecurityError::WrongFileType {
            expected: "directory",
            path,
        });
    }
    Ok(path)
}

/// Resolves an existing regular file after rejecting ambiguous input.
pub fn validate_file_path(input: &str) -> Result<PathBuf, SecurityError> {
    let path = canonicalize_path(input)?;
    if !path.is_file() {
        return Err(SecurityError::WrongFileType {
            expected: "file",
            path,
        });
    }
    Ok(path)
}

/// Validates a relative sync path and proves that its resolved parent remains below `project_root`.
///
/// The final path may not exist yet, which allows a validated destination for a newly-created file.
/// Its parent must already exist so symlinks and aliases can be resolved before use.
pub fn validate_relative_sync_path(
    project_root: &Path,
    input: &str,
) -> Result<PathBuf, SecurityError> {
    validate_path_text(input)?;
    let relative = Path::new(input);
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(SecurityError::UnsafeRelativePath(input.to_owned()));
    }

    let root = fs::canonicalize(project_root).map_err(|source| SecurityError::Canonicalize {
        path: project_root.to_path_buf(),
        source,
    })?;
    if !root.is_dir() {
        return Err(SecurityError::WrongFileType {
            expected: "directory",
            path: root,
        });
    }

    let joined = root.join(relative);
    let resolved = match fs::canonicalize(&joined) {
        Ok(path) => path,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            let Some(parent) = joined.parent() else {
                return Err(SecurityError::UnsafeRelativePath(input.to_owned()));
            };
            let parent =
                fs::canonicalize(parent).map_err(|source| SecurityError::Canonicalize {
                    path: parent.to_path_buf(),
                    source,
                })?;
            let Some(name) = joined.file_name() else {
                return Err(SecurityError::UnsafeRelativePath(input.to_owned()));
            };
            parent.join(name)
        }
        Err(source) => {
            return Err(SecurityError::Canonicalize {
                path: joined,
                source,
            });
        }
    };

    if !resolved.starts_with(&root) {
        return Err(SecurityError::UnsafeRelativePath(input.to_owned()));
    }
    Ok(resolved)
}

fn canonicalize_path(input: &str) -> Result<PathBuf, SecurityError> {
    validate_path_text(input)?;
    let path = PathBuf::from(input);
    fs::canonicalize(&path).map_err(|source| SecurityError::Canonicalize { path, source })
}

fn validate_path_text(input: &str) -> Result<(), SecurityError> {
    if input.is_empty() {
        return Err(SecurityError::EmptyPath);
    }
    if input
        .chars()
        .any(|character| character == '\0' || character.is_control())
    {
        return Err(SecurityError::InvalidCharacter);
    }
    Ok(())
}

fn canonicalize_if_present(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut unresolved = Vec::new();
    let mut existing = path;
    loop {
        if let Ok(canonical) = fs::canonicalize(existing) {
            return unresolved
                .iter()
                .rev()
                .fold(canonical, |resolved, component| resolved.join(component));
        }
        let Some(name) = existing.file_name() else {
            return path.to_path_buf();
        };
        unresolved.push(name.to_os_string());
        let Some(parent) = existing.parent() else {
            return path.to_path_buf();
        };
        existing = parent;
    }
}

fn paths_overlap(candidate: &Path, blocked: &Path) -> bool {
    candidate.starts_with(blocked) || blocked.starts_with(candidate)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn validates_existing_file_and_directory() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempdir()?;
        let file = root.path().join("file.txt");
        fs::write(&file, "safe")?;

        assert_eq!(
            validate_dir_path(&root.path().to_string_lossy())?,
            fs::canonicalize(root.path())?
        );
        assert_eq!(
            validate_file_path(&file.to_string_lossy())?,
            fs::canonicalize(file)?
        );
        Ok(())
    }

    #[test]
    fn rejects_empty_nul_control_and_missing_paths() {
        assert!(matches!(
            validate_dir_path(""),
            Err(SecurityError::EmptyPath)
        ));
        assert!(matches!(
            validate_dir_path("bad\0path"),
            Err(SecurityError::InvalidCharacter)
        ));
        assert!(matches!(
            validate_dir_path("bad\npath"),
            Err(SecurityError::InvalidCharacter)
        ));
        assert!(matches!(
            validate_dir_path("/definitely/not/a/real/cage/path"),
            Err(SecurityError::Canonicalize { .. })
        ));
    }

    #[test]
    fn relative_sync_path_rejects_escape_and_symlink_escape()
    -> Result<(), Box<dyn std::error::Error>> {
        let project = tempdir()?;
        let outside = tempdir()?;
        fs::create_dir(project.path().join("inside"))?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path(), project.path().join("escape"))?;

        assert!(matches!(
            validate_relative_sync_path(project.path(), "../outside"),
            Err(SecurityError::UnsafeRelativePath(_))
        ));
        assert!(matches!(
            validate_relative_sync_path(project.path(), "/absolute"),
            Err(SecurityError::UnsafeRelativePath(_))
        ));
        #[cfg(unix)]
        assert!(matches!(
            validate_relative_sync_path(project.path(), "escape/new.txt"),
            Err(SecurityError::UnsafeRelativePath(_))
        ));
        Ok(())
    }

    #[test]
    fn relative_sync_path_allows_existing_and_new_leaf_inside_root()
    -> Result<(), Box<dyn std::error::Error>> {
        let project = tempdir()?;
        fs::create_dir(project.path().join("nested"))?;
        let existing = project.path().join("nested/existing.txt");
        fs::write(&existing, "safe")?;

        assert_eq!(
            validate_relative_sync_path(project.path(), "nested/existing.txt")?,
            fs::canonicalize(existing)?
        );
        assert_eq!(
            validate_relative_sync_path(project.path(), "nested/new.txt")?,
            fs::canonicalize(project.path())?.join("nested/new.txt")
        );
        Ok(())
    }

    #[test]
    fn blocks_runtime_paths_and_home_configuration() -> Result<(), Box<dyn std::error::Error>> {
        let home = tempdir()?;
        let runtime = tempdir()?;
        fs::create_dir_all(home.path().join(".docker/run"))?;
        fs::write(home.path().join(".docker/run/docker.sock"), "socket")?;
        fs::create_dir(runtime.path().join("podman"))?;
        fs::write(runtime.path().join("podman/podman.sock"), "socket")?;
        let policy = PathPolicy::new(
            Some(home.path().to_path_buf()),
            Some(runtime.path().to_path_buf()),
        );

        assert!(matches!(
            policy.validate_user_mount_path(
                &home
                    .path()
                    .join(".docker/run/docker.sock")
                    .to_string_lossy()
            ),
            Err(SecurityError::BlockedMount(_))
        ));
        assert!(matches!(
            policy.validate_user_mount_path(
                &runtime.path().join("podman/podman.sock").to_string_lossy()
            ),
            Err(SecurityError::BlockedMount(_))
        ));
        assert!(matches!(
            policy.validate_user_mount_path(&home.path().to_string_lossy()),
            Err(SecurityError::BlockedMount(_))
        ));
        assert!(matches!(
            policy.validate_user_mount_path(&runtime.path().to_string_lossy()),
            Err(SecurityError::BlockedMount(_))
        ));
        Ok(())
    }

    #[test]
    fn blocks_ancestors_of_static_runtime_paths() {
        let policy = PathPolicy::new(None, None);

        assert!(matches!(
            policy.validate_user_mount_path("/"),
            Err(SecurityError::BlockedMount(_))
        ));
        if Path::new("/var").exists() {
            assert!(matches!(
                policy.validate_user_mount_path("/var"),
                Err(SecurityError::BlockedMount(_))
            ));
        }
        if Path::new("/run").exists() {
            assert!(matches!(
                policy.validate_user_mount_path("/run"),
                Err(SecurityError::BlockedMount(_))
            ));
        }
    }

    #[test]
    fn blocks_rootless_containerd_paths() -> Result<(), Box<dyn std::error::Error>> {
        let runtime = tempdir()?;
        fs::create_dir_all(runtime.path().join("containerd-rootless/api"))?;
        let socket = runtime
            .path()
            .join("containerd-rootless/api/containerd.sock");
        fs::write(&socket, "socket")?;
        let policy = PathPolicy::new(None, Some(runtime.path().to_path_buf()));

        assert!(matches!(
            policy.validate_user_mount_path(&socket.to_string_lossy()),
            Err(SecurityError::BlockedMount(_))
        ));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn blocks_symlink_alias_to_runtime_path() -> Result<(), Box<dyn std::error::Error>> {
        let home = tempdir()?;
        let exposed = tempdir()?;
        fs::create_dir_all(home.path().join(".docker/run"))?;
        let socket = home.path().join(".docker/run/docker.sock");
        fs::write(&socket, "socket")?;
        let alias = exposed.path().join("innocent.sock");
        std::os::unix::fs::symlink(&socket, &alias)?;
        let policy = PathPolicy::new(Some(home.path().to_path_buf()), None);

        assert!(matches!(
            policy.validate_user_mount_path(&alias.to_string_lossy()),
            Err(SecurityError::BlockedMount(_))
        ));
        Ok(())
    }
}
