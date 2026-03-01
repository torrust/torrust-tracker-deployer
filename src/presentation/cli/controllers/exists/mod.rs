//! Exists Command Presentation Module
//!
//! This module implements the CLI presentation layer for the exists command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The exists command presentation layer follows the DDD pattern, providing
//! a read-only existence check without any state modification.
//!
//! ## Components
//!
//! - `errors` - Presentation layer error types with `.help()` methods
//! - `handler` - Main command handler orchestrating the workflow
//!
//! ## Output Contract
//!
//! - Outputs bare `true` or `false` to stdout
//! - Exit code 0 for both true and false results (command succeeded)
//! - Exit code 1 only for errors (repository failures, invalid name)

pub mod errors;
pub mod handler;
pub use handler::ExistsCommandController;

// Re-export commonly used types for convenience
pub use errors::ExistsSubcommandError;
