//! Show Command Presentation Module
//!
//! This module implements the CLI presentation layer for the show command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The show command presentation layer follows the DDD pattern, providing
//! a read-only view of stored environment data without remote verification.
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
//! ```rust
//! use std::path::Path;
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
//! use torrust_tracker_deployer_lib::presentation::controllers::show;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//!
//! # fn main() {
//! let container = Container::new(VerbosityLevel::Normal, Path::new("."));
//! let context = ExecutionContext::new(Arc::new(container));
//!
//! // Call the show handler
//! if let Err(e) = context
//!     .container()
//!     .create_show_controller()
//!     .execute("my-environment")
//! {
//!     eprintln!("Show failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! # }
//! ```

pub mod errors;
pub mod handler;
pub use handler::ShowCommandController;

// Re-export commonly used types for convenience
pub use errors::ShowSubcommandError;
