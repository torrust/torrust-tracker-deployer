//! Command Router
//!
//! This module provides the central command routing functionality for the Dispatch Layer.
//! It contains the `route_command` function that matches parsed CLI commands to their
//! appropriate handler functions.
//!
//! ## Purpose
//!
//! The router extracts command dispatch logic from the main application bootstrap and
//! the presentation commands module, creating a clean separation between:
//!
//! - **Command parsing** (Input Layer - already done)
//! - **Command routing** (This module - routes commands to handlers)
//! - **Command execution** (Controller Layer - executes business logic)
//! - **Result presentation** (View Layer - displays results)
//!
//! ## Design
//!
//! ```text
//! Commands enum → route_command() → Handler function
//!      ↓                ↓                    ↓
//! Parsed input    Route decision      Business logic
//! ```
//!
//! ## Benefits
//!
//! - **Centralized Routing**: All command routing logic in one place
//! - **Type Safety**: Compile-time guarantees that all commands are handled
//! - **Testability**: Router can be tested independently of handlers
//! - **Maintainability**: Easy to add new commands or modify routing logic
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use std::path::Path;
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::dispatch::{route_command, ExecutionContext};
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//! // Note: Commands enum requires specific action parameters in practice
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let container = Container::new(VerbosityLevel::Normal);
//! let context = ExecutionContext::new(Arc::new(container));
//! let working_dir = Path::new(".");
//!
//! // Route command to appropriate handler
//! // Note: Commands require proper construction with actions
//! # Ok(())
//! # }
//! ```

use std::path::Path;

use crate::presentation::controllers::create;
use crate::presentation::errors::CommandError;
use crate::presentation::input::Commands;

use super::ExecutionContext;

/// Route a parsed command to its appropriate handler
///
/// This function serves as the central dispatch point for all CLI commands.
/// It takes a parsed command and an execution context, then routes the command
/// to the appropriate handler function in the Controllers layer.
///
/// # Arguments
///
/// * `command` - Parsed command from the Input Layer
/// * `working_dir` - Working directory for command execution
/// * `context` - Execution context providing access to application services
///
/// # Returns
///
/// Returns `Ok(())` on successful command execution, or a `CommandError`
/// if the command fails. The error contains detailed context and actionable
/// troubleshooting information.
///
/// # Errors
///
/// Returns an error if:
/// - Command handler execution fails
/// - Required services are not available in the context
/// - Command parameters are invalid
///
/// # Examples
///
/// ```text
/// use std::path::Path;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::bootstrap::Container;
/// use torrust_tracker_deployer_lib::presentation::dispatch::{route_command, ExecutionContext};
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
/// // Note: Commands enum requires specific action parameters in practice
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let container = Container::new(VerbosityLevel::Normal);
///     let context = ExecutionContext::new(Arc::new(container));
///     let working_dir = Path::new(".");
///
///     // Route command to appropriate handler - requires proper Commands construction
///     // route_command(command, working_dir, &context).await?;
///     Ok(())
/// }
/// ```
#[allow(clippy::too_many_lines)]
pub async fn route_command(
    command: Commands,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CommandError> {
    match command {
        Commands::Create { action } => {
            create::route_command(action, working_dir, context).await?;
            Ok(())
        }
        Commands::Destroy { environment } => {
            context
                .container()
                .create_destroy_controller()
                .execute(&environment)
                .await?;
            Ok(())
        }
        Commands::Purge { environment, force } => {
            context
                .container()
                .create_purge_controller()
                .execute(&environment, force)
                .await?;
            Ok(())
        }
        Commands::Provision { environment } => {
            context
                .container()
                .create_provision_controller()
                .execute(&environment)
                .await?;
            Ok(())
        }
        Commands::Configure { environment } => {
            context
                .container()
                .create_configure_controller()
                .execute(&environment)?;
            Ok(())
        }
        Commands::Test { environment } => {
            context
                .container()
                .create_test_controller()
                .execute(&environment)
                .await?;
            Ok(())
        }
        Commands::Validate { env_file } => {
            context
                .container()
                .create_validate_controller()
                .execute(&env_file)?;
            Ok(())
        }
        Commands::Register {
            environment,
            instance_ip,
            ssh_port,
        } => {
            context
                .container()
                .create_register_controller()
                .execute(&environment, &instance_ip, ssh_port)
                .await?;
            Ok(())
        }
        Commands::Release { environment } => {
            context
                .container()
                .create_release_controller()
                .execute(&environment)
                .await?;
            Ok(())
        }
        Commands::Render {
            env_name,
            env_file,
            instance_ip,
        } => {
            context
                .container()
                .create_render_controller()
                .execute(env_name.as_deref(), env_file.as_deref(), &instance_ip)
                .await?;
            Ok(())
        }
        Commands::Run { environment } => {
            context
                .container()
                .create_run_controller()
                .execute(&environment)
                .await?;
            Ok(())
        }
        Commands::Show { environment } => {
            context
                .container()
                .create_show_controller()
                .execute(&environment)?;
            Ok(())
        }
        Commands::List => {
            context.container().create_list_controller().execute()?;
            Ok(())
        }
    }
}
