//! JSON View for Environment Details
//!
//! This module provides JSON-based rendering for environment creation details.
//! It follows the Strategy Pattern, providing one specific rendering strategy
//! (machine-readable JSON) for environment details.

use super::super::EnvironmentDetailsData;

/// JSON view for rendering environment creation details
///
/// This view produces machine-readable JSON output suitable for programmatic
/// parsing, automation workflows, and AI agents.
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
/// use std::path::PathBuf;
/// use chrono::{TimeZone, Utc};
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::create::{
///     EnvironmentDetailsData, JsonView
/// };
///
/// let data = EnvironmentDetailsData {
///     environment_name: "my-env".to_string(),
///     instance_name: "torrust-tracker-vm-my-env".to_string(),
///     data_dir: PathBuf::from("./data/my-env"),
///     build_dir: PathBuf::from("./build/my-env"),
///     created_at: Utc.with_ymd_and_hms(2026, 2, 16, 14, 30, 0).unwrap(),
/// };
///
/// let json = JsonView::render(&data).expect("JSON serialization failed");
/// assert!(json.contains(r#""environment_name": "my-env""#));
/// ```
pub struct JsonView;

impl JsonView {
    /// Render environment details as JSON
    ///
    /// Takes environment creation data and produces a JSON-formatted string
    /// suitable for programmatic parsing and automation workflows.
    ///
    /// # Arguments
    ///
    /// * `data` - Environment details to render
    ///
    /// # Returns
    ///
    /// A JSON string containing:
    /// - `environment_name`: Name of the created environment
    /// - `instance_name`: Name of the VM instance
    /// - `data_dir`: Path to environment data directory
    /// - `build_dir`: Path to build artifacts directory
    /// - `created_at`: ISO 8601 timestamp of environment creation
    ///
    /// # Format
    ///
    /// The output is pretty-printed JSON for readability:
    /// ```json
    /// {
    ///   "environment_name": "my-env",
    ///   "instance_name": "torrust-tracker-vm-my-env",
    ///   "data_dir": "./data/my-env",
    ///   "build_dir": "./build/my-env",
    ///   "created_at": "2026-02-16T14:30:00Z"
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if JSON serialization fails (very rare,
    /// would indicate a bug in the serialization implementation).
    pub fn render(data: &EnvironmentDetailsData) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(data)
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::path::PathBuf;

    fn test_timestamp() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 2, 16, 14, 30, 0).unwrap()
    }

    #[test]
    fn it_should_render_environment_details_as_json_format() {
        // Given
        let data = EnvironmentDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            data_dir: PathBuf::from("./data/test-env"),
            build_dir: PathBuf::from("./build/test-env"),
            created_at: test_timestamp(),
        };

        // When
        let json = JsonView::render(&data).expect("JSON serialization failed");

        // Then
        assert!(json.contains(r#""environment_name": "test-env""#));
        assert!(json.contains(r#""instance_name": "torrust-tracker-vm-test-env""#));
        assert!(json.contains(r#""data_dir": "./data/test-env""#));
        assert!(json.contains(r#""build_dir": "./build/test-env""#));
        assert!(json.contains(r#""created_at": "2026-02-16T14:30:00Z""#));
    }

    #[test]
    fn it_should_produce_valid_json_parsable_by_serde() {
        // Given
        let data = EnvironmentDetailsData {
            environment_name: "prod".to_string(),
            instance_name: "vm-prod".to_string(),
            data_dir: PathBuf::from("/opt/data/prod"),
            build_dir: PathBuf::from("/opt/build/prod"),
            created_at: test_timestamp(),
        };

        // When
        let json = JsonView::render(&data).expect("JSON serialization failed");
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("Should produce valid JSON");

        // Then
        assert_eq!(parsed["environment_name"], "prod");
        assert_eq!(parsed["instance_name"], "vm-prod");
        assert_eq!(parsed["data_dir"], "/opt/data/prod");
        assert_eq!(parsed["build_dir"], "/opt/build/prod");
        assert_eq!(parsed["created_at"], "2026-02-16T14:30:00Z");
    }

    #[test]
    fn it_should_format_json_as_pretty_printed() {
        // Given
        let data = EnvironmentDetailsData {
            environment_name: "my-env".to_string(),
            instance_name: "vm-my-env".to_string(),
            data_dir: PathBuf::from("./data/my-env"),
            build_dir: PathBuf::from("./build/my-env"),
            created_at: test_timestamp(),
        };

        // When
        let json = JsonView::render(&data).expect("JSON serialization failed");

        // Then - pretty-printed JSON should have newlines and indentation
        assert!(
            json.contains('\n'),
            "JSON should be pretty-printed with newlines"
        );
        assert!(
            json.lines().count() > 1,
            "JSON should span multiple lines when pretty-printed"
        );
    }
}
