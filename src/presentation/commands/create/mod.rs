//! Create Command Presentation Module
//!
//! This module implements the CLI presentation layer for the create command,
//! handling Figment integration for configuration file parsing, argument
//! processing, and user interaction.
//!
//! ## Architecture
//!
//! The create command presentation layer follows the existing patterns from
//! the destroy command and integrates with the application layer's
//! `CreateCommandHandler`. Figment is used as a delivery mechanism and stays
//! in the presentation layer following DDD boundaries.
//!
//! ## Components
//!
//! - `config_loader` - Figment integration for JSON configuration loading
//! - `errors` - Presentation layer error types with `.help()` methods
//! - `subcommand` - Main command handler orchestrating the workflow
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::presentation::commands::create;
//!
//! if let Err(e) = create::handle(
//!     Path::new("config/environment.json"),
//!     Path::new(".")
//! ) {
//!     eprintln!("Create failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! ```

pub mod config_loader;
pub mod errors;
pub mod subcommand;

#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use config_loader::ConfigLoader;
pub use errors::{ConfigFormat, CreateSubcommandError};
pub use subcommand::handle_create_command;
