//! Render Details Data Transfer Object
//!
//! This module contains the presentation DTO for render command details.
//! It serves as the data structure passed to view renderers (`TextView`, `JsonView`, etc.).
//!
//! # Architecture
//!
//! This follows the Strategy Pattern where:
//! - This DTO is the data passed to all rendering strategies
//! - Different views (`TextView`, `JsonView`) consume this data
//! - Adding new formats doesn't modify this DTO or existing views
//!
//! # SOLID Principles
//!
//! - **Single Responsibility**: This file only defines the data structure
//! - **Open/Closed**: New formats extend by adding views, not modifying this
//! - **Separation of Concerns**: Data definition separate from rendering logic

use serde::Serialize;

use crate::application::command_handlers::render::RenderResult;

/// Render details data for rendering
///
/// This struct holds all the data needed to render render command
/// information for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples application types from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
///
/// # Named Constructor vs `From`
///
/// A named constructor `from_result` is used because it provides a clear API
/// and `RenderResult` is a single input (unlike DTOs combining multiple sources).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RenderDetailsData {
    /// Name of the environment whose artifacts were generated
    pub environment_name: String,
    /// Description of the configuration source (env name or config file)
    pub config_source: String,
    /// IP address used in artifact generation
    pub target_ip: String,
    /// Absolute path to the directory containing generated artifacts
    pub output_dir: String,
}

impl RenderDetailsData {
    /// Construct a `RenderDetailsData` from a render result
    ///
    /// # Arguments
    ///
    /// * `result` - Successful render result from the application layer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::net::IpAddr;
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::application::command_handlers::render::RenderResult;
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::render::RenderDetailsData;
    ///
    /// let result = RenderResult {
    ///     environment_name: "my-env".to_string(),
    ///     config_source: "Config file: envs/my-env.json".to_string(),
    ///     target_ip: "192.168.1.100".parse::<IpAddr>().unwrap(),
    ///     output_dir: PathBuf::from("/tmp/build/my-env"),
    /// };
    ///
    /// let data = RenderDetailsData::from_result(&result);
    ///
    /// assert_eq!(data.environment_name, "my-env");
    /// assert_eq!(data.target_ip, "192.168.1.100");
    /// ```
    #[must_use]
    pub fn from_result(result: &RenderResult) -> Self {
        Self {
            environment_name: result.environment_name.clone(),
            config_source: result.config_source.clone(),
            target_ip: result.target_ip.to_string(),
            output_dir: result.output_dir.display().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::path::PathBuf;

    use super::*;

    fn create_sample_result() -> RenderResult {
        RenderResult {
            environment_name: "test-env".to_string(),
            config_source: "Config file: envs/test-env.json".to_string(),
            target_ip: "192.168.1.100".parse::<IpAddr>().unwrap(),
            output_dir: PathBuf::from("/tmp/build/test-env"),
        }
    }

    #[test]
    fn it_should_build_dto_from_result() {
        // Arrange
        let result = create_sample_result();

        // Act
        let data = RenderDetailsData::from_result(&result);

        // Assert
        assert_eq!(data.environment_name, "test-env");
        assert_eq!(data.config_source, "Config file: envs/test-env.json");
        assert_eq!(data.target_ip, "192.168.1.100");
        assert_eq!(data.output_dir, "/tmp/build/test-env");
    }

    #[test]
    fn it_should_convert_ip_to_string() {
        // Arrange
        let result = create_sample_result();

        // Act
        let data = RenderDetailsData::from_result(&result);

        // Assert - IpAddr is converted to string
        assert_eq!(data.target_ip, "192.168.1.100");
    }

    #[test]
    fn it_should_convert_output_dir_to_string() {
        // Arrange
        let result = create_sample_result();

        // Act
        let data = RenderDetailsData::from_result(&result);

        // Assert - PathBuf is converted to string via display()
        assert_eq!(data.output_dir, "/tmp/build/test-env");
    }
}
