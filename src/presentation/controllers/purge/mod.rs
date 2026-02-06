//! Purge Command Presentation Module
//!
//! This module implements the CLI presentation layer for the purge command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The purge command presentation layer follows the DDD pattern, orchestrating
//! the application layer's `PurgeCommandHandler` while providing user-friendly
//! output and error handling.
//!
//! ## Components
//!
//! - `errors` - Presentation layer error types with `.help()` methods
//! - `handler` - Main command handler orchestrating the workflow
//!
//! ## Usage Example
//!
//! ### Via Container (Recommended)
//!
//! ```rust
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let container = Container::new(VerbosityLevel::Normal, Path::new("."));
//! if let Err(e) = container
//!     .create_purge_controller()
//!     .execute("test-env", false)
//!     .await
//! {
//!     eprintln!("Purge failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! # }
//! ```
//!
//! ## Direct Usage (For Testing)
//!
//! ```rust
//! use std::path::{Path, PathBuf};
//! use std::sync::Arc;
//! use std::time::Duration;
//! use parking_lot::ReentrantMutex;
//! use std::cell::RefCell;
//! use torrust_tracker_deployer_lib::presentation::controllers::purge::handler::PurgeCommandController;
//! use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
//! use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
//! use torrust_tracker_deployer_lib::shared::clock::SystemClock;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
//! let data_dir = PathBuf::from("./data");
//! let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
//! let repository = repository_factory.create(data_dir);
//! let clock = Arc::new(SystemClock);
//! if let Err(e) = PurgeCommandController::new(repository, clock, output).execute("test-env", false).await {
//!     eprintln!("Purge failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! # }
//! ```

pub mod errors;
pub mod handler;
pub use handler::PurgeCommandController;

#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use errors::PurgeSubcommandError;
