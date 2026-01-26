//! `MySQL` database service configuration domain type
//!
//! This module defines the `MySQL` service configuration domain type which implements
//! the `PortDerivation` and `NetworkDerivation` traits following the same
//! pattern as other services.
//!
//! ## Note on Type Names
//!
//! - `domain::mysql::MysqlServiceConfig` (this module): Docker service configuration
//!   for port/network derivation in Docker Compose topology
//! - `domain::tracker::MysqlConfig`: Database connection settings for the tracker
//!   (host, port, credentials)
//!
//! ## Port Rules Reference
//!
//! | Rule    | Description                               |
//! |---------|-------------------------------------------|
//! | PORT-11 | `MySQL` never exposes ports externally    |
//!
//! ## Network Rules Reference
//!
//! | Rule   | Description                                 |
//! |--------|---------------------------------------------|
//! | NET-08 | `MySQL` always connects to Database network |

use serde::{Deserialize, Serialize};

use crate::domain::topology::{
    EnabledServices, Network, NetworkDerivation, PortBinding, PortDerivation,
};

/// `MySQL` database service configuration for Docker Compose topology
///
/// `MySQL` is a special service with fixed behavior:
/// - Never exposes ports externally (security - internal only)
/// - Always connects to the Database network
///
/// Unlike other services, `MySQL` doesn't have user-configurable port behavior,
/// but it still implements `PortDerivation` for consistency.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::mysql::MysqlServiceConfig;
/// use torrust_tracker_deployer_lib::domain::topology::PortDerivation;
///
/// let config = MysqlServiceConfig::new();
/// let ports = config.derive_ports();
/// assert!(ports.is_empty()); // MySQL never exposes ports
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MysqlServiceConfig {
    // MySQL has no configurable fields for port/network derivation.
    // This is intentionally empty - the behavior is fixed.
    // Note: Database credentials are in domain::tracker::MysqlConfig.
}

impl MysqlServiceConfig {
    /// Creates a new `MysqlServiceConfig`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::mysql::MysqlServiceConfig;
    ///
    /// let config = MysqlServiceConfig::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl PortDerivation for MysqlServiceConfig {
    /// Derives port bindings for the `MySQL` database service
    ///
    /// Implements PORT-11: `MySQL` has no exposed ports
    ///
    /// `MySQL` is accessed only via Docker network by the tracker service.
    /// It should never be exposed to the host network for security reasons.
    fn derive_ports(&self) -> Vec<PortBinding> {
        vec![]
    }
}

impl NetworkDerivation for MysqlServiceConfig {
    /// Derives network assignments for the `MySQL` service
    ///
    /// Implements NET-08: `MySQL` always connects to Database network only
    fn derive_networks(&self, _enabled_services: &EnabledServices) -> Vec<Network> {
        vec![Network::Database]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::topology::Service;

    // =========================================================================
    // Constructor tests
    // =========================================================================

    mod constructor {
        use super::*;

        #[test]
        fn it_should_create_mysql_service_config() {
            let config = MysqlServiceConfig::new();
            assert_eq!(config, MysqlServiceConfig::default());
        }

        #[test]
        fn it_should_implement_default() {
            let config = MysqlServiceConfig::default();
            assert_eq!(config, MysqlServiceConfig::new());
        }
    }

    // =========================================================================
    // PortDerivation tests (PORT-11)
    // =========================================================================

    mod port_derivation {
        use super::*;

        #[test]
        fn it_should_expose_no_ports() {
            let config = MysqlServiceConfig::new();
            let ports = config.derive_ports();

            assert!(ports.is_empty());
        }
    }

    // =========================================================================
    // NetworkDerivation tests (NET-08)
    // =========================================================================

    mod network_derivation {
        use super::*;

        #[test]
        fn it_should_connect_to_database_network() {
            let config = MysqlServiceConfig::new();
            let enabled = EnabledServices::from(&[]);
            let networks = config.derive_networks(&enabled);

            assert_eq!(networks, vec![Network::Database]);
        }

        #[test]
        fn it_should_connect_only_to_database_network_regardless_of_enabled_services() {
            let config = MysqlServiceConfig::new();
            let enabled = EnabledServices::from(&[
                Service::Tracker,
                Service::Prometheus,
                Service::Grafana,
                Service::Caddy,
            ]);
            let networks = config.derive_networks(&enabled);

            // NET-08: MySQL only connects to Database network
            assert_eq!(networks, vec![Network::Database]);
        }
    }
}
