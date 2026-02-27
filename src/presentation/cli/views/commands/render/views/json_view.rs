//! JSON View for Render Command
//!
//! This module provides JSON-based rendering for the render command.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`RenderDetailsData` DTO).
//!
//! # Design
//!
//! The `JsonView` serializes render result information to JSON using `serde_json`.
//! The output includes the environment name, configuration source, target IP,
//! and output directory for the generated artifacts.

use crate::presentation::cli::views::commands::render::RenderDetailsData;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering render details as JSON
///
/// This view provides machine-readable JSON output for automation workflows
/// and AI agents. It serializes the render details without any transformations,
/// preserving all field names and structure from the DTO.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::render::{
///     RenderDetailsData, JsonView,
/// };
///
/// let data = RenderDetailsData {
///     environment_name: "my-env".to_string(),
///     config_source: "Config file: envs/my-env.json".to_string(),
///     target_ip: "192.168.1.100".to_string(),
///     output_dir: "/tmp/build/my-env".to_string(),
/// };
///
/// let output = JsonView::render(&data);
///
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["environment_name"], "my-env");
/// assert_eq!(parsed["target_ip"], "192.168.1.100");
/// ```
pub struct JsonView;

impl JsonView {
    /// Render render details as JSON
    ///
    /// Serializes the render details to pretty-printed JSON format.
    /// The JSON structure matches the DTO structure exactly:
    /// - `environment_name`: Name of the environment
    /// - `config_source`: Description of the configuration source
    /// - `target_ip`: IP address used in artifact generation
    /// - `output_dir`: Path to the generated artifacts directory
    ///
    /// # Arguments
    ///
    /// * `data` - Render details to render
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized render details.
    /// If serialization fails (which should never happen with valid data),
    /// returns an error JSON object with the serialization error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::render::{
    ///     RenderDetailsData, JsonView,
    /// };
    ///
    /// let data = RenderDetailsData {
    ///     environment_name: "prod-tracker".to_string(),
    ///     config_source: "Config file: envs/prod-tracker.json".to_string(),
    ///     target_ip: "10.0.0.1".to_string(),
    ///     output_dir: "/tmp/build/prod-tracker".to_string(),
    /// };
    ///
    /// let json = JsonView::render(&data);
    ///
    /// assert!(json.contains("\"environment_name\": \"prod-tracker\""));
    /// assert!(json.contains("\"target_ip\": \"10.0.0.1\""));
    /// ```
    #[must_use]
    pub fn render(data: &RenderDetailsData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Failed to serialize render details",
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

impl Render<RenderDetailsData> for JsonView {
    fn render(data: &RenderDetailsData) -> Result<String, ViewRenderError> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures

    fn create_test_data() -> RenderDetailsData {
        RenderDetailsData {
            environment_name: "test-env".to_string(),
            config_source: "Config file: envs/test-env.json".to_string(),
            target_ip: "192.168.1.100".to_string(),
            output_dir: "/tmp/build/test-env".to_string(),
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
    fn it_should_render_render_details_as_valid_json() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert - verify it's valid JSON with expected string field values
        assert_json_str_fields_eq(
            &json,
            &[
                ("environment_name", "test-env"),
                ("config_source", "Config file: envs/test-env.json"),
                ("target_ip", "192.168.1.100"),
                ("output_dir", "/tmp/build/test-env"),
            ],
        );
    }

    #[test]
    fn it_should_render_all_required_fields() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert - every documented field must be present
        assert_json_has_fields(
            &json,
            &[
                "environment_name",
                "config_source",
                "target_ip",
                "output_dir",
            ],
        );
    }

    #[test]
    fn it_should_produce_valid_json() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert
        let result = serde_json::from_str::<serde_json::Value>(&json);
        assert!(result.is_ok(), "Output should be valid JSON, got: {json}");
    }

    #[test]
    fn it_should_render_env_name_source_via_env_name() {
        // Arrange - simulate env-name based source
        let data = RenderDetailsData {
            environment_name: "my-env".to_string(),
            config_source: "Environment: my-env".to_string(),
            target_ip: "10.0.0.1".to_string(),
            output_dir: "/tmp/build/my-env".to_string(),
        };

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert_json_str_fields_eq(&json, &[("config_source", "Environment: my-env")]);
    }

    #[test]
    fn it_should_render_pretty_printed_json() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert - pretty-printed JSON has newlines and indentation
        assert!(
            json.contains('\n'),
            "Pretty-printed JSON should contain newlines"
        );
        assert!(
            json.contains("  "),
            "Pretty-printed JSON should contain indentation"
        );
    }
}
