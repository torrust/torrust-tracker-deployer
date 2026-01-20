//! `MySQL` service configuration for Docker Compose
//!
//! This module defines the `MySQL` service configuration for the docker-compose.yml template.
//!
//! ## Note on Configuration Separation
//!
//! There are two `MySQL`-related types in the docker-compose context:
//!
//! - `MysqlSetupConfig` (in database.rs): Contains credentials and initialization settings
//!   for Docker Compose environment variables (root password, database name, user, etc.)
//!
//! - `MysqlServiceConfig` (this module): Contains service definition settings like networks,
//!   following the same pattern as `CaddyServiceConfig`, `PrometheusServiceConfig`, etc.
//!
//! This separation keeps the pattern consistent across all services - each service
//! has its own config type for networks and service-specific settings.

use serde::Serialize;

/// Network names used by the `MySQL` service
const DATABASE_NETWORK: &str = "database_network";

/// `MySQL` service configuration for Docker Compose
///
/// Contains configuration for the `MySQL` service definition in docker-compose.yml.
/// This is intentionally minimal - the actual `MySQL` setup configuration (credentials)
/// is in `MysqlSetupConfig`.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::MysqlServiceConfig;
///
/// let mysql = MysqlServiceConfig::new();
/// assert_eq!(mysql.networks, vec!["database_network"]);
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MysqlServiceConfig {
    /// Networks this service connects to
    ///
    /// `MySQL` only connects to `database_network` for isolation.
    /// Only the tracker can access `MySQL` through this network.
    pub networks: Vec<String>,
}

impl MysqlServiceConfig {
    /// Creates a new `MysqlServiceConfig` with default networks
    ///
    /// `MySQL` connects to:
    /// - `database_network`: For database access by the tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            networks: vec![DATABASE_NETWORK.to_string()],
        }
    }
}

impl Default for MysqlServiceConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_mysql_config_with_database_network() {
        let mysql = MysqlServiceConfig::new();

        assert_eq!(mysql.networks, vec!["database_network"]);
    }

    #[test]
    fn it_should_implement_default() {
        let mysql = MysqlServiceConfig::default();

        assert_eq!(mysql.networks, vec!["database_network"]);
    }

    #[test]
    fn it_should_serialize_to_json() {
        let mysql = MysqlServiceConfig::new();

        let json = serde_json::to_value(&mysql).expect("serialization should succeed");

        assert_eq!(json["networks"][0], "database_network");
    }
}
