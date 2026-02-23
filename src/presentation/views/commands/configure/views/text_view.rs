//! Text View for Configure Command
//!
//! This module provides text-based rendering for the configure command.
//! It follows the Strategy Pattern, providing a human-readable output format
//! for the same underlying data (`ConfigureDetailsData` DTO).
//!
//! # Design
//!
//! The `TextView` formats configure details as human-readable text suitable
//! for terminal display and direct user consumption.

use crate::presentation::views::commands::configure::ConfigureDetailsData;

/// View for rendering configure details as human-readable text
///
/// This view produces formatted text output suitable for terminal display
/// and human consumption. It presents environment configuration details
/// in a clear, readable format.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::views::commands::configure::{
///     ConfigureDetailsData, TextView,
/// };
/// use chrono::{TimeZone, Utc};
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let details = ConfigureDetailsData {
///     environment_name: "my-env".to_string(),
///     instance_name: "torrust-tracker-vm-my-env".to_string(),
///     provider: "lxd".to_string(),
///     state: "Configured".to_string(),
///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))),
///     created_at: Utc.with_ymd_and_hms(2026, 2, 20, 10, 0, 0).unwrap(),
/// };
///
/// let output = TextView::render(&details);
/// assert!(output.contains("Environment Details:"));
/// assert!(output.contains("my-env"));
/// ```
pub struct TextView;

impl TextView {
    /// Render configure details as human-readable formatted text
    ///
    /// Takes configure details and produces a human-readable output
    /// suitable for displaying to users via stdout.
    ///
    /// # Arguments
    ///
    /// * `data` - Configure details to render
    ///
    /// # Returns
    ///
    /// A formatted string containing:
    /// - Environment Details section with name, instance, provider, state
    /// - Instance IP (if available)
    /// - Creation timestamp
    ///
    /// # Format
    ///
    /// The output follows this structure:
    /// ```text
    /// Environment Details:
    ///   Name:              <environment_name>
    ///   Instance:          <instance_name>
    ///   Provider:          <provider>
    ///   State:             <state>
    ///   Instance IP:       <ip or "Not available">
    ///   Created:           <timestamp>
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::commands::configure::{
    ///     ConfigureDetailsData, TextView,
    /// };
    /// use chrono::{TimeZone, Utc};
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// let details = ConfigureDetailsData {
    ///     environment_name: "prod-tracker".to_string(),
    ///     instance_name: "torrust-tracker-vm-prod-tracker".to_string(),
    ///     provider: "lxd".to_string(),
    ///     state: "Configured".to_string(),
    ///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
    ///     created_at: Utc.with_ymd_and_hms(2026, 1, 5, 10, 30, 0).unwrap(),
    /// };
    ///
    /// let text = TextView::render(&details);
    ///
    /// assert!(text.contains("Name:"));
    /// assert!(text.contains("prod-tracker"));
    /// assert!(text.contains("Configured"));
    /// ```
    #[must_use]
    pub fn render(data: &ConfigureDetailsData) -> String {
        let instance_ip = data
            .instance_ip
            .map_or_else(|| "Not available".to_string(), |ip| ip.to_string());

        format!(
            r"Environment Details:
  Name:              {}
  Instance:          {}
  Provider:          {}
  State:             {}
  Instance IP:       {}
  Created:           {}",
            data.environment_name,
            data.instance_name,
            data.provider,
            data.state,
            instance_ip,
            data.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn it_should_render_configure_details_as_formatted_text() {
        // Arrange
        let details = ConfigureDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            provider: "lxd".to_string(),
            state: "Configured".to_string(),
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))),
            created_at: Utc.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap(),
        };

        // Act
        let text = TextView::render(&details);

        // Assert
        assert!(text.contains("Environment Details:"));
        assert!(text.contains("Name:"));
        assert!(text.contains("test-env"));
        assert!(text.contains("Instance:"));
        assert!(text.contains("torrust-tracker-vm-test-env"));
        assert!(text.contains("Provider:"));
        assert!(text.contains("lxd"));
        assert!(text.contains("State:"));
        assert!(text.contains("Configured"));
        assert!(text.contains("Instance IP:"));
        assert!(text.contains("10.140.190.39"));
        assert!(text.contains("Created:"));
        assert!(text.contains("2026-02-23 10:00:00 UTC"));
    }

    #[test]
    fn it_should_display_not_available_when_instance_ip_is_none() {
        // Arrange
        let details = ConfigureDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            provider: "lxd".to_string(),
            state: "Configured".to_string(),
            instance_ip: None,
            created_at: Utc.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap(),
        };

        // Act
        let text = TextView::render(&details);

        // Assert
        assert!(text.contains("Instance IP:       Not available"));
    }

    #[test]
    fn it_should_include_all_required_sections() {
        // Arrange
        let details = ConfigureDetailsData {
            environment_name: "my-env".to_string(),
            instance_name: "torrust-tracker-vm-my-env".to_string(),
            provider: "hetzner".to_string(),
            state: "Configured".to_string(),
            instance_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42))),
            created_at: Utc.with_ymd_and_hms(2026, 2, 20, 14, 30, 45).unwrap(),
        };

        // Act
        let text = TextView::render(&details);

        // Assert - check all sections are present
        assert!(text.contains("Name:"));
        assert!(text.contains("Instance:"));
        assert!(text.contains("Provider:"));
        assert!(text.contains("State:"));
        assert!(text.contains("Instance IP:"));
        assert!(text.contains("Created:"));
    }
}
