//! Command Handlers Module
//!
//! This module provides unified command execution and error handling for all CLI commands.
//! It serves as the central dispatch point for command execution and provides consistent
//! error handling across all commands.

use crate::presentation::cli::Commands;
use crate::presentation::errors::CommandError;

// Re-export command modules
pub mod create;
pub mod destroy;

// Future command modules will be added here:
// pub mod provision;
// pub mod configure;
// pub mod release;

/// Execute the given command
///
/// This function serves as the central dispatcher for all CLI commands.
/// It matches the command type and delegates execution to the appropriate
/// command handler module.
///
/// # Arguments
///
/// * `command` - The parsed CLI command to execute
/// * `working_dir` - Working directory for environment data storage
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
/// use torrust_tracker_deployer_lib::presentation::{cli, commands};
/// use std::path::Path;
///
/// let cli = cli::Cli::parse();
/// if let Some(command) = cli.command {
///     let working_dir = Path::new(".");
///     let result = commands::execute(command, working_dir);
///     match result {
///         Ok(_) => println!("Command executed successfully"),
///         Err(e) => commands::handle_error(&e),
///     }
/// }
/// ```
pub fn execute(command: Commands, working_dir: &std::path::Path) -> Result<(), CommandError> {
    match command {
        Commands::Create { action } => {
            create::handle_create_command(action, working_dir)?;
            Ok(())
        }
        Commands::Destroy { environment } => {
            destroy::handle(&environment, working_dir)?;
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
///
/// # Example
///
/// ```rust
/// use clap::Parser;
/// use torrust_tracker_deployer_lib::presentation::{commands, cli, errors};
/// use torrust_tracker_deployer_lib::presentation::commands::destroy::DestroyError;
/// use torrust_tracker_deployer_lib::domain::environment::name::EnvironmentNameError;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Example of handling a command error (simulated for testing)
/// let name_error = EnvironmentNameError::InvalidFormat {
///     attempted_name: "invalid_name".to_string(),
///     reason: "contains invalid characters: _".to_string(),
///     valid_examples: vec!["dev".to_string(), "staging".to_string()],
/// };
/// let sample_error = errors::CommandError::Destroy(
///     Box::new(DestroyError::InvalidEnvironmentName {
///         name: "invalid_name".to_string(),
///         source: name_error,
///     })
/// );
/// commands::handle_error(&sample_error);
/// # Ok(())
/// # }
/// ```
pub fn handle_error(error: &CommandError) {
    eprintln!("Error: {error}");
    eprintln!("\nFor detailed troubleshooting:");
    eprintln!("{}", error.help());
}
