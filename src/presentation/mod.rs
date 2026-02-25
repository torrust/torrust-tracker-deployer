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
//! ├── cli/   # CLI delivery mechanism (Clap-based command-line interface)
//! ├── sdk/   # SDK delivery mechanism (programmatic Rust API)
//! └── mod.rs # This file - declares both delivery mechanisms
//! ```
//!
//! The two delivery mechanisms are independent — neither should import from the other.
//! Each sub-module handles its own input parsing, routing, and output formatting.

pub mod cli;
pub mod sdk;

// Re-export commonly used CLI types for backward compatibility.
// External consumers can use either `presentation::Cli` or `presentation::cli::Cli`.
pub use cli::{
    handle_error, Cli, CommandError, Commands, CreateCommandError, CreateEnvironmentCommandError,
    CreateEnvironmentTemplateCommandError, DestroySubcommandError, GlobalArgs, ProgressReporter,
    Theme, UserOutput, VerbosityLevel,
};
