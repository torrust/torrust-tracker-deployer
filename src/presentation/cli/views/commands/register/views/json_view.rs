//! JSON View for Register Command
//!
//! This module provides JSON-based rendering for the register command.
//! It follows the Strategy Pattern, providing a machine-readable output format
//! for the same underlying data (`RegisterDetailsData` DTO).
//!
//! # Design
//!
//! The `JsonView` serializes register result information to JSON using `serde_json`.
//! The output includes the environment name, instance IP, SSH port, and a boolean
//! confirming the registration.

use crate::presentation::cli::views::commands::register::RegisterDetailsData;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering register details as JSON
///
/// This view provides machine-readable JSON output for automation workflows
/// and AI agents. It serializes the register details without any transformations,
/// preserving all field names and structure from the DTO.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::register::{
///     RegisterDetailsData, JsonView,
/// };
///
/// let data = RegisterDetailsData {
///     environment_name: "my-env".to_string(),
///     instance_ip: "192.168.1.100".to_string(),
///     ssh_port: 22,
///     registered: true,
/// };
///
/// let output = JsonView::render(&data);
///
/// // Verify it's valid JSON
/// let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
/// assert_eq!(parsed["environment_name"], "my-env");
/// assert_eq!(parsed["instance_ip"], "192.168.1.100");
/// assert_eq!(parsed["ssh_port"], 22);
/// assert_eq!(parsed["registered"], true);
/// ```
pub struct JsonView;

impl JsonView {
    /// Render register details as JSON
    ///
    /// Serializes the register details to pretty-printed JSON format.
    /// The JSON structure matches the DTO structure exactly:
    /// - `environment_name`: Name of the registered environment
    /// - `instance_ip`: IP address of the registered instance
    /// - `ssh_port`: SSH port of the registered instance
    /// - `registered`: Always `true` on success
    ///
    /// # Arguments
    ///
    /// * `data` - Register details to render
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized register details.
    /// If serialization fails (which should never happen with valid data),
    /// returns an error JSON object with the serialization error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::register::{
    ///     RegisterDetailsData, JsonView,
    /// };
    ///
    /// let data = RegisterDetailsData {
    ///     environment_name: "prod-tracker".to_string(),
    ///     instance_ip: "10.0.0.1".to_string(),
    ///     ssh_port: 2222,
    ///     registered: true,
    /// };
    ///
    /// let json = JsonView::render(&data);
    ///
    /// assert!(json.contains("\"environment_name\": \"prod-tracker\""));
    /// assert!(json.contains("\"instance_ip\": \"10.0.0.1\""));
    /// assert!(json.contains("\"ssh_port\": 2222"));
    /// assert!(json.contains("\"registered\": true"));
    /// ```
    #[must_use]
    pub fn render(data: &RegisterDetailsData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Failed to serialize register details",
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

impl Render<RegisterDetailsData> for JsonView {
    fn render(data: &RegisterDetailsData) -> Result<String, ViewRenderError> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> RegisterDetailsData {
        RegisterDetailsData {
            environment_name: "test-env".to_string(),
            instance_ip: "192.168.1.100".to_string(),
            ssh_port: 22,
            registered: true,
        }
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
        assert_eq!(parsed["instance_ip"], "192.168.1.100");
        assert_eq!(parsed["ssh_port"], 22);
        assert_eq!(parsed["registered"], true);
    }

    #[test]
    fn it_should_include_environment_name_field() {
        // Arrange
        let data = RegisterDetailsData {
            environment_name: "my-env".to_string(),
            instance_ip: "10.0.0.1".to_string(),
            ssh_port: 22,
            registered: true,
        };

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert!(
            json.contains("\"environment_name\": \"my-env\""),
            "JSON should contain environment_name field"
        );
    }

    #[test]
    fn it_should_include_instance_ip_field() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert!(
            json.contains("\"instance_ip\": \"192.168.1.100\""),
            "JSON should contain instance_ip field"
        );
    }

    #[test]
    fn it_should_include_ssh_port_field() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert!(
            json.contains("\"ssh_port\": 22"),
            "JSON should contain ssh_port field"
        );
    }

    #[test]
    fn it_should_include_registered_true_field() {
        // Arrange
        let data = create_test_data();

        // Act
        let json = JsonView::render(&data);

        // Assert
        assert!(
            json.contains("\"registered\": true"),
            "JSON should contain registered: true"
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
