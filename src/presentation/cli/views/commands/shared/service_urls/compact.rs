//! Compact Service URLs View
//!
//! This module provides a compact view for rendering service URLs without
//! additional context. It filters out localhost-only services and shows
//! only publicly accessible endpoints.

use crate::application::command_handlers::show::info::{GrafanaInfo, ServiceInfo};

/// Compact view for rendering service URLs
///
/// This view renders service URLs in a compact format suitable for the
/// run command output. It shows only publicly accessible services and
/// excludes internal-only services (localhost addresses).
pub struct CompactServiceUrlsView;

impl CompactServiceUrlsView {
    /// Render service URLs in compact format
    ///
    /// # Arguments
    ///
    /// * `services` - Service information containing tracker endpoints
    /// * `grafana` - Optional Grafana service information
    ///
    /// # Returns
    ///
    /// A formatted string with all publicly accessible service URLs
    #[must_use]
    pub fn render(services: &ServiceInfo, grafana: Option<&GrafanaInfo>) -> String {
        let mut lines = vec!["Services are now accessible:".to_string()];

        // UDP Trackers
        Self::render_udp_trackers(services, &mut lines);

        // HTTP Trackers (HTTPS and direct)
        Self::render_http_trackers(services, &mut lines);

        // API Endpoint (if publicly accessible)
        Self::render_api(services, &mut lines);

        // Health Check (if publicly accessible)
        Self::render_health_check(services, &mut lines);

        // Grafana
        if let Some(grafana_info) = grafana {
            Self::render_grafana(grafana_info, &mut lines);
        }

        lines.join("\n")
    }

    fn render_udp_trackers(services: &ServiceInfo, lines: &mut Vec<String>) {
        if services.udp_trackers.is_empty() {
            return;
        }

        for url in &services.udp_trackers {
            lines.push(format!("  Tracker (UDP):  {url}"));
        }
    }

    fn render_http_trackers(services: &ServiceInfo, lines: &mut Vec<String>) {
        // HTTPS-enabled HTTP trackers (via Caddy)
        for url in &services.https_http_trackers {
            lines.push(format!("  Tracker (HTTP): {url}"));
        }

        // Direct HTTP trackers (no TLS)
        for url in &services.direct_http_trackers {
            lines.push(format!("  Tracker (HTTP): {url}"));
        }

        // Note: localhost_http_trackers are NOT shown (internal only)
    }

    fn render_api(services: &ServiceInfo, lines: &mut Vec<String>) {
        // Only show if API is publicly accessible
        if !services.api_is_localhost_only {
            lines.push(format!("  API:            {}", services.api_endpoint));
        }
    }

    fn render_health_check(services: &ServiceInfo, lines: &mut Vec<String>) {
        // Only show if health check is publicly accessible
        if !services.health_check_is_localhost_only {
            lines.push(format!("  Health Check:   {}", services.health_check_url));
        }
    }

    fn render_grafana(grafana: &GrafanaInfo, lines: &mut Vec<String>) {
        lines.push(format!("  Grafana:        {}", grafana.url));
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;
    use crate::application::command_handlers::show::info::LocalhostServiceInfo;

    #[test]
    fn it_should_render_udp_tracker() {
        let services = ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        assert!(output.contains("Tracker (UDP):  udp://10.0.0.1:6969/announce"));
    }

    #[test]
    fn it_should_render_http_tracker() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        assert!(output.contains("Tracker (HTTP): http://10.0.0.1:7070/announce"));
        // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_render_https_tracker() {
        let services = ServiceInfo::new(
            vec![],
            vec!["https://http.tracker.local/announce".to_string()],
            vec![],
            vec![],
            "https://api.tracker.local/api".to_string(),
            true,
            true,
            "https://health.tracker.local/health_check".to_string(),
            true,
            true,
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        assert!(output.contains("Tracker (HTTP): https://http.tracker.local/announce"));
    }

    #[test]
    fn it_should_render_api_when_publicly_accessible() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        assert!(output.contains("API:            http://10.0.0.1:1212/api")); // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_not_render_api_when_localhost_only() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "http://127.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            true,                                             // localhost only
            "http://127.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            true, // localhost only
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        assert!(!output.contains("API:"));
    }

    #[test]
    fn it_should_render_health_check_when_publicly_accessible() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        assert!(output.contains("Health Check:   http://10.0.0.1:1313/health_check"));
        // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_not_render_health_check_when_localhost_only() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://127.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            true, // localhost only
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        assert!(!output.contains("Health Check:"));
    }

    #[test]
    fn it_should_not_render_localhost_only_trackers() {
        let localhost_tracker = LocalhostServiceInfo {
            service_name: "http-tracker-1".to_string(),
            port: 7070,
        };

        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![localhost_tracker],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        // Should not contain localhost tracker
        assert!(!output.contains("localhost"));
        assert!(!output.contains("SSH tunnel"));
    }

    #[test]
    fn it_should_render_grafana_when_provided() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let grafana = GrafanaInfo::new(
            Url::parse("http://10.0.0.1:3000").unwrap(), // DevSkim: ignore DS137138
            false,
        );

        let output = CompactServiceUrlsView::render(&services, Some(&grafana));

        assert!(output.contains("Grafana:        http://10.0.0.1:3000")); // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_include_header() {
        let services = ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],
            vec![],
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let output = CompactServiceUrlsView::render(&services, None);

        assert!(output.starts_with("Services are now accessible:"));
    }
}
