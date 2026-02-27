//! JSON View for Purge Command
//!
//! This module provides JSON-based rendering for the purge command.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`PurgeDetailsData` DTO).
//!
//! # Design
//!
//! The `JsonView` serializes purge result information to JSON using `serde_json`.
//! The output includes the environment name and a boolean confirming the purge.

use crate::presentation::cli::views::commands::purge::PurgeDetailsData;

/// View for rendering purge details as JSON
///
/// This view provides machine-readable JSON output for automation workflows
/// and AI agents. It serializes the purge details without any transformations,
/// preserving all field names and structure from the DTO.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::purge::{
///     PurgeDetailsData, JsonView,
/// };
///
/// let data = PurgeDetailsData::from_environment_name("my-env");
///
/// let output = JsonView::render(&data);
///
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["environment_name"], "my-env");
/// assert_eq!(parsed["purged"], true);
/// ```
pub struct JsonView;

impl JsonView {
    /// Render purge details as JSON
    ///
    /// Serializes the purge details to pretty-printed JSON format.
    /// The JSON structure matches the DTO structure exactly:
    /// - `environment_name`: Name of the purged environment
    /// - `purged`: Always `true` on success
    ///
    /// # Arguments
    ///
    /// * `data` - Purge details to render
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized purge details.
    /// If serialization fails (which should never happen with valid data),
    /// returns an error JSON object with the serialization error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::purge::{
    ///     PurgeDetailsData, JsonView,
    /// };
    ///
    /// let data = PurgeDetailsData::from_environment_name("prod-tracker");
    ///
    /// let json = JsonView::render(&data);
    ///
    /// assert!(json.contains("\"environment_name\": \"prod-tracker\""));
    /// assert!(json.contains("\"purged\": true"));
    /// ```
    #[must_use]
    pub fn render(data: &PurgeDetailsData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Failed to serialize purge details",
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

    fn create_test_data() -> PurgeDetailsData {
        PurgeDetailsData::from_environment_name("test-env")
    }

    #[test]
    fn it_should_render_valid_json() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("Should produce valid JSON");
        assert_eq!(parsed["environment_name"], "test-env");
        assert_eq!(parsed["purged"], true);
    }

    #[test]
    fn it_should_include_environment_name_field() {
        // Arrange
        let data = PurgeDetailsData::from_environment_name("my-env");

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert!(
            json.contains("\"environment_name\": \"my-env\""),
            "JSON should contain environment_name field"
        );
    }

    #[test]
    fn it_should_include_purged_true_field() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert!(
            json.contains("\"purged\": true"),
            "JSON should contain purged: true"
        );
    }

    #[test]
    fn it_should_produce_pretty_printed_json() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert — pretty-printed JSON contains newlines
        assert!(json.contains('\n'), "JSON should be pretty-printed");
    }
}
