//! Tracker Services View
//!
//! This module provides a view for rendering tracker service endpoints
//! including UDP trackers, HTTP trackers (HTTPS and direct), API, and health check.

use crate::application::command_handlers::show::info::ServiceInfo;

/// View for rendering tracker service information
///
/// This view handles the display of tracker service endpoints that become
/// available after services have been started (Released/Running states).
pub struct TrackerServicesView;

impl TrackerServicesView {
    /// Render tracker service information as formatted lines
    ///
    /// # Arguments
    ///
    /// * `services` - Service information containing tracker endpoints
    ///
    /// # Returns
    ///
    /// A vector of formatted lines ready to be joined
    #[must_use]
    pub fn render(services: &ServiceInfo) -> Vec<String> {
        let mut lines = vec![
            String::new(), // blank line
            "Tracker Services:".to_string(),
        ];

        // UDP Trackers
        if !services.udp_trackers.is_empty() {
            lines.push("  UDP Trackers:".to_string());
            for url in &services.udp_trackers {
                lines.push(format!("    - {url}"));
            }
        }

        // HTTPS-enabled HTTP trackers (via Caddy)
        if !services.https_http_trackers.is_empty() {
            lines.push("  HTTP Trackers (HTTPS via Caddy):".to_string());
            for url in &services.https_http_trackers {
                lines.push(format!("    - {url}"));
            }
        }

        // Direct HTTP trackers (no TLS)
        if !services.direct_http_trackers.is_empty() {
            lines.push("  HTTP Trackers (direct):".to_string());
            for url in &services.direct_http_trackers {
                lines.push(format!("    - {url}"));
            }
        }

        // API endpoint with HTTPS indicator
        if services.api_uses_https {
            lines.push("  API Endpoint (HTTPS via Caddy):".to_string());
        } else {
            lines.push("  API Endpoint:".to_string());
        }
        lines.push(format!("    - {}", services.api_endpoint));

        // Health check
        if services.health_check_uses_https {
            lines.push("  Health Check (HTTPS via Caddy):".to_string());
        } else {
            lines.push("  Health Check:".to_string());
        }
        lines.push(format!("    - {}", services.health_check_url));

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::command_handlers::show::info::TlsDomainInfo;

    fn sample_http_only_services() -> ServiceInfo {
        ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],                                            // No HTTPS trackers
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            false,                                             // API doesn't use HTTPS
            "http://10.0.0.1:1313/health_check".to_string(),   // DevSkim: ignore DS137138
            false,                                             // Health check doesn't use HTTPS
            vec![],                                            // No TLS domains
        )
    }

    fn sample_https_services() -> ServiceInfo {
        ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![
                "https://http1.tracker.local/announce".to_string(),
                "https://http2.tracker.local/announce".to_string(),
            ],
            vec!["http://10.0.0.1:7072/announce".to_string()], // DevSkim: ignore DS137138
            "https://api.tracker.local/api".to_string(),
            true,                                            // API uses HTTPS
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,                                           // Health check doesn't use HTTPS (yet)
            vec![
                TlsDomainInfo::new("api.tracker.local".to_string(), 1212),
                TlsDomainInfo::new("http1.tracker.local".to_string(), 7070),
            ],
        )
    }

    #[test]
    fn it_should_render_udp_trackers() {
        let lines = TrackerServicesView::render(&sample_http_only_services());
        assert!(lines.iter().any(|l| l.contains("UDP Trackers:")));
        assert!(lines
            .iter()
            .any(|l| l.contains("udp://10.0.0.1:6969/announce")));
    }

    #[test]
    fn it_should_render_direct_http_trackers() {
        let lines = TrackerServicesView::render(&sample_http_only_services());
        assert!(lines.iter().any(|l| l.contains("HTTP Trackers (direct):")));
        assert!(lines
            .iter()
            .any(|l| l.contains("http://10.0.0.1:7070/announce"))); // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_render_https_http_trackers() {
        let lines = TrackerServicesView::render(&sample_https_services());
        assert!(lines
            .iter()
            .any(|l| l.contains("HTTP Trackers (HTTPS via Caddy):")));
        assert!(lines
            .iter()
            .any(|l| l.contains("https://http1.tracker.local/announce")));
        assert!(lines
            .iter()
            .any(|l| l.contains("https://http2.tracker.local/announce")));
    }

    #[test]
    fn it_should_render_api_endpoint_without_https_indicator() {
        let lines = TrackerServicesView::render(&sample_http_only_services());
        assert!(lines.iter().any(|l| l == "  API Endpoint:"));
        assert!(lines.iter().any(|l| l.contains("http://10.0.0.1:1212/api"))); // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_render_api_endpoint_with_https_indicator() {
        let lines = TrackerServicesView::render(&sample_https_services());
        assert!(lines
            .iter()
            .any(|l| l.contains("API Endpoint (HTTPS via Caddy):")));
        assert!(lines
            .iter()
            .any(|l| l.contains("https://api.tracker.local/api")));
    }

    #[test]
    fn it_should_render_health_check() {
        let lines = TrackerServicesView::render(&sample_http_only_services());
        assert!(lines.iter().any(|l| l.contains("Health Check:")));
        assert!(!lines
            .iter()
            .any(|l| l.contains("Health Check (HTTPS via Caddy):")));
        assert!(lines
            .iter()
            .any(|l| l.contains("http://10.0.0.1:1313/health_check"))); // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_render_health_check_with_https_indicator() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            "https://health.tracker.local/health_check".to_string(),
            true, // Health check uses HTTPS
            vec![TlsDomainInfo::new("health.tracker.local".to_string(), 1313)],
        );

        let lines = TrackerServicesView::render(&services);
        assert!(lines
            .iter()
            .any(|l| l.contains("Health Check (HTTPS via Caddy):")));
        assert!(lines
            .iter()
            .any(|l| l.contains("https://health.tracker.local/health_check")));
    }

    #[test]
    fn it_should_not_show_empty_sections() {
        let services = ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],                                 // No HTTPS trackers
            vec![],                                 // No direct HTTP trackers
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,                                           // Health check doesn't use HTTPS
            vec![],
        );

        let lines = TrackerServicesView::render(&services);
        assert!(!lines.iter().any(|l| l.contains("HTTP Trackers")));
    }
}
