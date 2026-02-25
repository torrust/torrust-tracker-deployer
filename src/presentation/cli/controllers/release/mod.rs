//! Release Command Presentation Module
//!
//! This module implements the CLI presentation layer for the release command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The release command presentation layer follows the DDD pattern, orchestrating
//! the application layer's `ReleaseCommandHandler` while providing user-friendly
//! output and error handling.
//!
//! ## Components
//!
//! - `errors` - Presentation layer error types with `.help()` methods
//! - `handler` - Main command handler orchestrating the workflow
//!
//! ## Usage Example
//!
//! ### Basic Usage
//!
//! ```ignore
//! use std::path::Path;
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::cli::dispatch::ExecutionContext;
//! use torrust_tracker_deployer_lib::presentation::cli::controllers::release;
//! use torrust_tracker_deployer_lib::presentation::cli::views::VerbosityLevel;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let container = Container::new(VerbosityLevel::Normal, Path::new("."));
//! let context = ExecutionContext::new(Arc::new(container), global_args);
//!
//! // Call the release handler
//! let result = context
//!     .container()
//!     .create_release_controller()
//!     .execute("my-environment")
//!     .await;
//! # }
//! ```

pub mod errors;
pub mod handler;
pub use handler::ReleaseCommandController;

#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use errors::ReleaseSubcommandError;
