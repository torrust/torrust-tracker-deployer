//! List Command Presentation Module
//!
//! This module implements the CLI presentation layer for the list command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The list command presentation layer follows the DDD pattern, providing
//! a read-only view of all environments in the workspace.
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
//! use torrust_tracker_deployer_lib::presentation::controllers::list;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//!
//! # fn main() {
//! let container = Container::new(VerbosityLevel::Normal, Path::new("."));
//! let context = ExecutionContext::new(Arc::new(container));
//!
//! // Call the list handler
//! if let Err(e) = context
//!     .container()
//!     .create_list_controller()
//!     .execute()
//! {
//!     eprintln!("List failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! # }
//! ```

pub mod errors;
pub mod handler;
pub use handler::ListCommandController;

// Re-export commonly used types for convenience
pub use errors::ListSubcommandError;
