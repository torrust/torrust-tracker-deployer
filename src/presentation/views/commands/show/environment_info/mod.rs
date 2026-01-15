//! Environment Information View for Show Command
//!
//! This module provides a view for rendering environment information
//! with state-aware details.
//!
//! # Module Structure
//!
//! The view is composed of specialized child views for each section:
//! - `basic`: Basic environment info (name, state, provider, created)
//! - `infrastructure`: Infrastructure details (IP, SSH credentials)
//! - `tracker_services`: Tracker service endpoints
//! - `prometheus`: Prometheus metrics service
//! - `grafana`: Grafana visualization service
//! - `https_hint`: HTTPS configuration hints (/etc/hosts)
//! - `next_step`: State-aware guidance

mod basic;
mod grafana;
mod https_hint;
mod infrastructure;
mod next_step;
mod prometheus;
mod tracker_services;

use basic::BasicInfoView;
use grafana::GrafanaView;
use https_hint::HttpsHintView;
use infrastructure::InfrastructureView;
use next_step::NextStepGuidanceView;
use prometheus::PrometheusView;
use tracker_services::TrackerServicesView;

use crate::application::command_handlers::show::info::EnvironmentInfo;

/// View for rendering environment information
///
/// This view is responsible for formatting and rendering the environment
/// information that users see when running the `show` command.
///
/// # Design
///
/// Following MVC pattern with composition, this view:
/// - Receives data from the controller via the `EnvironmentInfo` DTO
/// - Delegates rendering to specialized child views
/// - Composes the final output from child view results
/// - Returns a string ready for output to stdout
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::command_handlers::show::info::EnvironmentInfo;
/// use torrust_tracker_deployer_lib::presentation::views::commands::show::EnvironmentInfoView;
/// use chrono::{TimeZone, Utc};
///
/// let created_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
/// let info = EnvironmentInfo::new(
///     "my-env".to_string(),
///     "Created".to_string(),
///     "LXD".to_string(),
///     created_at,
///     "created".to_string(),
/// );
///
/// let output = EnvironmentInfoView::render(&info);
/// assert!(output.contains("Environment: my-env"));
/// assert!(output.contains("State: Created"));
/// ```
pub struct EnvironmentInfoView;

impl EnvironmentInfoView {
    /// Render environment information as a formatted string
    ///
    /// Takes environment info and produces a human-readable output suitable
    /// for displaying to users via stdout. Uses composition to delegate
    /// rendering to specialized child views.
    ///
    /// # Arguments
    ///
    /// * `info` - Environment information to render
    ///
    /// # Returns
    ///
    /// A formatted string containing:
    /// - Basic information (name, state, provider)
    /// - Infrastructure details (if available)
    /// - Service information (if available, for Released/Running states)
    /// - Next step guidance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::show::info::{
    ///     EnvironmentInfo, InfrastructureInfo,
    /// };
    /// use torrust_tracker_deployer_lib::presentation::views::commands::show::EnvironmentInfoView;
    /// use std::net::{IpAddr, Ipv4Addr};
    /// use chrono::Utc;
    ///
    /// let info = EnvironmentInfo::new(
    ///     "prod-env".to_string(),
    ///     "Provisioned".to_string(),
    ///     "LXD".to_string(),
    ///     Utc::now(),
    ///     "provisioned".to_string(),
    /// ).with_infrastructure(InfrastructureInfo::new(
    ///     IpAddr::V4(Ipv4Addr::new(10, 140, 190, 171)),
    ///     22,
    ///     "torrust".to_string(),
    ///     "~/.ssh/id_rsa".to_string(),
    /// ));
    ///
    /// let output = EnvironmentInfoView::render(&info);
    /// assert!(output.contains("10.140.190.171"));
    /// assert!(output.contains("ssh -i"));
    /// ```
    #[must_use]
    pub fn render(info: &EnvironmentInfo) -> String {
        let mut lines = Vec::new();

        // Basic information (always present)
        lines.extend(BasicInfoView::render(
            &info.name,
            &info.state,
            &info.provider,
            info.created_at,
        ));

        // Infrastructure details (if available)
        if let Some(ref infra) = info.infrastructure {
            lines.extend(InfrastructureView::render(infra));
        }

        // Tracker service information (if available)
        if let Some(ref services) = info.services {
            lines.extend(TrackerServicesView::render(services));
        }

        // Prometheus service (if configured)
        if let Some(ref prometheus) = info.prometheus {
            lines.extend(PrometheusView::render(prometheus));
        }

        // Grafana service (if configured)
        if let Some(ref grafana) = info.grafana {
            lines.extend(GrafanaView::render(grafana));
        }

        // HTTPS hint with /etc/hosts (if TLS is configured)
        if let Some(ref services) = info.services {
            let instance_ip = info.infrastructure.as_ref().map(|i| i.instance_ip);
            lines.extend(HttpsHintView::render(services, instance_ip));
        }

        // Next step guidance (always present)
        lines.extend(NextStepGuidanceView::render(&info.state_name));

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::application::command_handlers::show::info::{
        InfrastructureInfo, ServiceInfo, TlsDomainInfo,
    };

    /// Helper to create a fixed test timestamp
    fn test_timestamp() -> chrono::DateTime<chrono::Utc> {
        Utc.with_ymd_and_hms(2025, 1, 7, 12, 30, 45).unwrap()
    }

    #[test]
    fn it_should_render_basic_environment_info() {
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Created".to_string(),
            "LXD".to_string(),
            test_timestamp(),
            "created".to_string(),
        );

        let output = EnvironmentInfoView::render(&info);

        assert!(output.contains("Environment: test-env"));
        assert!(output.contains("State: Created"));
        assert!(output.contains("Provider: LXD"));
        assert!(output.contains("Created: 2025-01-07 12:30:45 UTC"));
        assert!(output.contains("Run 'provision' to create infrastructure."));
    }

    #[test]
    fn it_should_render_infrastructure_details_when_available() {
        let info = EnvironmentInfo::new(
            "prod-env".to_string(),
            "Provisioned".to_string(),
            "LXD".to_string(),
            test_timestamp(),
            "provisioned".to_string(),
        )
        .with_infrastructure(InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 140, 190, 171)),
            22,
            "torrust".to_string(),
            "~/.ssh/id_rsa".to_string(),
        ));

        let output = EnvironmentInfoView::render(&info);

        assert!(output.contains("Infrastructure:"));
        assert!(output.contains("Instance IP: 10.140.190.171"));
        assert!(output.contains("SSH Port: 22"));
        assert!(output.contains("SSH User: torrust"));
        assert!(output.contains("SSH Key: ~/.ssh/id_rsa"));
        assert!(output.contains("Connection:"));
        assert!(output.contains("ssh -i"));
    }

    #[test]
    fn it_should_render_service_info_when_available() {
        let info = EnvironmentInfo::new(
            "running-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            test_timestamp(),
            "running".to_string(),
        )
        .with_services(ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],                                            // No HTTPS trackers
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            false,                                             // API doesn't use HTTPS
            "http://10.0.0.1:1313/health_check".to_string(),   // DevSkim: ignore DS137138
            vec![],                                            // No TLS domains
        ));

        let output = EnvironmentInfoView::render(&info);

        assert!(output.contains("Tracker Services:"));
        assert!(output.contains("UDP Trackers:"));
        assert!(output.contains("udp://10.0.0.1:6969/announce"));
        assert!(output.contains("HTTP Trackers (direct):"));
        assert!(output.contains("http://10.0.0.1:7070/announce")); // DevSkim: ignore DS137138
        assert!(output.contains("API Endpoint:"));
        assert!(output.contains("http://10.0.0.1:1212/api")); // DevSkim: ignore DS137138
        assert!(output.contains("Health Check:"));
        assert!(output.contains("http://10.0.0.1:1313/health_check")); // DevSkim: ignore DS137138
    }

    #[test]
    fn it_should_render_complete_info_with_infrastructure_and_services() {
        let info = EnvironmentInfo::new(
            "full-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            test_timestamp(),
            "running".to_string(),
        )
        .with_infrastructure(InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            2222,
            "admin".to_string(),
            "/path/to/key".to_string(),
        ))
        .with_services(ServiceInfo::new(
            vec!["udp://192.168.1.100:6969/announce".to_string()],
            vec![],                                      // No HTTPS trackers
            vec![],                                      // No direct trackers
            "http://192.168.1.100:1212/api".to_string(), // DevSkim: ignore DS137138
            false,
            "http://192.168.1.100:1313/health_check".to_string(), // DevSkim: ignore DS137138
            vec![],
        ));

        let output = EnvironmentInfoView::render(&info);

        // Should have all sections
        assert!(output.contains("Environment: full-env"));
        assert!(output.contains("Infrastructure:"));
        assert!(output.contains("192.168.1.100"));
        assert!(output.contains("Tracker Services:"));
        assert!(output.contains("UDP Trackers:"));
        // Should not have HTTP Trackers section when empty
        assert!(!output.contains("HTTP Trackers"));
    }

    #[test]
    fn it_should_render_https_services_with_hosts_hint() {
        let info = EnvironmentInfo::new(
            "https-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            test_timestamp(),
            "running".to_string(),
        )
        .with_infrastructure(InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 140, 190, 214)),
            22,
            "torrust".to_string(),
            "~/.ssh/id_rsa".to_string(),
        ))
        .with_services(ServiceInfo::new(
            vec!["udp://10.140.190.214:6969/announce".to_string()],
            vec![
                "https://http1.tracker.local/announce".to_string(),
                "https://http2.tracker.local/announce".to_string(),
            ],
            vec!["http://10.140.190.214:7072/announce".to_string()], // DevSkim: ignore DS137138
            "https://api.tracker.local/api".to_string(),
            true,                                                  // API uses HTTPS
            "http://10.140.190.214:1313/health_check".to_string(), // DevSkim: ignore DS137138
            vec![
                TlsDomainInfo::new("api.tracker.local".to_string(), 1212),
                TlsDomainInfo::new("http1.tracker.local".to_string(), 7070),
                TlsDomainInfo::new("http2.tracker.local".to_string(), 7071),
                TlsDomainInfo::new("grafana.tracker.local".to_string(), 3000),
            ],
        ));

        let output = EnvironmentInfoView::render(&info);

        // Check HTTPS trackers section
        assert!(output.contains("HTTP Trackers (HTTPS via Caddy):"));
        assert!(output.contains("https://http1.tracker.local/announce"));
        assert!(output.contains("https://http2.tracker.local/announce"));

        // Check direct HTTP trackers section
        assert!(output.contains("HTTP Trackers (direct):"));
        assert!(output.contains("http://10.140.190.214:7072/announce")); // DevSkim: ignore DS137138

        // Check API shows HTTPS
        assert!(output.contains("API Endpoint (HTTPS via Caddy):"));
        assert!(output.contains("https://api.tracker.local/api"));

        // Check /etc/hosts hint
        assert!(output.contains("Note: HTTPS services require domain-based access"));
        assert!(output.contains("/etc/hosts"));
        assert!(output.contains("10.140.190.214"));
        assert!(output.contains("api.tracker.local"));
        assert!(output.contains("http1.tracker.local"));
        assert!(output.contains("grafana.tracker.local"));

        // Check unexposed ports message
        assert!(output.contains("Internal ports"));
        assert!(output.contains("not directly accessible when TLS is enabled"));
    }

    #[test]
    fn it_should_include_port_in_ssh_command_when_non_standard() {
        let info = EnvironmentInfo::new(
            "custom-port-env".to_string(),
            "Provisioned".to_string(),
            "LXD".to_string(),
            test_timestamp(),
            "provisioned".to_string(),
        )
        .with_infrastructure(InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            2222,
            "user".to_string(),
            "/key".to_string(),
        ));

        let output = EnvironmentInfoView::render(&info);

        assert!(output.contains("-p 2222"));
    }
}
