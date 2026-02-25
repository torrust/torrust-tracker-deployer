//! DNS Setup Reminder View for Provision Command
//!
//! This module provides a view for rendering DNS setup reminders after
//! successful infrastructure provisioning when domains are configured.

use std::fmt::Write;
use std::net::IpAddr;

use crate::application::command_handlers::show::info::ServiceInfo;

/// DNS reminder data for rendering
///
/// This struct holds all the data needed to render DNS setup reminders
/// for a provisioned instance with configured domains.
#[derive(Debug, Clone)]
pub struct DnsReminderData {
    /// Instance IP address
    pub instance_ip: IpAddr,
    /// List of all configured domains
    pub domains: Vec<String>,
}

/// View for rendering DNS setup reminders
///
/// This view is responsible for formatting and rendering DNS setup information
/// that users need to configure after provisioning when domains are used.
///
/// # Design
///
/// Following MVC pattern, this view:
/// - Receives data from the controller
/// - Formats the output for display
/// - Only displays when domains are actually configured
/// - Returns a string ready for output to stdout
///
/// # Examples
///
/// ```rust
/// use std::net::{IpAddr, Ipv4Addr};
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::provision::view_data::dns_reminder::DnsReminderData;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::provision::DnsReminderView;
///
/// let data = DnsReminderData {
///     instance_ip: IpAddr::V4(Ipv4Addr::new(10, 140, 190, 171)),
///     domains: vec![
///         "http.tracker.example.com".to_string(),
///         "api.tracker.example.com".to_string(),
///         "grafana.example.com".to_string(),
///     ],
/// };
///
/// let output = DnsReminderView::render(&data);
/// assert!(output.contains("DNS Setup Required"));
/// assert!(output.contains("http.tracker.example.com"));
/// ```
pub struct DnsReminderView;

impl DnsReminderView {
    /// Render DNS setup reminder as a formatted string
    ///
    /// Takes DNS reminder data and produces a human-readable output suitable
    /// for displaying to users via stdout.
    ///
    /// # Arguments
    ///
    /// * `data` - DNS reminder data to render
    ///
    /// # Returns
    ///
    /// A formatted string containing:
    /// - Warning icon and header
    /// - Explanation message
    /// - Server IP address
    /// - List of all configured domains
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::net::{IpAddr, Ipv4Addr};
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::provision::view_data::dns_reminder::DnsReminderData;
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::provision::DnsReminderView;
    ///
    /// let data = DnsReminderData {
    ///     instance_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
    ///     domains: vec![
    ///         "tracker.example.com".to_string(),
    ///     ],
    /// };
    ///
    /// let output = DnsReminderView::render(&data);
    /// assert!(output.contains("192.168.1.100"));
    /// assert!(output.contains("tracker.example.com"));
    /// ```
    #[must_use]
    pub fn render(data: &DnsReminderData) -> String {
        let mut output = String::new();

        output.push_str("\n⚠️  DNS Setup Required:\n");
        output.push_str(
            "  Your configuration uses custom domains. Remember to update your DNS records\n",
        );
        let _ = writeln!(
            output,
            "  to point your domains to the server IP: {}",
            data.instance_ip
        );
        output.push_str("\n  Configured domains:\n");

        for domain in &data.domains {
            let _ = writeln!(output, "    - {domain}");
        }

        output
    }

    /// Extract all domains from `ServiceInfo`
    ///
    /// This helper method collects all unique domains from the service configuration,
    /// including domains from HTTP trackers, API, health check, and Grafana.
    ///
    /// # Arguments
    ///
    /// * `services` - Service information containing domain configuration
    ///
    /// # Returns
    ///
    /// A vector of unique domain names, or empty vector if no domains are configured.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::show::info::{ServiceInfo, TlsDomainInfo};
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::provision::DnsReminderView;
    ///
    /// let services = ServiceInfo::new(
    ///     vec![],
    ///     vec!["https://http.tracker.local/announce".to_string()],
    ///     vec![],
    ///     vec![],
    ///     "https://api.tracker.local/api".to_string(),
    ///     true,
    ///     false,
    ///     "https://health.tracker.local/health_check".to_string(),
    ///     true,
    ///     false,
    ///     vec![
    ///         TlsDomainInfo::new("http.tracker.local".to_string(), 7070),
    ///         TlsDomainInfo::new("api.tracker.local".to_string(), 1212),
    ///         TlsDomainInfo::new("health.tracker.local".to_string(), 1313),
    ///         TlsDomainInfo::new("grafana.tracker.local".to_string(), 3000),
    ///     ],
    /// );
    ///
    /// let domains = DnsReminderView::extract_all_domains(&services);
    /// assert_eq!(domains.len(), 4);
    /// assert!(domains.contains(&"http.tracker.local".to_string()));
    /// assert!(domains.contains(&"api.tracker.local".to_string()));
    /// ```
    #[must_use]
    pub fn extract_all_domains(services: &ServiceInfo) -> Vec<String> {
        // Currently, ServiceInfo only tracks TLS domains
        // This returns all domain names from tls_domains
        services
            .tls_domain_names()
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    use crate::application::command_handlers::show::info::TlsDomainInfo;

    #[test]
    fn it_should_render_dns_reminder_with_single_domain() {
        let data = DnsReminderData {
            instance_ip: IpAddr::V4(Ipv4Addr::new(10, 140, 190, 171)),
            domains: vec!["tracker.example.com".to_string()],
        };

        let output = DnsReminderView::render(&data);

        assert!(output.contains("⚠️  DNS Setup Required:"));
        assert!(output.contains("10.140.190.171"));
        assert!(output.contains("tracker.example.com"));
        assert!(output.contains("Configured domains:"));
    }

    #[test]
    fn it_should_render_dns_reminder_with_multiple_domains() {
        let data = DnsReminderData {
            instance_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            domains: vec![
                "http.tracker.example.com".to_string(),
                "api.tracker.example.com".to_string(),
                "grafana.example.com".to_string(),
            ],
        };

        let output = DnsReminderView::render(&data);

        assert!(output.contains("DNS Setup Required"));
        assert!(output.contains("192.168.1.100"));
        assert!(output.contains("http.tracker.example.com"));
        assert!(output.contains("api.tracker.example.com"));
        assert!(output.contains("grafana.example.com"));
    }

    #[test]
    fn it_should_extract_all_domains_from_service_info() {
        let services = ServiceInfo::new(
            vec![],
            vec!["https://http.tracker.local/announce".to_string()],
            vec![],
            vec![],
            "https://api.tracker.local/api".to_string(),
            true,
            false,
            "https://health.tracker.local/health_check".to_string(),
            true,
            false,
            vec![
                TlsDomainInfo::new("http.tracker.local".to_string(), 7070),
                TlsDomainInfo::new("api.tracker.local".to_string(), 1212),
                TlsDomainInfo::new("health.tracker.local".to_string(), 1313),
            ],
        );

        let domains = DnsReminderView::extract_all_domains(&services);

        assert_eq!(domains.len(), 3);
        assert!(domains.contains(&"http.tracker.local".to_string()));
        assert!(domains.contains(&"api.tracker.local".to_string()));
        assert!(domains.contains(&"health.tracker.local".to_string()));
    }

    #[test]
    fn it_should_return_empty_vec_when_no_domains_configured() {
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
            vec![], // No TLS domains
        );

        let domains = DnsReminderView::extract_all_domains(&services);

        assert!(domains.is_empty());
    }

    #[test]
    fn it_should_format_output_with_proper_indentation() {
        let data = DnsReminderData {
            instance_ip: IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)),
            domains: vec!["example.com".to_string()],
        };

        let output = DnsReminderView::render(&data);

        // Check for proper indentation and formatting
        assert!(output.contains("  Your configuration"));
        assert!(output.contains("  to point"));
        assert!(output.contains("  Configured domains:"));
        assert!(output.contains("    - example.com"));
    }
}
