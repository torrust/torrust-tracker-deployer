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
//! ```rust
//! use std::sync::Arc;
//! use crate::bootstrap::Container;
//! use crate::presentation::dispatch::{route_command, ExecutionContext};
//! use crate::presentation::input::Commands;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let container = Arc::new(Container::new());
//! let context = ExecutionContext::new(container);
//! let command = Commands::Create { action: todo!() };
//!
//! // Route command to appropriate handler
//! route_command(command, &context).await?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;

use crate::presentation::commands::{create, destroy};
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
/// ```rust
/// use std::path::Path;
/// use std::sync::Arc;
/// use crate::bootstrap::Container;
/// use crate::presentation::dispatch::{route_command, ExecutionContext};
/// use crate::presentation::input::Commands;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let container = Arc::new(Container::new());
/// let context = ExecutionContext::new(container);
/// let command = Commands::Create { action: todo!() };
/// let working_dir = Path::new(".");
///
/// match route_command(command, working_dir, &context) {
///     Ok(()) => println!("Command completed successfully"),
///     Err(e) => eprintln!("Command failed: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
pub fn route_command(
    command: Commands,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CommandError> {
    match command {
        Commands::Create { action } => {
            create::handle_create_command(action, working_dir, &context.user_output())?;
            Ok(())
        }
        Commands::Destroy { environment } => {
            destroy::handle_destroy_command(&environment, working_dir, &context.user_output())?;
            Ok(())
        } // Future commands will be added here as the Controller Layer expands:
          //
          // Commands::Provision { environment, provider } => {
          //     provision::handle_provision_command(&environment, &provider, context)?;
          //     Ok(())
          // }
          //
          // Commands::Configure { environment } => {
          //     configure::handle_configure_command(&environment, context)?;
          //     Ok(())
          // }
          //
          // Commands::Release { environment, version } => {
          //     release::handle_release_command(&environment, &version, context)?;
          //     Ok(())
          // }
    }
}
