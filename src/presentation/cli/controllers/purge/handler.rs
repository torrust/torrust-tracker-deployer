//! Purge Command Handler
//!
//! This module handles the purge command execution at the presentation layer,
//! including environment validation, confirmation prompts, and user interaction.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::purge::handler::PurgeCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::commands::purge::{JsonView, PurgeDetailsData, TextView};
use crate::presentation::cli::views::progress::ProgressReporter;
use crate::presentation::cli::views::Render;
use crate::presentation::cli::views::UserOutput;

use super::errors::PurgeSubcommandError;

/// Steps in the purge workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PurgeStep {
    ValidateEnvironment,
    ConfirmOperation,
    PurgeLocalData,
}

impl PurgeStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateEnvironment,
        Self::ConfirmOperation,
        Self::PurgeLocalData,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateEnvironment => "Validating environment",
            Self::ConfirmOperation => "Confirming operation",
            Self::PurgeLocalData => "Purging local data",
        }
    }
}

/// Presentation layer controller for purge command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// before delegating to the application layer `PurgeCommandHandler`.
///
/// # Responsibilities
///
/// - Validate user input (environment name format)
/// - Show progress updates to the user
/// - Handle confirmation prompts (unless --force is provided)
/// - Format success/error messages for display
/// - Delegate business logic to application layer
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. It delegates actual business logic to the application layer's
/// `PurgeCommandHandler`, maintaining clear separation of concerns.
pub struct PurgeCommandController {
    handler: PurgeCommandHandler,
    progress: ProgressReporter,
}

impl PurgeCommandController {
    /// Create a new purge command controller
    ///
    /// Creates a `PurgeCommandController` with the application handler.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        handler: PurgeCommandHandler,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, PurgeStep::count());

        Self { handler, progress }
    }

    /// Execute the complete purge workflow
    ///
    /// Orchestrates all steps of the purge command:
    /// 1. Validate environment name
    /// 2. Confirm operation (unless --force is provided)
    /// 3. Purge local data
    /// 4. Complete with success message
    ///
    /// # Arguments
    ///
    /// * `environment_name` - The name of the environment to purge
    /// * `force` - Skip confirmation prompt if true
    /// * `output_format` - Output format (text or JSON)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment name is invalid (format validation fails)
    /// - Environment cannot be loaded from repository
    /// - User cancels operation at confirmation prompt
    /// - Purge operation fails
    /// - Progress reporting encounters a poisoned mutex
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `PurgeSubcommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    #[allow(clippy::unused_async)] // Part of uniform async presentation layer interface
    pub async fn execute(
        &mut self,
        environment_name: &str,
        force: bool,
        output_format: OutputFormat,
    ) -> Result<(), PurgeSubcommandError> {
        let env_name = self.validate_environment_name(environment_name)?;

        // Handle confirmation unless --force flag provided
        if !force {
            self.progress
                .start_step(PurgeStep::ConfirmOperation.description())?;

            // Show warning and prompt for confirmation
            self.show_confirmation_prompt(environment_name);

            // Read user response
            if !Self::read_user_confirmation()? {
                self.progress.complete_step(None)?;
                return Err(PurgeSubcommandError::UserCancelled);
            }

            self.progress.complete_step(None)?;
        }

        // Execute purge via application handler
        self.progress
            .start_step(PurgeStep::PurgeLocalData.description())?;
        self.handler.execute(&env_name).map_err(|source| {
            PurgeSubcommandError::PurgeOperationFailed {
                name: environment_name.to_string(),
                source,
            }
        })?;
        self.progress.complete_step(None)?;

        self.complete_workflow(environment_name, output_format)?;

        Ok(())
    }

    /// Validate the environment name format
    ///
    /// Shows progress to user and validates that the environment name
    /// meets domain requirements (1-63 chars, alphanumeric + hyphens).
    #[allow(clippy::result_large_err)]
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, PurgeSubcommandError> {
        self.progress
            .start_step(PurgeStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            PurgeSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress.complete_step(None)?;

        Ok(env_name)
    }

    /// Complete the workflow with success message
    ///
    /// Shows final success message to the user with workflow summary.
    /// Dispatches to `TextView` or `JsonView` based on `output_format`.
    #[allow(clippy::result_large_err)]
    fn complete_workflow(
        &mut self,
        environment_name: &str,
        output_format: OutputFormat,
    ) -> Result<(), PurgeSubcommandError> {
        let data = PurgeDetailsData::from_environment_name(environment_name);
        match output_format {
            OutputFormat::Text => {
                self.progress.complete(&TextView::render(&data)?)?;
            }
            OutputFormat::Json => {
                self.progress.result(&JsonView::render(&data)?)?;
            }
        }
        Ok(())
    }

    /// Show confirmation prompt with warning message
    ///
    /// Displays a warning about the irreversible nature of the purge operation
    /// and prompts the user to confirm.
    fn show_confirmation_prompt(&mut self, environment_name: &str) {
        let warning = format!(
            "⚠️  WARNING: This will permanently delete all local data for '{environment_name}':\n\
             • data/{environment_name}/ directory\n\
             • build/{environment_name}/ directory\n\
             • Environment registry entry\n\
             \n\
             This operation CANNOT be undone!\n"
        );

        self.progress.output().lock().borrow_mut().warn(&warning);

        self.progress
            .output()
            .lock()
            .borrow_mut()
            .progress("Are you sure you want to continue? (y/N): ");
    }

    /// Read user confirmation from stdin
    ///
    /// Returns `true` if user confirms (enters 'y' or 'Y'), `false` otherwise.
    #[allow(clippy::result_large_err)]
    fn read_user_confirmation() -> Result<bool, PurgeSubcommandError> {
        use std::io::{self, BufRead};

        let stdin = io::stdin();
        let mut line = String::new();

        stdin
            .lock()
            .read_line(&mut line)
            .map_err(|source| PurgeSubcommandError::IoError {
                operation: "reading user confirmation".to_string(),
                source,
            })?;

        let response = line.trim().to_lowercase();
        Ok(response == "y" || response == "yes")
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add unit tests in Phase 3 when implementing actual purge logic
    // Tests should cover:
    // - Valid environment name validation
    // - Invalid environment name rejection
    // - Force flag behavior
    // - Error handling for non-existent environments
}
