//! Text View for Validate Command
//!
//! This module provides text-based rendering for the validate command.
//! It follows the Strategy Pattern, providing a human-readable output format
//! for the same underlying data (`ValidateDetailsData` DTO).
//!
//! # Design
//!
//! The `TextView` formats validation details as human-readable text suitable
//! for terminal display and direct user consumption. It preserves the exact
//! output format produced before the Strategy Pattern was introduced.

use crate::presentation::cli::views::commands::validate::ValidateDetailsData;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering validate details as human-readable text
///
/// This view produces formatted text output suitable for terminal display
/// and human consumption. It presents environment validation details
/// in a clear, readable format.
///
/// The rendered string is intended to be passed to `ProgressReporter::complete()`,
/// which adds the `✅` prefix to the first line.
///
/// # Examples
///
/// ```rust
/// # use torrust_tracker_deployer_lib::presentation::cli::views::Render;
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::validate::{
///     ValidateDetailsData, TextView,
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
/// let output = TextView::render(&data).unwrap();
/// assert!(output.contains("Configuration file 'envs/my-env.json' is valid"));
/// assert!(output.contains("Environment Details:"));
/// assert!(output.contains("my-env"));
/// ```
pub struct TextView;

impl Render<ValidateDetailsData> for TextView {
    fn render(data: &ValidateDetailsData) -> Result<String, ViewRenderError> {
        Ok(format!(
            "Configuration file '{}' is valid\n\nEnvironment Details:\n\
            • Name: {}\n\
            • Provider: {}\n\
            • Prometheus: {}\n\
            • Grafana: {}\n\
            • HTTPS: {}\n\
            • Backups: {}",
            data.config_file,
            data.environment_name,
            data.provider,
            if data.has_prometheus {
                "Enabled"
            } else {
                "Disabled"
            },
            if data.has_grafana {
                "Enabled"
            } else {
                "Disabled"
            },
            if data.has_https {
                "Enabled"
            } else {
                "Disabled"
            },
            if data.has_backup {
                "Enabled"
            } else {
                "Disabled"
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures

    fn create_test_data_all_enabled() -> ValidateDetailsData {
        ValidateDetailsData {
            environment_name: "test-env".to_string(),
            config_file: "envs/test-env.json".to_string(),
            provider: "lxd".to_string(),
            is_valid: true,
            has_prometheus: true,
            has_grafana: true,
            has_https: true,
            has_backup: true,
        }
    }

    fn create_test_data_all_disabled() -> ValidateDetailsData {
        ValidateDetailsData {
            environment_name: "minimal-env".to_string(),
            config_file: "envs/minimal-env.json".to_string(),
            provider: "lxd".to_string(),
            is_valid: true,
            has_prometheus: false,
            has_grafana: false,
            has_https: false,
            has_backup: false,
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
    fn it_should_render_validate_details_as_formatted_text() {
        // Arrange
        let data = create_test_data_all_enabled();

        // Act
        let text = TextView::render(&data).unwrap();

        // Assert
        assert_contains_all(
            &text,
            &[
                "Configuration file 'envs/test-env.json' is valid",
                "Environment Details:",
                "Name:",
                "test-env",
                "Provider:",
                "lxd",
                "Prometheus:",
                "Grafana:",
                "HTTPS:",
                "Backups:",
            ],
        );
    }

    #[test]
    fn it_should_display_enabled_when_features_are_configured() {
        // Arrange
        let data = create_test_data_all_enabled();

        // Act
        let text = TextView::render(&data).unwrap();

        // Assert - all feature sections show "Enabled"
        let enabled_count = text.matches("Enabled").count();
        assert_eq!(
            enabled_count, 4,
            "Expected 4 'Enabled' occurrences (prometheus, grafana, https, backup) but got {enabled_count}\nText: {text}"
        );
    }

    #[test]
    fn it_should_display_disabled_when_features_are_not_configured() {
        // Arrange
        let data = create_test_data_all_disabled();

        // Act
        let text = TextView::render(&data).unwrap();

        // Assert - all feature sections show "Disabled"
        let disabled_count = text.matches("Disabled").count();
        assert_eq!(
            disabled_count, 4,
            "Expected 4 'Disabled' occurrences (prometheus, grafana, https, backup) but got {disabled_count}\nText: {text}"
        );
    }

    #[test]
    fn it_should_include_config_file_path_in_first_line() {
        // Arrange
        let data = create_test_data_all_disabled();

        // Act
        let text = TextView::render(&data).unwrap();

        // Assert
        assert!(
            text.starts_with("Configuration file 'envs/minimal-env.json' is valid"),
            "Text should start with config file path validation message.\nActual:\n{text}"
        );
    }

    #[test]
    fn it_should_include_all_required_sections() {
        // Arrange
        let data = create_test_data_all_enabled();

        // Act
        let text = TextView::render(&data).unwrap();

        // Assert
        assert_contains_all(
            &text,
            &[
                "Environment Details:",
                "• Name:",
                "• Provider:",
                "• Prometheus:",
                "• Grafana:",
                "• HTTPS:",
                "• Backups:",
            ],
        );
    }
}
