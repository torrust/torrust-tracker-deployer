//! Caddy service configuration for Docker Compose
//!
//! This module defines the Caddy reverse proxy service configuration
//! for the docker-compose.yml template.
//!
//! ## Note on Context Separation
//!
//! This type (`CaddyDockerServiceConfig`) is separate from the `CaddyContext` used
//! for rendering the Caddyfile.tera template. Each template has its own context:
//!
//! - `CaddyDockerServiceConfig` (this module): For docker-compose.yml service definition
//! - `CaddyContext` (in caddy/template/wrapper): For Caddyfile content with domains/ports
//!
//! The docker-compose template only needs to know that Caddy is enabled
//! (for network/volume definitions), not the detailed service configurations.

use serde::Serialize;

use crate::domain::caddy::CaddyConfig;
use crate::domain::topology::{EnabledServices, Network, NetworkDerivation, PortDerivation};

use super::port_definition::PortDefinition;

/// Caddy reverse proxy service configuration for Docker Compose
///
/// Contains configuration for the Caddy service definition in docker-compose.yml.
/// This is intentionally minimal - the actual Caddy configuration (domains, ports)
/// is in the Caddyfile, rendered separately.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::CaddyDockerServiceConfig;
/// use torrust_tracker_deployer_lib::domain::caddy::CaddyConfig;
/// use torrust_tracker_deployer_lib::domain::topology::{EnabledServices, Network};
///
/// let caddy = CaddyDockerServiceConfig::from_domain_config(&CaddyConfig::new(), &EnabledServices::default());
/// assert_eq!(caddy.networks, vec![Network::Proxy]);
/// assert_eq!(caddy.ports.len(), 3); // 80, 443, 443/udp
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CaddyDockerServiceConfig {
    /// Port bindings for Docker Compose
    ///
    /// Caddy exposes ports 80 (HTTP for ACME), 443 (HTTPS), and 443/udp (QUIC).
    pub ports: Vec<PortDefinition>,
    /// Networks this service connects to
    ///
    /// Caddy always connects to `proxy_network` for reverse proxying
    /// to backend services (tracker API, HTTP trackers, Grafana).
    pub networks: Vec<Network>,
}

impl CaddyDockerServiceConfig {
    /// Creates a new `CaddyDockerServiceConfig` from domain configuration
    ///
    /// Uses the domain `PortDerivation` and `NetworkDerivation` traits,
    /// ensuring business rules live in the domain layer.
    ///
    /// # Arguments
    ///
    /// * `config` - The domain Caddy configuration
    /// * `enabled_services` - Topology context with information about enabled services
    #[must_use]
    pub fn from_domain_config(config: &CaddyConfig, enabled_services: &EnabledServices) -> Self {
        let port_bindings = config.derive_ports();
        let ports = port_bindings.iter().map(PortDefinition::from).collect();
        let networks = config.derive_networks(enabled_services);

        Self { ports, networks }
    }

    /// Creates a new `CaddyDockerServiceConfig` with default configuration
    ///
    /// Convenience method that creates a default `CaddyConfig` and empty enabled services.
    #[must_use]
    pub fn new() -> Self {
        Self::from_domain_config(&CaddyConfig::new(), &EnabledServices::default())
    }
}

impl Default for CaddyDockerServiceConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for backward compatibility
pub type CaddyServiceConfig = CaddyDockerServiceConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_connect_caddy_to_proxy_network() {
        let caddy = CaddyDockerServiceConfig::new();

        assert_eq!(caddy.networks, vec![Network::Proxy]);
    }

    #[test]
    fn it_should_implement_default() {
        let caddy = CaddyDockerServiceConfig::default();

        assert_eq!(caddy.networks, vec![Network::Proxy]);
    }

    #[test]
    fn it_should_serialize_network_to_name_string() {
        let caddy = CaddyDockerServiceConfig::new();

        let json = serde_json::to_value(&caddy).expect("serialization should succeed");

        // Network serializes to its name string for template compatibility
        assert_eq!(json["networks"][0], "proxy_network");
    }

    #[test]
    fn it_should_expose_three_ports() {
        let caddy = CaddyDockerServiceConfig::new();

        assert_eq!(caddy.ports.len(), 3);
    }

    #[test]
    fn it_should_serialize_ports_with_binding_and_description() {
        let caddy = CaddyDockerServiceConfig::new();

        let json = serde_json::to_value(&caddy).expect("serialization should succeed");

        // Each port has binding and description fields
        assert!(json["ports"][0]["binding"].is_string());
        assert!(json["ports"][0]["description"].is_string());
    }

    #[test]
    fn it_should_use_domain_traits_for_port_derivation() {
        let config = CaddyConfig::new();
        let enabled_services = EnabledServices::default();
        let caddy = CaddyDockerServiceConfig::from_domain_config(&config, &enabled_services);

        // Verify ports come from domain trait
        assert_eq!(caddy.ports.len(), 3);
    }

    #[test]
    fn it_should_use_domain_traits_for_network_derivation() {
        let config = CaddyConfig::new();
        let enabled_services = EnabledServices::default();
        let caddy = CaddyDockerServiceConfig::from_domain_config(&config, &enabled_services);

        // Verify networks come from domain trait
        assert_eq!(caddy.networks, vec![Network::Proxy]);
    }
}
