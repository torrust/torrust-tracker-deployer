//! Environment Details View for Create Command
//!
//! This module provides a view for rendering environment creation details
//! after successful environment initialization.

use std::path::PathBuf;

use crate::domain::environment::state::Created;
use crate::domain::environment::Environment;

/// Environment details data for rendering
///
/// This struct holds all the data needed to render environment creation
/// information for display to the user.
#[derive(Debug, Clone)]
pub struct EnvironmentDetailsData {
    /// Name of the created environment
    pub environment_name: String,
    /// Name of the instance that will be created
    pub instance_name: String,
    /// Path to the data directory
    pub data_dir: PathBuf,
    /// Path to the build directory
    pub build_dir: PathBuf,
}

/// Conversion from domain model to presentation DTO
///
/// This `From` trait implementation is placed in the presentation layer
/// (not in the domain layer) to maintain proper DDD layering:
///
/// - Domain layer should not depend on presentation layer DTOs
/// - Presentation layer can depend on domain models (allowed)
/// - This keeps the domain clean and focused on business logic
///
/// Alternative approaches considered:
/// - Adding method to `Environment<Created>`: Would violate DDD by making
///   domain depend on presentation DTOs
/// - Keeping mapping in controller: Works but less idiomatic than `From` trait
impl From<&Environment<Created>> for EnvironmentDetailsData {
    fn from(environment: &Environment<Created>) -> Self {
        Self {
            environment_name: environment.name().as_str().to_string(),
            instance_name: environment.instance_name().as_str().to_string(),
            data_dir: environment.data_dir().clone(),
            build_dir: environment.build_dir().clone(),
        }
    }
}

/// View for rendering environment creation details
///
/// This view is responsible for formatting and rendering the environment
/// information that users see after creating a new environment.
///
/// # Design
///
/// Following MVC pattern, this view:
/// - Receives data from the controller via the `EnvironmentDetailsData` DTO
/// - Formats the output for display
/// - Returns a string ready for output to stdout
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use torrust_tracker_deployer_lib::presentation::views::commands::create::EnvironmentDetailsView;
/// use torrust_tracker_deployer_lib::presentation::views::commands::create::environment_details::EnvironmentDetailsData;
///
/// let data = EnvironmentDetailsData {
///     environment_name: "my-env".to_string(),
///     instance_name: "torrust-tracker-vm-my-env".to_string(),
///     data_dir: PathBuf::from("./data/my-env"),
///     build_dir: PathBuf::from("./build/my-env"),
/// };
///
/// let output = EnvironmentDetailsView::render_human_readable(&data);
/// assert!(output.contains("Environment Details:"));
/// assert!(output.contains("my-env"));
/// ```
pub struct EnvironmentDetailsView;

impl EnvironmentDetailsView {
    /// Render environment details as human-readable formatted string
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
    pub fn render_human_readable(data: &EnvironmentDetailsData) -> String {
        let mut lines = Vec::new();

        lines.push("Environment Details:".to_string());
        lines.push(format!("1. Environment name: {}", data.environment_name));
        lines.push(format!("2. Instance name: {}", data.instance_name));
        lines.push(format!("3. Data directory: {}", data.data_dir.display()));
        lines.push(format!("4. Build directory: {}", data.build_dir.display()));

        lines.join("\n")
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_render_environment_details_as_human_readable_format() {
        // Given
        let data = EnvironmentDetailsData {
            environment_name: "test-env".to_string(),
            instance_name: "torrust-tracker-vm-test-env".to_string(),
            data_dir: PathBuf::from("./data/test-env"),
            build_dir: PathBuf::from("./build/test-env"),
        };

        // When
        let output = EnvironmentDetailsView::render_human_readable(&data);

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
        };

        // When
        let output = EnvironmentDetailsView::render_human_readable(&data);
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
        };

        // When
        let output = EnvironmentDetailsView::render_human_readable(&data);

        // Then
        assert!(output.contains("/absolute/path/data/my-env"));
        assert!(output.contains("/absolute/path/build/my-env"));
    }
}
