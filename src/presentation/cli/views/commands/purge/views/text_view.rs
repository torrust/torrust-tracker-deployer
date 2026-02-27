//! Text View for Purge Command
//!
//! This module provides text-based rendering for the purge command.
//! It follows the Strategy Pattern, providing a human-readable output format
//! for the same underlying data (`PurgeDetailsData` DTO).
//!
//! # Design
//!
//! The `TextView` formats purge details as human-readable text suitable
//! for terminal display and direct user consumption. It preserves the exact
//! output format produced before the Strategy Pattern was introduced.

use crate::presentation::cli::views::commands::purge::PurgeDetailsData;

/// View for rendering purge details as human-readable text
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
/// use torrust_tracker_deployer_lib::presentation::cli::views::commands::purge::{
///     PurgeDetailsData, TextView,
/// };
///
/// let data = PurgeDetailsData::from_environment_name("my-env");
///
/// let output = TextView::render(&data);
/// assert!(output.contains("Environment 'my-env' purged successfully"));
/// ```
pub struct TextView;

impl TextView {
    /// Render purge details as human-readable text
    ///
    /// Takes purge details and produces a human-readable output
    /// intended to be wrapped by `ProgressReporter::complete()`.
    ///
    /// # Arguments
    ///
    /// * `data` - Purge details to render
    ///
    /// # Returns
    ///
    /// A formatted string: `"Environment '<name>' purged successfully"`.
    /// The `✅` prefix is added by `ProgressReporter::complete()`, not here.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::purge::{
    ///     PurgeDetailsData, TextView,
    /// };
    ///
    /// let data = PurgeDetailsData::from_environment_name("prod-tracker");
    ///
    /// let text = TextView::render(&data);
    ///
    /// assert!(text.contains("Environment 'prod-tracker' purged successfully"));
    /// ```
    #[must_use]
    pub fn render(data: &PurgeDetailsData) -> String {
        format!(
            "Environment '{}' purged successfully",
            data.environment_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_render_success_message_with_environment_name() {
        // Arrange
        let data = PurgeDetailsData::from_environment_name("test-env");

        // Act
        let text = TextView::render(&data);

        // Assert
        assert_eq!(text, "Environment 'test-env' purged successfully");
    }

    #[test]
    fn it_should_include_environment_name_in_output() {
        // Arrange
        let data = PurgeDetailsData::from_environment_name("my-production-env");

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
        let data = PurgeDetailsData::from_environment_name("test-env");

        // Act
        let text = TextView::render(&data);

        // Assert
        assert!(
            !text.starts_with('✅'),
            "TextView should not add the ✅ prefix — that is ProgressReporter's job"
        );
    }
}
