//! Release Details Data Transfer Object
//!
//! This module contains the presentation DTO for release command details.
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

use crate::domain::environment::state::Released;
use crate::domain::environment::Environment;

/// Release details data for rendering
///
/// This struct holds all the data needed to render release command
/// information for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples domain models from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReleaseDetailsData {
    /// Name of the released environment
    pub environment_name: String,
    /// Name of the released instance
    pub instance_name: String,
    /// Infrastructure provider (lowercase: "lxd", "hetzner", etc.)
    pub provider: String,
    /// State name (always "Released" for this command)
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
/// - Adding method to `Environment<Released>`: Would violate DDD by making
///   domain depend on presentation DTOs
/// - Keeping mapping in controller: Works but less idiomatic than `From` trait
impl From<&Environment<Released>> for ReleaseDetailsData {
    fn from(env: &Environment<Released>) -> Self {
        Self {
            environment_name: env.name().as_str().to_string(),
            instance_name: env.instance_name().as_str().to_string(),
            provider: env.provider_config().provider_name().to_string(),
            state: "Released".to_string(),
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
    use chrono::{DateTime, TimeZone, Utc};
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    // Test fixtures and helpers

    fn create_test_ssh_credentials() -> SshCredentials {
        let ssh_username = Username::new("deployer".to_string()).unwrap();
        SshCredentials::new(
            PathBuf::from("./keys/test_rsa"),
            PathBuf::from("./keys/test_rsa.pub"),
            ssh_username,
        )
    }

    fn create_test_provider_config() -> ProviderConfig {
        ProviderConfig::Lxd(LxdConfig {
            profile_name: ProfileName::new("lxd-test-env".to_string()).unwrap(),
        })
    }

    fn create_test_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap()
    }

    fn create_released_environment_with_ip(ip: IpAddr) -> Environment<Released> {
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let ssh_credentials = create_test_ssh_credentials();
        let provider_config = create_test_provider_config();
        let created_at = create_test_timestamp();

        Environment::new(env_name, provider_config, ssh_credentials, 22, created_at)
            .start_provisioning()
            .provisioned(ip, ProvisionMethod::Provisioned)
            .start_configuring()
            .configured()
            .start_releasing()
            .released()
    }

    fn create_test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))
    }

    fn create_expected_dto(ip: IpAddr) -> ReleaseDetailsData {
        ReleaseDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            provider: "lxd".to_string(),
            state: "Released".to_string(),
            instance_ip: Some(ip),
            created_at: create_test_timestamp(),
        }
    }

    // Tests

    #[test]
    fn it_should_convert_released_environment_to_dto() {
        // Arrange
        let test_ip = create_test_ip();
        let env = create_released_environment_with_ip(test_ip);
        let expected = create_expected_dto(test_ip);

        // Act
        let dto = ReleaseDetailsData::from(&env);

        // Assert
        assert_eq!(dto, expected);
    }

    #[test]
    fn it_should_have_released_state_string() {
        // Arrange
        let env = create_released_environment_with_ip(create_test_ip());

        // Act
        let dto = ReleaseDetailsData::from(&env);

        // Assert
        assert_eq!(dto.state, "Released");
    }

    #[test]
    fn it_should_have_instance_ip_present_for_provisioned_environment() {
        // Arrange - create environment with IP
        let env = create_released_environment_with_ip(create_test_ip());

        // Act
        let dto = ReleaseDetailsData::from(&env);

        // Assert - in a real released environment, IP should always be present
        // This test documents that released environments have IP addresses
        assert!(dto.instance_ip.is_some());
    }
}
