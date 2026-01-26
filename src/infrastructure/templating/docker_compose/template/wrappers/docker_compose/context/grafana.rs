//! Grafana service configuration for Docker Compose

// External crates
use serde::Serialize;

use crate::domain::grafana::GrafanaConfig;
use crate::domain::topology::{EnabledServices, Network, NetworkDerivation, PortDerivation};

use super::port_definition::PortDefinition;

/// Grafana service configuration for Docker Compose
///
/// Contains configuration needed for the Grafana service definition in docker-compose.yml.
/// Only includes fields actually used by the template (ports and networks).
/// Credentials are handled separately by the env context for .env template.
#[derive(Serialize, Debug, Clone)]
pub struct GrafanaServiceContext {
    /// Port bindings for Docker Compose
    ///
    /// When TLS is disabled, Grafana exposes port 3000 directly.
    /// When TLS is enabled, Caddy handles the port and this is empty.
    pub ports: Vec<PortDefinition>,
    /// Networks the Grafana service should connect to
    ///
    /// Pre-computed list based on enabled features:
    /// - Always includes `visualization_network` (queries Prometheus)
    /// - Includes `proxy_network` if Caddy TLS proxy is enabled
    pub networks: Vec<Network>,
}

impl GrafanaServiceContext {
    /// Creates a new `GrafanaServiceContext` from domain configuration
    ///
    /// Uses the domain `PortDerivation` and `NetworkDerivation` traits,
    /// ensuring business rules live in the domain layer.
    ///
    /// # Arguments
    ///
    /// * `config` - The domain Grafana configuration
    /// * `context` - Topology context with information about enabled services
    #[must_use]
    pub fn from_domain_config(config: &GrafanaConfig, enabled_services: &EnabledServices) -> Self {
        let networks = config.derive_networks(enabled_services);
        let ports = config
            .derive_ports()
            .iter()
            .map(PortDefinition::from)
            .collect();
        Self { ports, networks }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::DomainName;

    fn make_config(use_tls_proxy: bool) -> GrafanaConfig {
        if use_tls_proxy {
            GrafanaConfig::new(
                "admin".to_string(),
                "password".to_string(),
                Some(DomainName::new("grafana.example.com").unwrap()),
                true,
            )
        } else {
            GrafanaConfig::new("admin".to_string(), "password".to_string(), None, false)
        }
    }

    fn make_context(has_caddy: bool) -> EnabledServices {
        if has_caddy {
            EnabledServices::from(&[crate::domain::topology::Service::Caddy])
        } else {
            EnabledServices::from(&[])
        }
    }

    #[test]
    fn it_should_connect_grafana_to_visualization_network() {
        let context = make_context(false);
        let config = GrafanaServiceContext::from_domain_config(&make_config(false), &context);

        assert!(config.networks.contains(&Network::Visualization));
    }

    #[test]
    fn it_should_not_connect_grafana_to_proxy_network_when_caddy_disabled() {
        let context = make_context(false);
        let config = GrafanaServiceContext::from_domain_config(&make_config(false), &context);

        assert_eq!(config.networks, vec![Network::Visualization]);
        assert!(!config.networks.contains(&Network::Proxy));
    }

    #[test]
    fn it_should_connect_grafana_to_proxy_network_when_caddy_enabled() {
        let context = make_context(true);
        let config = GrafanaServiceContext::from_domain_config(&make_config(true), &context);

        assert_eq!(
            config.networks,
            vec![Network::Visualization, Network::Proxy]
        );
    }

    #[test]
    fn it_should_serialize_networks_to_name_strings() {
        let context = make_context(true);
        let config = GrafanaServiceContext::from_domain_config(&make_config(true), &context);

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        // Networks serialize to their name strings for template compatibility
        assert_eq!(json["networks"][0], "visualization_network");
        assert_eq!(json["networks"][1], "proxy_network");
    }

    #[test]
    fn it_should_expose_port_3000_when_tls_disabled() {
        let context = make_context(false);
        let config = GrafanaServiceContext::from_domain_config(&make_config(false), &context);

        assert_eq!(config.ports.len(), 1);
        assert_eq!(config.ports[0].binding(), "3000:3000");
    }

    #[test]
    fn it_should_not_expose_ports_when_tls_enabled() {
        let context = make_context(true);
        let config = GrafanaServiceContext::from_domain_config(&make_config(true), &context);

        assert!(config.ports.is_empty());
    }
}
