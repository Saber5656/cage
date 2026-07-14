use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use dialoguer::Confirm;
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::{
    CageConfig, CustomAdapterConfig, DefaultsConfig, EnvironmentConfig, ProfileConfig, SyncConfig,
};

/// Maximum accepted size for either configuration file.
pub const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

/// Explicit locations used by the configuration loader.
#[derive(Clone, Debug)]
pub struct ConfigPaths {
    pub global: PathBuf,
    pub project: PathBuf,
    pub cage_home: PathBuf,
}

impl ConfigPaths {
    /// Resolve conventional global/project paths for a project directory.
    #[must_use]
    pub fn for_project(project_dir: &Path) -> Self {
        let config_home = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config"));
        let cage_home =
            std::env::var_os("CAGE_HOME").map_or_else(|| config_home.join("cage"), PathBuf::from);

        Self {
            global: config_home.join("cage").join("cage.toml"),
            project: project_dir.join("cage.toml"),
            cage_home,
        }
    }
}

/// Typed configuration failures. Invalid or untrusted inputs always fail closed.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to inspect configuration {path}: {source}")]
    Inspect {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("configuration {path} exceeds the {limit}-byte limit")]
    TooLarge { path: PathBuf, limit: u64 },
    #[error("failed to read configuration {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("configuration {path} is not valid UTF-8")]
    InvalidUtf8 { path: PathBuf },
    #[error("configuration {path} must be a regular file and cannot be a symlink")]
    InvalidFileType { path: PathBuf },
    #[error("invalid TOML in {path}: {source}")]
    InvalidToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid configuration value at {field}: {reason}")]
    InvalidValue { field: String, reason: String },
    #[error("profile '{0}' is not defined")]
    UnknownProfile(String),
    #[error(
        "project configuration {path} is not trusted; review and approve it interactively before using non-interactive mode"
    )]
    UntrustedProject { path: PathBuf },
    #[error("project configuration {path} was rejected")]
    ProjectRejected { path: PathBuf },
    #[error("failed to render effective project configuration: {0}")]
    Render(#[from] toml::ser::Error),
    #[error("failed to prompt for project configuration trust: {0}")]
    Prompt(#[source] dialoguer::Error),
    #[error("failed to update trust cache {path}: {source}")]
    TrustCache {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Policy boundary for first-use project configuration approval.
pub trait ProjectConfigConfirmer {
    /// Show the effective configuration and return whether it is approved.
    ///
    /// # Errors
    ///
    /// Returns a typed error if confirmation cannot be obtained.
    fn confirm(&mut self, project_path: &Path, effective_toml: &str) -> Result<bool, ConfigError>;
}

/// Fail-closed confirmer for automation and auto execution.
pub struct NonInteractiveConfirmer;

impl ProjectConfigConfirmer for NonInteractiveConfirmer {
    fn confirm(&mut self, project_path: &Path, _effective_toml: &str) -> Result<bool, ConfigError> {
        Err(ConfigError::UntrustedProject {
            path: project_path.to_owned(),
        })
    }
}

/// Interactive terminal confirmer.
pub struct DialoguerConfirmer;

impl ProjectConfigConfirmer for DialoguerConfirmer {
    fn confirm(&mut self, project_path: &Path, effective_toml: &str) -> Result<bool, ConfigError> {
        Confirm::new()
            .with_prompt(format!(
                "Apply project configuration {}?\n\n{}",
                project_path.display(),
                effective_toml
            ))
            .default(false)
            .interact()
            .map_err(ConfigError::Prompt)
    }
}

/// Two-level configuration loader.
pub struct ConfigLoader {
    paths: ConfigPaths,
}

impl ConfigLoader {
    #[must_use]
    pub const fn new(paths: ConfigPaths) -> Self {
        Self { paths }
    }

    /// Load configuration for non-interactive execution.
    ///
    /// Previously acknowledged project content is accepted. New or changed project content fails
    /// closed until an interactive load records approval.
    ///
    /// # Errors
    ///
    /// Returns a typed error for I/O, parsing, validation, or trust failures.
    pub fn load_non_interactive(&self) -> Result<CageConfig, ConfigError> {
        self.load(&mut NonInteractiveConfirmer)
    }

    /// Load, merge, validate, and (when necessary) confirm project configuration.
    ///
    /// # Errors
    ///
    /// Returns a typed error for I/O, parsing, validation, or trust failures.
    pub fn load(
        &self,
        confirmer: &mut impl ProjectConfigConfirmer,
    ) -> Result<CageConfig, ConfigError> {
        let global = read_optional(&self.paths.global)?.unwrap_or_default();
        let Some((project, project_bytes)) = read_optional_with_bytes(&self.paths.project)? else {
            validate(&global)?;
            return Ok(global);
        };

        let effective = merge(global, project);
        validate(&effective)?;

        let canonical_project =
            fs::canonicalize(&self.paths.project).map_err(|source| ConfigError::Inspect {
                path: self.paths.project.clone(),
                source,
            })?;
        let trust_key = trust_key(&canonical_project, &project_bytes);
        let acknowledgement = self
            .paths
            .cage_home
            .join("trusted-configs")
            .join(format!("{trust_key}.ack"));

        if acknowledgement_is_valid(&acknowledgement, &canonical_project, &project_bytes)? {
            return Ok(effective);
        }

        let rendered = toml::to_string_pretty(&effective)?;
        if !confirmer.confirm(&canonical_project, &rendered)? {
            return Err(ConfigError::ProjectRejected {
                path: canonical_project,
            });
        }

        write_acknowledgement(&acknowledgement, &canonical_project, &project_bytes)?;
        Ok(effective)
    }
}

fn read_optional(path: &Path) -> Result<Option<CageConfig>, ConfigError> {
    read_optional_with_bytes(path).map(|value| value.map(|(config, _)| config))
}

fn read_optional_with_bytes(path: &Path) -> Result<Option<(CageConfig, Vec<u8>)>, ConfigError> {
    let path_metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ConfigError::Inspect {
                path: path.to_owned(),
                source,
            });
        }
    };

    if !path_metadata.file_type().is_file() || path_metadata.file_type().is_symlink() {
        return Err(ConfigError::InvalidFileType {
            path: path.to_owned(),
        });
    }
    if path_metadata.len() > MAX_CONFIG_BYTES {
        return Err(ConfigError::TooLarge {
            path: path.to_owned(),
            limit: MAX_CONFIG_BYTES,
        });
    }

    let file = open_config(path).map_err(|source| ConfigError::Read {
        path: path.to_owned(),
        source,
    })?;
    let opened_metadata = file.metadata().map_err(|source| ConfigError::Inspect {
        path: path.to_owned(),
        source,
    })?;
    if !opened_metadata.file_type().is_file() {
        return Err(ConfigError::InvalidFileType {
            path: path.to_owned(),
        });
    }
    if opened_metadata.len() > MAX_CONFIG_BYTES {
        return Err(ConfigError::TooLarge {
            path: path.to_owned(),
            limit: MAX_CONFIG_BYTES,
        });
    }
    let mut bytes = Vec::new();
    file.take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| ConfigError::Read {
            path: path.to_owned(),
            source,
        })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_CONFIG_BYTES {
        return Err(ConfigError::TooLarge {
            path: path.to_owned(),
            limit: MAX_CONFIG_BYTES,
        });
    }

    let content =
        std::str::from_utf8(&bytes).map_err(|_| ConfigError::InvalidUtf8 { path: path.into() })?;
    let config = toml::from_str(content).map_err(|source| ConfigError::InvalidToml {
        path: path.to_owned(),
        source,
    })?;
    Ok(Some((config, bytes)))
}

fn open_config(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
    }
    options.open(path)
}

fn merge(mut base: CageConfig, project: CageConfig) -> CageConfig {
    merge_environment(&mut base.environment, project.environment);
    merge_sync(&mut base.sync, project.sync);
    merge_defaults(&mut base.defaults, project.defaults);

    for (name, project_profile) in project.profiles {
        merge_profile(base.profiles.entry(name).or_default(), project_profile);
    }
    for (name, project_adapter) in project.adapters {
        merge_adapter(base.adapters.entry(name).or_default(), project_adapter);
    }
    base
}

fn merge_environment(base: &mut EnvironmentConfig, project: EnvironmentConfig) {
    override_option(&mut base.runtime, project.runtime);
}

fn merge_sync(base: &mut SyncConfig, project: SyncConfig) {
    override_option(&mut base.include, project.include);
    override_option(&mut base.exclude, project.exclude);
}

fn merge_defaults(base: &mut DefaultsConfig, project: DefaultsConfig) {
    override_option(&mut base.profile, project.profile);
    override_option(&mut base.agent, project.agent);
}

fn merge_profile(base: &mut ProfileConfig, project: ProfileConfig) {
    override_option(&mut base.image, project.image);
    override_option(&mut base.memory, project.memory);
    override_option(&mut base.cpus, project.cpus);
    override_option(&mut base.env, project.env);
    override_option(&mut base.gpus, project.gpus);
    override_option(&mut base.with_hooks, project.with_hooks);
    override_option(&mut base.dind, project.dind);
}

fn merge_adapter(base: &mut CustomAdapterConfig, project: CustomAdapterConfig) {
    override_option(&mut base.image, project.image);
    override_option(&mut base.command, project.command);
    override_option(&mut base.env, project.env);
    override_option(&mut base.config_files, project.config_files);
}

fn override_option<T>(base: &mut Option<T>, project: Option<T>) {
    if project.is_some() {
        *base = project;
    }
}

fn validate(config: &CageConfig) -> Result<(), ConfigError> {
    if let Some(runtime) = &config.environment.runtime
        && runtime != "docker"
        && runtime != "podman"
    {
        return invalid("environment.runtime", "expected 'docker' or 'podman'");
    }

    for (name, profile) in &config.profiles {
        validate_name(&format!("profiles.{name}"), name)?;
        if let Some(image) = &profile.image {
            validate_image(&format!("profiles.{name}.image"), image)?;
        }
        if let Some(memory) = &profile.memory
            && !valid_memory(memory)
        {
            return invalid(
                &format!("profiles.{name}.memory"),
                "expected a positive integer with an optional b, k, m, g, or t suffix",
            );
        }
        if let Some(cpus) = &profile.cpus {
            let parsed = cpus.parse::<f64>().ok();
            if !parsed.is_some_and(|value| value.is_finite() && value > 0.0) {
                return invalid(
                    &format!("profiles.{name}.cpus"),
                    "expected a finite number greater than zero",
                );
            }
        }
        validate_env_names(&format!("profiles.{name}.env"), profile.env.as_deref())?;
        if let Some(gpus) = &profile.gpus {
            for gpu in gpus {
                if gpu.is_empty() || !gpu.chars().all(safe_identifier_char) {
                    return invalid(
                        &format!("profiles.{name}.gpus"),
                        "GPU identifiers may contain only letters, digits, '.', '_', '-', ':' or '/'",
                    );
                }
            }
        }
        if profile.dind == Some(true) {
            return invalid(
                &format!("profiles.{name}.dind"),
                "DinD cannot be enabled until the safe sidecar policy is implemented",
            );
        }
    }

    for (name, adapter) in &config.adapters {
        validate_name(&format!("adapters.{name}"), name)?;
        let image = adapter
            .image
            .as_deref()
            .ok_or_else(|| ConfigError::InvalidValue {
                field: format!("adapters.{name}.image"),
                reason: "custom adapters require an image".into(),
            })?;
        validate_image(&format!("adapters.{name}.image"), image)?;
        if adapter.command.as_ref().is_none_or(Vec::is_empty) {
            return invalid(
                &format!("adapters.{name}.command"),
                "custom adapters require a non-empty command",
            );
        }
        validate_env_names(&format!("adapters.{name}.env"), adapter.env.as_deref())?;
        if let Some(paths) = &adapter.config_files {
            validate_patterns(&format!("adapters.{name}.config_files"), paths)?;
        }
    }

    if let Some(profile) = &config.defaults.profile
        && !config.profiles.contains_key(profile)
    {
        return Err(ConfigError::UnknownProfile(profile.clone()));
    }
    if let Some(patterns) = &config.sync.include {
        validate_patterns("sync.include", patterns)?;
    }
    if let Some(patterns) = &config.sync.exclude {
        validate_patterns("sync.exclude", patterns)?;
    }
    Ok(())
}

fn validate_name(field: &str, value: &str) -> Result<(), ConfigError> {
    if value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        Ok(())
    } else {
        invalid(field, "must contain only letters, digits, '_' or '-'")
    }
}

fn validate_image(field: &str, image: &str) -> Result<(), ConfigError> {
    if !image.is_empty() && image.chars().all(safe_image_char) {
        Ok(())
    } else {
        invalid(
            field,
            "contains characters that are unsafe for an image reference",
        )
    }
}

fn validate_env_names(field: &str, values: Option<&[String]>) -> Result<(), ConfigError> {
    for value in values.unwrap_or_default() {
        if !value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
            || !value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return invalid(
                field,
                &format!("'{value}' is not an environment variable name"),
            );
        }
    }
    Ok(())
}

fn safe_image_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | '/' | ':' | '@')
}

fn safe_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | '/' | ':')
}

fn valid_memory(memory: &str) -> bool {
    let digits = match memory.as_bytes().last() {
        Some(b'b' | b'k' | b'm' | b'g' | b't') => &memory[..memory.len() - 1],
        _ => memory,
    };
    !digits.is_empty()
        && digits.chars().all(|character| character.is_ascii_digit())
        && digits.parse::<u64>().is_ok_and(|value| value > 0)
}

fn validate_patterns(field: &str, patterns: &[String]) -> Result<(), ConfigError> {
    for pattern in patterns {
        if pattern.is_empty()
            || pattern.chars().any(char::is_control)
            || Path::new(pattern).is_absolute()
            || Path::new(pattern)
                .components()
                .any(|component| component == Component::ParentDir)
        {
            return invalid(
                field,
                &format!("'{pattern}' must be a safe relative path or glob"),
            );
        }
    }
    Ok(())
}

fn invalid<T>(field: &str, reason: &str) -> Result<T, ConfigError> {
    Err(ConfigError::InvalidValue {
        field: field.to_owned(),
        reason: reason.to_owned(),
    })
}

fn trust_key(project_path: &Path, project_bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(project_path.as_os_str().as_encoded_bytes());
    digest.update([0]);
    digest.update(project_bytes);
    hex(&digest.finalize())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn acknowledgement_is_valid(
    path: &Path,
    project_path: &Path,
    project_bytes: &[u8],
) -> Result<bool, ConfigError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() && !metadata.file_type().is_symlink() => {
            let expected = acknowledgement_content(project_path, project_bytes);
            let actual = fs::read_to_string(path).map_err(|source| ConfigError::TrustCache {
                path: path.to_owned(),
                source,
            })?;
            if actual == expected {
                Ok(true)
            } else {
                Err(ConfigError::TrustCache {
                    path: path.to_owned(),
                    source: std::io::Error::other("trust acknowledgement content is invalid"),
                })
            }
        }
        Ok(_) => Err(ConfigError::TrustCache {
            path: path.to_owned(),
            source: std::io::Error::other("trust acknowledgement is not a regular file"),
        }),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(ConfigError::TrustCache {
            path: path.to_owned(),
            source,
        }),
    }
}

fn write_acknowledgement(
    path: &Path,
    project_path: &Path,
    project_bytes: &[u8],
) -> Result<(), ConfigError> {
    let parent = path.parent().ok_or_else(|| ConfigError::TrustCache {
        path: path.to_owned(),
        source: std::io::Error::other("trust cache path has no parent"),
    })?;
    fs::create_dir_all(parent).map_err(|source| ConfigError::TrustCache {
        path: parent.to_owned(),
        source,
    })?;

    let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => file,
        Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
            if acknowledgement_is_valid(path, project_path, project_bytes)? {
                return Ok(());
            }
            return Err(ConfigError::TrustCache {
                path: path.to_owned(),
                source: std::io::Error::other(
                    "concurrent trust acknowledgement did not match the approved configuration",
                ),
            });
        }
        Err(source) => {
            return Err(ConfigError::TrustCache {
                path: path.to_owned(),
                source,
            });
        }
    };
    file.write_all(acknowledgement_content(project_path, project_bytes).as_bytes())
        .map_err(|source| ConfigError::TrustCache {
            path: path.to_owned(),
            source,
        })
}

fn acknowledgement_content(project_path: &Path, project_bytes: &[u8]) -> String {
    format!(
        "project={}\ncontent_sha256={}\n",
        project_path.display(),
        hex(&Sha256::digest(project_bytes))
    )
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct Accept;

    impl ProjectConfigConfirmer for Accept {
        fn confirm(&mut self, _path: &Path, effective: &str) -> Result<bool, ConfigError> {
            assert!(effective.contains("[environment]"));
            Ok(true)
        }
    }

    struct Reject;

    impl ProjectConfigConfirmer for Reject {
        fn confirm(&mut self, _path: &Path, _effective: &str) -> Result<bool, ConfigError> {
            Ok(false)
        }
    }

    fn paths(root: &TempDir) -> ConfigPaths {
        ConfigPaths {
            global: root.path().join("global.toml"),
            project: root.path().join("project").join("cage.toml"),
            cage_home: root.path().join("home"),
        }
    }

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn defaults_when_files_are_absent() {
        let root = TempDir::new().unwrap();
        let loaded = ConfigLoader::new(paths(&root))
            .load_non_interactive()
            .unwrap();
        assert_eq!(loaded, CageConfig::default());
    }

    #[test]
    fn loads_global_only_without_project_confirmation() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(
            &paths.global,
            "[environment]\nruntime = \"docker\"\n[profiles.base]\nmemory = \"4g\"\n",
        );
        let loaded = ConfigLoader::new(paths).load_non_interactive().unwrap();
        assert_eq!(loaded.runtime(), Some("docker"));
        assert_eq!(
            loaded.profile("base").unwrap().memory.as_deref(),
            Some("4g")
        );
    }

    #[test]
    fn resolved_profile_applies_safe_resource_defaults() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.global, "[profiles.base]\nimage = \"node:20\"\n");
        let loaded = ConfigLoader::new(paths).load_non_interactive().unwrap();
        let profile = loaded.resolved_profile("base").unwrap();
        assert_eq!(profile.memory.as_deref(), Some(CageConfig::DEFAULT_MEMORY));
        assert_eq!(profile.cpus.as_deref(), Some(CageConfig::DEFAULT_CPUS));
    }

    #[test]
    fn project_overrides_global_fields_and_preserves_other_fields() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(
            &paths.global,
            "[environment]\nruntime = \"docker\"\n[profiles.dev]\nmemory = \"4g\"\ncpus = \"2.0\"\nenv = [\"GLOBAL_TOKEN\"]\n",
        );
        write(
            &paths.project,
            "[environment]\nruntime = \"podman\"\n[profiles.dev]\nmemory = \"8g\"\nimage = \"node:20\"\n",
        );
        let loaded = ConfigLoader::new(paths).load(&mut Accept).unwrap();
        let profile = loaded.profile("dev").unwrap();
        assert_eq!(loaded.runtime(), Some("podman"));
        assert_eq!(profile.memory.as_deref(), Some("8g"));
        assert_eq!(profile.cpus.as_deref(), Some("2.0"));
        assert_eq!(profile.image.as_deref(), Some("node:20"));
        assert_eq!(
            profile.env.as_deref(),
            Some(&[String::from("GLOBAL_TOKEN")][..])
        );
    }

    #[test]
    fn unchanged_project_is_accepted_noninteractively_after_approval() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.project, "[profiles.dev]\nmemory = \"8g\"\n");
        ConfigLoader::new(paths.clone()).load(&mut Accept).unwrap();
        ConfigLoader::new(paths).load_non_interactive().unwrap();
    }

    #[test]
    fn changed_project_requires_fresh_approval() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.project, "[profiles.dev]\nmemory = \"8g\"\n");
        ConfigLoader::new(paths.clone()).load(&mut Accept).unwrap();
        write(&paths.project, "[profiles.dev]\nmemory = \"16g\"\n");
        let error = ConfigLoader::new(paths).load_non_interactive().unwrap_err();
        assert!(matches!(error, ConfigError::UntrustedProject { .. }));
    }

    #[test]
    fn rejected_project_is_not_cached() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.project, "[profiles.dev]\nmemory = \"8g\"\n");
        let error = ConfigLoader::new(paths.clone())
            .load(&mut Reject)
            .unwrap_err();
        assert!(matches!(error, ConfigError::ProjectRejected { .. }));
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::UntrustedProject { .. })
        ));
    }

    #[test]
    fn invalid_toml_returns_an_error() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.global, "[[not valid");
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::InvalidToml { .. })
        ));
    }

    #[test]
    fn oversized_configuration_is_rejected() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        let oversized = usize::try_from(MAX_CONFIG_BYTES + 1).unwrap();
        fs::write(&paths.global, vec![b'x'; oversized]).unwrap();
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::TooLarge { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn configuration_symlink_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        let target = root.path().join("target.toml");
        write(&target, "[environment]\nruntime = \"docker\"\n");
        symlink(target, &paths.global).unwrap();
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::InvalidFileType { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn configuration_fifo_is_rejected_without_blocking() {
        use std::process::Command;

        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        let status = Command::new("mkfifo").arg(&paths.global).status().unwrap();
        assert!(status.success());
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::InvalidFileType { .. })
        ));
    }

    #[test]
    fn malicious_image_and_environment_value_are_rejected() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(
            &paths.global,
            "[profiles.bad]\nimage = \"ubuntu; curl attacker\"\nenv = [\"TOKEN=value\"]\n",
        );
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::InvalidValue { .. })
        ));
    }

    #[test]
    fn hardening_adjacent_and_unknown_keys_are_rejected() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.global, "[profiles.bad]\nprivileged = true\n");
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::InvalidToml { .. })
        ));
    }

    #[test]
    fn invalid_resources_and_dind_enablement_are_rejected() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(
            &paths.global,
            "[profiles.bad]\nmemory = \"4g;rm\"\ncpus = \"NaN\"\ndind = true\n",
        );
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::InvalidValue { .. })
        ));
    }

    #[test]
    fn invalid_default_profile_is_rejected() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.global, "[defaults]\nprofile = \"missing\"\n");
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::UnknownProfile(name)) if name == "missing"
        ));
    }

    #[test]
    fn unsafe_sync_paths_are_rejected() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.global, "[sync]\ninclude = [\"../secrets/**\"]\n");
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::InvalidValue { .. })
        ));
    }

    #[test]
    fn corrupted_acknowledgement_fails_closed() {
        let root = TempDir::new().unwrap();
        let paths = paths(&root);
        write(&paths.project, "[profiles.dev]\nmemory = \"8g\"\n");
        ConfigLoader::new(paths.clone()).load(&mut Accept).unwrap();
        let acknowledgement = fs::read_dir(paths.cage_home.join("trusted-configs"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        fs::write(acknowledgement, "corrupted").unwrap();
        assert!(matches!(
            ConfigLoader::new(paths).load_non_interactive(),
            Err(ConfigError::TrustCache { .. })
        ));
    }
}
