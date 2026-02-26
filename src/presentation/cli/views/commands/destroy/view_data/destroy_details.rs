//! Destroy Details Data Transfer Object
//!
//! This module contains the presentation DTO for destroy command details.
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

use crate::domain::environment::state::Destroyed;
use crate::domain::environment::Environment;

/// Destroy details data for rendering
///
/// This struct holds all the data needed to render destroy command
/// information for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples domain models from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
///
/// # Note on `instance_ip`
///
/// `instance_ip` may be `None` (rendered as `null` in JSON) because `destroy`
/// accepts environments in any state, including those that were never provisioned.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DestroyDetailsData {
    /// Name of the destroyed environment
    pub environment_name: String,
    /// Name of the destroyed instance
    pub instance_name: String,
    /// Infrastructure provider (lowercase: "lxd", "hetzner", etc.)
    pub provider: String,
    /// State name (always "Destroyed" for this command)
    pub state: String,
    /// IP address of the instance, or `None` if never provisioned
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
impl From<&Environment<Destroyed>> for DestroyDetailsData {
    fn from(env: &Environment<Destroyed>) -> Self {
        Self {
            environment_name: env.name().as_str().to_string(),
            instance_name: env.instance_name().as_str().to_string(),
            provider: env.provider_config().provider_name().to_string(),
            state: "Destroyed".to_string(),
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

    fn create_destroyed_environment_with_ip(ip: IpAddr) -> Environment<Destroyed> {
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
            .start_destroying()
            .destroyed()
    }

    fn create_destroyed_environment_without_ip() -> Environment<Destroyed> {
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let ssh_credentials = create_test_ssh_credentials();
        let provider_config = create_test_provider_config();
        let created_at = create_test_timestamp();

        // Destroy from Created state — never provisioned, so no IP
        Environment::new(env_name, provider_config, ssh_credentials, 22, created_at)
            .start_destroying()
            .destroyed()
    }

    fn create_test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))
    }

    // Tests

    #[test]
    fn it_should_convert_destroyed_environment_with_ip_to_dto() {
        // Arrange
        let test_ip = create_test_ip();
        let env = create_destroyed_environment_with_ip(test_ip);

        // Act
        let dto = DestroyDetailsData::from(&env);

        // Assert
        assert_eq!(dto.environment_name, "test-env");
        assert_eq!(dto.instance_name, "torrust-tracker-vm-test-env");
        assert_eq!(dto.provider, "lxd");
        assert_eq!(dto.state, "Destroyed");
        assert_eq!(dto.instance_ip, Some(test_ip));
        assert_eq!(dto.created_at, create_test_timestamp());
    }

    #[test]
    fn it_should_convert_destroyed_environment_without_ip_to_dto() {
        // Arrange
        let env = create_destroyed_environment_without_ip();

        // Act
        let dto = DestroyDetailsData::from(&env);

        // Assert
        assert_eq!(dto.instance_ip, None);
        assert_eq!(dto.state, "Destroyed");
    }

    #[test]
    fn it_should_have_destroyed_state_string() {
        // Arrange
        let env = create_destroyed_environment_with_ip(create_test_ip());

        // Act
        let dto = DestroyDetailsData::from(&env);

        // Assert
        assert_eq!(dto.state, "Destroyed");
    }
}
