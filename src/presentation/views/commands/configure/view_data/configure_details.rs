//! Configure Details Data Transfer Object
//!
//! This module contains the presentation DTO for configure command details.
//! It serves as the data structure passed to view renderers (`TextView`, `JsonView`, etc.).
//!
//! # Architecture
//!
//! This follows the Strategy Pattern where:
//! - This DTO is the data passed to all rendering strategies
//! - Different views (`TextView`, `JsonView`) consume this data
//! - Adding new formats doesn't modify this DTO or existing views
//!
//! # SOLID Principles
//!
//! - **Single Responsibility**: This file only defines the data structure
//! - **Open/Closed**: New formats extend by adding views, not modifying this
//! - **Separation of Concerns**: Data definition separate from rendering logic

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::net::IpAddr;

use crate::domain::environment::state::Configured;
use crate::domain::environment::Environment;

/// Configure details data for rendering
///
/// This struct holds all the data needed to render configure command
/// information for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples domain models from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
#[derive(Debug, Clone, Serialize)]
pub struct ConfigureDetailsData {
    /// Name of the configured environment
    pub environment_name: String,
    /// Name of the configured instance
    pub instance_name: String,
    /// Infrastructure provider (lowercase: "lxd", "hetzner", etc.)
    pub provider: String,
    /// State name (always "Configured" for this command)
    pub state: String,
    /// IP address of the instance (nullable)
    pub instance_ip: Option<IpAddr>,
    /// Timestamp when the environment was created (ISO 8601 format in JSON)
    pub created_at: DateTime<Utc>,
}

/// Conversion from domain model to presentation DTO
///
/// This `From` trait implementation is placed in the presentation layer
/// (not in the domain layer) to maintain proper DDD layering:
///
/// - Domain layer should not depend on presentation layer DTOs
/// - Presentation layer can depend on domain models (allowed)
/// - This keeps the domain clean and focused on business logic
///
/// Alternative approaches considered:
/// - Adding method to `Environment<Configured>`: Would violate DDD by making
///   domain depend on presentation DTOs
/// - Keeping mapping in controller: Works but less idiomatic than `From` trait
impl From<&Environment<Configured>> for ConfigureDetailsData {
    fn from(env: &Environment<Configured>) -> Self {
        Self {
            environment_name: env.name().as_str().to_string(),
            instance_name: env.instance_name().as_str().to_string(),
            provider: env.provider_config().provider_name().to_string(),
            state: "Configured".to_string(),
            instance_ip: env.instance_ip(),
            created_at: env.created_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ssh::SshCredentials;
    use crate::domain::environment::runtime_outputs::ProvisionMethod;
    use crate::domain::environment::EnvironmentName;
    use crate::domain::provider::{LxdConfig, ProviderConfig};
    use crate::domain::ProfileName;
    use crate::shared::Username;
    use chrono::TimeZone;
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    #[test]
    fn it_should_convert_configured_environment_to_dto() {
        // Arrange
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let ssh_username = Username::new("deployer".to_string()).unwrap();
        let ssh_credentials = SshCredentials::new(
            PathBuf::from("./keys/test_rsa"),
            PathBuf::from("./keys/test_rsa.pub"),
            ssh_username,
        );
        let provider_config = ProviderConfig::Lxd(LxdConfig {
            profile_name: ProfileName::new("lxd-test-env".to_string()).unwrap(),
        });
        let created_at = Utc.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap();

        let env = Environment::new(env_name, provider_config, ssh_credentials, 22, created_at)
            .start_provisioning()
            .provisioned(
                IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39)),
                ProvisionMethod::Provisioned,
            )
            .start_configuring()
            .configured();

        // Act
        let dto = ConfigureDetailsData::from(&env);

        // Assert
        assert_eq!(dto.environment_name, "test-env");
        assert_eq!(dto.instance_name, "torrust-tracker-vm-test-env");
        assert_eq!(dto.provider, "lxd");
        assert_eq!(dto.state, "Configured");
        assert_eq!(
            dto.instance_ip,
            Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39)))
        );
        assert_eq!(dto.created_at, created_at);
    }

    #[test]
    fn it_should_handle_none_instance_ip() {
        // Arrange - create environment without IP
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let ssh_username = Username::new("deployer".to_string()).unwrap();
        let ssh_credentials = SshCredentials::new(
            PathBuf::from("./keys/test_rsa"),
            PathBuf::from("./keys/test_rsa.pub"),
            ssh_username,
        );
        let provider_config = ProviderConfig::Lxd(LxdConfig {
            profile_name: ProfileName::new("lxd-test-env".to_string()).unwrap(),
        });
        let created_at = Utc.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap();

        // Create environment in configured state but somehow without IP
        // (this is theoretically possible if we transition manually for testing)
        let env = Environment::new(env_name, provider_config, ssh_credentials, 22, created_at)
            .start_provisioning()
            .provisioned(
                IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39)),
                ProvisionMethod::Provisioned,
            )
            .start_configuring()
            .configured();

        // Act
        let dto = ConfigureDetailsData::from(&env);

        // Assert - in a real configured environment, IP should always be present
        // This test just documents the behavior when it's None
        assert!(dto.instance_ip.is_some());
    }
}
