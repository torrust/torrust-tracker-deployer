//! Port definition for Docker Compose template rendering
//!
//! This module provides the `PortDefinition` type that bridges the domain
//! `PortBinding` type with the Tera template context.

use serde::Serialize;

use crate::domain::topology::PortBinding;

/// A port definition for Docker Compose template rendering
///
/// This type is serialized to the Tera template context and provides:
/// - `binding`: The Docker Compose port string (e.g., "6969:6969/udp")
/// - `description`: A human-readable description for YAML comments
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::topology::PortBinding;
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::PortDefinition;
///
/// let port_binding = PortBinding::udp(6969, "BitTorrent UDP announce");
/// let definition = PortDefinition::from(&port_binding);
///
/// assert_eq!(definition.binding(), "6969:6969/udp");
/// assert_eq!(definition.description(), "BitTorrent UDP announce");
/// ```
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PortDefinition {
    /// Docker Compose port binding string (e.g., "6969:6969/udp")
    binding: String,
    /// Human-readable description for YAML comments
    description: String,
}

impl PortDefinition {
    /// Creates a new port definition
    #[must_use]
    pub fn new(binding: String, description: String) -> Self {
        Self {
            binding,
            description,
        }
    }

    /// Returns the Docker Compose port binding string
    #[must_use]
    pub fn binding(&self) -> &str {
        &self.binding
    }

    /// Returns the description for YAML comments
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl From<&PortBinding> for PortDefinition {
    fn from(port: &PortBinding) -> Self {
        Self {
            binding: port.docker_compose_binding(),
            description: port.description().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_udp_port_binding_to_definition() {
        let binding = PortBinding::udp(6969, "BitTorrent UDP announce");

        let definition = PortDefinition::from(&binding);

        assert_eq!(definition.binding(), "6969:6969/udp");
        assert_eq!(definition.description(), "BitTorrent UDP announce");
    }

    #[test]
    fn it_should_convert_tcp_port_binding_to_definition() {
        let binding = PortBinding::tcp(7070, "HTTP tracker");

        let definition = PortDefinition::from(&binding);

        assert_eq!(definition.binding(), "7070:7070");
        assert_eq!(definition.description(), "HTTP tracker");
    }

    #[test]
    fn it_should_convert_localhost_binding_to_definition() {
        let binding = PortBinding::localhost_tcp(9090, "Prometheus");

        let definition = PortDefinition::from(&binding);

        assert_eq!(definition.binding(), "127.0.0.1:9090:9090");
        assert_eq!(definition.description(), "Prometheus");
    }

    #[test]
    fn it_should_convert_multiple_bindings_using_from_trait() {
        let bindings = [PortBinding::udp(6969, "UDP"), PortBinding::tcp(7070, "TCP")];

        // Use From trait directly - idiomatic Rust pattern
        let definitions: Vec<PortDefinition> = bindings.iter().map(PortDefinition::from).collect();

        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0].binding(), "6969:6969/udp");
        assert_eq!(definitions[1].binding(), "7070:7070");
    }

    #[test]
    fn it_should_serialize_to_json_for_template() {
        let definition =
            PortDefinition::new("6969:6969/udp".to_string(), "BitTorrent UDP".to_string());

        let json = serde_json::to_value(&definition).expect("serialization should succeed");

        assert_eq!(json["binding"], "6969:6969/udp");
        assert_eq!(json["description"], "BitTorrent UDP");
    }
}
