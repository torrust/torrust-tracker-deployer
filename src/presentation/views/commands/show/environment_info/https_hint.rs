//! HTTPS Hint View
//!
//! This module provides a view for rendering the /etc/hosts hint
//! when HTTPS/TLS is configured for services.

use std::net::IpAddr;

use crate::application::command_handlers::show::info::ServiceInfo;

/// View for rendering HTTPS configuration hints
///
/// This view displays helpful information about accessing TLS-enabled services,
/// including the /etc/hosts entry needed for local domains and a note about
/// internal ports not being directly accessible.
pub struct HttpsHintView;

impl HttpsHintView {
    /// Render HTTPS hint information as formatted lines
    ///
    /// Only renders content if there are TLS-enabled services.
    ///
    /// # Arguments
    ///
    /// * `services` - Service information containing TLS domain info
    /// * `instance_ip` - Optional instance IP for /etc/hosts entry
    ///
    /// # Returns
    ///
    /// A vector of formatted lines ready to be joined.
    /// Returns empty vector if no TLS is configured.
    #[must_use]
    pub fn render(services: &ServiceInfo, instance_ip: Option<IpAddr>) -> Vec<String> {
        if !services.has_any_tls() {
            return vec![];
        }

        let mut lines = vec![
            String::new(), // blank line
            "Note: HTTPS services require domain-based access. For local domains (*.local),"
                .to_string(),
            "add the following to your /etc/hosts file:".to_string(),
            String::new(), // blank line
        ];

        // Build /etc/hosts entry
        if let Some(ip) = instance_ip {
            let domains = services.tls_domain_names().join(" ");
            lines.push(format!("  {ip}   {domains}"));
        }

        lines.push(String::new()); // blank line

        // Internal ports note
        let ports: Vec<String> = services
            .unexposed_ports()
            .iter()
            .map(ToString::to_string)
            .collect();
        lines.push(format!(
            "Internal ports ({}) are not directly accessible when TLS is enabled.",
            ports.join(", ")
        ));

        lines
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;
    use crate::application::command_handlers::show::info::TlsDomainInfo;

    fn services_without_tls() -> ServiceInfo {
        ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],                                            // No HTTPS trackers
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,                                           // Health check doesn't use HTTPS
            vec![],                                          // No TLS domains
        )
    }

    fn services_with_tls() -> ServiceInfo {
        ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec!["https://http1.tracker.local/announce".to_string()],
            vec![],
            "https://api.tracker.local/api".to_string(),
            true,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            false,                                           // Health check doesn't use HTTPS
            vec![
                TlsDomainInfo::new("api.tracker.local".to_string(), 1212),
                TlsDomainInfo::new("http1.tracker.local".to_string(), 7070),
                TlsDomainInfo::new("grafana.tracker.local".to_string(), 3000),
            ],
        )
    }

    #[test]
    fn it_should_return_empty_when_no_tls_configured() {
        let lines = HttpsHintView::render(&services_without_tls(), None);
        assert!(lines.is_empty());
    }

    #[test]
    fn it_should_render_note_about_domain_access() {
        let ip = IpAddr::V4(Ipv4Addr::new(10, 140, 190, 214));
        let lines = HttpsHintView::render(&services_with_tls(), Some(ip));
        assert!(lines
            .iter()
            .any(|l| l.contains("HTTPS services require domain-based access")));
    }

    #[test]
    fn it_should_render_etc_hosts_instruction() {
        let ip = IpAddr::V4(Ipv4Addr::new(10, 140, 190, 214));
        let lines = HttpsHintView::render(&services_with_tls(), Some(ip));
        assert!(lines.iter().any(|l| l.contains("/etc/hosts")));
    }

    #[test]
    fn it_should_render_etc_hosts_entry_with_ip_and_domains() {
        let ip = IpAddr::V4(Ipv4Addr::new(10, 140, 190, 214));
        let lines = HttpsHintView::render(&services_with_tls(), Some(ip));
        assert!(lines.iter().any(|l| l.contains("10.140.190.214")));
        assert!(lines.iter().any(|l| l.contains("api.tracker.local")));
        assert!(lines.iter().any(|l| l.contains("http1.tracker.local")));
        assert!(lines.iter().any(|l| l.contains("grafana.tracker.local")));
    }

    #[test]
    fn it_should_render_internal_ports_note() {
        let ip = IpAddr::V4(Ipv4Addr::new(10, 140, 190, 214));
        let lines = HttpsHintView::render(&services_with_tls(), Some(ip));
        assert!(lines
            .iter()
            .any(|l| l.contains("Internal ports") && l.contains("not directly accessible")));
    }

    #[test]
    fn it_should_list_unexposed_ports() {
        let ip = IpAddr::V4(Ipv4Addr::new(10, 140, 190, 214));
        let lines = HttpsHintView::render(&services_with_tls(), Some(ip));
        let ports_line = lines.iter().find(|l| l.contains("Internal ports")).unwrap();
        assert!(ports_line.contains("1212"));
        assert!(ports_line.contains("7070"));
        assert!(ports_line.contains("3000"));
    }

    #[test]
    fn it_should_still_render_message_without_ip() {
        let lines = HttpsHintView::render(&services_with_tls(), None);
        assert!(lines
            .iter()
            .any(|l| l.contains("HTTPS services require domain-based access")));
        // But no IP in the /etc/hosts entry
        assert!(!lines.iter().any(|l| l.contains("10.140")));
    }
}
