//! `MySQL` service configuration for Docker Compose
//!
//! This module defines the `MySQL` service configuration for the docker-compose.yml template.
//!
//! ## Note on Configuration Separation
//!
//! There are multiple `MySQL`-related types:
//!
//! - `MysqlSetupConfig` (in database.rs): Contains credentials and initialization settings
//!   for Docker Compose environment variables (root password, database name, user, etc.)
//!
//! - `MysqlServiceContext` (this module): Contains service definition settings like networks,
//!   following the same pattern as `CaddyDockerServiceConfig`, `PrometheusServiceConfig`, etc.
//!
//! - `domain::mysql::MysqlServiceConfig`: Domain configuration for port/network derivation
//!
//! This separation keeps the pattern consistent across all services - each service
//! has its own config type for networks and service-specific settings.

use serde::Serialize;

use crate::domain::mysql::MysqlServiceConfig as DomainMysqlConfig;
use crate::domain::topology::{EnabledServices, Network, NetworkDerivation, PortDerivation};

use super::port_definition::PortDefinition;

/// `MySQL` service configuration for Docker Compose
///
/// Contains configuration for the `MySQL` service definition in docker-compose.yml.
/// This is intentionally minimal - the actual `MySQL` setup configuration (credentials)
/// is in `MysqlSetupConfig`.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::MysqlServiceContext;
/// use torrust_tracker_deployer_lib::domain::mysql::MysqlServiceConfig;
/// use torrust_tracker_deployer_lib::domain::topology::EnabledServices;
///
/// let mysql = MysqlServiceContext::from_domain_config(&MysqlServiceConfig::new(), &EnabledServices::default());
/// assert_eq!(mysql.networks, vec![torrust_tracker_deployer_lib::domain::topology::Network::Database]);
/// assert!(mysql.ports.is_empty()); // MySQL never exposes ports externally
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MysqlServiceContext {
    /// Port bindings for Docker Compose
    ///
    /// `MySQL` never exposes ports externally - only tracker can access it via internal network.
    pub ports: Vec<PortDefinition>,
    /// Networks this service connects to
    ///
    /// `MySQL` only connects to `database_network` for isolation.
    /// Only the tracker can access `MySQL` through this network.
    pub networks: Vec<Network>,
}

impl MysqlServiceContext {
    /// Creates a new `MysqlServiceContext` from domain configuration
    ///
    /// Uses the domain `PortDerivation` and `NetworkDerivation` traits,
    /// ensuring business rules live in the domain layer.
    ///
    /// # Arguments
    ///
    /// * `config` - The domain `MySQL` service configuration
    /// * `enabled_services` - Topology context with information about enabled services
    #[must_use]
    pub fn from_domain_config(
        config: &DomainMysqlConfig,
        enabled_services: &EnabledServices,
    ) -> Self {
        let port_bindings = config.derive_ports();
        let ports = port_bindings.iter().map(PortDefinition::from).collect();
        let networks = config.derive_networks(enabled_services);

        Self { ports, networks }
    }

    /// Creates a new `MysqlServiceContext` with default configuration
    ///
    /// Convenience method that creates a default domain config and empty enabled services.
    #[must_use]
    pub fn new() -> Self {
        Self::from_domain_config(&DomainMysqlConfig::new(), &EnabledServices::default())
    }
}

impl Default for MysqlServiceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_connect_mysql_to_database_network() {
        let mysql = MysqlServiceContext::new();

        assert_eq!(mysql.networks, vec![Network::Database]);
    }

    #[test]
    fn it_should_implement_default() {
        let mysql = MysqlServiceContext::default();

        assert_eq!(mysql.networks, vec![Network::Database]);
    }

    #[test]
    fn it_should_serialize_network_to_name_string() {
        let mysql = MysqlServiceContext::new();

        let json = serde_json::to_value(&mysql).expect("serialization should succeed");

        // Network serializes to its name string for template compatibility
        assert_eq!(json["networks"][0], "database_network");
    }

    #[test]
    fn it_should_not_expose_any_ports() {
        let mysql = MysqlServiceContext::new();

        assert!(mysql.ports.is_empty());
    }

    #[test]
    fn it_should_use_domain_traits_for_port_derivation() {
        let config = DomainMysqlConfig::new();
        let enabled_services = EnabledServices::default();
        let mysql = MysqlServiceContext::from_domain_config(&config, &enabled_services);

        // Verify ports come from domain trait (empty for MySQL)
        assert!(mysql.ports.is_empty());
    }

    #[test]
    fn it_should_use_domain_traits_for_network_derivation() {
        let config = DomainMysqlConfig::new();
        let enabled_services = EnabledServices::default();
        let mysql = MysqlServiceContext::from_domain_config(&config, &enabled_services);

        // Verify networks come from domain trait
        assert_eq!(mysql.networks, vec![Network::Database]);
    }
}
