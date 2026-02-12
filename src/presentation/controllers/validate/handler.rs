//! Validate Command Handler
//!
//! This module handles the validate command execution at the presentation layer,
//! including file validation and user feedback.

use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::validate::{ValidateCommandHandler, ValidationResult};
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

use super::errors::ValidateSubcommandError;

/// Steps in the validate workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidateStep {
    LoadConfiguration,
    ValidateSchema,
    ValidateFields,
}

impl ValidateStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::LoadConfiguration,
        Self::ValidateSchema,
        Self::ValidateFields,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::LoadConfiguration => "Loading configuration file",
            Self::ValidateSchema => "Validating JSON schema",
            Self::ValidateFields => "Validating configuration fields",
        }
    }
}

/// Presentation layer controller for validate command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// for validating environment configuration files without deployment.
///
/// # Responsibilities
///
/// - Validate file path exists and is readable
/// - Show progress updates to the user
/// - Format validation results for display
/// - Delegate validation logic to application layer
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. Business logic is delegated to the application layer's
/// `ValidateCommandHandler`.
pub struct ValidateCommandController {
    progress: ProgressReporter,
    handler: ValidateCommandHandler,
}

impl ValidateCommandController {
    /// Create a new validate command controller
    ///
    /// Creates a `ValidateCommandController` with user output.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>) -> Self {
        Self {
            progress: ProgressReporter::new(user_output, ValidateStep::count()),
            handler: ValidateCommandHandler::new(),
        }
    }

    /// Execute the validate command workflow
    ///
    /// Main entry point for the validate command. Orchestrates the validation
    /// workflow: loads configuration, validates schema and fields.
    ///
    /// # Arguments
    ///
    /// * `env_file` - Path to the environment configuration file
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful validation, or a `ValidateSubcommandError`
    /// if validation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File path does not exist
    /// - File is not readable
    /// - Configuration validation fails
    pub fn execute(&mut self, env_file: &Path) -> Result<(), ValidateSubcommandError> {
        // Step 1: Load Configuration (file existence check)
        self.progress
            .start_step(ValidateStep::LoadConfiguration.description())?;
        Self::validate_file_exists(env_file)?;
        self.progress
            .complete_step(Some("Configuration file loaded"))?;

        // Step 2: Validate Schema (JSON parsing)
        self.progress
            .start_step(ValidateStep::ValidateSchema.description())?;

        // Delegate actual validation to application layer
        let result = self.handler.validate(env_file).map_err(|source| {
            ValidateSubcommandError::ValidationFailed {
                path: env_file.to_path_buf(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some("Schema validation passed"))?;

        // Step 3: Validate Fields (domain rules)
        self.progress
            .start_step(ValidateStep::ValidateFields.description())?;
        self.progress
            .complete_step(Some("Field validation passed"))?;

        // Complete workflow with detailed results
        self.complete_workflow(env_file, &result)?;

        Ok(())
    }

    /// Validate that the configuration file exists and is readable
    fn validate_file_exists(env_file: &Path) -> Result<(), ValidateSubcommandError> {
        if !env_file.exists() {
            return Err(ValidateSubcommandError::ConfigFileNotFound {
                path: env_file.to_path_buf(),
            });
        }

        if !env_file.is_file() {
            return Err(ValidateSubcommandError::ConfigPathNotFile {
                path: env_file.to_path_buf(),
            });
        }

        Ok(())
    }

    /// Complete the workflow with success message and validation details
    fn complete_workflow(
        &mut self,
        env_file: &Path,
        result: &ValidationResult,
    ) -> Result<(), ValidateSubcommandError> {
        self.progress.blank_line()?;
        self.progress.complete(&format!(
            "Configuration file '{}' is valid\n\n\
            Environment Details:\n\
              • Name: {}\n\
              • Provider: {}\n\
              • Prometheus: {}\n\
              • Grafana: {}\n\
              • HTTPS: {}\n\
              • Backups: {}",
            env_file.display(),
            result.environment_name,
            result.provider,
            if result.has_prometheus {
                "Enabled"
            } else {
                "Disabled"
            },
            if result.has_grafana {
                "Enabled"
            } else {
                "Disabled"
            },
            if result.has_https {
                "Enabled"
            } else {
                "Disabled"
            },
            if result.has_backup {
                "Enabled"
            } else {
                "Disabled"
            }
        ))?;
        Ok(())
    }
}
