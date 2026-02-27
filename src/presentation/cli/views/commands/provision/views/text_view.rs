//! Text View for Provision Details
//!
//! This module provides text-based rendering for provision command details.
//! It follows the Strategy Pattern, providing one specific rendering strategy
//! (human-readable text) for provision details.

use crate::presentation::cli::views::{Render, ViewRenderError};
use super::super::ProvisionDetailsData;

/// Text view for rendering provision details
///
/// This view produces human-readable formatted text output suitable for
/// terminal display and human consumption.
///
/// # Design
///
/// This view is part of a Strategy Pattern implementation where:
/// - Each format (Text, JSON, XML, etc.) has its own dedicated view
/// - Adding new formats requires creating new view files, not modifying existing ones
/// - Follows Open/Closed Principle from SOLID
///
/// # Examples
///
/// ```rust
/// use std::net::{IpAddr, Ipv4Addr};
/// use std::path::PathBuf;
/// use chrono::{TimeZone, Utc};
/// use torrust_tracker_deployer_lib::domain::provider::Provider;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::provision::{
///     ProvisionDetailsData, TextView
/// };
///
/// let data = ProvisionDetailsData {
///     environment_name: "my-env".to_string(),
///     instance_name: "torrust-tracker-vm-my-env".to_string(),
///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))),
///     ssh_username: "torrust".to_string(),
///     ssh_port: 22,
///     ssh_private_key_path: PathBuf::from("/path/to/key"),
///     provider: Provider::Lxd.to_string(),
///     provisioned_at: Utc.with_ymd_and_hms(2026, 2, 16, 14, 30, 0).unwrap(),
///     domains: vec!["tracker.example.com".to_string()],
/// };
///
/// let output = TextView::render(&data);
/// assert!(output.contains("Instance Connection Details:"));
/// assert!(output.contains("10.140.190.39"));
/// ```
pub struct TextView;

impl TextView {
    /// Render provision details as human-readable formatted text
    ///
    /// Takes provision data and produces a human-readable output
    /// suitable for displaying to users via stdout.
    ///
    /// # Arguments
    ///
    /// * `data` - Provision details to render
    ///
    /// # Returns
    ///
    /// A formatted string containing:
    /// - Instance Connection Details section with IP, SSH port, private key path, username
    /// - SSH connection command (if IP is available)
    /// - DNS setup reminder (if domains are configured)
    ///
    /// # Format
    ///
    /// The output follows this structure:
    /// ```text
    /// Instance Connection Details:
    ///   IP Address:        <ip>
    ///   SSH Port:          <port>
    ///   SSH Private Key:   <path>
    ///   SSH Username:      <username>
    ///
    /// Connect using:
    ///   ssh -i <path> <username>@<ip> -p <port>
    ///
    /// ⚠️  DNS Setup Required:
    ///   Your configuration uses custom domains...
    ///   Configured domains:
    ///     - <domain1>
    ///     - <domain2>
    /// ```
    #[must_use]
    pub fn render(data: &ProvisionDetailsData) -> String {
        let mut lines = Vec::new();

        // Connection details section
        lines.push("\nInstance Connection Details:".to_string());

        if let Some(ip) = data.instance_ip {
            lines.push(format!("  IP Address:        {ip}"));
            lines.push(format!("  SSH Port:          {}", data.ssh_port));
            lines.push(format!(
                "  SSH Private Key:   {}",
                data.ssh_private_key_path.display()
            ));
            lines.push(format!("  SSH Username:      {}", data.ssh_username));
            lines.push(String::new());
            lines.push("Connect using:".to_string());
            lines.push(format!(
                "  ssh -i {} {}@{} -p {}",
                data.ssh_private_key_path.display(),
                data.ssh_username,
                ip,
                data.ssh_port
            ));
        } else {
            lines.push("  IP Address:        <not available>".to_string());
            lines.push(
                "  WARNING: Instance IP not captured - this is an unexpected state.".to_string(),
            );
            lines.push("           The environment may not be fully provisioned.".to_string());
            lines.push(format!("  SSH Port:          {}", data.ssh_port));
            lines.push(format!(
                "  SSH Private Key:   {}",
                data.ssh_private_key_path.display()
            ));
            lines.push(format!("  SSH Username:      {}", data.ssh_username));
        }

        // DNS reminder section (only if domains are configured)
        if !data.domains.is_empty() {
            if let Some(ip) = data.instance_ip {
                lines.push(String::new());
                lines.push("⚠️  DNS Setup Required:".to_string());
                lines.push(
                    "  Your configuration uses custom domains. Remember to update your DNS records"
                        .to_string(),
                );
                lines.push(format!("  to point your domains to the server IP: {ip}"));
                lines.push(String::new());
                lines.push("  Configured domains:".to_string());
                for domain in &data.domains {
                    lines.push(format!("    - {domain}"));
                }
            }
        }

        lines.join("\n")
    }
}

impl Render<ProvisionDetailsData> for TextView {
    fn render(data: &ProvisionDetailsData) -> Result<String, ViewRenderError> {
        Ok(TextView::render(data))
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use crate::domain::provider::Provider;

    fn test_timestamp() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 2, 16, 14, 30, 0).unwrap()
    }

    #[test]
    fn it_should_render_provision_details_with_https_domains() {
        // Given
        let data = ProvisionDetailsData {
            environment_name: "full-stack-lxd".to_string(),
            instance_name: "torrust-tracker-vm-full-stack-lxd".to_string(),
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))),
            ssh_username: "torrust".to_string(),
            ssh_port: 22,
            ssh_private_key_path: PathBuf::from("/path/to/testing_rsa"),
            provider: Provider::Lxd.to_string(),
            provisioned_at: test_timestamp(),
            domains: vec![
                "tracker1.example.com".to_string(),
                "tracker2.example.com".to_string(),
                "api.example.com".to_string(),
            ],
        };

        // When
        let output = TextView::render(&data);

        // Then
        assert!(output.contains("Instance Connection Details:"));
        assert!(output.contains("10.140.190.39"));
        assert!(output.contains("SSH Port:          22"));
        assert!(output.contains("SSH Username:      torrust"));
        assert!(output.contains("Connect using:"));
        assert!(output.contains("ssh -i"));
        assert!(output.contains("⚠️  DNS Setup Required:"));
        assert!(output.contains("tracker1.example.com"));
        assert!(output.contains("tracker2.example.com"));
        assert!(output.contains("api.example.com"));
    }

    #[test]
    fn it_should_render_provision_details_without_https_domains() {
        // Given
        let data = ProvisionDetailsData {
            environment_name: "simple-tracker".to_string(),
            instance_name: "torrust-tracker-vm-simple-tracker".to_string(),
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 40))),
            ssh_username: "torrust".to_string(),
            ssh_port: 22,
            ssh_private_key_path: PathBuf::from("/path/to/testing_rsa"),
            provider: Provider::Lxd.to_string(),
            provisioned_at: test_timestamp(),
            domains: vec![],
        };

        // When
        let output = TextView::render(&data);

        // Then
        assert!(output.contains("Instance Connection Details:"));
        assert!(output.contains("10.140.190.40"));
        assert!(output.contains("SSH Port:          22"));
        assert!(output.contains("Connect using:"));
        assert!(!output.contains("⚠️  DNS Setup Required:"));
    }

    #[test]
    fn it_should_render_warning_when_ip_is_missing() {
        // Given
        let data = ProvisionDetailsData {
            environment_name: "broken-env".to_string(),
            instance_name: "torrust-tracker-vm-broken-env".to_string(),
            instance_ip: None,
            ssh_username: "torrust".to_string(),
            ssh_port: 22,
            ssh_private_key_path: PathBuf::from("/path/to/testing_rsa"),
            provider: Provider::Lxd.to_string(),
            provisioned_at: test_timestamp(),
            domains: vec![],
        };

        // When
        let output = TextView::render(&data);

        // Then
        assert!(output.contains("IP Address:        <not available>"));
        assert!(output.contains("WARNING: Instance IP not captured"));
        assert!(!output.contains("Connect using:"));
    }
}
