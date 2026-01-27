//! Datasource template context
//!
//! Defines the variables needed for the prometheus.yml.tera datasource template rendering.

use serde::Serialize;

use crate::infrastructure::templating::TemplateMetadata;

/// Context for rendering prometheus.yml.tera datasource template
///
/// Contains all variables needed for Grafana Prometheus datasource configuration.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::grafana::template::DatasourceContext;
/// use torrust_tracker_deployer_lib::infrastructure::templating::TemplateMetadata;
/// use torrust_tracker_deployer_lib::shared::clock::{Clock, SystemClock};
///
/// let metadata = TemplateMetadata::new(SystemClock.now());
/// let context = DatasourceContext::new(metadata, 15);
/// ```
///
/// # Data Flow
///
/// Prometheus Config (`scrape_interval_in_secs`) → Application Layer → `DatasourceContext`
///
/// - `prometheus_scrape_interval_in_secs`: From `prometheus.scrape_interval_in_secs()`
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DatasourceContext {
    /// Template metadata (timestamp, etc.)
    ///
    /// Contains information about when the template was generated, useful for
    /// tracking template versions and ensuring reproducibility.
    #[serde(flatten)]
    pub metadata: TemplateMetadata,

    /// Prometheus scrape interval in seconds
    ///
    /// This matches the Prometheus `scrape_interval` configuration to ensure
    /// Grafana's time interval aligns with data collection intervals.
    pub prometheus_scrape_interval_in_secs: u32,
}

impl DatasourceContext {
    /// Creates a new `DatasourceContext`
    ///
    /// # Arguments
    ///
    /// * `metadata` - Template metadata (timestamp, etc.)
    /// * `prometheus_scrape_interval_in_secs` - Prometheus scrape interval in seconds
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::grafana::template::DatasourceContext;
    /// use torrust_tracker_deployer_lib::infrastructure::templating::TemplateMetadata;
    /// use torrust_tracker_deployer_lib::shared::clock::{Clock, SystemClock};
    ///
    /// let metadata = TemplateMetadata::new(SystemClock.now());
    /// let context = DatasourceContext::new(metadata, 15);
    /// assert_eq!(context.prometheus_scrape_interval_in_secs, 15);
    /// ```
    #[must_use]
    pub fn new(metadata: TemplateMetadata, prometheus_scrape_interval_in_secs: u32) -> Self {
        Self {
            metadata,
            prometheus_scrape_interval_in_secs,
        }
    }
}

impl Default for DatasourceContext {
    fn default() -> Self {
        use chrono::{TimeZone, Utc};
        Self {
            metadata: TemplateMetadata::new(Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()),
            prometheus_scrape_interval_in_secs: 15,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn create_test_metadata() -> TemplateMetadata {
        TemplateMetadata::new(Utc.with_ymd_and_hms(2026, 1, 27, 13, 41, 56).unwrap())
    }

    #[test]
    fn it_should_create_datasource_context() {
        let context = DatasourceContext::new(create_test_metadata(), 30);

        assert_eq!(context.prometheus_scrape_interval_in_secs, 30);
    }

    #[test]
    fn it_should_have_default_values() {
        let context = DatasourceContext::default();

        assert_eq!(context.prometheus_scrape_interval_in_secs, 15);
    }

    #[test]
    fn it_should_be_serializable_for_tera() {
        let context = DatasourceContext::new(create_test_metadata(), 20);

        let json = serde_json::to_value(&context).expect("Failed to serialize");

        assert_eq!(json["generated_at"], "2026-01-27T13:41:56Z");
        assert_eq!(json["prometheus_scrape_interval_in_secs"], 20);
    }
}
