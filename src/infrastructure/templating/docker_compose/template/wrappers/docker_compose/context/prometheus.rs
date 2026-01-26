//! Prometheus service configuration for Docker Compose

// External crates
use serde::Serialize;

use crate::domain::prometheus::PrometheusConfig;
use crate::domain::topology::{EnabledServices, Network, NetworkDerivation, PortDerivation};

use super::port_definition::PortDefinition;

/// Prometheus service configuration for Docker Compose
///
/// Contains all configuration needed for the Prometheus service in Docker Compose,
/// including the scrape interval, port mappings, and network connections. All logic
/// is pre-computed in Rust to keep the Tera template simple.
#[derive(Serialize, Debug, Clone)]
pub struct PrometheusServiceConfig {
    /// Scrape interval in seconds
    pub scrape_interval_in_secs: u32,
    /// Port bindings for Docker Compose
    ///
    /// Prometheus exposes port 9090 on localhost only for security.
    pub ports: Vec<PortDefinition>,
    /// Networks the Prometheus service should connect to
    ///
    /// Pre-computed list based on enabled features:
    /// - Always includes `metrics_network` (scrapes metrics from tracker)
    /// - Includes `visualization_network` if Grafana is enabled
    pub networks: Vec<Network>,
}

impl PrometheusServiceConfig {
    /// Creates a new `PrometheusServiceConfig` from domain configuration
    ///
    /// Uses the domain `PortDerivation` and `NetworkDerivation` traits,
    /// ensuring business rules live in the domain layer.
    ///
    /// # Arguments
    ///
    /// * `config` - The domain Prometheus configuration
    /// * `context` - Topology context with information about enabled services
    #[must_use]
    pub fn from_domain_config(
        config: &PrometheusConfig,
        enabled_services: &EnabledServices,
    ) -> Self {
        // Use domain NetworkDerivation trait for network logic
        let networks = config.derive_networks(enabled_services);
        // Use domain PortDerivation trait for port logic
        let port_bindings = config.derive_ports();
        let ports = port_bindings.iter().map(PortDefinition::from).collect();

        Self {
            scrape_interval_in_secs: config.scrape_interval_in_secs(),
            ports,
            networks,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;

    fn make_config(scrape_interval_secs: u32) -> PrometheusConfig {
        PrometheusConfig::new(
            NonZeroU32::new(scrape_interval_secs).expect("non-zero scrape interval"),
        )
    }

    fn make_context(has_grafana: bool) -> EnabledServices {
        if has_grafana {
            EnabledServices::from(&[crate::domain::topology::Service::Grafana])
        } else {
            EnabledServices::from(&[])
        }
    }

    #[test]
    fn it_should_connect_prometheus_to_metrics_network() {
        let context = make_context(false);
        let config = PrometheusServiceConfig::from_domain_config(&make_config(15), &context);

        assert!(config.networks.contains(&Network::Metrics));
    }

    #[test]
    fn it_should_not_connect_prometheus_to_visualization_network_when_grafana_disabled() {
        let context = make_context(false);
        let config = PrometheusServiceConfig::from_domain_config(&make_config(15), &context);

        assert_eq!(config.networks, vec![Network::Metrics]);
        assert!(!config.networks.contains(&Network::Visualization));
    }

    #[test]
    fn it_should_connect_prometheus_to_visualization_network_when_grafana_enabled() {
        let context = make_context(true);
        let config = PrometheusServiceConfig::from_domain_config(&make_config(30), &context);

        assert_eq!(
            config.networks,
            vec![Network::Metrics, Network::Visualization]
        );
    }

    #[test]
    fn it_should_serialize_networks_to_name_strings() {
        let context = make_context(true);
        let config = PrometheusServiceConfig::from_domain_config(&make_config(15), &context);

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        // Networks serialize to their name strings for template compatibility
        assert_eq!(json["networks"][0], "metrics_network");
        assert_eq!(json["networks"][1], "visualization_network");
    }

    #[test]
    fn it_should_expose_localhost_port_9090() {
        let context = make_context(false);
        let config = PrometheusServiceConfig::from_domain_config(&make_config(15), &context);

        assert_eq!(config.ports.len(), 1);
        assert_eq!(config.ports[0].binding(), "127.0.0.1:9090:9090");
    }

    #[test]
    fn it_should_serialize_ports_with_binding_and_description() {
        let context = make_context(false);
        let config = PrometheusServiceConfig::from_domain_config(&make_config(15), &context);

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        assert!(json["ports"][0]["binding"].is_string());
        assert!(json["ports"][0]["description"].is_string());
    }
}
