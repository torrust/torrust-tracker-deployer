//! DTO to Domain Conversion for Environment Parameters
//!
//! This module provides the `TryFrom` implementation that converts
//! `EnvironmentCreationConfig` (a DTO) to `EnvironmentParams` (a domain type).
//!
//! # Architecture
//!
//! The conversion lives in the Application layer because it references the DTO,
//! but the target type (`EnvironmentParams`) is a pure domain value object.
//!
//! ```text
//! EnvironmentCreationConfig (DTO - Application Layer)
//!         │
//!         │ TryFrom (this module - Application Layer)
//!         ▼
//! EnvironmentParams (Domain Value Object)
//!         │
//!         │ Environment::create(params, working_dir, timestamp)
//!         ▼
//! Environment<Created> (Domain Aggregate)
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::convert::TryInto;
//! use torrust_tracker_deployer_lib::application::command_handlers::create::config::EnvironmentCreationConfig;
//! use torrust_tracker_deployer_lib::domain::environment::EnvironmentParams;
//!
//! let config: EnvironmentCreationConfig = // ... load from JSON
//! # todo!();
//! let params: EnvironmentParams = config.try_into()?;
//!
//! // Access validated domain objects by name
//! println!("Environment: {}", params.environment_name.as_str());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! See `docs/decisions/tryfrom-for-dto-to-domain-conversion.md` for rationale.

use std::convert::TryFrom;
use std::convert::TryInto;

use crate::domain::environment::EnvironmentParams;
use crate::domain::https::HttpsConfig;
use crate::domain::{EnvironmentName, InstanceName};

use super::errors::CreateConfigError;
use super::EnvironmentCreationConfig;

impl TryFrom<EnvironmentCreationConfig> for EnvironmentParams {
    type Error = CreateConfigError;

    /// Converts DTO configuration to validated domain parameters
    ///
    /// This performs all validation and type conversion from string-based
    /// DTO fields to strongly-typed domain objects.
    ///
    /// # Validation
    ///
    /// - Environment name must follow naming rules
    /// - Instance name (if provided) must follow instance naming rules
    /// - Provider config must be valid (e.g., valid profile name for LXD)
    /// - SSH username must follow Linux username requirements
    /// - SSH key files must exist and be accessible
    /// - SSH key paths must be absolute
    ///
    /// # Instance Name Auto-Generation
    ///
    /// If `instance_name` is not provided in the configuration, it will be
    /// auto-generated using the format: `torrust-tracker-vm-{env_name}`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError` if any validation fails. All error variants
    /// implement `.help()` with detailed troubleshooting guidance.
    fn try_from(config: EnvironmentCreationConfig) -> Result<Self, Self::Error> {
        // Convert environment name string to domain type
        let environment_name = EnvironmentName::new(&config.environment.name)?;

        // Instance name: use provided or auto-generate from environment name
        let instance_name = match &config.environment.instance_name {
            Some(name_str) => InstanceName::new(name_str.clone()).map_err(|e| {
                CreateConfigError::InvalidInstanceName {
                    name: name_str.clone(),
                    reason: e.to_string(),
                }
            })?,
            None => generate_instance_name(&environment_name),
        };

        // Convert ProviderSection (DTO) to domain ProviderConfig
        let provider_config = config.provider.try_into()?;

        // Get SSH port before consuming ssh_credentials
        let ssh_port = config.ssh_credentials.port;

        // Convert SSH credentials config to domain type
        let ssh_credentials = config.ssh_credentials.try_into()?;

        // Convert TrackerSection (DTO) to domain TrackerConfig
        let tracker_config = config.tracker.try_into()?;

        // Convert Prometheus and Grafana sections to domain types
        let prometheus_config = config.prometheus.map(TryInto::try_into).transpose()?;
        let grafana_config = config.grafana.map(TryInto::try_into).transpose()?;

        // Convert HTTPS section to domain type with email validation
        let https_config = config
            .https
            .map(|section| HttpsConfig::new(section.admin_email, section.use_staging))
            .transpose()?;

        Ok(EnvironmentParams::new(
            environment_name,
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker_config,
            prometheus_config,
            grafana_config,
            https_config,
        ))
    }
}

/// Generates an instance name from the environment name
///
/// Format: `torrust-tracker-vm-{env_name}`
///
/// # Panics
///
/// This function does not panic. The generated instance name is guaranteed
/// to be valid for any valid environment name.
fn generate_instance_name(env_name: &EnvironmentName) -> InstanceName {
    let instance_name_str = format!("torrust-tracker-vm-{}", env_name.as_str());
    InstanceName::new(instance_name_str)
        .expect("Generated instance name should always be valid for valid environment names")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::command_handlers::create::config::provider::LxdProviderSection;
    use crate::application::command_handlers::create::config::tracker::TrackerSection;
    use crate::application::command_handlers::create::config::{
        EnvironmentSection, ProviderSection, SshCredentialsConfig,
    };

    /// Helper to create a valid configuration for testing
    fn valid_config() -> EnvironmentCreationConfig {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "test-env".to_string(),
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            ProviderSection::Lxd(LxdProviderSection {
                profile_name: "lxd-test-env".to_string(),
            }),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn it_should_convert_valid_config_to_environment_params() {
        let config = valid_config();
        let result: Result<EnvironmentParams, _> = config.try_into();

        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.environment_name.as_str(), "test-env");
        assert_eq!(params.instance_name.as_str(), "torrust-tracker-vm-test-env");
        assert_eq!(params.ssh_port, 22);
    }

    #[test]
    fn it_should_use_custom_instance_name_when_provided() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "my-env".to_string(),
                instance_name: Some("custom-vm-name".to_string()),
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            ProviderSection::Lxd(LxdProviderSection {
                profile_name: "lxd-my-env".to_string(),
            }),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let params: EnvironmentParams = config.try_into().unwrap();
        assert_eq!(params.instance_name.as_str(), "custom-vm-name");
    }

    #[test]
    fn it_should_reject_invalid_environment_name() {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config = EnvironmentCreationConfig::new(
            EnvironmentSection {
                name: "INVALID_NAME".to_string(), // uppercase not allowed
                instance_name: None,
            },
            SshCredentialsConfig::new(private_key_path, public_key_path, "torrust".to_string(), 22),
            ProviderSection::Lxd(LxdProviderSection {
                profile_name: "lxd-test".to_string(),
            }),
            TrackerSection::default(),
            None,
            None,
            None,
            None,
        );

        let result: Result<EnvironmentParams, CreateConfigError> = config.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn it_should_provide_named_field_access() {
        use crate::adapters::ssh::SshCredentials;
        use crate::domain::grafana::GrafanaConfig;
        use crate::domain::https::HttpsConfig;
        use crate::domain::prometheus::PrometheusConfig;
        use crate::domain::provider::ProviderConfig;
        use crate::domain::tracker::TrackerConfig;

        let config = valid_config();
        let params: EnvironmentParams = config.try_into().unwrap();

        // All fields are accessible by name (not by position)
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
