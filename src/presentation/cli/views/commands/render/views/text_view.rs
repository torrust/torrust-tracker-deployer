//! Text View for Render Command
//!
//! This module provides text-based rendering for the render command.
//! It follows the Strategy Pattern, providing a human-readable output format
//! for the same underlying data (`RenderDetailsData` DTO).
//!
//! # Design
//!
//! The `TextView` formats render details as human-readable text suitable
//! for terminal display and direct user consumption. It preserves the exact
//! output format produced before the Strategy Pattern was introduced.

use crate::presentation::cli::views::commands::render::RenderDetailsData;

/// View for rendering render details as human-readable text
///
/// This view produces formatted text output suitable for terminal display
/// and human consumption. It presents artifact generation details
/// in a clear, readable format including next steps for the user.
///
/// The rendered string is intended to be passed to `ProgressReporter::complete()`,
/// which adds the `✅` prefix to the first line.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::render::{
///     RenderDetailsData, TextView,
/// };
///
/// let data = RenderDetailsData {
///     environment_name: "my-env".to_string(),
///     config_source: "Config file: envs/my-env.json".to_string(),
///     target_ip: "192.168.1.100".to_string(),
///     output_dir: "/tmp/build/my-env".to_string(),
/// };
///
/// let output = TextView::render(&data);
/// assert!(output.contains("Deployment artifacts generated successfully!"));
/// assert!(output.contains("192.168.1.100"));
/// assert!(output.contains("/tmp/build/my-env"));
/// ```
pub struct TextView;

impl TextView {
    /// Render render details as human-readable formatted text
    ///
    /// Takes render details and produces a human-readable output
    /// intended to be wrapped by `ProgressReporter::complete()`.
    ///
    /// # Arguments
    ///
    /// * `data` - Render details to render
    ///
    /// # Returns
    ///
    /// A formatted string containing:
    /// - "Deployment artifacts generated successfully!"
    /// - Source, Target IP, and Output path
    /// - Next steps section with guidance
    ///
    /// # Format
    ///
    /// The output follows this structure:
    ///
    /// ```text
    /// Deployment artifacts generated successfully!
    ///
    ///   Source: <config_source>
    ///   Target IP: <target_ip>
    ///   Output: <output_dir>
    ///
    /// Next steps:
    ///   - Review artifacts in the output directory
    ///   - Use 'provision' command to deploy infrastructure
    ///   - Or use artifacts manually with your deployment tools
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::render::{
    ///     RenderDetailsData, TextView,
    /// };
    ///
    /// let data = RenderDetailsData {
    ///     environment_name: "prod-tracker".to_string(),
    ///     config_source: "Config file: envs/prod-tracker.json".to_string(),
    ///     target_ip: "10.0.0.1".to_string(),
    ///     output_dir: "/tmp/build/prod-tracker".to_string(),
    /// };
    ///
    /// let text = TextView::render(&data);
    ///
    /// assert!(text.contains("Deployment artifacts generated successfully!"));
    /// assert!(text.contains("10.0.0.1"));
    /// assert!(text.contains("Next steps:"));
    /// ```
    #[must_use]
    pub fn render(data: &RenderDetailsData) -> String {
        format!(
            "Deployment artifacts generated successfully!\n\n\
             \x20\x20Source: {}\n\
             \x20\x20Target IP: {}\n\
             \x20\x20Output: {}\n\n\
             Next steps:\n\
             \x20\x20- Review artifacts in the output directory\n\
             \x20\x20- Use 'provision' command to deploy infrastructure\n\
             \x20\x20- Or use artifacts manually with your deployment tools",
            data.config_source, data.target_ip, data.output_dir,
        )
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
    fn it_should_render_render_details_as_formatted_text() {
        // Arrange
        let data = create_test_data();

        // Act
        let text = TextView::render(&data);

        // Assert
        assert_contains_all(
            &text,
            &[
                "Deployment artifacts generated successfully!",
                "Config file: envs/test-env.json",
                "192.168.1.100",
                "/tmp/build/test-env",
                "Next steps:",
            ],
        );
    }

    #[test]
    fn it_should_include_all_result_fields() {
        // Arrange
        let data = create_test_data();

        // Act
        let text = TextView::render(&data);

        // Assert - each significant data point appears in the output
        assert_contains_all(
            &text,
            &[
                "Config file: envs/test-env.json",
                "192.168.1.100",
                "/tmp/build/test-env",
            ],
        );
    }

    #[test]
    fn it_should_include_next_steps_guidance() {
        // Arrange
        let data = create_test_data();

        // Act
        let text = TextView::render(&data);

        // Assert - next steps section is present
        assert_contains_all(&text, &["Next steps:", "Review artifacts", "provision"]);
    }

    #[test]
    fn it_should_render_env_name_source_when_used() {
        // Arrange - simulate env-name based source
        let data = RenderDetailsData {
            environment_name: "my-env".to_string(),
            config_source: "Environment: my-env".to_string(),
            target_ip: "10.0.0.1".to_string(),
            output_dir: "/tmp/build/my-env".to_string(),
        };

        // Act
        let text = TextView::render(&data);

        // Assert
        assert_contains_all(
            &text,
            &["Environment: my-env", "10.0.0.1", "/tmp/build/my-env"],
        );
    }
}
