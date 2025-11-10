//! Command Handlers Module
//!
//! This module provides unified command execution and error handling for all CLI commands.
//! It serves as the central dispatch point for command execution and provides consistent
//! error handling across all commands.

use std::sync::{Arc, Mutex};

use crate::presentation::errors::CommandError;
use crate::presentation::input::cli::Commands;
use crate::presentation::user_output::UserOutput;

// Re-export command modules
pub mod constants;
pub mod context;
pub mod create;
pub mod destroy;
pub mod factory;

// Shared test utilities
#[cfg(test)]
pub mod tests;

// Future command modules will be added here:
// pub mod provision;
// pub mod configure;
// pub mod release;

/// Execute the given command
///
/// **DEPRECATED**: This function is deprecated in favor of the new Dispatch Layer.
/// Use `crate::presentation::dispatch::route_command` instead.
///
/// This function will be removed in a future version. The new dispatch layer
/// provides better separation of concerns and cleaner architecture.
///
/// # Migration Guide
///
/// Old code:
/// ```rust,ignore
/// use std::sync::{Arc, Mutex};
/// use crate::presentation::{commands, user_output::UserOutput};
///
/// let user_output = Arc::new(Mutex::new(UserOutput::new(/* ... */)));
/// commands::execute(command, working_dir, &user_output)?;
/// ```
///
/// New code:
/// ```rust,ignore
/// use std::sync::Arc;
/// use crate::bootstrap::Container;
/// use crate::presentation::dispatch::{route_command, ExecutionContext};
///
/// let container = Arc::new(Container::new());
/// let context = ExecutionContext::new(container);
/// route_command(command, working_dir, &context)?;
/// ```
///
/// This function serves as the central dispatcher for all CLI commands.
/// It matches the command type and delegates execution to the appropriate
/// command handler module.
///
/// # Arguments
///
/// * `command` - The parsed CLI command to execute
/// * `working_dir` - Working directory for environment data storage
/// * `user_output` - Shared user output service for consistent output formatting
///
/// # Returns
///
/// Returns `Ok(())` on successful execution, or a `CommandError` if execution fails.
/// The error contains detailed context and actionable troubleshooting information.
///
/// # Errors
///
/// Returns an error if command execution fails.
///
/// # Example
///
/// ```rust
/// use clap::Parser;
/// use torrust_tracker_deployer_lib::presentation::{input::cli, commands, user_output};
/// use std::{path::Path, sync::{Arc, Mutex}};
///
/// let cli = cli::Cli::parse();
/// if let Some(command) = cli.command {
///     let working_dir = Path::new(".");
///     let user_output = Arc::new(Mutex::new(user_output::UserOutput::new(user_output::VerbosityLevel::Normal)));
///     let result = commands::execute(command, working_dir, &user_output);
///     match result {
///         Ok(_) => println!("Command executed successfully"),
///         Err(e) => commands::handle_error(&e, &user_output),
///     }
/// }
/// ```
#[deprecated(
    since = "0.1.0",
    note = "Use `crate::presentation::dispatch::route_command` instead"
)]
///
/// ```rust
/// use clap::Parser;
/// use torrust_tracker_deployer_lib::presentation::{input::cli, commands, user_output};
/// use std::{path::Path, sync::{Arc, Mutex}};
///
/// let cli = cli::Cli::parse();
/// if let Some(command) = cli.command {
///     let working_dir = Path::new(".");
///     let user_output = Arc::new(Mutex::new(user_output::UserOutput::new(user_output::VerbosityLevel::Normal)));
///     let result = commands::execute(command, working_dir, &user_output);
///     match result {
///         Ok(_) => println!("Command executed successfully"),
///         Err(e) => commands::handle_error(&e, &user_output),
///     }
/// }
/// ```
pub fn execute(
    command: Commands,
    working_dir: &std::path::Path,
    user_output: &Arc<Mutex<UserOutput>>,
) -> Result<(), CommandError> {
    match command {
        Commands::Create { action } => {
            create::handle_create_command(action, working_dir, user_output)?;
            Ok(())
        }
        Commands::Destroy { environment } => {
            destroy::handle_destroy_command(&environment, working_dir, user_output)?;
            Ok(())
        } // Future commands will be added here:
          //
          // Commands::Provision { environment, provider } => {
          //     provision::handle(&environment, &provider)?;
          //     Ok(())
          // }
          //
          // Commands::Configure { environment } => {
          //     configure::handle(&environment)?;
          //     Ok(())
          // }
          //
          // Commands::Release { environment, version } => {
          //     release::handle(&environment, &version)?;
          //     Ok(())
          // }
    }
}

/// Handle command errors with consistent user output
///
/// This function provides standardized error output for all command failures.
/// It displays the error message and detailed troubleshooting information
/// to help users resolve issues.
///
/// # Arguments
///
/// * `error` - The command error to handle and display
/// * `user_output` - Shared user output service for consistent output formatting
///
/// # Example
///
/// ```rust
/// use clap::Parser;
/// use torrust_tracker_deployer_lib::presentation::{commands, input::cli, errors, user_output};
/// use torrust_tracker_deployer_lib::presentation::commands::destroy::DestroySubcommandError;
/// use torrust_tracker_deployer_lib::domain::environment::name::EnvironmentNameError;
/// use std::sync::{Arc, Mutex};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Example of handling a command error (simulated for testing)
/// let name_error = EnvironmentNameError::InvalidFormat {
///     attempted_name: "invalid_name".to_string(),
///     reason: "contains invalid characters: _".to_string(),
///     valid_examples: vec!["dev".to_string(), "staging".to_string()],
/// };
/// let sample_error = errors::CommandError::Destroy(
///     Box::new(DestroySubcommandError::InvalidEnvironmentName {
///         name: "invalid_name".to_string(),
///         source: name_error,
///     })
/// );
/// let user_output = Arc::new(Mutex::new(user_output::UserOutput::new(user_output::VerbosityLevel::Normal)));
/// commands::handle_error(&sample_error, &user_output);
/// # Ok(())
/// # }
/// ```
pub fn handle_error(error: &CommandError, user_output: &Arc<Mutex<UserOutput>>) {
    let help_text = error.help();

    if let Ok(mut output) = user_output.lock() {
        output.error(&format!("{error}"));
        output.blank_line();
        output.info_block("For detailed troubleshooting:", &[help_text]);
    } else {
        // Cannot acquire lock - print to stderr directly as fallback
        //
        // RATIONALE: Plain text formatting without emojis/styling is intentional.
        // When the mutex is poisoned, we're in a degraded error state where another
        // thread has panicked. Using plain eprintln! ensures maximum compatibility
        // and avoids any additional complexity that could fail in this critical path.
        // The goal here is reliability over aesthetics - get the error message to
        // the user no matter what, even if it's not pretty.
        eprintln!("ERROR: {error}");
        eprintln!();
        eprintln!("CRITICAL: Failed to acquire user output lock.");
        eprintln!("This indicates a panic occurred in another thread.");
        eprintln!();
        eprintln!("For detailed troubleshooting:");
        eprintln!("{help_text}");
    }
}
