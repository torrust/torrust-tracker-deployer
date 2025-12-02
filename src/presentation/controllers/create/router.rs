//! Create Command Router
//!
//! This module handles the create command execution at the presentation layer,
//! routing between different subcommands (environment creation or template generation).

use std::path::Path;

use crate::presentation::dispatch::ExecutionContext;
use crate::presentation::input::cli::commands::CreateAction;

use super::errors::CreateCommandError;

/// Route the create command to its appropriate subcommand
///
/// This function routes between different create subcommands (environment or template).
///
/// # Arguments
///
/// * `action` - The create action to perform (environment creation or template generation)
/// * `working_dir` - Root directory for environment data storage
/// * `context` - Execution context providing access to application services
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateCommandError` on failure.
///
/// # Errors
///
/// Returns an error if the subcommand execution fails.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub async fn route_command(
    action: CreateAction,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CreateCommandError> {
    match action {
        CreateAction::Environment { env_file } => context
            .container()
            .create_environment_controller()
            .execute(&env_file, working_dir)
            .await
            .map(|_| ()) // Convert Environment<Created> to ()
            .map_err(CreateCommandError::Environment),
        CreateAction::Template {
            output_path,
            provider,
        } => {
            let template_path = output_path.unwrap_or_else(CreateAction::default_template_path);
            context
                .container()
                .create_template_controller()
                .execute(&template_path, provider)
                .await
                .map_err(CreateCommandError::Template)
        }
    }
}
