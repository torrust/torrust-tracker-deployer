//! Text View for Environment Details
//!
//! This module provides text-based rendering for environment creation details.
//! It follows the Strategy Pattern, providing one specific rendering strategy
//! (human-readable text) for environment details.

use crate::presentation::cli::views::{Render, ViewRenderError};
use super::super::EnvironmentDetailsData;

/// Text view for rendering environment creation details
///
/// This view produces human-readable formatted text output suitable for
/// terminal display and human consumption.
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
///     EnvironmentDetailsData, TextView
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
/// let output = TextView::render(&data);
/// assert!(output.contains("Environment Details:"));
/// assert!(output.contains("my-env"));
/// ```
pub struct TextView;

impl TextView {
    /// Render environment details as human-readable formatted text
    ///
    /// Takes environment creation data and produces a human-readable output
    /// suitable for displaying to users via stdout.
    ///
    /// # Arguments
    ///
    /// * `data` - Environment details to render
    ///
    /// # Returns
    ///
    /// A formatted string containing:
    /// - Section header ("Environment Details:")
    /// - Numbered list of environment information
    /// - Environment name
    /// - Instance name
    /// - Data directory path
    /// - Build directory path
    ///
    /// # Format
    ///
    /// The output follows this structure:
    /// ```text
    /// Environment Details:
    /// 1. Environment name: <name>
    /// 2. Instance name: <instance-name>
    /// 3. Data directory: <path>
    /// 4. Build directory: <path>
    /// ```
    #[must_use]
    pub fn render(data: &EnvironmentDetailsData) -> String {
        let mut lines = Vec::new();

        lines.push("Environment Details:".to_string());
        lines.push(format!("1. Environment name: {}", data.environment_name));
        lines.push(format!("2. Instance name: {}", data.instance_name));
        lines.push(format!("3. Data directory: {}", data.data_dir.display()));
        lines.push(format!("4. Build directory: {}", data.build_dir.display()));

        lines.join("\n")
    }
}

impl Render<EnvironmentDetailsData> for TextView {
    fn render(data: &EnvironmentDetailsData) -> Result<String, ViewRenderError> {
        Ok(TextView::render(data))
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
    fn it_should_render_environment_details_as_human_readable_format() {
        // Given
        let data = EnvironmentDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            data_dir: PathBuf::from("./data/test-env"),
            build_dir: PathBuf::from("./build/test-env"),
            created_at: test_timestamp(),
        };

        // When
        let output = TextView::render(&data);

        // Then
        assert!(output.contains("Environment Details:"));
        assert!(output.contains("1. Environment name: test-env"));
        assert!(
            output.contains("2. Instance name: torrust-tracker-vm-test-env"),
            "Output missing instance name"
        );
        assert!(output.contains("3. Data directory: ./data/test-env"));
        assert!(output.contains("4. Build directory: ./build/test-env"));
    }

    #[test]
    fn it_should_format_output_with_proper_line_structure() {
        // Given
        let data = EnvironmentDetailsData {
            environment_name: "prod".to_string(),
            instance_name: "vm-prod".to_string(),
            data_dir: PathBuf::from("/opt/deployer/data/prod"),
            build_dir: PathBuf::from("/opt/deployer/build/prod"),
            created_at: test_timestamp(),
        };

        // When
        let output = TextView::render(&data);
        let lines: Vec<&str> = output.lines().collect();

        // Then
        assert_eq!(lines.len(), 5, "Should have 5 lines (header + 4 details)");
        assert_eq!(lines[0], "Environment Details:");
        assert!(lines[1].starts_with("1. "));
        assert!(lines[2].starts_with("2. "));
        assert!(lines[3].starts_with("3. "));
        assert!(lines[4].starts_with("4. "));
    }

    #[test]
    fn it_should_handle_absolute_paths_correctly() {
        // Given
        let data = EnvironmentDetailsData {
            environment_name: "my-env".to_string(),
            instance_name: "torrust-tracker-vm-my-env".to_string(),
            data_dir: PathBuf::from("/absolute/path/data/my-env"),
            build_dir: PathBuf::from("/absolute/path/build/my-env"),
            created_at: test_timestamp(),
        };

        // When
        let output = TextView::render(&data);

        // Then
        assert!(output.contains("/absolute/path/data/my-env"));
        assert!(output.contains("/absolute/path/build/my-env"));
    }
}
