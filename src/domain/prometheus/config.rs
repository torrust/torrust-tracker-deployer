//! Prometheus configuration domain model
//!
//! Defines the configuration for Prometheus metrics scraping.

use serde::{Deserialize, Serialize};

/// Prometheus metrics collection configuration
///
/// Configures how Prometheus scrapes metrics from the tracker.
/// When present in environment configuration, Prometheus service is enabled.
/// When absent, Prometheus service is disabled.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::prometheus::PrometheusConfig;
///
/// let config = PrometheusConfig {
///     scrape_interval: 15,
/// };
/// ```
///
/// # Default Behavior
///
/// - Default scrape interval: 15 seconds
/// - Minimum recommended: 5 seconds (to avoid overwhelming the tracker)
/// - Maximum recommended: 300 seconds (5 minutes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrometheusConfig {
    /// Scrape interval in seconds
    ///
    /// How often Prometheus should scrape metrics from the tracker's HTTP API endpoints.
    /// Default: 15 seconds
    pub scrape_interval: u32,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            scrape_interval: 15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_prometheus_config_with_default_values() {
        let config = PrometheusConfig::default();
        assert_eq!(config.scrape_interval, 15);
    }

    #[test]
    fn it_should_create_prometheus_config_with_custom_interval() {
        let config = PrometheusConfig {
            scrape_interval: 30,
        };
        assert_eq!(config.scrape_interval, 30);
    }

    #[test]
    fn it_should_serialize_to_json() {
        let config = PrometheusConfig {
            scrape_interval: 20,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["scrape_interval"], 20);
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = serde_json::json!({
            "scrape_interval": 25
        });

        let config: PrometheusConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.scrape_interval, 25);
    }

    #[test]
    fn it_should_support_different_scrape_intervals() {
        let fast = PrometheusConfig { scrape_interval: 5 };
        let medium = PrometheusConfig {
            scrape_interval: 15,
        };
        let slow = PrometheusConfig {
            scrape_interval: 300,
        };

        assert_eq!(fast.scrape_interval, 5);
        assert_eq!(medium.scrape_interval, 15);
        assert_eq!(slow.scrape_interval, 300);
    }
}
