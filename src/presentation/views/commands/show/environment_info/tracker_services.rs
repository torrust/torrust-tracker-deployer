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

        Self::render_udp_trackers(services, &mut lines);
        Self::render_http_trackers(services, &mut lines);
        Self::render_api_endpoint(services, &mut lines);
        Self::render_health_check(services, &mut lines);

        lines
    }

    fn render_udp_trackers(services: &ServiceInfo, lines: &mut Vec<String>) {
        if services.udp_trackers.is_empty() {
            return;
        }

        lines.push("  UDP Trackers:".to_string());
        for url in &services.udp_trackers {
            lines.push(format!("    - {url}"));
        }
    }

    fn render_http_trackers(services: &ServiceInfo, lines: &mut Vec<String>) {
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

        // Localhost-only HTTP trackers
        if !services.localhost_http_trackers.is_empty() {
            lines.push("  HTTP Trackers (internal only):".to_string());
            for tracker in &services.localhost_http_trackers {
                lines.push(format!(
                    "    - {} - localhost:{} (access via SSH tunnel)",
                    tracker.service_name, tracker.port
                ));
            }
        }
    }

    fn render_api_endpoint(services: &ServiceInfo, lines: &mut Vec<String>) {
        if services.api_is_localhost_only {
            lines.push("  API Endpoint (internal only):".to_string());
            lines.push(format!(
                "    - {} (access via SSH tunnel)",
                services.api_endpoint
            ));
        } else if services.api_uses_https {
            lines.push("  API Endpoint (HTTPS via Caddy):".to_string());
            lines.push(format!("    - {}", services.api_endpoint));
        } else {
            lines.push("  API Endpoint:".to_string());
            lines.push(format!("    - {}", services.api_endpoint));
        }
    }

    fn render_health_check(services: &ServiceInfo, lines: &mut Vec<String>) {
        if services.health_check_is_localhost_only {
            lines.push("  Health Check (internal only):".to_string());
            lines.push(format!(
                "    - {} (access via SSH tunnel)",
                services.health_check_url
            ));
        } else if services.health_check_uses_https {
            lines.push("  Health Check (HTTPS via Caddy):".to_string());
            lines.push(format!("    - {}", services.health_check_url));
        } else {
            lines.push("  Health Check:".to_string());
            lines.push(format!("    - {}", services.health_check_url));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::command_handlers::show::info::{LocalhostServiceInfo, TlsDomainInfo};

    fn sample_http_only_services() -> ServiceInfo {
        ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],                                            // No HTTPS trackers
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            vec![],                                            // No localhost HTTP trackers
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            false,                                             // API doesn't use HTTPS
            false,                                             // API not localhost-only
            "http://10.0.0.1:1313/health_check".to_string(),   // DevSkim: ignore DS137138
            false,                                             // Health check doesn't use HTTPS
            false,                                             // Health check not localhost-only
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
            vec![],                                            // No localhost HTTP trackers
            "https://api.tracker.local/api".to_string(),
            true,                                            // API uses HTTPS
            false,                                           // API not localhost-only
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,                                           // Health check doesn't use HTTPS (yet)
            false,                                           // Health check not localhost-only
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
            vec![],                                 // No localhost HTTP trackers
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false, // API not localhost-only
            "https://health.tracker.local/health_check".to_string(),
            true,  // Health check uses HTTPS
            false, // Health check not localhost-only
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
            vec![],                                 // No localhost HTTP trackers
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,                                           // API not localhost-only
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,                                           // Health check doesn't use HTTPS
            false,                                           // Health check not localhost-only
            vec![],
        );

        let lines = TrackerServicesView::render(&services);
        assert!(!lines.iter().any(|l| l.contains("HTTP Trackers")));
    }

    #[test]
    fn it_should_render_localhost_only_api() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "http://127.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            true,                                            // API is localhost-only
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let lines = TrackerServicesView::render(&services);
        assert!(lines
            .iter()
            .any(|l| l.contains("API Endpoint (internal only):")));
        assert!(lines.iter().any(|l| l.contains("access via SSH tunnel")));
    }

    #[test]
    fn it_should_render_localhost_only_health_check() {
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
            true, // Health check is localhost-only
            vec![],
        );

        let lines = TrackerServicesView::render(&services);
        assert!(lines
            .iter()
            .any(|l| l.contains("Health Check (internal only):")));
        assert!(lines.iter().any(|l| l.contains("access via SSH tunnel")));
    }

    #[test]
    fn it_should_render_localhost_only_http_trackers() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![
                LocalhostServiceInfo {
                    service_name: "http_tracker_1".to_string(),
                    port: 7070,
                },
                LocalhostServiceInfo {
                    service_name: "http_tracker_2".to_string(),
                    port: 7071,
                },
            ],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let lines = TrackerServicesView::render(&services);
        assert!(lines
            .iter()
            .any(|l| l.contains("HTTP Trackers (internal only):")));
        assert!(lines
            .iter()
            .any(|l| l.contains("http_tracker_1 - localhost:7070")));
        assert!(lines
            .iter()
            .any(|l| l.contains("http_tracker_2 - localhost:7071")));
        assert!(
            lines
                .iter()
                .filter(|l| l.contains("access via SSH tunnel"))
                .count()
                >= 2
        );
    }
}
