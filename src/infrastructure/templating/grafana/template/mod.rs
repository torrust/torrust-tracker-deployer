//! Grafana Template Rendering
//!
//! Provides template rendering capabilities for Grafana provisioning configuration.
//!
//! ## Components
//!
//! - `renderer` - Project generator and template renderers

pub mod renderer;

use serde::Serialize;

/// Context for rendering Grafana datasource configuration templates
///
/// Contains all variables needed to render the Prometheus datasource template.
#[derive(Debug, Clone, Serialize)]
pub struct GrafanaContext {
    /// Prometheus scrape interval in seconds
    ///
    /// Used to configure the datasource's `timeInterval` setting, which should match
    /// Prometheus's `scrape_interval` for optimal query performance.
    pub prometheus_scrape_interval_in_secs: u32,
}

impl GrafanaContext {
    /// Creates a new Grafana context
    ///
    /// # Arguments
    ///
    /// * `prometheus_scrape_interval_in_secs` - Scrape interval from Prometheus config
    #[must_use]
    pub fn new(prometheus_scrape_interval_in_secs: u32) -> Self {
        Self {
            prometheus_scrape_interval_in_secs,
        }
    }
}
