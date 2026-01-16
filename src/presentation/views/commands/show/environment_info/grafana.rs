//! Grafana Service View
//!
//! This module provides a view for rendering Grafana visualization service information.

use crate::application::command_handlers::show::info::GrafanaInfo;

/// View for rendering Grafana service information
///
/// This view handles the display of Grafana visualization service details,
/// including HTTPS status when configured with Caddy.
pub struct GrafanaView;

impl GrafanaView {
    /// Render Grafana service information as formatted lines
    ///
    /// # Arguments
    ///
    /// * `grafana` - Grafana service information
    ///
    /// # Returns
    ///
    /// A vector of formatted lines ready to be joined
    #[must_use]
    pub fn render(grafana: &GrafanaInfo) -> Vec<String> {
        let header = if grafana.uses_https {
            "Grafana (HTTPS via Caddy):".to_string()
        } else {
            "Grafana:".to_string()
        };

        vec![
            String::new(), // blank line
            header,
            format!("  {}", grafana.url),
        ]
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    fn http_grafana() -> GrafanaInfo {
        GrafanaInfo::new(
            Url::parse("http://10.0.0.1:3100").unwrap(), // DevSkim: ignore DS137138
            false,
        )
    }

    fn https_grafana() -> GrafanaInfo {
        GrafanaInfo::new(Url::parse("https://grafana.tracker.local").unwrap(), true)
    }

    #[test]
    fn it_should_render_http_grafana_header() {
        let lines = GrafanaView::render(&http_grafana());
        assert!(lines.iter().any(|l| l == "Grafana:"));
    }

    #[test]
    fn it_should_render_https_grafana_header_with_caddy_indicator() {
        let lines = GrafanaView::render(&https_grafana());
        assert!(lines
            .iter()
            .any(|l| l.contains("Grafana (HTTPS via Caddy):")));
    }

    #[test]
    fn it_should_render_http_url() {
        let lines = GrafanaView::render(&http_grafana());
        assert!(lines.iter().any(|l| l.contains("http://10.0.0.1:3100"))); // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_render_https_url() {
        let lines = GrafanaView::render(&https_grafana());
        assert!(lines
            .iter()
            .any(|l| l.contains("https://grafana.tracker.local")));
    }

    #[test]
    fn it_should_start_with_blank_line() {
        let lines = GrafanaView::render(&http_grafana());
        assert!(lines.first().is_some_and(String::is_empty));
    }
}
