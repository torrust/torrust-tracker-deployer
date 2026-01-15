//! Prometheus Service View
//!
//! This module provides a view for rendering Prometheus metrics service information.

use crate::application::command_handlers::show::info::PrometheusInfo;

/// View for rendering Prometheus service information
///
/// This view handles the display of Prometheus metrics service details.
/// Prometheus is typically internal-only and accessed via SSH tunnel.
pub struct PrometheusView;

impl PrometheusView {
    /// Render Prometheus service information as formatted lines
    ///
    /// # Arguments
    ///
    /// * `prometheus` - Prometheus service information
    ///
    /// # Returns
    ///
    /// A vector of formatted lines ready to be joined
    #[must_use]
    pub fn render(prometheus: &PrometheusInfo) -> Vec<String> {
        vec![
            String::new(), // blank line
            "Prometheus:".to_string(),
            format!("  {}", prometheus.access_note),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_render_prometheus_header() {
        let prometheus = PrometheusInfo::default_internal();
        let lines = PrometheusView::render(&prometheus);
        assert!(lines.iter().any(|l| l == "Prometheus:"));
    }

    #[test]
    fn it_should_render_access_note() {
        let prometheus = PrometheusInfo::default_internal();
        let lines = PrometheusView::render(&prometheus);
        assert!(lines.iter().any(|l| l.contains("Internal only")));
    }

    #[test]
    fn it_should_start_with_blank_line() {
        let prometheus = PrometheusInfo::default_internal();
        let lines = PrometheusView::render(&prometheus);
        assert!(lines.first().is_some_and(String::is_empty));
    }
}
