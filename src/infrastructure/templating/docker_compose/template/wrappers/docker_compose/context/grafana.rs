//! Grafana service configuration for Docker Compose

// External crates
use serde::Serialize;

use crate::domain::topology::Network;
use crate::shared::secrets::Password;

use super::port_definition::PortDefinition;
use super::port_derivation::derive_grafana_ports;

/// Grafana service configuration for Docker Compose
///
/// Contains all configuration needed for the Grafana service in Docker Compose,
/// including admin credentials, TLS settings, port mappings, and network connections.
/// All logic is pre-computed in Rust to keep the Tera template simple.
#[derive(Serialize, Debug, Clone)]
pub struct GrafanaServiceConfig {
    /// Grafana admin username
    pub admin_user: String,
    /// Grafana admin password
    pub admin_password: Password,
    /// Whether Grafana has TLS enabled (port should not be exposed if true)
    #[serde(default)]
    pub has_tls: bool,
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

impl GrafanaServiceConfig {
    /// Creates a new `GrafanaServiceConfig` with pre-computed networks and ports
    ///
    /// # Arguments
    ///
    /// * `admin_user` - Grafana admin username
    /// * `admin_password` - Grafana admin password
    /// * `has_tls` - Whether Grafana has TLS enabled (via Caddy)
    /// * `has_caddy` - Whether Caddy TLS proxy is enabled (adds `proxy_network`)
    #[must_use]
    pub fn new(
        admin_user: String,
        admin_password: Password,
        has_tls: bool,
        has_caddy: bool,
    ) -> Self {
        let networks = Self::compute_networks(has_caddy);
        let port_bindings = derive_grafana_ports(has_tls);
        let ports = port_bindings.iter().map(PortDefinition::from).collect();

        Self {
            admin_user,
            admin_password,
            has_tls,
            ports,
            networks,
        }
    }

    /// Computes the list of networks for the Grafana service
    fn compute_networks(has_caddy: bool) -> Vec<Network> {
        let mut networks = vec![Network::Visualization];

        if has_caddy {
            networks.push(Network::Proxy);
        }

        networks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_connect_grafana_to_visualization_network() {
        let config =
            GrafanaServiceConfig::new("admin".to_string(), Password::new("password"), false, false);

        assert!(config.networks.contains(&Network::Visualization));
    }

    #[test]
    fn it_should_not_connect_grafana_to_proxy_network_when_caddy_disabled() {
        let config =
            GrafanaServiceConfig::new("admin".to_string(), Password::new("password"), false, false);

        assert_eq!(config.networks, vec![Network::Visualization]);
        assert!(!config.networks.contains(&Network::Proxy));
    }

    #[test]
    fn it_should_connect_grafana_to_proxy_network_when_caddy_enabled() {
        let config =
            GrafanaServiceConfig::new("admin".to_string(), Password::new("password"), true, true);

        assert_eq!(
            config.networks,
            vec![Network::Visualization, Network::Proxy]
        );
    }

    #[test]
    fn it_should_serialize_networks_to_name_strings() {
        let config =
            GrafanaServiceConfig::new("admin".to_string(), Password::new("password"), true, true);

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        // Networks serialize to their name strings for template compatibility
        assert_eq!(json["networks"][0], "visualization_network");
        assert_eq!(json["networks"][1], "proxy_network");
    }

    #[test]
    fn it_should_expose_port_3000_when_tls_disabled() {
        let config =
            GrafanaServiceConfig::new("admin".to_string(), Password::new("password"), false, false);

        assert_eq!(config.ports.len(), 1);
        assert_eq!(config.ports[0].binding(), "3000:3000");
    }

    #[test]
    fn it_should_not_expose_ports_when_tls_enabled() {
        let config =
            GrafanaServiceConfig::new("admin".to_string(), Password::new("password"), true, true);

        assert!(config.ports.is_empty());
    }
}
