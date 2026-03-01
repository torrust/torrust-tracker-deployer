//! Exists Command Handler
//!
//! This module handles the exists command execution at the presentation layer,
//! checking whether an environment exists and outputting a boolean result.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::exists::{
    ExistsCommandHandler, ExistsCommandHandlerError,
};
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::commands::exists::{ExistsResult, JsonView, TextView};
use crate::presentation::cli::views::Render;
use crate::presentation::cli::views::UserOutput;

use super::errors::ExistsSubcommandError;

/// Presentation layer controller for exists command workflow
///
/// Checks whether an environment exists and outputs a boolean result.
/// This is a read-only command that checks local data only.
///
/// ## Responsibilities
///
/// - Validate environment name format
/// - Delegate to application layer for existence check
/// - Output `true` or `false` to stdout
///
/// ## Output Contract
///
/// - **stdout**: `true` or `false` (bare value, valid JSON)
/// - **exit code 0**: Command completed successfully (result is on stdout)
/// - **exit code 1**: An error occurred (e.g., repository failure)
///
/// ## Architecture
///
/// This controller intentionally does NOT use `ProgressReporter` because
/// the exists check is a sub-millisecond operation. Progress reporting
/// would add noise without value.
pub struct ExistsCommandController {
    handler: ExistsCommandHandler,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
}

impl ExistsCommandController {
    /// Create a new `ExistsCommandController` with dependencies
    ///
    /// # Arguments
    ///
    /// * `repository` - Environment repository for checking existence
    /// * `user_output` - Shared output service for result display
    #[allow(clippy::needless_pass_by_value)] // Arc parameters are moved to constructor for ownership
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let handler = ExistsCommandHandler::new(repository);

        Self {
            handler,
            user_output,
        }
    }

    /// Execute the exists command workflow
    ///
    /// This method orchestrates a simple workflow:
    /// 1. Validate environment name
    /// 2. Check existence via application layer
    /// 3. Output boolean result to stdout
    ///
    /// # Arguments
    ///
    /// * `environment_name` - Name of the environment to check
    /// * `output_format` - Output format (Text or Json)
    ///
    /// # Errors
    ///
    /// Returns `ExistsSubcommandError` if any step fails
    pub fn execute(
        &self,
        environment_name: &str,
        output_format: OutputFormat,
    ) -> Result<(), ExistsSubcommandError> {
        // Step 1: Validate environment name
        let env_name = Self::validate_environment_name(environment_name)?;

        // Step 2: Check existence via application layer
        let result = self
            .handler
            .execute(&env_name)
            .map_err(|e| Self::map_handler_error(e, &env_name))?;

        // Step 3: Output boolean result
        self.display_result(&result, output_format)?;

        Ok(())
    }

    /// Step 1: Validate environment name format
    fn validate_environment_name(name: &str) -> Result<EnvironmentName, ExistsSubcommandError> {
        EnvironmentName::new(name.to_string()).map_err(|source| {
            ExistsSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })
    }

    /// Map application layer errors to presentation errors
    fn map_handler_error(
        error: ExistsCommandHandlerError,
        env_name: &EnvironmentName,
    ) -> ExistsSubcommandError {
        match error {
            ExistsCommandHandlerError::RepositoryError(e) => {
                ExistsSubcommandError::ExistenceCheckFailed {
                    name: env_name.to_string(),
                    message: e.to_string(),
                }
            }
        }
    }

    /// Step 3: Display boolean result
    ///
    /// Outputs `true` or `false` to stdout. The output is the same
    /// for both Text and Json formats since bare `true`/`false` are
    /// valid JSON values.
    fn display_result(
        &self,
        result: &ExistsResult,
        output_format: OutputFormat,
    ) -> Result<(), ExistsSubcommandError> {
        let output = match output_format {
            OutputFormat::Text => TextView::render(result)?,
            OutputFormat::Json => JsonView::render(result)?,
        };

        // Write result to stdout via UserOutput
        self.user_output.lock().borrow_mut().result(&output);

        Ok(())
    }
}
