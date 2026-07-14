//! Cage configuration loading and validation.

mod loader;

use std::collections::BTreeMap;

pub use loader::{
    ConfigError, ConfigLoader, ConfigPaths, DialoguerConfirmer, MAX_CONFIG_BYTES,
    NonInteractiveConfirmer, ProjectConfigConfirmer,
};
use serde::{Deserialize, Serialize};

/// Fully merged Cage configuration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CageConfig {
    pub environment: EnvironmentConfig,
    pub profiles: BTreeMap<String, ProfileConfig>,
    pub sync: SyncConfig,
    pub adapters: BTreeMap<String, CustomAdapterConfig>,
    pub defaults: DefaultsConfig,
}

/// Container environment selection.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EnvironmentConfig {
    /// Container runtime preference (docker or podman).
    pub runtime: Option<String>,
}

/// Named agent profile.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProfileConfig {
    pub image: Option<String>,
    pub memory: Option<String>,
    pub cpus: Option<String>,
    /// Environment variable names. Values are never stored in configuration.
    pub env: Option<Vec<String>>,
    pub gpus: Option<Vec<String>>,
    pub with_hooks: Option<bool>,
    pub dind: Option<bool>,
}

/// Workspace synchronization filters.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SyncConfig {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

/// User-defined agent adapter.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CustomAdapterConfig {
    pub image: Option<String>,
    pub command: Option<Vec<String>>,
    /// Environment variable names. Values are resolved later by the credential layer.
    pub env: Option<Vec<String>>,
    pub config_files: Option<Vec<String>>,
}

/// Default selections used when the CLI does not provide an override.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DefaultsConfig {
    pub profile: Option<String>,
    pub agent: Option<String>,
}

impl CageConfig {
    pub const DEFAULT_MEMORY: &'static str = "4g";
    pub const DEFAULT_CPUS: &'static str = "2.0";

    /// Resolve a required profile by name.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::UnknownProfile` when the profile does not exist.
    pub fn profile(&self, name: &str) -> Result<&ProfileConfig, ConfigError> {
        self.profiles
            .get(name)
            .ok_or_else(|| ConfigError::UnknownProfile(name.to_owned()))
    }

    /// Resolve a profile and apply the resource defaults owned by the configuration layer.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::UnknownProfile` when the profile does not exist.
    pub fn resolved_profile(&self, name: &str) -> Result<ProfileConfig, ConfigError> {
        let mut profile = self.profile(name)?.clone();
        profile
            .memory
            .get_or_insert_with(|| Self::DEFAULT_MEMORY.to_owned());
        profile
            .cpus
            .get_or_insert_with(|| Self::DEFAULT_CPUS.to_owned());
        Ok(profile)
    }

    /// Runtime preference for the Docker/Podman selector.
    #[must_use]
    pub fn runtime(&self) -> Option<&str> {
        self.environment.runtime.as_deref()
    }
}
