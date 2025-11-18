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

use crate::presentation::controllers::{create, destroy, provision};
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
            destroy::handle(&environment, working_dir, context).await?;
            Ok(())
        }
        Commands::Provision { environment } => {
            provision::handle(&environment, working_dir, context).await?;
            Ok(())
        } // Future commands will be added here as the Controller Layer expands:
          //
          // Commands::Configure { environment } => {
          //     configure::handle_configure_command(&environment, context).await?;
          //     Ok(())
          // }
          //
          // Commands::Release { environment, version } => {
          //     release::handle_release_command(&environment, &version, context).await?;
          //     Ok(())
          // }
    }
}
