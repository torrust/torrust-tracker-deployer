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
//! - `handler` - Main command handler routing between subcommands
//! - `subcommands` - Individual subcommand implementations (environment, template)
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use std::path::{Path, PathBuf};
//! use std::sync::{Arc, Mutex};
//! use torrust_tracker_deployer_lib::presentation::input::cli::commands::CreateAction;
//! use torrust_tracker_deployer_lib::presentation::controllers::create;
//! use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
//!
//! let action = CreateAction::Environment {
//!     env_file: PathBuf::from("config/environment.json")
//! };
//! // Note: ExecutionContext would be provided by the application bootstrap
//! # let context = todo!(); // Mock for documentation example
//!
//! if let Err(e) = create::handle(action, Path::new("."), &context) {
//!     eprintln!("Create failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! ```

pub mod config_loader;
pub mod errors;
pub mod handler;
pub mod subcommands;

// TODO: Update tests to use ExecutionContext interface
// #[cfg(test)]
// mod tests;

// Re-export commonly used types for convenience
pub use config_loader::ConfigLoader;
pub use errors::{ConfigFormat, CreateSubcommandError};
pub use handler::handle;
