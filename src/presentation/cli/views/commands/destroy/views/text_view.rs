//! Text View for Destroy Command
//!
//! This module provides text-based rendering for the destroy command.
//! It follows the Strategy Pattern, providing a human-readable output format
//! for the same underlying data (`DestroyDetailsData` DTO).
//!
//! # Design
//!
//! The `TextView` formats destroy details as human-readable text suitable
//! for terminal display and direct user consumption.

use crate::presentation::cli::views::commands::destroy::DestroyDetailsData;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering destroy details as human-readable text
///
/// This view produces formatted text output suitable for terminal display
/// and human consumption. It presents environment destroy details
/// in a clear, readable format.
///
/// # Examples
///
/// ```rust
/// # use torrust_tracker_deployer_lib::presentation::cli::views::Render;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::destroy::{
///     DestroyDetailsData, TextView,
/// };
/// use chrono::{TimeZone, Utc};
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let details = DestroyDetailsData {
///     environment_name: "my-env".to_string(),
///     instance_name: "torrust-tracker-vm-my-env".to_string(),
///     provider: "lxd".to_string(),
///     state: "Destroyed".to_string(),
///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))),
///     created_at: Utc.with_ymd_and_hms(2026, 2, 20, 10, 0, 0).unwrap(),
/// };
///
/// let output = TextView::render(&details).unwrap();
/// assert!(output.contains("Environment Details:"));
/// assert!(output.contains("my-env"));
/// ```
pub struct TextView;

impl Render<DestroyDetailsData> for TextView {
    fn render(data: &DestroyDetailsData) -> Result<String, ViewRenderError> {
        let instance_ip = data
            .instance_ip
            .map_or_else(|| "Not available".to_string(), |ip| ip.to_string());

        Ok(format!(
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
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use std::net::{IpAddr, Ipv4Addr};

    // Test fixtures and helpers

    fn create_test_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap()
    }

    fn create_test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))
    }

    fn create_test_details_with_ip(ip: Option<IpAddr>) -> DestroyDetailsData {
        DestroyDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            provider: "lxd".to_string(),
            state: "Destroyed".to_string(),
            instance_ip: ip,
            created_at: create_test_timestamp(),
        }
    }

    /// Helper to assert text contains all expected substrings
    fn assert_contains_all(text: &str, expected: &[&str]) {
        for substring in expected {
            assert!(
                text.contains(substring),
                "Expected text to contain '{substring}' but it didn't.\nActual text:\n{text}"
            );
        }
    }

    // Tests

    #[test]
    fn it_should_render_destroy_details_as_formatted_text() {
        // Arrange
        let details = create_test_details_with_ip(Some(create_test_ip()));

        // Act
        let text = TextView::render(&details).unwrap();

        // Assert
        assert_contains_all(
            &text,
            &[
                "Environment Details:",
                "Name:",
                "test-env",
                "Instance:",
                "torrust-tracker-vm-test-env",
                "Provider:",
                "lxd",
                "State:",
                "Destroyed",
                "Instance IP:",
                "10.140.190.39",
                "Created:",
                "2026-02-23 10:00:00 UTC",
            ],
        );
    }

    #[test]
    fn it_should_display_not_available_when_instance_ip_is_none() {
        // Arrange
        let details = create_test_details_with_ip(None);

        // Act
        let text = TextView::render(&details).unwrap();

        // Assert
        assert!(text.contains("Instance IP:       Not available"));
    }

    #[test]
    fn it_should_include_all_required_sections() {
        // Arrange
        let details = create_test_details_with_ip(Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42))));

        // Act
        let text = TextView::render(&details).unwrap();

        // Assert - check all sections are present
        assert_contains_all(
            &text,
            &[
                "Name:",
                "Instance:",
                "Provider:",
                "State:",
                "Instance IP:",
                "Created:",
            ],
        );
    }
}
