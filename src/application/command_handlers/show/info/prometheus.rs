//! Prometheus metrics service information for display purposes
//!
//! This module contains DTOs for the Prometheus service.

/// Prometheus metrics service information for display purposes
///
/// This information shows the status of the Prometheus service when configured.
/// Prometheus collects and stores metrics from the tracker service.
/// It can be used independently or as a data source for Grafana.
#[derive(Debug, Clone)]
pub struct PrometheusInfo {
    /// Description of how to access Prometheus (internal only)
    pub access_note: String,
}

impl PrometheusInfo {
    /// Create a new `PrometheusInfo`
    #[must_use]
    pub fn new(access_note: String) -> Self {
        Self { access_note }
    }

    /// Create default `PrometheusInfo` for standard deployment
    ///
    /// Prometheus is always bound to localhost:9090 and not exposed externally.
    #[must_use]
    pub fn default_internal() -> Self {
        Self::new("Internal only (localhost:9090) - not exposed externally".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_prometheus_info() {
        let info = PrometheusInfo::new("Custom access note".to_string());
        assert_eq!(info.access_note, "Custom access note");
    }

    #[test]
    fn it_should_create_default_internal_prometheus_info() {
        let info = PrometheusInfo::default_internal();
        assert!(info.access_note.contains("localhost:9090"));
        assert!(info.access_note.contains("not exposed"));
    }
}
