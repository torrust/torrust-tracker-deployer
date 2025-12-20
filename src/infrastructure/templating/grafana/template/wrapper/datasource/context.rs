//! Datasource template context
//!
//! Defines the variables needed for the prometheus.yml.tera datasource template rendering.

use serde::Serialize;

/// Context for rendering prometheus.yml.tera datasource template
///
/// Contains all variables needed for Grafana Prometheus datasource configuration.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::grafana::template::DatasourceContext;
///
/// let context = DatasourceContext::new(15);
/// ```
///
/// # Data Flow
///
/// Prometheus Config (`scrape_interval_in_secs`) → Application Layer → `DatasourceContext`
///
/// - `prometheus_scrape_interval_in_secs`: From `prometheus.scrape_interval_in_secs()`
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DatasourceContext {
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
    /// * `prometheus_scrape_interval_in_secs` - Prometheus scrape interval in seconds
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::grafana::template::DatasourceContext;
    ///
    /// let context = DatasourceContext::new(15);
    /// assert_eq!(context.prometheus_scrape_interval_in_secs, 15);
    /// ```
    #[must_use]
    pub fn new(prometheus_scrape_interval_in_secs: u32) -> Self {
        Self {
            prometheus_scrape_interval_in_secs,
        }
    }
}

impl Default for DatasourceContext {
    fn default() -> Self {
        Self {
            prometheus_scrape_interval_in_secs: 15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_datasource_context() {
        let context = DatasourceContext::new(30);

        assert_eq!(context.prometheus_scrape_interval_in_secs, 30);
    }

    #[test]
    fn it_should_have_default_values() {
        let context = DatasourceContext::default();

        assert_eq!(context.prometheus_scrape_interval_in_secs, 15);
    }

    #[test]
    fn it_should_be_serializable_for_tera() {
        let context = DatasourceContext::new(20);

        let json = serde_json::to_value(&context).expect("Failed to serialize");

        assert_eq!(json["prometheus_scrape_interval_in_secs"], 20);
    }
}
