//! DNS Configuration Hint View
//!
//! This module provides a view for rendering DNS configuration hints
//! when TLS/HTTPS services are configured.

use crate::application::command_handlers::show::info::ServiceInfo;

/// View for rendering DNS configuration hints
///
/// This view displays a note about DNS configuration requirements
/// when HTTPS services are configured.
pub struct DnsHintView;

impl DnsHintView {
    /// Render DNS configuration hint if TLS is configured
    ///
    /// # Arguments
    ///
    /// * `services` - Service information to check for HTTPS usage
    ///
    /// # Returns
    ///
    /// An optional string with DNS configuration hint if HTTPS is configured
    #[must_use]
    pub fn render(services: &ServiceInfo) -> Option<String> {
        if Self::has_https_services(services) {
            Some(
                "\nNote: HTTPS services require DNS configuration. See 'show' command for details."
                    .to_string(),
            )
        } else {
            None
        }
    }

    /// Check if any service uses HTTPS
    fn has_https_services(services: &ServiceInfo) -> bool {
        !services.https_http_trackers.is_empty()
            || services.api_uses_https
            || services.health_check_uses_https
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_none_when_no_https_services() {
        let services = ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],                                            // No HTTPS trackers
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,                                  // No HTTPS API
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,                                           // No HTTPS health check
            false,
            vec![],
        );

        let result = DnsHintView::render(&services);

        assert!(result.is_none());
    }

    #[test]
    fn it_should_return_hint_when_https_trackers_configured() {
        let services = ServiceInfo::new(
            vec![],
            vec!["https://http.tracker.local/announce".to_string()], // HTTPS tracker
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

        let result = DnsHintView::render(&services);

        assert!(result.is_some());
        assert!(result.unwrap().contains("DNS configuration"));
    }

    #[test]
    fn it_should_return_hint_when_https_api_configured() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "https://api.tracker.local/api".to_string(),
            true, // HTTPS API
            true,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            vec![],
        );

        let result = DnsHintView::render(&services);

        assert!(result.is_some());
        assert!(result.unwrap().contains("DNS configuration"));
    }

    #[test]
    fn it_should_return_hint_when_https_health_check_configured() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "http://10.0.0.1:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            false,
            "https://health.tracker.local/health_check".to_string(),
            true, // HTTPS health check
            true,
            vec![],
        );

        let result = DnsHintView::render(&services);

        assert!(result.is_some());
        assert!(result.unwrap().contains("DNS configuration"));
    }

    #[test]
    fn it_should_mention_show_command() {
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

        let result = DnsHintView::render(&services);

        assert!(result.is_some());
        let hint = result.unwrap();
        assert!(hint.contains("show"));
        assert!(hint.contains("details"));
    }
}
