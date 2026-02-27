//! JSON View for Test Command
//!
//! This module provides JSON-based rendering for the test command.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`TestResultData` DTO).
//!
//! # Design
//!
//! The `JsonView` serializes test command results to JSON using `serde_json`.
//! The output includes test result status and any advisory DNS warnings.

use crate::presentation::cli::views::commands::test::TestResultData;

/// View for rendering test results as JSON
///
/// This view provides machine-readable JSON output for automation workflows,
/// CI/CD pipelines, and AI agents. It serializes the test results without
/// any transformations, preserving all field names and structure from the DTO.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::test::{
///     TestResultData, JsonView,
/// };
///
/// let data = TestResultData {
///     environment_name: "my-env".to_string(),
///     instance_ip: "10.140.190.39".to_string(),
///     result: "pass".to_string(),
///     dns_warnings: vec![],
/// };
///
/// let output = JsonView::render(&data);
///
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["environment_name"], "my-env");
/// assert_eq!(parsed["result"], "pass");
/// ```
pub struct JsonView;

impl JsonView {
    /// Render test results as JSON
    ///
    /// Serializes the test results to pretty-printed JSON format.
    /// The JSON structure matches the DTO structure exactly:
    /// - `environment_name`: Name of the tested environment
    /// - `instance_ip`: IP address of the tested instance
    /// - `result`: Always "pass" on success
    /// - `dns_warnings`: Array of advisory DNS warnings (may be empty)
    ///
    /// # Arguments
    ///
    /// * `data` - Test result data to render
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized test results.
    /// If serialization fails (which should never happen with valid data),
    /// returns an error JSON object with the serialization error message.
    #[must_use]
    pub fn render(data: &TestResultData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Failed to serialize test results",
                "message": e.to_string(),
            }))
            .unwrap_or_else(|_| {
                r#"{
  "error": "Failed to serialize error message"
}"#
                .to_string()
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::cli::views::commands::test::DnsWarningData;

    // Test fixtures and helpers

    fn create_test_data_no_warnings() -> TestResultData {
        TestResultData {
            environment_name: "test-env".to_string(),
            instance_ip: "10.140.190.39".to_string(),
            result: "pass".to_string(),
            dns_warnings: vec![],
        }
    }

    fn create_test_data_with_warnings() -> TestResultData {
        TestResultData {
            environment_name: "test-env".to_string(),
            instance_ip: "10.140.190.39".to_string(),
            result: "pass".to_string(),
            dns_warnings: vec![
                DnsWarningData {
                    domain: "tracker.local".to_string(),
                    expected_ip: "10.140.190.39".to_string(),
                    issue: "tracker.local does not resolve (expected: 10.140.190.39): name resolution failed".to_string(),
                },
                DnsWarningData {
                    domain: "api.tracker.local".to_string(),
                    expected_ip: "10.140.190.39".to_string(),
                    issue: "api.tracker.local resolves to [192.168.1.1] but expected 10.140.190.39".to_string(),
                },
            ],
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
    fn it_should_render_test_results_as_valid_json() {
        // Arrange
        let data = create_test_data_no_warnings();

        // Act
        let json = JsonView::render(&data);

        // Assert - verify it's valid JSON with expected field values
        assert_json_fields_eq(
            &json,
            &[
                ("environment_name", "test-env"),
                ("instance_ip", "10.140.190.39"),
                ("result", "pass"),
            ],
        );
    }

    #[test]
    fn it_should_include_all_required_fields() {
        // Arrange
        let data = create_test_data_no_warnings();

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert_json_has_fields(
            &json,
            &["environment_name", "instance_ip", "result", "dns_warnings"],
        );
    }

    #[test]
    fn it_should_render_empty_dns_warnings_as_empty_array() {
        // Arrange
        let data = create_test_data_no_warnings();

        // Act
        let json = JsonView::render(&data);

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
        let warnings = parsed["dns_warnings"].as_array().expect("Should be array");
        assert!(warnings.is_empty());
    }

    #[test]
    fn it_should_render_dns_warnings_with_all_fields() {
        // Arrange
        let data = create_test_data_with_warnings();

        // Act
        let json = JsonView::render(&data);

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
        let warnings = parsed["dns_warnings"].as_array().expect("Should be array");
        assert_eq!(warnings.len(), 2);

        // First warning
        assert_eq!(warnings[0]["domain"], "tracker.local");
        assert_eq!(warnings[0]["expected_ip"], "10.140.190.39");
        assert!(warnings[0]["issue"]
            .as_str()
            .unwrap()
            .contains("does not resolve"));

        // Second warning
        assert_eq!(warnings[1]["domain"], "api.tracker.local");
        assert!(warnings[1]["issue"]
            .as_str()
            .unwrap()
            .contains("192.168.1.1"));
    }
}
