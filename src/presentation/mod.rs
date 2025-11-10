//! Presentation Layer
//!
//! This layer handles user-facing output and presentation concerns following DDD architecture.
//! It manages how information is presented to users, separate from internal logging and
//! application logic.
//!
//! ## Responsibilities
//!
//! - **User Output**: Managing user-facing messages, progress updates, and result presentation
//! - **CLI Interface**: Command-line argument parsing and subcommand definitions
//! - **Command Execution**: Coordinating command handlers and providing unified error handling
//! - **Output Channels**: Implementing proper stdout/stderr separation for CLI applications
//! - **Verbosity Control**: Handling different levels of output detail based on user preferences
//! - **Output Formatting**: Structuring output for both human consumption and automation/piping
//!
//! ## Design Principles
//!
//! - **Channel Separation**: Following Unix conventions with stdout for results and stderr for operational messages
//! - **Automation Friendly**: Supporting clean piping and redirection for scripting
//! - **User Experience**: Providing clear, actionable feedback without interfering with result data
//! - **Verbosity Levels**: Respecting user preferences for output detail
//! - **Error Conventions**: Following project error handling guidelines with structured errors and tiered help
//!
//! ## Module Structure
//!
//! ```text
//! presentation/
//! ├── input/            # Input Layer - Input parsing and validation
//! │   └── cli/          # CLI argument parsing and structure
//! │       ├── args.rs   # Global CLI arguments (logging config)
//! │       ├── commands.rs # Subcommand definitions
//! │       └── mod.rs    # Main Cli struct and parsing logic
//! ├── dispatch/         # Dispatch Layer - Command routing and execution context
//! │   ├── mod.rs        # Layer exports and documentation
//! │   ├── router.rs     # Command routing logic (route_command function)
//! │   └── context.rs    # ExecutionContext wrapper around Container
//! ├── commands/         # Command execution handlers
//! │   ├── destroy.rs    # Destroy command handler
//! │   └── mod.rs        # Unified command dispatch and error handling
//! ├── errors.rs         # Unified error types for all commands
//! ├── user_output.rs    # User-facing output management
//! └── mod.rs            # This file - layer exports and documentation
//! ```

// Core presentation modules
pub mod commands;
pub mod dispatch;
pub mod error;
pub mod errors;
pub mod input;
pub mod progress;
pub mod user_output;

// Re-export commonly used presentation types for convenience
pub use commands::create::CreateSubcommandError;
pub use commands::destroy::DestroySubcommandError;

// Re-export error handling function from error module
pub use error::handle_error;

pub use errors::CommandError;
pub use input::{Cli, Commands, GlobalArgs};
pub use progress::ProgressReporter;
pub use user_output::{Theme, UserOutput, VerbosityLevel};
