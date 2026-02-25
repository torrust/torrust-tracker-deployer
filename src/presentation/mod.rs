//! Presentation Layer
//!
//! This layer contains user-facing delivery mechanisms for the deployer application.
//! It follows DDD architecture, separating presentation concerns from application
//! and domain logic.
//!
//! ## Module Structure
//!
//! ```text
//! presentation/
//! └── cli/   # CLI delivery mechanism (Clap-based command-line interface)
//! ```
//!
//! The SDK delivery mechanism has been extracted into the `torrust-tracker-deployer-sdk`
//! workspace package (`packages/sdk/`).

pub mod cli;

// Re-export commonly used CLI types for backward compatibility.
// External consumers can use either `presentation::Cli` or `presentation::cli::Cli`.
pub use cli::{
    handle_error, Cli, CommandError, Commands, CreateCommandError, CreateEnvironmentCommandError,
    CreateEnvironmentTemplateCommandError, DestroySubcommandError, GlobalArgs, ProgressReporter,
    Theme, UserOutput, VerbosityLevel,
};
