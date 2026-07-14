//! Container runtime mount parsing and validation.

use std::path::PathBuf;

use super::path::{PathPolicy, SecurityError};

/// Identifies whether a mount came from Cage internals or untrusted user input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeSource {
    /// A mount assembled by Cage itself.
    Internal,
    /// A mount supplied through CLI or configuration input.
    UserSpecified,
}

/// Runtime mount semantics relevant to validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MountKind {
    /// A bind mount backed by a host filesystem path.
    Bind,
    /// A host device mapping.
    Device,
    /// A runtime-managed named volume with no direct host path.
    NamedVolume,
}

/// A normalized mount extracted from container runtime arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mount {
    /// Provenance controlling whether host-path policy applies.
    source: VolumeSource,
    /// Mount semantics.
    kind: MountKind,
    /// Host path for bind/device mounts; absent for named volumes.
    host: Option<PathBuf>,
    /// Runtime-managed volume name; absent for bind/device mounts and anonymous volumes.
    volume_name: Option<String>,
    /// Container destination when one is present.
    container: Option<String>,
    /// Runtime-specific options retained for downstream argument construction.
    options: Vec<String>,
}

impl Mount {
    /// Constructs a Cage-owned internal bind mount.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "consumed by the Issue #8 runtime integration")
    )]
    #[must_use]
    pub(crate) fn internal_bind(host: PathBuf, container: impl Into<String>) -> Self {
        Self {
            source: VolumeSource::Internal,
            kind: MountKind::Bind,
            host: Some(host),
            volume_name: None,
            container: Some(container.into()),
            options: Vec::new(),
        }
    }

    /// Returns the trust provenance assigned during parsing or internal construction.
    #[must_use]
    pub const fn source(&self) -> VolumeSource {
        self.source
    }

    /// Returns the normalized mount kind.
    #[must_use]
    pub const fn kind(&self) -> MountKind {
        self.kind
    }

    /// Returns the host path for bind/device mounts.
    #[must_use]
    pub fn host(&self) -> Option<&std::path::Path> {
        self.host.as_deref()
    }

    /// Returns the runtime-managed volume name when one was supplied.
    #[must_use]
    pub fn volume_name(&self) -> Option<&str> {
        self.volume_name.as_deref()
    }

    /// Returns the container destination when present.
    #[must_use]
    pub fn container(&self) -> Option<&str> {
        self.container.as_deref()
    }

    /// Returns runtime-specific mount options.
    #[must_use]
    pub fn options(&self) -> &[String] {
        &self.options
    }
}

/// Parses Docker/Podman `-v`, `--volume`, `--mount`, and `--device` arguments.
///
/// Unrelated runtime arguments are ignored. Malformed recognized mount flags fail closed.
pub fn parse_runtime_mounts(args: &[String]) -> Result<Vec<Mount>, SecurityError> {
    let mut mounts = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        match argument.as_str() {
            "-v" | "--volume" => {
                let value = required_next(args, index, argument)?;
                mounts.push(parse_volume(value)?);
                index += 2;
            }
            "--mount" => {
                let value = required_next(args, index, argument)?;
                if let Some(mount) = parse_mount(value)? {
                    mounts.push(mount);
                }
                index += 2;
            }
            "--device" => {
                let value = required_next(args, index, argument)?;
                mounts.push(parse_device(value)?);
                index += 2;
            }
            _ if argument.starts_with("--volume=") => {
                mounts.push(parse_volume(&argument["--volume=".len()..])?);
                index += 1;
            }
            _ if argument.starts_with("--mount=") => {
                if let Some(mount) = parse_mount(&argument["--mount=".len()..])? {
                    mounts.push(mount);
                }
                index += 1;
            }
            _ if argument.starts_with("--device=") => {
                mounts.push(parse_device(&argument["--device=".len()..])?);
                index += 1;
            }
            _ if argument.starts_with("-v") && argument.len() > 2 => {
                mounts.push(parse_volume(&argument[2..])?);
                index += 1;
            }
            _ => index += 1,
        }
    }
    Ok(mounts)
}

/// Validates every user-specified host path.
///
/// Internal mounts and named volumes are trusted construction-time values and are preserved.
pub fn validate_mounts(mounts: &[Mount]) -> Result<(), SecurityError> {
    validate_mounts_with_policy(mounts, &PathPolicy::default())
}

fn validate_mounts_with_policy(mounts: &[Mount], policy: &PathPolicy) -> Result<(), SecurityError> {
    for mount in mounts {
        validate_mount_shape(mount)?;
        if mount.source == VolumeSource::Internal || mount.kind == MountKind::NamedVolume {
            continue;
        }
        let host = mount
            .host
            .as_ref()
            .ok_or_else(|| SecurityError::InvalidMount("host path is required".to_owned()))?;
        let host_text = host.to_str().ok_or_else(|| {
            SecurityError::InvalidMount("host path is not valid UTF-8".to_owned())
        })?;
        policy.validate_user_mount_path(host_text)?;
    }
    Ok(())
}

fn validate_mount_shape(mount: &Mount) -> Result<(), SecurityError> {
    let valid = match mount.kind {
        MountKind::Bind => {
            mount.host.is_some() && mount.volume_name.is_none() && mount.container.is_some()
        }
        MountKind::Device => mount.host.is_some() && mount.volume_name.is_none(),
        MountKind::NamedVolume => {
            mount.host.is_none()
                && mount.container.is_some()
                && mount.volume_name.as_deref().is_none_or(is_named_volume)
        }
    };
    if !valid {
        return Err(SecurityError::InvalidMount(
            "mount contains an inconsistent kind/host/container combination".to_owned(),
        ));
    }
    Ok(())
}

fn required_next<'a>(
    args: &'a [String],
    index: usize,
    flag: &str,
) -> Result<&'a str, SecurityError> {
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| SecurityError::InvalidMount(format!("{flag} requires a value")))
}

fn parse_volume(value: &str) -> Result<Mount, SecurityError> {
    if value.is_empty() {
        return Err(SecurityError::InvalidMount(
            "volume specification is empty".to_owned(),
        ));
    }
    let mut parts = value.splitn(3, ':');
    let host = parts.next().unwrap_or_default();
    let container = parts.next().ok_or_else(|| {
        SecurityError::InvalidMount(format!("volume must include a container path: {value}"))
    })?;
    if host.is_empty() || container.is_empty() {
        return Err(SecurityError::InvalidMount(format!(
            "volume has an empty host or container path: {value}"
        )));
    }
    let options = parts
        .next()
        .map(|raw| raw.split(',').map(str::to_owned).collect())
        .unwrap_or_default();

    if is_named_volume(host) {
        Ok(Mount {
            source: VolumeSource::UserSpecified,
            kind: MountKind::NamedVolume,
            host: None,
            volume_name: Some(host.to_owned()),
            container: Some(container.to_owned()),
            options,
        })
    } else {
        Ok(Mount {
            source: VolumeSource::UserSpecified,
            kind: MountKind::Bind,
            host: Some(PathBuf::from(host)),
            volume_name: None,
            container: Some(container.to_owned()),
            options,
        })
    }
}

fn parse_mount(value: &str) -> Result<Option<Mount>, SecurityError> {
    if value.is_empty() {
        return Err(SecurityError::InvalidMount(
            "mount specification is empty".to_owned(),
        ));
    }
    let fields = value.split(',').collect::<Vec<_>>();
    let mount_type = unique_field(&fields, &["type"])?.ok_or_else(|| {
        SecurityError::InvalidMount(format!("mount is missing required type: {value}"))
    })?;
    match mount_type {
        "volume" => return parse_named_mount(&fields, value).map(Some),
        "tmpfs" => return parse_tmpfs_mount(&fields, value),
        "bind" => {}
        unknown => {
            return Err(SecurityError::InvalidMount(format!(
                "unsupported mount type {unknown}: {value}"
            )));
        }
    }
    let host = unique_field(&fields, &["source", "src"])?.ok_or_else(|| {
        SecurityError::InvalidMount(format!("bind mount is missing source/src: {value}"))
    })?;
    let container = unique_field(&fields, &["destination", "dst", "target"])?.ok_or_else(|| {
        SecurityError::InvalidMount(format!("bind mount is missing target: {value}"))
    })?;
    if host.is_empty() || container.is_empty() {
        return Err(SecurityError::InvalidMount(format!(
            "bind mount has an empty source or target: {value}"
        )));
    }

    Ok(Some(Mount {
        source: VolumeSource::UserSpecified,
        kind: MountKind::Bind,
        host: Some(PathBuf::from(host)),
        volume_name: None,
        container: Some(container.to_owned()),
        options: fields
            .iter()
            .filter(|field| !field.contains('='))
            .map(|field| (*field).to_owned())
            .collect(),
    }))
}

fn parse_device(value: &str) -> Result<Mount, SecurityError> {
    if value.is_empty() {
        return Err(SecurityError::InvalidMount(
            "device specification is empty".to_owned(),
        ));
    }
    let mut parts = value.splitn(3, ':');
    let host = parts.next().unwrap_or_default();
    if host.is_empty() {
        return Err(SecurityError::InvalidMount(format!(
            "device has an empty host path: {value}"
        )));
    }
    let container = parts
        .next()
        .filter(|part| !part.is_empty())
        .map(str::to_owned);
    let options = parts
        .next()
        .filter(|part| !part.is_empty())
        .map(|part| vec![part.to_owned()])
        .unwrap_or_default();
    Ok(Mount {
        source: VolumeSource::UserSpecified,
        kind: MountKind::Device,
        host: Some(PathBuf::from(host)),
        volume_name: None,
        container,
        options,
    })
}

fn parse_named_mount(fields: &[&str], value: &str) -> Result<Mount, SecurityError> {
    if fields.iter().any(|field| {
        let name = field.split_once('=').map_or(*field, |(name, _)| name);
        name == "volume-driver" || name == "volume-opt"
    }) {
        return Err(SecurityError::InvalidMount(format!(
            "custom volume drivers and volume options are forbidden: {value}"
        )));
    }
    reject_unknown_fields(
        fields,
        &[
            "type",
            "source",
            "src",
            "destination",
            "dst",
            "target",
            "readonly",
            "volume-nocopy",
        ],
        value,
    )?;
    let name = unique_field(fields, &["source", "src"])?;
    if name.is_some_and(|name| !is_named_volume(name)) {
        return Err(SecurityError::InvalidMount(format!(
            "volume source is not a valid named volume: {value}"
        )));
    }
    let container = unique_field(fields, &["destination", "dst", "target"])?
        .filter(|target| !target.is_empty())
        .ok_or_else(|| {
            SecurityError::InvalidMount(format!("volume mount is missing target: {value}"))
        })?;
    Ok(Mount {
        source: VolumeSource::UserSpecified,
        kind: MountKind::NamedVolume,
        host: None,
        volume_name: name.map(str::to_owned),
        container: Some(container.to_owned()),
        options: fields
            .iter()
            .filter(|field| !field.contains('='))
            .map(|field| (*field).to_owned())
            .collect(),
    })
}

fn parse_tmpfs_mount(fields: &[&str], value: &str) -> Result<Option<Mount>, SecurityError> {
    reject_unknown_fields(
        fields,
        &[
            "type",
            "destination",
            "dst",
            "target",
            "readonly",
            "tmpfs-size",
            "tmpfs-mode",
        ],
        value,
    )?;
    if unique_field(fields, &["source", "src"])?.is_some() {
        return Err(SecurityError::InvalidMount(format!(
            "tmpfs mount must not include a host source: {value}"
        )));
    }
    unique_field(fields, &["destination", "dst", "target"])?
        .filter(|target| !target.is_empty())
        .ok_or_else(|| {
            SecurityError::InvalidMount(format!("tmpfs mount is missing target: {value}"))
        })?;
    Ok(None)
}

fn reject_unknown_fields(
    fields: &[&str],
    allowed: &[&str],
    value: &str,
) -> Result<(), SecurityError> {
    for field in fields {
        let name = field.split_once('=').map_or(*field, |(name, _)| name);
        if !allowed.contains(&name) {
            return Err(SecurityError::InvalidMount(format!(
                "unsupported mount field {name}: {value}"
            )));
        }
    }
    Ok(())
}

fn unique_field<'a>(fields: &'a [&str], names: &[&str]) -> Result<Option<&'a str>, SecurityError> {
    let mut found = None;
    for field in fields {
        let Some((name, value)) = field.split_once('=') else {
            continue;
        };
        if names.contains(&name) && found.replace(value).is_some() {
            return Err(SecurityError::InvalidMount(format!(
                "mount field is specified more than once: {}",
                names.join("/")
            )));
        }
    }
    Ok(found)
}

fn is_named_volume(source: &str) -> bool {
    !source.is_empty()
        && !source.contains('/')
        && !source.contains('\\')
        && !source.starts_with('.')
        && source
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_.-".contains(character))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn parses_all_supported_mount_flag_forms() -> Result<(), Box<dyn std::error::Error>> {
        let mounts = parse_runtime_mounts(&strings(&[
            "run",
            "-v",
            "/host/a:/container/a:ro",
            "--volume=/host/b:/container/b",
            "-vnamed-data:/container/data",
            "--mount",
            "type=bind,src=/host/c,dst=/container/c,readonly",
            "--device=/dev/null:/dev/safe:r",
        ]))?;

        assert_eq!(mounts.len(), 5);
        assert_eq!(mounts[0].kind(), MountKind::Bind);
        assert_eq!(mounts[0].options(), ["ro"]);
        assert_eq!(mounts[2].kind(), MountKind::NamedVolume);
        assert_eq!(mounts[3].host(), Some(std::path::Path::new("/host/c")));
        assert_eq!(mounts[4].kind(), MountKind::Device);
        Ok(())
    }

    #[test]
    fn rejects_malformed_recognized_flags() {
        assert!(parse_runtime_mounts(&strings(&["--volume"])).is_err());
        assert!(parse_runtime_mounts(&strings(&["--volume=host-only"])).is_err());
        assert!(parse_runtime_mounts(&strings(&["--mount=type=bind,target=/workspace"])).is_err());
        assert!(
            parse_runtime_mounts(&strings(&[
                "--mount=type=bind,src=/one,source=/two,target=/workspace"
            ]))
            .is_err()
        );
        assert!(
            parse_runtime_mounts(&strings(&[
                "--mount=type=unknown,source=/one,target=/workspace"
            ]))
            .is_err()
        );
        assert!(
            parse_runtime_mounts(&strings(&["--mount=source=/one,target=/workspace"])).is_err()
        );
        assert!(parse_runtime_mounts(&strings(&["--device="])).is_err());
    }

    #[test]
    fn accepts_plain_named_anonymous_and_tmpfs_mounts() -> Result<(), Box<dyn std::error::Error>> {
        let mounts = parse_runtime_mounts(&strings(&[
            "--mount=type=volume,src=workspace-data,dst=/workspace,readonly",
            "--mount=type=volume,target=/cache,volume-nocopy",
            "--mount=type=tmpfs,target=/run/cache,tmpfs-size=1m,tmpfs-mode=0700",
        ]))?;

        assert_eq!(mounts.len(), 2);
        assert_eq!(mounts[0].kind(), MountKind::NamedVolume);
        assert_eq!(mounts[0].volume_name(), Some("workspace-data"));
        assert_eq!(mounts[1].volume_name(), None);
        validate_mounts_with_policy(&mounts, &PathPolicy::new(None, None))?;
        Ok(())
    }

    #[test]
    fn rejects_local_volume_driver_host_bind_escape() {
        let exploit = [
            "--mount=type=volume,src=evil,dst=/workspace,volume-driver=local,",
            "volume-opt=type=none,volume-opt=device=/var/run/docker,volume-opt=o=bind",
        ]
        .concat();

        assert!(parse_runtime_mounts(&strings(&[&exploit])).is_err());
        assert!(
            parse_runtime_mounts(&strings(&[
                "--mount=type=tmpfs,source=/var/run/docker,target=/workspace"
            ]))
            .is_err()
        );
        assert!(
            parse_runtime_mounts(&strings(&[
                "--mount=type=volume,source=/var/run/docker,target=/workspace"
            ]))
            .is_err()
        );
    }

    #[test]
    fn validates_user_bind_and_device_paths() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempdir()?;
        let file = root.path().join("device");
        fs::write(&file, "device")?;
        let mounts = vec![
            Mount {
                source: VolumeSource::UserSpecified,
                kind: MountKind::Bind,
                host: Some(root.path().to_path_buf()),
                volume_name: None,
                container: Some("/workspace".to_owned()),
                options: Vec::new(),
            },
            Mount {
                source: VolumeSource::UserSpecified,
                kind: MountKind::Device,
                host: Some(file.clone()),
                volume_name: None,
                container: None,
                options: Vec::new(),
            },
        ];

        validate_mounts_with_policy(&mounts, &PathPolicy::new(None, None))?;
        Ok(())
    }

    #[test]
    fn internal_mounts_and_named_volumes_bypass_user_path_blocklist()
    -> Result<(), Box<dyn std::error::Error>> {
        let home = tempdir()?;
        fs::create_dir(home.path().join(".docker"))?;
        let internal = Mount::internal_bind(home.path().join(".docker"), "/internal");
        let named = Mount {
            source: VolumeSource::UserSpecified,
            kind: MountKind::NamedVolume,
            host: None,
            volume_name: Some("workspace-data".to_owned()),
            container: Some("/workspace".to_owned()),
            options: Vec::new(),
        };
        let policy = PathPolicy::new(Some(home.path().to_path_buf()), None);

        validate_mounts_with_policy(&[internal, named], &policy)?;
        Ok(())
    }

    #[test]
    fn rejects_user_mount_below_protected_directory() -> Result<(), Box<dyn std::error::Error>> {
        let home = tempdir()?;
        fs::create_dir_all(home.path().join(".docker/run"))?;
        let socket = home.path().join(".docker/run/docker.sock");
        fs::write(&socket, "socket")?;
        let mount = Mount {
            source: VolumeSource::UserSpecified,
            kind: MountKind::Bind,
            host: Some(socket),
            volume_name: None,
            container: Some("/var/run/docker.sock".to_owned()),
            options: Vec::new(),
        };
        let policy = PathPolicy::new(Some(home.path().to_path_buf()), None);

        assert!(matches!(
            validate_mounts_with_policy(&[mount], &policy),
            Err(SecurityError::BlockedMount(_))
        ));
        Ok(())
    }

    #[test]
    fn rejects_inconsistent_mount_state() {
        let forged = Mount {
            source: VolumeSource::Internal,
            kind: MountKind::NamedVolume,
            host: Some(PathBuf::from("/var/run/docker.sock")),
            volume_name: Some("forged".to_owned()),
            container: Some("/workspace".to_owned()),
            options: Vec::new(),
        };

        assert!(matches!(
            validate_mounts_with_policy(&[forged], &PathPolicy::new(None, None)),
            Err(SecurityError::InvalidMount(_))
        ));
    }
}
