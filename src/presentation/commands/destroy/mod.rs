//! Destroy Command Presentation Module
//!
//! This module implements the CLI presentation layer for the destroy command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The destroy command presentation layer follows the DDD pattern, orchestrating
//! the application layer's `DestroyCommandHandler` while providing user-friendly
//! output and error handling.
//!
//! ## Components
//!
//! - `errors` - Presentation layer error types with `.help()` methods
//! - `subcommand` - Main command handler orchestrating the workflow
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::presentation::commands::destroy;
//!
//! if let Err(e) = destroy::handle_destroy_command("test-env", Path::new(".")) {
//!     eprintln!("Destroy failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! ```

pub mod errors;
pub mod subcommand;

#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use errors::DestroySubcommandError;
pub use subcommand::handle_destroy_command;
