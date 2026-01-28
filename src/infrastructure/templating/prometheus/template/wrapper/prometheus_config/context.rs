//! Prometheus template context
//!
//! Defines the variables needed for prometheus.yml.tera template rendering.

use serde::Serialize;

use crate::infrastructure::templating::metadata::TemplateMetadata;

/// Context for rendering prometheus.yml.tera template
///
/// Contains all variables needed for Prometheus scrape configuration.
/// The context extracts metrics endpoint details from the tracker configuration.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::prometheus::PrometheusContext;
/// use torrust_tracker_deployer_lib::infrastructure::templating::metadata::TemplateMetadata;
/// use torrust_tracker_deployer_lib::shared::clock::{Clock, SystemClock};
///
/// let clock = SystemClock;
/// let metadata = TemplateMetadata::new(clock.now());
/// let context = PrometheusContext {
///     metadata,
///     scrape_interval: "15s".to_string(),
///     api_token: "MyAccessToken".to_string(),
///     api_port: 1212,
/// };
/// ```
///
/// # Data Flow
///
/// Environment Config (`tracker.http_api`) → Application Layer → `PrometheusContext`
///
/// - `scrape_interval`: From `prometheus.scrape_interval` (e.g., "15s", "30s", "1m")
/// - `api_token`: From `tracker.http_api.admin_token`
/// - `api_port`: Parsed from `tracker.http_api.bind_address` (e.g., 1212 from "0.0.0.0:1212")
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PrometheusContext {
    /// Template metadata (generation timestamp, etc.)
    ///
    /// Flattened for template compatibility - serializes metadata at top level.
    #[serde(flatten)]
    pub metadata: TemplateMetadata,

    /// How often to scrape metrics from tracker (e.g., "15s", "30s", "1m")
    ///
    /// Default: "15s"
    /// Examples: "5s" (minimum to avoid overwhelming), "5m" (maximum reasonable interval)
    pub scrape_interval: String,

    /// Tracker HTTP API admin token for authentication
    ///
    /// This token is required to access the tracker's metrics endpoints:
    /// - `/api/v1/stats` - Aggregate statistics
    /// - `/api/v1/metrics` - Detailed operational metrics
    pub api_token: String,

    /// Tracker HTTP API port
    ///
    /// The port where the tracker's HTTP API is listening.
    /// Prometheus scrapes metrics from this API.
    /// Extracted from the tracker's HTTP API bind address.
    /// Example: 1212 from "0.0.0.0:1212"
    pub api_port: u16,
}

impl PrometheusContext {
    /// Creates a new `PrometheusContext`
    ///
    /// # Arguments
    ///
    /// * `metadata` - Template metadata (generation timestamp)
    /// * `scrape_interval` - How often to scrape metrics (e.g., "15s", "30s", "1m")
    /// * `api_token` - Tracker HTTP API admin token
    /// * `api_port` - Tracker HTTP API port
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::prometheus::PrometheusContext;
    /// use torrust_tracker_deployer_lib::infrastructure::templating::metadata::TemplateMetadata;
    /// use torrust_tracker_deployer_lib::shared::clock::{Clock, SystemClock};
    ///
    /// let clock = SystemClock;
    /// let metadata = TemplateMetadata::new(clock.now());
    /// let context = PrometheusContext::new(metadata, "15s".to_string(), "MyToken".to_string(), 1212);
    /// ```
    #[must_use]
    pub fn new(
        metadata: TemplateMetadata,
        scrape_interval: String,
        api_token: String,
        api_port: u16,
    ) -> Self {
        Self {
            metadata,
            scrape_interval,
            api_token,
            api_port,
        }
    }
}

impl Default for PrometheusContext {
    fn default() -> Self {
        use chrono::TimeZone;
        let epoch = chrono::Utc.timestamp_opt(0, 0).unwrap();
        Self {
            metadata: TemplateMetadata::new(epoch),
            scrape_interval: "15s".to_string(),
            api_token: String::new(),
            api_port: 1212,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    /// Helper to create test metadata with a fixed timestamp
    fn create_test_metadata() -> TemplateMetadata {
        let fixed_time = Utc.with_ymd_and_hms(2026, 1, 27, 13, 41, 56).unwrap();
        TemplateMetadata::new(fixed_time)
    }

    #[test]
    fn it_should_create_prometheus_context() {
        let metadata = create_test_metadata();
        let context =
            PrometheusContext::new(metadata, "15s".to_string(), "test_token".to_string(), 1212);

        assert_eq!(context.scrape_interval, "15s");
        assert_eq!(context.api_token, "test_token");
        assert_eq!(context.api_port, 1212);
    }

    #[test]
    fn it_should_create_default_context() {
        let context = PrometheusContext::default();

        assert_eq!(context.scrape_interval, "15s");
        assert_eq!(context.api_token, "");
        assert_eq!(context.api_port, 1212);
    }

    #[test]
    fn it_should_serialize_to_json() {
        let metadata = create_test_metadata();
        let context =
            PrometheusContext::new(metadata, "30s".to_string(), "admin_token".to_string(), 8080);

        let json = serde_json::to_value(&context).unwrap();
        assert_eq!(json["generated_at"], "2026-01-27T13:41:56Z");
        assert_eq!(json["scrape_interval"], "30s");
        assert_eq!(json["api_token"], "admin_token");
        assert_eq!(json["api_port"], 8080);
    }

    #[test]
    fn it_should_support_different_scrape_intervals() {
        let metadata = create_test_metadata();
        let fast_scrape = PrometheusContext::new(
            metadata.clone(),
            "5s".to_string(),
            "token".to_string(),
            1212,
        );
        let slow_scrape =
            PrometheusContext::new(metadata, "5m".to_string(), "token".to_string(), 1212);

        assert_eq!(fast_scrape.scrape_interval, "5s");
        assert_eq!(slow_scrape.scrape_interval, "5m");
    }

    #[test]
    fn it_should_support_different_ports() {
        let metadata = create_test_metadata();
        let default_port = PrometheusContext::new(
            metadata.clone(),
            "15s".to_string(),
            "token".to_string(),
            1212,
        );
        let custom_port =
            PrometheusContext::new(metadata, "15s".to_string(), "token".to_string(), 8080);

        assert_eq!(default_port.api_port, 1212);
        assert_eq!(custom_port.api_port, 8080);
    }
}
