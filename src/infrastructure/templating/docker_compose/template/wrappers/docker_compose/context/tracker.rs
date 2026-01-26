//! Tracker service configuration for Docker Compose

// External crates
use serde::Serialize;

use crate::domain::topology::{EnabledServices, Network, NetworkDerivation, PortDerivation};
use crate::domain::tracker::TrackerConfig;

use super::port_definition::PortDefinition;

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
    /// Creates a new `TrackerServiceConfig` from domain configuration
    ///
    /// Uses the domain `PortDerivation` trait for port derivation logic,
    /// ensuring business rules live in the domain layer.
    ///
    /// # Arguments
    ///
    /// * `config` - The domain Tracker configuration
    /// * `context` - Topology context with information about enabled services
    #[must_use]
    pub fn from_domain_config(config: &TrackerConfig, enabled_services: &EnabledServices) -> Self {
        // Extract port info for legacy fields
        let udp_tracker_ports: Vec<u16> = config
            .udp_trackers()
            .iter()
            .map(|t| t.bind_address().port())
            .collect();

        let http_tracker_ports_without_tls: Vec<u16> = config
            .http_trackers()
            .iter()
            .filter(|t| !t.use_tls_proxy())
            .map(|t| t.bind_address().port())
            .collect();

        let http_api_port = config.http_api().bind_address().port();
        let http_api_has_tls = config.http_api().use_tls_proxy();

        let needs_ports_section = !udp_tracker_ports.is_empty()
            || !http_tracker_ports_without_tls.is_empty()
            || !http_api_has_tls;

        // Use domain NetworkDerivation trait for network logic
        let networks = config.derive_networks(enabled_services);

        // Use domain PortDerivation trait for port logic
        let port_bindings = config.derive_ports();
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
}

// Type alias for backward compatibility
pub type TrackerPorts = TrackerServiceConfig;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::tracker::{
        DatabaseConfig as TrackerDatabaseConfig, HealthCheckApiConfig, HttpApiConfig,
        HttpTrackerConfig, SqliteConfig, TrackerConfig, TrackerCoreConfig, UdpTrackerConfig,
    };

    /// Helper to create `EnabledServices` from boolean flags
    fn make_context(has_prometheus: bool, has_mysql: bool, has_caddy: bool) -> EnabledServices {
        use crate::domain::topology::Service;
        let mut services = Vec::new();
        if has_prometheus {
            services.push(Service::Prometheus);
        }
        if has_mysql {
            services.push(Service::MySQL);
        }
        if has_caddy {
            services.push(Service::Caddy);
        }
        EnabledServices::from(&services)
    }

    /// Helper to create a basic domain `TrackerConfig` for network tests
    fn basic_domain_tracker_config() -> TrackerConfig {
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap()],
            vec![],
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                None,
                false,
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    /// Helper to create a domain `TrackerConfig` with TLS on API
    fn domain_tracker_config_with_api_tls() -> TrackerConfig {
        use crate::shared::DomainName;
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap()],
            vec![],
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                Some(DomainName::new("api.example.com").unwrap()),
                true, // TLS enabled
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    /// Helper to create a domain `TrackerConfig` with multiple UDP ports
    fn domain_tracker_config_with_udp_ports(ports: &[u16]) -> TrackerConfig {
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            ports
                .iter()
                .map(|p| {
                    UdpTrackerConfig::new(format!("0.0.0.0:{p}").parse().unwrap(), None).unwrap()
                })
                .collect(),
            vec![],
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                None,
                false,
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    /// Helper to create a domain `TrackerConfig` with HTTP ports (no TLS)
    fn domain_tracker_config_with_http_ports(ports: &[u16]) -> TrackerConfig {
        use crate::shared::DomainName;
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![],
            ports
                .iter()
                .map(|p| {
                    HttpTrackerConfig::new(format!("0.0.0.0:{p}").parse().unwrap(), None, false)
                        .unwrap()
                })
                .collect(),
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                Some(DomainName::new("api.example.com").unwrap()),
                true, // API has TLS so only HTTP tracker ports are tested
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    /// Helper to create a minimal domain `TrackerConfig` (no UDP, no HTTP, API has TLS)
    fn minimal_domain_tracker_config_with_api_tls() -> TrackerConfig {
        use crate::shared::DomainName;
        TrackerConfig::new(
            TrackerCoreConfig::new(
                TrackerDatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![],
            vec![],
            HttpApiConfig::new(
                "0.0.0.0:1212".parse().unwrap(),
                "TestToken".to_string().into(),
                Some(DomainName::new("api.example.com").unwrap()),
                true,
            )
            .unwrap(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .unwrap()
    }

    // ==========================================================================
    // Network assignment tests
    // ==========================================================================

    #[test]
    fn it_should_connect_tracker_to_metrics_network_when_prometheus_enabled() {
        let domain_config = basic_domain_tracker_config();
        let context = make_context(true, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        assert!(config.networks.contains(&Network::Metrics));
    }

    #[test]
    fn it_should_not_connect_tracker_to_metrics_network_when_prometheus_disabled() {
        let domain_config = basic_domain_tracker_config();
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        assert!(!config.networks.contains(&Network::Metrics));
    }

    #[test]
    fn it_should_connect_tracker_to_database_network_when_mysql_enabled() {
        let domain_config = basic_domain_tracker_config();
        let context = make_context(false, true, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        assert!(config.networks.contains(&Network::Database));
    }

    #[test]
    fn it_should_not_connect_tracker_to_database_network_when_mysql_disabled() {
        let domain_config = basic_domain_tracker_config();
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        assert!(!config.networks.contains(&Network::Database));
    }

    #[test]
    fn it_should_connect_tracker_to_proxy_network_when_caddy_enabled() {
        let domain_config = domain_tracker_config_with_api_tls();
        let context = make_context(false, false, true);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        assert!(config.networks.contains(&Network::Proxy));
    }

    #[test]
    fn it_should_not_connect_tracker_to_proxy_network_when_caddy_disabled() {
        let domain_config = basic_domain_tracker_config();
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        assert!(!config.networks.contains(&Network::Proxy));
    }

    #[test]
    fn it_should_connect_tracker_to_all_networks_when_all_services_enabled() {
        let domain_config = domain_tracker_config_with_api_tls();
        let context = make_context(true, true, true);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        assert_eq!(
            config.networks,
            vec![Network::Metrics, Network::Database, Network::Proxy]
        );
    }

    #[test]
    fn it_should_have_no_networks_when_minimal_deployment() {
        let domain_config = basic_domain_tracker_config();
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        assert!(config.networks.is_empty());
    }

    // ==========================================================================
    // Serialization tests
    // ==========================================================================

    #[test]
    fn it_should_serialize_networks_to_name_strings() {
        let domain_config = domain_tracker_config_with_api_tls();
        let context = make_context(true, true, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        // Networks serialize to their name strings for template compatibility
        assert_eq!(json["networks"][0], "metrics_network");
        assert_eq!(json["networks"][1], "database_network");
    }

    // ==========================================================================
    // Port derivation tests (testing from_domain_config delegation to domain)
    // ==========================================================================

    #[test]
    fn it_should_derive_udp_ports() {
        let domain_config = domain_tracker_config_with_udp_ports(&[6969, 6970]);
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        // UDP ports + API port (no TLS) = 3 ports
        // Filter just UDP ports
        let udp_ports: Vec<_> = config
            .ports
            .iter()
            .filter(|p| p.binding().ends_with("/udp"))
            .collect();
        assert_eq!(udp_ports.len(), 2);
        assert_eq!(udp_ports[0].binding(), "6969:6969/udp");
        assert_eq!(udp_ports[1].binding(), "6970:6970/udp");
    }

    #[test]
    fn it_should_derive_http_ports_without_tls() {
        let domain_config = domain_tracker_config_with_http_ports(&[7070, 7071]);
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        // HTTP tracker ports only (API has TLS so not exposed)
        assert_eq!(config.ports.len(), 2);
        assert_eq!(config.ports[0].binding(), "7070:7070");
        assert_eq!(config.ports[1].binding(), "7071:7071");
    }

    #[test]
    fn it_should_derive_api_port_when_no_tls() {
        let domain_config = basic_domain_tracker_config();
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        // UDP port + API port = 2 ports
        // API port is the second one
        let api_port = config.ports.iter().find(|p| p.binding() == "1212:1212");
        assert!(api_port.is_some());
    }

    #[test]
    fn it_should_not_derive_api_port_when_tls_enabled() {
        let domain_config = minimal_domain_tracker_config_with_api_tls();
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        // No UDP, no HTTP without TLS, API has TLS = no ports
        assert!(config.ports.is_empty());
    }

    #[test]
    fn it_should_serialize_ports_with_binding_and_description() {
        let domain_config = basic_domain_tracker_config();
        let context = make_context(false, false, false);
        let config = TrackerServiceConfig::from_domain_config(&domain_config, &context);

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        assert!(json["ports"][0]["binding"].is_string());
        assert!(json["ports"][0]["description"].is_string());
    }
}
