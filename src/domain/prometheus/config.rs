//! Prometheus configuration domain model
//!
//! Defines the configuration for Prometheus metrics scraping.

use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

/// Default scrape interval in seconds
///
/// This is the recommended interval for most use cases, balancing
/// monitoring frequency with resource usage.
const DEFAULT_SCRAPE_INTERVAL_SECS: u32 = 15;

/// Prometheus metrics collection configuration
///
/// Configures how Prometheus scrapes metrics from the tracker.
/// When present in environment configuration, Prometheus service is enabled.
/// When absent, Prometheus service is disabled.
///
/// # Example
///
/// ```rust
/// use std::num::NonZeroU32;
/// use torrust_tracker_deployer_lib::domain::prometheus::PrometheusConfig;
///
/// let interval = NonZeroU32::new(15).expect("15 is non-zero");
/// let config = PrometheusConfig::new(interval);
/// ```
///
/// # Default Behavior
///
/// - Default scrape interval: 15 seconds
/// - Minimum: 1 second (to avoid zero or negative values)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrometheusConfig {
    /// Scrape interval in seconds
    ///
    /// Guaranteed to be non-zero at the type level.
    /// The Prometheus template will append 's' suffix.
    /// Examples: 15 → "15s", 30 → "30s", 60 → "60s" (1 minute)
    scrape_interval_in_secs: NonZeroU32,
}

impl PrometheusConfig {
    /// Creates a new Prometheus configuration with the specified scrape interval
    ///
    /// # Arguments
    ///
    /// * `scrape_interval_in_secs` - Non-zero interval in seconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::num::NonZeroU32;
    /// use torrust_tracker_deployer_lib::domain::prometheus::PrometheusConfig;
    ///
    /// let interval = NonZeroU32::new(30).expect("30 is non-zero");
    /// let config = PrometheusConfig::new(interval);
    /// assert_eq!(config.scrape_interval_in_secs(), 30);
    /// ```
    #[must_use]
    pub const fn new(scrape_interval_in_secs: NonZeroU32) -> Self {
        Self {
            scrape_interval_in_secs,
        }
    }

    /// Returns the scrape interval in seconds
    ///
    /// The value is guaranteed to be non-zero.
    #[must_use]
    pub fn scrape_interval_in_secs(&self) -> u32 {
        self.scrape_interval_in_secs.get()
    }
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            // SAFETY: DEFAULT_SCRAPE_INTERVAL_SECS is non-zero
            scrape_interval_in_secs: NonZeroU32::new(DEFAULT_SCRAPE_INTERVAL_SECS)
                .expect("DEFAULT_SCRAPE_INTERVAL_SECS is non-zero"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;

    #[test]
    fn it_should_create_prometheus_config_with_default_values() {
        let config = PrometheusConfig::default();
        assert_eq!(
            config.scrape_interval_in_secs(),
            DEFAULT_SCRAPE_INTERVAL_SECS
        );
    }

    #[test]
    fn it_should_create_prometheus_config_with_custom_interval() {
        let interval = NonZeroU32::new(30).expect("30 is non-zero");
        let config = PrometheusConfig::new(interval);
        assert_eq!(config.scrape_interval_in_secs(), 30);
    }

    #[test]
    fn it_should_serialize_to_json() {
        let interval = NonZeroU32::new(20).expect("20 is non-zero");
        let config = PrometheusConfig::new(interval);

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["scrape_interval_in_secs"], 20);
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = serde_json::json!({
            "scrape_interval_in_secs": 25
        });

        let config: PrometheusConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.scrape_interval_in_secs(), 25);
    }

    #[test]
    fn it_should_support_different_scrape_intervals() {
        let fast = PrometheusConfig::new(NonZeroU32::new(5).expect("5 is non-zero"));
        let medium = PrometheusConfig::new(NonZeroU32::new(15).expect("15 is non-zero"));
        let slow = PrometheusConfig::new(NonZeroU32::new(300).expect("300 is non-zero"));

        assert_eq!(fast.scrape_interval_in_secs(), 5);
        assert_eq!(medium.scrape_interval_in_secs(), 15);
        assert_eq!(slow.scrape_interval_in_secs(), 300);
    }

    #[test]
    fn it_should_reject_zero_interval_at_type_level() {
        // Cannot construct NonZeroU32 with 0
        let result = NonZeroU32::new(0);
        assert!(result.is_none());
    }
}
