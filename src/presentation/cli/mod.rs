//! CLI Delivery Mechanism
//!
//! This module contains the CLI-specific presentation code that implements the
//! command-line interface for the deployer. It follows a four-layer MVC architecture:
//!
//! ```text
//! Input → Dispatch → Controllers → Views
//! ```
//!
//! | Layer           | Purpose                                        |
//! |-----------------|------------------------------------------------|
//! | **Input**       | CLI argument parsing and validation (Clap)     |
//! | **Dispatch**    | Command routing and execution context          |
//! | **Controllers** | Command handling and business logic coordination |
//! | **Views**       | Output formatting and presentation             |

pub mod controllers;
pub mod dispatch;
pub mod error;
pub mod errors;
pub mod input;
pub mod views;

// Re-export commonly used CLI types for convenience
pub use controllers::create::{
    CreateCommandError, CreateEnvironmentCommandError, CreateEnvironmentTemplateCommandError,
};
pub use controllers::destroy::DestroySubcommandError;
pub use error::handle_error;
pub use errors::CommandError;
pub use input::{Cli, Commands, GlobalArgs};
pub use views::progress::ProgressReporter;
pub use views::{Theme, UserOutput, VerbosityLevel};

#[cfg(test)]
mod tests {
    mod reentrancy_fix_test;
}
