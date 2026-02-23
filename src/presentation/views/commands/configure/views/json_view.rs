//! JSON View for Configure Command
//!
//! This module provides JSON-based rendering for the configure command.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`ConfigureDetailsData` DTO).
//!
//! # Design
//!
//! The `JsonView` serializes configure command information to JSON using `serde_json`.
//! The output includes environment details and configuration state.

use crate::presentation::views::commands::configure::ConfigureDetailsData;

/// View for rendering configure details as JSON
///
/// This view provides machine-readable JSON output for automation workflows
/// and AI agents. It serializes the configure details without any transformations,
/// preserving all field names and structure from the DTO.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::views::commands::configure::{
///     ConfigureDetailsData, JsonView,
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
/// let output = JsonView::render(&details);
///
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["environment_name"], "my-env");
/// assert_eq!(parsed["state"], "Configured");
/// ```
pub struct JsonView;

impl JsonView {
    /// Render configure details as JSON
    ///
    /// Serializes the configure details to pretty-printed JSON format.
    /// The JSON structure matches the DTO structure exactly:
    /// - `environment_name`: Name of the environment
    /// - `instance_name`: VM instance name
    /// - `provider`: Infrastructure provider
    /// - `state`: Always "Configured" on success
    /// - `instance_ip`: IP address (nullable)
    /// - `created_at`: ISO 8601 UTC timestamp
    ///
    /// # Arguments
    ///
    /// * `data` - Configure details to render
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized configure details.
    /// If serialization fails (which should never happen with valid data),
    /// returns an error JSON object with the serialization error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::commands::configure::{
    ///     ConfigureDetailsData, JsonView,
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
    /// let json = JsonView::render(&details);
    ///
    /// assert!(json.contains("\"environment_name\": \"prod-tracker\""));
    /// assert!(json.contains("\"state\": \"Configured\""));
    /// ```
    #[must_use]
    pub fn render(data: &ConfigureDetailsData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            format!(
                r#"{{
  "error": "Failed to serialize configure details",
  "message": "{e}"
}}"#
            )
        })
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

    fn create_test_details_with_ip(ip: Option<IpAddr>) -> ConfigureDetailsData {
        ConfigureDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            provider: "lxd".to_string(),
            state: "Configured".to_string(),
            instance_ip: ip,
            created_at: create_test_timestamp(),
        }
    }

    /// Helper to assert JSON fields match expected values
    fn assert_json_fields_eq(json: &str, expected_fields: &[(&str, &str)]) {
        let parsed: serde_json::Value = serde_json::from_str(json).expect("Should be valid JSON");
        for (field, expected_value) in expected_fields {
            assert_eq!(
                parsed[field].as_str().unwrap_or(""),
                *expected_value,
                "Field '{field}' should be '{expected_value}'"
            );
        }
    }

    /// Helper to assert JSON contains all required field names
    fn assert_json_has_fields(json: &str, field_names: &[&str]) {
        let parsed: serde_json::Value = serde_json::from_str(json).expect("Should be valid JSON");
        for field_name in field_names {
            assert!(
                parsed.get(field_name).is_some(),
                "Expected JSON to have field '{field_name}' but it didn't.\nActual JSON:\n{json}"
            );
        }
    }

    // Tests

    #[test]
    fn it_should_render_configure_details_as_valid_json() {
        // Arrange
        let details = create_test_details_with_ip(Some(create_test_ip()));

        // Act
        let json = JsonView::render(&details);

        // Assert - verify it's valid JSON with expected field values
        assert_json_fields_eq(
            &json,
            &[
                ("environment_name", "test-env"),
                ("instance_name", "torrust-tracker-vm-test-env"),
                ("provider", "lxd"),
                ("state", "Configured"),
                ("instance_ip", "10.140.190.39"),
                ("created_at", "2026-02-23T10:00:00Z"),
            ],
        );
    }

    #[test]
    fn it_should_render_null_instance_ip_as_json_null() {
        // Arrange
        let details = create_test_details_with_ip(None);

        // Act
        let json = JsonView::render(&details);

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
        assert!(parsed["instance_ip"].is_null());
    }

    #[test]
    fn it_should_include_all_required_fields() {
        // Arrange
        let details = create_test_details_with_ip(Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42))));

        // Act
        let json = JsonView::render(&details);

        // Assert - check all required fields are present
        assert_json_has_fields(
            &json,
            &[
                "environment_name",
                "instance_name",
                "provider",
                "state",
                "instance_ip",
                "created_at",
            ],
        );
    }
}
