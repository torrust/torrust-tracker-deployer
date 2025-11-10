//! Create Command Handler
//!
//! This module handles the create command execution at the presentation layer,
//! routing between different subcommands (environment creation or template generation).

use std::path::Path;

use crate::presentation::dispatch::ExecutionContext;
use crate::presentation::input::cli::commands::CreateAction;

use super::errors::CreateSubcommandError;
use super::subcommands;

/// Handle the create command with its subcommands
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
/// Returns `Ok(())` on success, or a `CreateSubcommandError` on failure.
///
/// # Errors
///
/// Returns an error if the subcommand execution fails.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle(
    action: CreateAction,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CreateSubcommandError> {
    match action {
        CreateAction::Environment { env_file } => {
            // TODO: Temporarily using old signature until environment subcommand is updated
            subcommands::handle_environment_creation(&env_file, working_dir, &context.user_output())
        }
        CreateAction::Template { output_path } => {
            let template_path = output_path.unwrap_or_else(CreateAction::default_template_path);
            subcommands::handle_template_generation(&template_path, context)
        }
    }
}
