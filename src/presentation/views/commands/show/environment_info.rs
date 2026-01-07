//! Environment Information View for Show Command
//!
//! This module provides a view for rendering environment information
//! with state-aware details.

use crate::application::command_handlers::show::info::EnvironmentInfo;

/// View for rendering environment information
///
/// This view is responsible for formatting and rendering the environment
/// information that users see when running the `show` command.
///
/// # Design
///
/// Following MVC pattern, this view:
/// - Receives data from the controller via the `EnvironmentInfo` DTO
/// - Formats the output for display
/// - Handles optional fields gracefully (infrastructure, services)
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
///     "Next: Run 'provision my-env' to create infrastructure".to_string(),
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
    /// for displaying to users via stdout.
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
    ///     "Next: Run 'configure prod-env' to install software".to_string(),
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

        // Basic information
        lines.push(String::new()); // blank line
        lines.push(format!("Environment: {}", info.name));
        lines.push(format!("State: {}", info.state));
        lines.push(format!("Provider: {}", info.provider));
        lines.push(format!(
            "Created: {}",
            info.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Infrastructure details (if available)
        if let Some(ref infra) = info.infrastructure {
            lines.push(String::new()); // blank line
            lines.push("Infrastructure:".to_string());
            lines.push(format!("  Instance IP: {}", infra.instance_ip));
            lines.push(format!("  SSH Port: {}", infra.ssh_port));
            lines.push(format!("  SSH User: {}", infra.ssh_user));
            lines.push(format!("  SSH Key: {}", infra.ssh_key_path));
            lines.push(String::new()); // blank line
            lines.push("Connection:".to_string());
            lines.push(format!("  {}", infra.ssh_command()));
        }

        // Service information (if available)
        if let Some(ref services) = info.services {
            lines.push(String::new()); // blank line
            lines.push("Tracker Services:".to_string());

            if !services.udp_trackers.is_empty() {
                lines.push("  UDP Trackers:".to_string());
                for url in &services.udp_trackers {
                    lines.push(format!("    - {url}"));
                }
            }

            if !services.http_trackers.is_empty() {
                lines.push("  HTTP Trackers:".to_string());
                for url in &services.http_trackers {
                    lines.push(format!("    - {url}"));
                }
            }

            lines.push("  API Endpoint:".to_string());
            lines.push(format!("    - {}", services.api_endpoint));

            lines.push("  Health Check:".to_string());
            lines.push(format!("    - {}", services.health_check_url));
        }

        // Next step guidance
        lines.push(String::new()); // blank line
        lines.push(info.next_step.clone());

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::application::command_handlers::show::info::{InfrastructureInfo, ServiceInfo};

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
            "Next: Run 'provision test-env' to create infrastructure".to_string(),
        );

        let output = EnvironmentInfoView::render(&info);

        assert!(output.contains("Environment: test-env"));
        assert!(output.contains("State: Created"));
        assert!(output.contains("Provider: LXD"));
        assert!(output.contains("Created: 2025-01-07 12:30:45 UTC"));
        assert!(output.contains("Next: Run 'provision test-env'"));
    }

    #[test]
    fn it_should_render_infrastructure_details_when_available() {
        let info = EnvironmentInfo::new(
            "prod-env".to_string(),
            "Provisioned".to_string(),
            "LXD".to_string(),
            test_timestamp(),
            "Next: Run 'configure prod-env' to install software".to_string(),
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
            "Tracker is running!".to_string(),
        )
        .with_services(ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            "http://10.0.0.1:1313/health_check".to_string(),   // DevSkim: ignore DS137138
        ));

        let output = EnvironmentInfoView::render(&info);

        assert!(output.contains("Tracker Services:"));
        assert!(output.contains("UDP Trackers:"));
        assert!(output.contains("udp://10.0.0.1:6969/announce"));
        assert!(output.contains("HTTP Trackers:"));
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
            "Tracker is running!".to_string(),
        )
        .with_infrastructure(InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            2222,
            "admin".to_string(),
            "/path/to/key".to_string(),
        ))
        .with_services(ServiceInfo::new(
            vec!["udp://192.168.1.100:6969/announce".to_string()],
            vec![],
            "http://192.168.1.100:1212/api".to_string(), // DevSkim: ignore DS137138
            "http://192.168.1.100:1313/health_check".to_string(), // DevSkim: ignore DS137138
        ));

        let output = EnvironmentInfoView::render(&info);

        // Should have all sections
        assert!(output.contains("Environment: full-env"));
        assert!(output.contains("Infrastructure:"));
        assert!(output.contains("192.168.1.100"));
        assert!(output.contains("Tracker Services:"));
        assert!(output.contains("UDP Trackers:"));
        // Should not have HTTP Trackers section when empty
        assert!(!output.contains("HTTP Trackers:"));
    }

    #[test]
    fn it_should_include_port_in_ssh_command_when_non_standard() {
        let info = EnvironmentInfo::new(
            "custom-port-env".to_string(),
            "Provisioned".to_string(),
            "LXD".to_string(),
            test_timestamp(),
            "Next step".to_string(),
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
