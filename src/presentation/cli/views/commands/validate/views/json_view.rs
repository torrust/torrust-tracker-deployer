//! JSON View for Validate Command
//!
//! This module provides JSON-based rendering for the validate command.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`ValidateDetailsData` DTO).
//!
//! # Design
//!
//! The `JsonView` serializes validation result information to JSON using `serde_json`.
//! The output includes the environment name, configuration file path, provider,
//! and feature flags for the validated configuration.

use crate::presentation::cli::views::commands::validate::ValidateDetailsData;

/// View for rendering validate details as JSON
///
/// This view provides machine-readable JSON output for automation workflows
/// and AI agents. It serializes the validation details without any transformations,
/// preserving all field names and structure from the DTO.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::validate::{
///     ValidateDetailsData, JsonView,
/// };
///
/// let data = ValidateDetailsData {
///     environment_name: "my-env".to_string(),
///     config_file: "envs/my-env.json".to_string(),
///     provider: "lxd".to_string(),
///     is_valid: true,
///     has_prometheus: true,
///     has_grafana: false,
///     has_https: false,
///     has_backup: false,
/// };
///
/// let output = JsonView::render(&data);
///
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["environment_name"], "my-env");
/// assert_eq!(parsed["is_valid"], true);
/// ```
pub struct JsonView;

impl JsonView {
    /// Render validate details as JSON
    ///
    /// Serializes the validation details to pretty-printed JSON format.
    /// The JSON structure matches the DTO structure exactly:
    /// - `environment_name`: Name of the validated environment
    /// - `config_file`: Path to the validated configuration file
    /// - `provider`: Infrastructure provider (lowercase)
    /// - `is_valid`: Always `true` on success
    /// - `has_prometheus`: Whether Prometheus is configured
    /// - `has_grafana`: Whether Grafana is configured
    /// - `has_https`: Whether HTTPS is configured
    /// - `has_backup`: Whether backups are configured
    ///
    /// # Arguments
    ///
    /// * `data` - Validate details to render
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized validate details.
    /// If serialization fails (which should never happen with valid data),
    /// returns an error JSON object with the serialization error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::validate::{
    ///     ValidateDetailsData, JsonView,
    /// };
    ///
    /// let data = ValidateDetailsData {
    ///     environment_name: "prod-tracker".to_string(),
    ///     config_file: "envs/prod-tracker.json".to_string(),
    ///     provider: "lxd".to_string(),
    ///     is_valid: true,
    ///     has_prometheus: true,
    ///     has_grafana: true,
    ///     has_https: true,
    ///     has_backup: false,
    /// };
    ///
    /// let json = JsonView::render(&data);
    ///
    /// assert!(json.contains("\"environment_name\": \"prod-tracker\""));
    /// assert!(json.contains("\"is_valid\": true"));
    /// ```
    #[must_use]
    pub fn render(data: &ValidateDetailsData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Failed to serialize validate details",
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

    // Test fixtures

    fn create_test_data() -> ValidateDetailsData {
        ValidateDetailsData {
            environment_name: "test-env".to_string(),
            config_file: "envs/test-env.json".to_string(),
            provider: "lxd".to_string(),
            is_valid: true,
            has_prometheus: true,
            has_grafana: false,
            has_https: false,
            has_backup: true,
        }
    }

    /// Helper to assert JSON fields match expected string values
    fn assert_json_str_fields_eq(json: &str, expected_fields: &[(&str, &str)]) {
        let parsed: serde_json::Value = serde_json::from_str(json).expect("Should be valid JSON");
        for (field, expected_value) in expected_fields {
            assert_eq!(
                parsed[field].as_str().unwrap_or(""),
                *expected_value,
                "Field '{field}' should be '{expected_value}'"
            );
        }
    }

    /// Helper to assert JSON bool fields match expected values
    fn assert_json_bool_fields_eq(json: &str, expected_fields: &[(&str, bool)]) {
        let parsed: serde_json::Value = serde_json::from_str(json).expect("Should be valid JSON");
        for (field, expected_value) in expected_fields {
            assert_eq!(
                parsed[field].as_bool().unwrap_or(false),
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
    fn it_should_render_validate_details_as_valid_json() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert - verify it's valid JSON with expected string field values
        assert_json_str_fields_eq(
            &json,
            &[
                ("environment_name", "test-env"),
                ("config_file", "envs/test-env.json"),
                ("provider", "lxd"),
            ],
        );
    }

    #[test]
    fn it_should_render_boolean_fields_correctly() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert_json_bool_fields_eq(
            &json,
            &[
                ("is_valid", true),
                ("has_prometheus", true),
                ("has_grafana", false),
                ("has_https", false),
                ("has_backup", true),
            ],
        );
    }

    #[test]
    fn it_should_include_all_required_fields() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert - check all required fields are present
        assert_json_has_fields(
            &json,
            &[
                "environment_name",
                "config_file",
                "provider",
                "is_valid",
                "has_prometheus",
                "has_grafana",
                "has_https",
                "has_backup",
            ],
        );
    }

    #[test]
    fn it_should_render_all_features_enabled() {
        // Arrange
        let data = ValidateDetailsData {
            environment_name: "full-stack".to_string(),
            config_file: "envs/full-stack.json".to_string(),
            provider: "lxd".to_string(),
            is_valid: true,
            has_prometheus: true,
            has_grafana: true,
            has_https: true,
            has_backup: true,
        };

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert_json_bool_fields_eq(
            &json,
            &[
                ("has_prometheus", true),
                ("has_grafana", true),
                ("has_https", true),
                ("has_backup", true),
            ],
        );
    }

    #[test]
    fn it_should_render_all_features_disabled() {
        // Arrange
        let data = ValidateDetailsData {
            environment_name: "minimal-env".to_string(),
            config_file: "envs/minimal-env.json".to_string(),
            provider: "lxd".to_string(),
            is_valid: true,
            has_prometheus: false,
            has_grafana: false,
            has_https: false,
            has_backup: false,
        };

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert_json_bool_fields_eq(
            &json,
            &[
                ("has_prometheus", false),
                ("has_grafana", false),
                ("has_https", false),
                ("has_backup", false),
            ],
        );
    }
}
