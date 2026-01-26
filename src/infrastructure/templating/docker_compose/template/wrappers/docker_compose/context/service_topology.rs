//! Service topology for Docker Compose
//!
//! This module defines the common topology structure shared by all Docker Compose
//! service contexts. The topology contains only inter-service relationship data
//! (ports and networks), not internal service configuration.
//!
//! ## Design Rationale
//!
//! All Docker Compose service contexts (Tracker, Grafana, Prometheus, Caddy, `MySQL`)
//! share the same topology structure because:
//!
//! 1. **Topology vs Configuration separation**: The docker-compose.yml template only
//!    needs topology data (how services connect). Internal configuration (credentials,
//!    intervals) is injected via `.env` files or service-specific templates.
//!
//! 2. **Template simplicity**: Tera templates stay simple by only dealing with
//!    ports and networks. All business logic is computed in Rust.
//!
//! 3. **Explicit pattern**: This shared type makes the architectural pattern explicit
//!    and ensures consistency across all services.

// External crates
use serde::Serialize;

use crate::domain::topology::Network;

use super::port_definition::PortDefinition;

/// Service topology for Docker Compose templates
///
/// Contains the inter-service relationship data that all Docker Compose services share:
/// - **Ports**: Which ports to expose (derived from service config + TLS settings)
/// - **Networks**: Which internal networks to connect to (derived from enabled services)
///
/// This type is embedded in all `*ServiceContext` types using `#[serde(flatten)]`
/// to provide a flat JSON structure for template compatibility.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::ServiceTopology;
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::PortDefinition;
/// use torrust_tracker_deployer_lib::domain::topology::Network;
///
/// let topology = ServiceTopology {
///     ports: vec![PortDefinition::new("6969:6969/udp".to_string(), "UDP Tracker".to_string())],
///     networks: vec![Network::Metrics, Network::Database],
/// };
///
/// // Serializes to flat structure for template compatibility
/// let json = serde_json::to_value(&topology).unwrap();
/// assert!(json["ports"].is_array());
/// assert!(json["networks"].is_array());
/// ```
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct ServiceTopology {
    /// Port bindings for Docker Compose
    ///
    /// Pre-computed list of ports the service should expose.
    /// Empty when TLS is enabled (Caddy handles the ports instead).
    pub ports: Vec<PortDefinition>,
    /// Networks the service should connect to
    ///
    /// Pre-computed list of internal Docker networks based on enabled services.
    /// Examples: `metrics_network`, `database_network`, `proxy_network`.
    pub networks: Vec<Network>,
}

impl ServiceTopology {
    /// Creates a new `ServiceTopology` with the given ports and networks
    #[must_use]
    pub fn new(ports: Vec<PortDefinition>, networks: Vec<Network>) -> Self {
        Self { ports, networks }
    }

    /// Creates an empty topology with no ports or networks
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_empty_topology() {
        let topology = ServiceTopology::empty();

        assert!(topology.ports.is_empty());
        assert!(topology.networks.is_empty());
    }

    #[test]
    fn it_should_create_topology_with_ports_and_networks() {
        let ports = vec![PortDefinition::new(
            "6969:6969/udp".to_string(),
            "UDP Tracker".to_string(),
        )];
        let networks = vec![Network::Metrics, Network::Database];

        let topology = ServiceTopology::new(ports.clone(), networks.clone());

        assert_eq!(topology.ports.len(), 1);
        assert_eq!(topology.networks, networks);
    }

    #[test]
    fn it_should_serialize_to_flat_json_structure() {
        let topology = ServiceTopology {
            ports: vec![PortDefinition::new(
                "6969:6969".to_string(),
                "Test port".to_string(),
            )],
            networks: vec![Network::Metrics],
        };

        let json = serde_json::to_value(&topology).expect("serialization should succeed");

        assert!(json["ports"].is_array());
        assert!(json["networks"].is_array());
        assert_eq!(json["networks"][0], "metrics_network");
    }
}
