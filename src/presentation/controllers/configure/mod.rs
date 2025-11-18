//! Configure Command Presentation Module
//!
//! This module implements the CLI presentation layer for the configure command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The configure command presentation layer follows the DDD pattern, orchestrating
//! the application layer's `ConfigureCommandHandler` while providing user-friendly
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
//! ```rust
//! use std::path::Path;
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
//! use torrust_tracker_deployer_lib::presentation::controllers::configure;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//!
//! let container = Container::new(VerbosityLevel::Normal);
//! let context = ExecutionContext::new(Arc::new(container));
//!
//! // Call the configure handler
//! let result = configure::handler::handle("my-environment", Path::new("."), &context);
//! ```
//!
//! ### Direct Usage (For Testing)
//!
//! ```rust
//! use std::path::Path;
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
//! use torrust_tracker_deployer_lib::presentation::controllers::configure;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let container = Container::new(VerbosityLevel::Normal);
//! let context = ExecutionContext::new(Arc::new(container));
//!
//! if let Err(e) = configure::handle("test-env", Path::new("."), &context).await {
//!     eprintln!("Configure failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! # }
//! ```
//!
//! ## Direct Usage (For Testing)
//!
//! ```rust
//! use std::path::Path;
//! use std::sync::Arc;
//! use std::time::Duration;
//! use parking_lot::ReentrantMutex;
//! use std::cell::RefCell;
//! use torrust_tracker_deployer_lib::presentation::controllers::configure;
//! use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
//! use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
//! use torrust_tracker_deployer_lib::shared::clock::SystemClock;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
//! let repository_factory = Arc::new(RepositoryFactory::new(Duration::from_secs(30)));
//! let clock = Arc::new(SystemClock);
//! if let Err(e) = configure::handle_configure_command("test-env", Path::new("."), repository_factory, clock, &output).await {
//!     eprintln!("Configure failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! # }
//! ```

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use errors::ConfigureSubcommandError;
pub use handler::{handle, handle_configure_command};
