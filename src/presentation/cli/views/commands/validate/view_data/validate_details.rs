//! Validate Details Data Transfer Object
//!
//! This module contains the presentation DTO for validate command details.
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

use std::path::Path;

use serde::Serialize;

use crate::application::command_handlers::validate::ValidationResult;

/// Validate details data for rendering
///
/// This struct holds all the data needed to render validate command
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
/// Unlike domain-backed DTOs (which use `From<&Environment<State>>`),
/// `ValidateDetailsData` combines two inputs (`&Path` + `&ValidationResult`).
/// A named constructor `from_result` is used instead of `From` to keep the API clear.
#[allow(clippy::struct_excessive_bools)] // Intentional: presentation data with feature flags
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidateDetailsData {
    /// Name of the validated environment
    pub environment_name: String,
    /// Path to the validated configuration file (as displayed to the user)
    pub config_file: String,
    /// Infrastructure provider (lowercase: "lxd", "hetzner", etc.)
    pub provider: String,
    /// Always `true` when the command exits successfully
    pub is_valid: bool,
    /// Whether Prometheus monitoring is configured
    pub has_prometheus: bool,
    /// Whether Grafana dashboard is configured
    pub has_grafana: bool,
    /// Whether HTTPS is configured
    pub has_https: bool,
    /// Whether backups are configured
    pub has_backup: bool,
}

impl ValidateDetailsData {
    /// Construct a `ValidateDetailsData` from an env file path and validation result
    ///
    /// # Arguments
    ///
    /// * `env_file` - Path to the configuration file that was validated
    /// * `result` - Successful validation result from the application layer
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::validate::ValidateDetailsData;
    /// use torrust_tracker_deployer_lib::application::command_handlers::validate::ValidationResult;
    ///
    /// let result = ValidationResult {
    ///     environment_name: "my-env".to_string(),
    ///     provider: "lxd".to_string(),
    ///     has_prometheus: true,
    ///     has_grafana: false,
    ///     has_https: false,
    ///     has_backup: false,
    /// };
    ///
    /// let data = ValidateDetailsData::from_result(Path::new("envs/my-env.json"), &result);
    ///
    /// assert_eq!(data.environment_name, "my-env");
    /// assert_eq!(data.is_valid, true);
    /// ```
    #[must_use]
    pub fn from_result(env_file: &Path, result: &ValidationResult) -> Self {
        Self {
            environment_name: result.environment_name.clone(),
            config_file: env_file.display().to_string(),
            provider: result.provider.clone(),
            is_valid: true,
            has_prometheus: result.has_prometheus,
            has_grafana: result.has_grafana,
            has_https: result.has_https,
            has_backup: result.has_backup,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_result() -> ValidationResult {
        ValidationResult {
            environment_name: "test-env".to_string(),
            provider: "lxd".to_string(),
            has_prometheus: true,
            has_grafana: false,
            has_https: false,
            has_backup: true,
        }
    }

    #[test]
    fn it_should_build_dto_from_result() {
        // Arrange
        let result = create_sample_result();
        let path = Path::new("envs/test-env.json");

        // Act
        let data = ValidateDetailsData::from_result(path, &result);

        // Assert
        assert_eq!(data.environment_name, "test-env");
        assert_eq!(data.config_file, "envs/test-env.json");
        assert_eq!(data.provider, "lxd");
        assert!(data.is_valid);
        assert!(data.has_prometheus);
        assert!(!data.has_grafana);
        assert!(!data.has_https);
        assert!(data.has_backup);
    }

    #[test]
    fn it_should_always_set_is_valid_to_true() {
        // Arrange — no validation failure scenario can produce a ValidateDetailsData
        // (only success paths call from_result)
        let result = create_sample_result();
        let path = Path::new("envs/test-env.json");

        // Act
        let data = ValidateDetailsData::from_result(path, &result);

        // Assert
        assert!(data.is_valid, "is_valid must always be true");
    }
}
