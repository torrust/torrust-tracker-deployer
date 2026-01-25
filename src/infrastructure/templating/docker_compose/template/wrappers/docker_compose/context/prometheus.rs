//! Prometheus service configuration for Docker Compose

// External crates
use serde::Serialize;

use crate::domain::topology::Network;

/// Prometheus service configuration for Docker Compose
///
/// Contains all configuration needed for the Prometheus service in Docker Compose,
/// including the scrape interval and network connections. All logic is pre-computed
/// in Rust to keep the Tera template simple.
#[derive(Serialize, Debug, Clone)]
pub struct PrometheusServiceConfig {
    /// Scrape interval in seconds
    pub scrape_interval_in_secs: u32,
    /// Networks the Prometheus service should connect to
    ///
    /// Pre-computed list based on enabled features:
    /// - Always includes `metrics_network` (scrapes metrics from tracker)
    /// - Includes `visualization_network` if Grafana is enabled
    pub networks: Vec<Network>,
}

impl PrometheusServiceConfig {
    /// Creates a new `PrometheusServiceConfig` with pre-computed networks
    ///
    /// # Arguments
    ///
    /// * `scrape_interval_in_secs` - The scrape interval in seconds
    /// * `has_grafana` - Whether Grafana is enabled (adds `visualization_network`)
    #[must_use]
    pub fn new(scrape_interval_in_secs: u32, has_grafana: bool) -> Self {
        let networks = Self::compute_networks(has_grafana);

        Self {
            scrape_interval_in_secs,
            networks,
        }
    }

    /// Computes the list of networks for the Prometheus service
    fn compute_networks(has_grafana: bool) -> Vec<Network> {
        let mut networks = vec![Network::Metrics];

        if has_grafana {
            networks.push(Network::Visualization);
        }

        networks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_connect_prometheus_to_metrics_network() {
        let config = PrometheusServiceConfig::new(15, false);

        assert!(config.networks.contains(&Network::Metrics));
    }

    #[test]
    fn it_should_not_connect_prometheus_to_visualization_network_when_grafana_disabled() {
        let config = PrometheusServiceConfig::new(15, false);

        assert_eq!(config.networks, vec![Network::Metrics]);
        assert!(!config.networks.contains(&Network::Visualization));
    }

    #[test]
    fn it_should_connect_prometheus_to_visualization_network_when_grafana_enabled() {
        let config = PrometheusServiceConfig::new(30, true);

        assert_eq!(
            config.networks,
            vec![Network::Metrics, Network::Visualization]
        );
    }

    #[test]
    fn it_should_serialize_networks_to_name_strings() {
        let config = PrometheusServiceConfig::new(15, true);

        let json = serde_json::to_value(&config).expect("serialization should succeed");

        // Networks serialize to their name strings for template compatibility
        assert_eq!(json["networks"][0], "metrics_network");
        assert_eq!(json["networks"][1], "visualization_network");
    }
}
