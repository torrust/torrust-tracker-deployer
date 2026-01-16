//! Prometheus service configuration for Docker Compose

// External crates
use serde::Serialize;

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
    pub networks: Vec<String>,
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
    fn compute_networks(has_grafana: bool) -> Vec<String> {
        let mut networks = vec!["metrics_network".to_string()];

        if has_grafana {
            networks.push("visualization_network".to_string());
        }

        networks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_prometheus_config_with_only_metrics_network_when_grafana_disabled() {
        let config = PrometheusServiceConfig::new(15, false);

        assert_eq!(config.scrape_interval_in_secs, 15);
        assert_eq!(config.networks, vec!["metrics_network"]);
    }

    #[test]
    fn it_should_create_prometheus_config_with_both_networks_when_grafana_enabled() {
        let config = PrometheusServiceConfig::new(30, true);

        assert_eq!(config.scrape_interval_in_secs, 30);
        assert_eq!(
            config.networks,
            vec!["metrics_network", "visualization_network"]
        );
    }
}
