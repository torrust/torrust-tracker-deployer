//! Environment Creation Parameters
//!
//! This module provides `EnvironmentParams`, a domain value object that holds
//! all validated parameters needed to create an `Environment` aggregate.
//!
//! # DDD Pattern: Factory Input
//!
//! This is a value object that groups all the inputs required by the
//! `Environment::create()` factory method. It provides:
//!
//! - **Named fields**: Self-documenting, no positional confusion
//! - **Type safety**: All fields are validated domain types
//! - **Clean API**: Single parameter instead of 10+ arguments
//!
//! # Architecture
//!
//! ```text
//! EnvironmentCreationConfig (DTO - Application Layer)
//!         │
//!         │ TryFrom (in Application Layer)
//!         ▼
//! EnvironmentParams (Domain Value Object)
//!         │
//!         │ Environment::create(params, working_dir, timestamp)
//!         ▼
//! Environment<Created> (Domain Aggregate)
//! ```
//!
//! The `TryFrom` implementation lives in the Application layer since it needs
//! to reference the DTO, but `EnvironmentParams` itself is a pure domain type.
//!
//! # Usage
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_lib::domain::environment::EnvironmentParams;
//! use torrust_tracker_deployer_lib::domain::{EnvironmentName, InstanceName};
//!
//! // EnvironmentParams is typically constructed via TryFrom in application layer
//! // or directly in domain tests
//! ```

use crate::adapters::ssh::SshCredentials;
use crate::domain::backup::BackupConfig;
use crate::domain::grafana::GrafanaConfig;
use crate::domain::https::HttpsConfig;
use crate::domain::prometheus::PrometheusConfig;
use crate::domain::provider::ProviderConfig;
use crate::domain::tracker::TrackerConfig;
use crate::domain::{EnvironmentName, InstanceName};

/// Parameters for creating a new Environment aggregate
///
/// This value object contains all validated domain objects needed to construct
/// an `Environment<Created>` aggregate. It serves as a "factory input" pattern,
/// grouping related parameters into a single, self-documenting type.
///
/// # Field Categories
///
/// - **Identity**: `environment_name`, `instance_name`
/// - **Infrastructure**: `provider_config`, `ssh_credentials`, `ssh_port`
/// - **Application**: `tracker_config`
/// - **Observability**: `prometheus_config`, `grafana_config`
/// - **Security**: `https_config`
///
/// # Invariants
///
/// All fields are pre-validated domain types. Cross-field validation
/// (e.g., Grafana requires Prometheus) happens in `Environment::create()`.
#[derive(Debug, Clone)]
pub struct EnvironmentParams {
    /// Validated environment name (e.g., "production", "staging")
    pub environment_name: EnvironmentName,

    /// Validated instance name for the VM/container
    ///
    /// Either user-provided or auto-generated as `torrust-tracker-vm-{env_name}`
    pub instance_name: InstanceName,

    /// Provider-specific configuration (LXD, Hetzner, etc.)
    pub provider_config: ProviderConfig,

    /// SSH credentials for remote access to the deployed instance
    pub ssh_credentials: SshCredentials,

    /// SSH port for remote connections (typically 22)
    pub ssh_port: u16,

    /// Tracker application configuration
    pub tracker_config: TrackerConfig,

    /// Optional Prometheus monitoring configuration
    pub prometheus_config: Option<PrometheusConfig>,

    /// Optional Grafana dashboard configuration
    ///
    /// Note: Requires `prometheus_config` to be set (validated in `Environment::create()`)
    pub grafana_config: Option<GrafanaConfig>,

    /// Optional HTTPS/TLS configuration for secure endpoints
    pub https_config: Option<HttpsConfig>,

    /// Optional backup service configuration
    pub backup_config: Option<BackupConfig>,
}

impl EnvironmentParams {
    /// Creates a new `EnvironmentParams` instance
    ///
    /// This constructor is primarily used in domain tests. In production,
    /// `EnvironmentParams` is typically constructed via `TryFrom` conversion
    /// from a configuration DTO in the application layer.
    ///
    /// # Arguments
    ///
    /// * `environment_name` - Validated environment name
    /// * `instance_name` - Validated instance name
    /// * `provider_config` - Provider configuration
    /// * `ssh_credentials` - SSH access credentials
    /// * `ssh_port` - SSH port number
    /// * `tracker_config` - Tracker application configuration
    /// * `prometheus_config` - Optional Prometheus configuration
    /// * `grafana_config` - Optional Grafana configuration
    /// * `https_config` - Optional HTTPS configuration
    /// * `backup_config` - Optional backup configuration
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        environment_name: EnvironmentName,
        instance_name: InstanceName,
        provider_config: ProviderConfig,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
        tracker_config: TrackerConfig,
        prometheus_config: Option<PrometheusConfig>,
        grafana_config: Option<GrafanaConfig>,
        https_config: Option<HttpsConfig>,
        backup_config: Option<BackupConfig>,
    ) -> Self {
        Self {
            environment_name,
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker_config,
            prometheus_config,
            grafana_config,
            https_config,
            backup_config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::provider::LxdConfig;
    use crate::domain::ProfileName;
    use crate::shared::Username;
    use std::path::PathBuf;

    fn sample_ssh_credentials() -> SshCredentials {
        let project_root = env!("CARGO_MANIFEST_DIR");
        SshCredentials::new(
            PathBuf::from(format!("{project_root}/fixtures/testing_rsa")),
            PathBuf::from(format!("{project_root}/fixtures/testing_rsa.pub")),
            Username::new("torrust").unwrap(),
        )
    }

    fn sample_tracker_config() -> TrackerConfig {
        TrackerConfig::default()
    }

    #[test]
    fn it_should_create_environment_params_with_all_fields() {
        let params = EnvironmentParams::new(
            EnvironmentName::new("test-env").unwrap(),
            InstanceName::new("test-instance".to_string()).unwrap(),
            ProviderConfig::Lxd(LxdConfig {
                profile_name: ProfileName::new("lxd-test").unwrap(),
            }),
            sample_ssh_credentials(),
            22,
            sample_tracker_config(),
            None,
            None,
            None,
            None,
        );

        assert_eq!(params.environment_name.as_str(), "test-env");
        assert_eq!(params.instance_name.as_str(), "test-instance");
        assert_eq!(params.ssh_port, 22);
    }

    #[test]
    fn it_should_provide_named_field_access() {
        let params = EnvironmentParams::new(
            EnvironmentName::new("prod").unwrap(),
            InstanceName::new("prod-vm".to_string()).unwrap(),
            ProviderConfig::Lxd(LxdConfig {
                profile_name: ProfileName::new("lxd-prod").unwrap(),
            }),
            sample_ssh_credentials(),
            2222,
            sample_tracker_config(),
            None,
            None,
            None,
            None,
        );

        // All fields accessible by name
        let _name: &EnvironmentName = &params.environment_name;
        let _instance: &InstanceName = &params.instance_name;
        let _provider: &ProviderConfig = &params.provider_config;
        let _ssh: &SshCredentials = &params.ssh_credentials;
        let _port: u16 = params.ssh_port;
        let _tracker: &TrackerConfig = &params.tracker_config;
        let _prometheus: &Option<PrometheusConfig> = &params.prometheus_config;
        let _grafana: &Option<GrafanaConfig> = &params.grafana_config;
        let _https: &Option<HttpsConfig> = &params.https_config;
    }
}
