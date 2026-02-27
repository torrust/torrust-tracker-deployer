//! Text View for Register Command
//!
//! This module provides text-based rendering for the register command.
//! It follows the Strategy Pattern, providing a human-readable output format
//! for the same underlying data (`RegisterDetailsData` DTO).
//!
//! # Design
//!
//! The `TextView` formats register details as human-readable text suitable
//! for terminal display and direct user consumption. It preserves the exact
//! output format produced before the Strategy Pattern was introduced.

use crate::presentation::cli::views::commands::register::RegisterDetailsData;
use crate::presentation::cli::views::{Render, ViewRenderError};

/// View for rendering register details as human-readable text
///
/// This view produces formatted text output suitable for terminal display
/// and human consumption.
///
/// The rendered string is intended to be passed to `ProgressReporter::complete()`,
/// which adds the `✅` prefix to the first line.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::register::{
///     RegisterDetailsData, TextView,
/// };
///
/// let data = RegisterDetailsData {
///     environment_name: "my-env".to_string(),
///     instance_ip: "192.168.1.100".to_string(),
///     ssh_port: 22,
///     registered: true,
/// };
///
/// let output = TextView::render(&data);
/// assert!(output.contains("Instance registered successfully with environment 'my-env'"));
/// ```
pub struct TextView;

impl TextView {
    /// Render register details as human-readable text
    ///
    /// Takes register details and produces a human-readable output
    /// intended to be wrapped by `ProgressReporter::complete()`.
    ///
    /// # Arguments
    ///
    /// * `data` - Register details to render
    ///
    /// # Returns
    ///
    /// A formatted string: `"Instance registered successfully with environment '<name>'"`.
    /// The `✅` prefix is added by `ProgressReporter::complete()`, not here.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::register::{
    ///     RegisterDetailsData, TextView,
    /// };
    ///
    /// let data = RegisterDetailsData {
    ///     environment_name: "prod-tracker".to_string(),
    ///     instance_ip: "10.0.0.1".to_string(),
    ///     ssh_port: 22,
    ///     registered: true,
    /// };
    ///
    /// let text = TextView::render(&data);
    ///
    /// assert!(text.contains("Instance registered successfully with environment 'prod-tracker'"));
    /// ```
    #[must_use]
    pub fn render(data: &RegisterDetailsData) -> String {
        format!(
            "Instance registered successfully with environment '{}'",
            data.environment_name
        )
    }
}

impl Render<RegisterDetailsData> for TextView {
    fn render(data: &RegisterDetailsData) -> Result<String, ViewRenderError> {
        Ok(TextView::render(data))
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
    fn it_should_render_success_message_with_environment_name() {
        // Arrange
        let data = create_test_data();

        // Act
        let text = TextView::render(&data);

        // Assert
        assert_eq!(
            text,
            "Instance registered successfully with environment 'test-env'"
        );
    }

    #[test]
    fn it_should_include_environment_name_in_output() {
        // Arrange
        let data = RegisterDetailsData {
            environment_name: "my-production-env".to_string(),
            instance_ip: "10.0.0.1".to_string(),
            ssh_port: 22,
            registered: true,
        };

        // Act
        let text = TextView::render(&data);

        // Assert
        assert!(
            text.contains("my-production-env"),
            "Output should contain the environment name"
        );
    }

    #[test]
    fn it_should_not_include_checkmark_prefix() {
        // Arrange — the ✅ is added by ProgressReporter::complete(), not here
        let data = create_test_data();

        // Act
        let text = TextView::render(&data);

        // Assert
        assert!(
            !text.starts_with('✅'),
            "TextView should not add the ✅ prefix — that is ProgressReporter's job"
        );
    }
}
