//! Test utilities for Environment
//!
//! This module provides test helpers and builders for creating Environment
//! instances in tests with proper isolation using temporary directories.

use super::*;
use crate::adapters::ssh::SshCredentials;
use crate::domain::backup::BackupConfig;
use crate::domain::grafana::GrafanaConfig;
use crate::domain::prometheus::PrometheusConfig;
use crate::domain::provider::{LxdConfig, ProviderConfig};
use crate::domain::tracker::TrackerConfig;
use crate::domain::EnvironmentName;
use crate::shared::Username;
use chrono::{TimeZone, Utc};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Returns a fixed test timestamp for deterministic tests (2025-01-01 00:00:00 UTC)
#[must_use]
pub fn test_timestamp() -> chrono::DateTime<chrono::Utc> {
    Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()
}

/// Test builder for creating Environment instances with sensible defaults
///
/// This builder simplifies test setup by providing default values and allowing
/// customization through a fluent API. It automatically manages temporary
/// directories and creates all required value objects.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::environment::testing::EnvironmentTestBuilder;
///
/// // Simple environment with defaults
/// let (env, _temp_dir) = EnvironmentTestBuilder::new()
///     .build_with_custom_paths();
///
/// // Customized environment
/// let (env, data_dir, build_dir, _temp_dir) = EnvironmentTestBuilder::new()
///     .with_name("staging")
///     .with_ssh_key_name("custom_key")
///     .build_with_custom_paths();
/// ```
pub struct EnvironmentTestBuilder {
    env_name: String,
    ssh_key_name: String,
    ssh_username: String,
    temp_dir: TempDir,
    prometheus_config: Option<PrometheusConfig>,
    backup_config: Option<BackupConfig>,
}

impl EnvironmentTestBuilder {
    /// Creates a new builder with sensible defaults
    ///
    /// # Panics
    ///
    /// Panics if the temporary directory cannot be created (e.g., due to filesystem issues).
    #[must_use]
    pub fn new() -> Self {
        Self {
            env_name: "test-env".to_string(),
            ssh_key_name: "test_key".to_string(),
            ssh_username: "torrust".to_string(),
            temp_dir: TempDir::new().expect("Failed to create temp directory"),
            prometheus_config: Some(PrometheusConfig::default()),
            backup_config: None,
        }
    }

    /// Sets the environment name
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.env_name = name.to_string();
        self
    }

    /// Sets the SSH key name (without .pub extension)
    #[must_use]
    pub fn with_ssh_key_name(mut self, key_name: &str) -> Self {
        self.ssh_key_name = key_name.to_string();
        self
    }

    /// Sets the SSH username
    #[must_use]
    pub fn with_ssh_username(mut self, username: &str) -> Self {
        self.ssh_username = username.to_string();
        self
    }

    /// Sets the Prometheus configuration
    #[must_use]
    pub fn with_prometheus_config(mut self, config: Option<PrometheusConfig>) -> Self {
        self.prometheus_config = config;
        self
    }

    /// Sets the Backup configuration
    #[must_use]
    pub fn with_backup_config(mut self, config: Option<BackupConfig>) -> Self {
        self.backup_config = config;
        self
    }

    /// Builds an Environment with custom paths inside a temporary directory
    ///
    /// This is the recommended way to create test environments as it ensures
    /// complete isolation and automatic cleanup. All paths are created within
    /// a temporary directory that is automatically cleaned up when the `TempDir`
    /// is dropped.
    ///
    /// # Returns
    ///
    /// Returns a tuple of:
    /// - The created `Environment` instance
    /// - The data directory path (inside temp dir)
    /// - The build directory path (inside temp dir)
    /// - The `TempDir` which must be kept alive for the test duration
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The environment name is invalid
    /// - The SSH username is invalid
    /// - SSH key files cannot be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::testing::EnvironmentTestBuilder;
    ///
    /// let (env, data_dir, build_dir, _temp_dir) = EnvironmentTestBuilder::new()
    ///     .with_name("my-test")
    ///     .build_with_custom_paths();
    ///
    /// // Use environment in tests
    /// assert!(data_dir.to_str().unwrap().contains("my-test"));
    /// assert!(build_dir.to_str().unwrap().contains("my-test"));
    /// // _temp_dir is automatically cleaned up when it goes out of scope
    /// ```
    #[must_use]
    pub fn build_with_custom_paths(self) -> (Environment<Created>, PathBuf, PathBuf, TempDir) {
        let temp_path = self.temp_dir.path();
        let data_dir = temp_path.join("data").join(&self.env_name);
        let build_dir = temp_path.join("build").join(&self.env_name);

        let env_name = EnvironmentName::new(self.env_name).unwrap();
        let ssh_username = Username::new(self.ssh_username).unwrap();
        let ssh_credentials = SshCredentials::new(
            temp_path.join(&self.ssh_key_name),
            temp_path.join(format!("{}.pub", &self.ssh_key_name)),
            ssh_username,
        );

        let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();
        let provider_config = ProviderConfig::Lxd(LxdConfig { profile_name });

        let user_inputs = UserInputs::with_tracker(
            &env_name,
            provider_config,
            ssh_credentials,
            22,
            TrackerConfig::default(),
            self.prometheus_config.clone(),
            // Grafana is only enabled when Prometheus is enabled (cross-service invariant)
            self.prometheus_config
                .as_ref()
                .map(|_| GrafanaConfig::default()),
            None,
            self.backup_config,
        )
        .expect("Test UserInputs should always be valid with defaults");

        let context = EnvironmentContext {
            created_at: test_timestamp(),
            user_inputs,
            internal_config: InternalConfig {
                data_dir: data_dir.clone(),
                build_dir: build_dir.clone(),
            },
            runtime_outputs: crate::domain::environment::RuntimeOutputs::new(),
        };

        let environment = Environment {
            context,
            state: Created,
        };

        (environment, data_dir, build_dir, self.temp_dir)
    }

    /// Returns a reference to the temp directory path
    #[must_use]
    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }
}

impl Default for EnvironmentTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
