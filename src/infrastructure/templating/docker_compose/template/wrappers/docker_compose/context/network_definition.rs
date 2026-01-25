//! Network definition for Docker Compose template context
//!
//! This module contains the [`NetworkDefinition`] type used for the global
//! `networks:` section in docker-compose.yml.

use serde::Serialize;

use crate::domain::topology::Network;

/// A network definition for the global `networks:` section
///
/// This type is used in the template context to render the networks section.
/// It contains the network name and driver as strings for direct template use.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::topology::Network;
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::NetworkDefinition;
///
/// let definition = NetworkDefinition::from(Network::Database);
/// assert_eq!(definition.name(), "database_network");
/// assert_eq!(definition.driver(), "bridge");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetworkDefinition {
    /// The network name as used in docker-compose.yml
    name: String,
    /// The Docker network driver (e.g., "bridge")
    driver: String,
}

impl NetworkDefinition {
    /// Returns the network name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the network driver
    #[must_use]
    pub fn driver(&self) -> &str {
        &self.driver
    }
}

impl From<Network> for NetworkDefinition {
    fn from(network: Network) -> Self {
        Self {
            name: network.name().to_string(),
            driver: network.driver().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_definition_from_database_network() {
        let definition = NetworkDefinition::from(Network::Database);

        assert_eq!(definition.name(), "database_network");
        assert_eq!(definition.driver(), "bridge");
    }

    #[test]
    fn it_should_create_definition_from_metrics_network() {
        let definition = NetworkDefinition::from(Network::Metrics);

        assert_eq!(definition.name(), "metrics_network");
        assert_eq!(definition.driver(), "bridge");
    }

    #[test]
    fn it_should_create_definition_from_visualization_network() {
        let definition = NetworkDefinition::from(Network::Visualization);

        assert_eq!(definition.name(), "visualization_network");
        assert_eq!(definition.driver(), "bridge");
    }

    #[test]
    fn it_should_create_definition_from_proxy_network() {
        let definition = NetworkDefinition::from(Network::Proxy);

        assert_eq!(definition.name(), "proxy_network");
        assert_eq!(definition.driver(), "bridge");
    }

    #[test]
    fn it_should_serialize_network_definition() {
        let definition = NetworkDefinition::from(Network::Database);

        let json = serde_json::to_string(&definition).unwrap();
        assert!(json.contains("\"name\":\"database_network\""));
        assert!(json.contains("\"driver\":\"bridge\""));
    }
}
