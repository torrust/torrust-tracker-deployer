//! Tracker service configuration for Docker Compose

// External crates
use serde::Serialize;

use crate::domain::topology::Network;

use super::port_definition::PortDefinition;
use super::port_derivation::derive_tracker_ports;

/// Tracker service configuration for Docker Compose
///
/// Contains all configuration needed for the tracker service in Docker Compose,
/// including port mappings and network connections. All logic is pre-computed
/// in Rust to keep the Tera template simple.
#[derive(Serialize, Debug, Clone)]
pub struct TrackerServiceConfig {
    /// UDP tracker ports (always exposed - UDP doesn't use TLS termination via Caddy)
    pub udp_tracker_ports: Vec<u16>,
    /// HTTP tracker ports without TLS (only these are exposed in Docker Compose)
    ///
    /// Ports with TLS enabled are handled by Caddy and NOT included here.
    pub http_tracker_ports_without_tls: Vec<u16>,
    /// HTTP API port
    pub http_api_port: u16,
    /// Whether the HTTP API has TLS enabled (port should not be exposed if true)
    #[serde(default)]
    pub http_api_has_tls: bool,
    /// Whether the tracker service needs a ports section at all
    ///
    /// Pre-computed flag: true if there are UDP ports, HTTP ports without TLS,
    /// or the API port is exposed (no TLS).
    #[serde(default)]
    pub needs_ports_section: bool,
    /// Port bindings for Docker Compose
    ///
    /// Pre-computed list of all ports the tracker should expose.
    /// Derived from UDP ports, HTTP ports without TLS, and API port (if no TLS).
    pub ports: Vec<PortDefinition>,
    /// Networks the tracker service should connect to
    ///
    /// Pre-computed list based on enabled features (prometheus, mysql, caddy).
    pub networks: Vec<Network>,
}

impl TrackerServiceConfig {
    /// Creates a new `TrackerServiceConfig` with pre-computed flags
    ///
    /// # Arguments
    ///
    /// * `udp_tracker_ports` - UDP tracker ports (always exposed)
    /// * `http_tracker_ports_without_tls` - HTTP tracker ports that don't have TLS
    /// * `http_api_port` - The HTTP API port number
    /// * `http_api_has_tls` - Whether the API uses TLS (Caddy handles it)
    /// * `has_prometheus` - Whether Prometheus is enabled (adds `metrics_network`)
    /// * `has_mysql` - Whether `MySQL` is the database driver (adds `database_network`)
    /// * `has_caddy` - Whether Caddy TLS proxy is enabled (adds `proxy_network`)
    #[must_use]
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
        udp_tracker_ports: Vec<u16>,
        http_tracker_ports_without_tls: Vec<u16>,
        http_api_port: u16,
        http_api_has_tls: bool,
        has_prometheus: bool,
        has_mysql: bool,
        has_caddy: bool,
    ) -> Self {
        let needs_ports_section = !udp_tracker_ports.is_empty()
            || !http_tracker_ports_without_tls.is_empty()
            || !http_api_has_tls;

        let networks = Self::compute_networks(has_prometheus, has_mysql, has_caddy);

        // Derive ports using the domain logic
        let port_bindings = derive_tracker_ports(
            &udp_tracker_ports,
            &http_tracker_ports_without_tls,
            http_api_port,
            http_api_has_tls,
        );
        let ports = port_bindings.iter().map(PortDefinition::from).collect();

        Self {
            udp_tracker_ports,
            http_tracker_ports_without_tls,
            http_api_port,
            http_api_has_tls,
            needs_ports_section,
            ports,
            networks,
        }
    }

    /// Computes the list of networks for the tracker service
    fn compute_networks(has_prometheus: bool, has_mysql: bool, has_caddy: bool) -> Vec<Network> {
        let mut networks = Vec::new();

        if has_prometheus {
            networks.push(Network::Metrics);
        }
        if has_mysql {
            networks.push(Network::Database);
        }
        if has_caddy {
            networks.push(Network::Proxy);
        }

        networks
    }
}

// Type alias for backward compatibility
pub type TrackerPorts = TrackerServiceConfig;

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // Network assignment tests
    // ==========================================================================

    #[test]
    fn it_should_connect_tracker_to_metrics_network_when_prometheus_enabled() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            false,
            true,  // has_prometheus
            false, // has_mysql
            false, // has_caddy
        );

        assert!(config.networks.contains(&Network::Metrics));
    }

    #[test]
    fn it_should_not_connect_tracker_to_metrics_network_when_prometheus_disabled() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            false,
            false, // has_prometheus
            false, // has_mysql
            false, // has_caddy
        );

        assert!(!config.networks.contains(&Network::Metrics));
    }

    #[test]
    fn it_should_connect_tracker_to_database_network_when_mysql_enabled() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            false,
            false, // has_prometheus
            true,  // has_mysql
            false, // has_caddy
        );

        assert!(config.networks.contains(&Network::Database));
    }

    #[test]
    fn it_should_not_connect_tracker_to_database_network_when_mysql_disabled() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            false,
            false, // has_prometheus
            false, // has_mysql (SQLite)
            false, // has_caddy
        );

        assert!(!config.networks.contains(&Network::Database));
    }

    #[test]
    fn it_should_connect_tracker_to_proxy_network_when_caddy_enabled() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            true,  // http_api_has_tls
            false, // has_prometheus
            false, // has_mysql
            true,  // has_caddy
        );

        assert!(config.networks.contains(&Network::Proxy));
    }

    #[test]
    fn it_should_not_connect_tracker_to_proxy_network_when_caddy_disabled() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            false,
            false, // has_prometheus
            false, // has_mysql
            false, // has_caddy
        );

        assert!(!config.networks.contains(&Network::Proxy));
    }

    #[test]
    fn it_should_connect_tracker_to_all_networks_when_all_services_enabled() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            true,
            true, // has_prometheus
            true, // has_mysql
            true, // has_caddy
        );

        assert_eq!(
            config.networks,
            vec![Network::Metrics, Network::Database, Network::Proxy]
        );
    }

    #[test]
    fn it_should_have_no_networks_when_minimal_deployment() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            false,
            false, // has_prometheus
            false, // has_mysql (SQLite)
            false, // has_caddy
        );

        assert!(config.networks.is_empty());
    }

    // ==========================================================================
    // Serialization tests
    // ==========================================================================

    #[test]
    fn it_should_serialize_networks_to_name_strings() {
        let config = TrackerServiceConfig::new(
            vec![6969],
            vec![],
            1212,
            true,
            true,  // has_prometheus
            true,  // has_mysql
            false, // has_caddy
        );

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        // Networks serialize to their name strings for template compatibility
        assert_eq!(json["networks"][0], "metrics_network");
        assert_eq!(json["networks"][1], "database_network");
    }

    // ==========================================================================
    // Port derivation tests
    // ==========================================================================

    #[test]
    fn it_should_derive_udp_ports() {
        let config = TrackerServiceConfig::new(
            vec![6969, 6970],
            vec![],
            1212,
            true, // TLS enabled, so API port not exposed
            false,
            false,
            false,
        );

        assert_eq!(config.ports.len(), 2);
        assert_eq!(config.ports[0].binding(), "6969:6969/udp");
        assert_eq!(config.ports[1].binding(), "6970:6970/udp");
    }

    #[test]
    fn it_should_derive_http_ports_without_tls() {
        let config = TrackerServiceConfig::new(
            vec![],
            vec![7070, 7071],
            1212,
            true, // TLS enabled, so API port not exposed
            false,
            false,
            false,
        );

        assert_eq!(config.ports.len(), 2);
        assert_eq!(config.ports[0].binding(), "7070:7070");
        assert_eq!(config.ports[1].binding(), "7071:7071");
    }

    #[test]
    fn it_should_derive_api_port_when_no_tls() {
        let config = TrackerServiceConfig::new(
            vec![],
            vec![],
            1212,
            false, // No TLS, so API port exposed
            false,
            false,
            false,
        );

        assert_eq!(config.ports.len(), 1);
        assert_eq!(config.ports[0].binding(), "1212:1212");
    }

    #[test]
    fn it_should_not_derive_api_port_when_tls_enabled() {
        let config = TrackerServiceConfig::new(
            vec![],
            vec![],
            1212,
            true, // TLS enabled, so API port not exposed
            false,
            false,
            false,
        );

        assert!(config.ports.is_empty());
    }

    #[test]
    fn it_should_serialize_ports_with_binding_and_description() {
        let config =
            TrackerServiceConfig::new(vec![6969], vec![], 1212, false, false, false, false);

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        assert!(json["ports"][0]["binding"].is_string());
        assert!(json["ports"][0]["description"].is_string());
    }
}
