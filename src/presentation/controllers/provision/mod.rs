//! Provision Command Presentation Module
//!
//! This module implements the CLI presentation layer for the provision command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The provision command presentation layer follows the DDD pattern, orchestrating
//! the application layer's `ProvisionCommandHandler` while providing user-friendly
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
//! use torrust_tracker_deployer_lib::presentation::controllers::provision;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let container = Container::new(VerbosityLevel::Normal, Path::new("."));
//! let context = ExecutionContext::new(Arc::new(container));
//!
//! // Call the provision handler
//! let result = context
//!     .container()
//!     .create_provision_controller()
//!     .execute("my-environment")
//!     .await;
//! # }
//! ```
//!
//! ### Direct Usage (For Testing)
//!
//! ```rust
//! use std::path::Path;
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
//! use torrust_tracker_deployer_lib::presentation::controllers::provision;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let container = Container::new(VerbosityLevel::Normal, Path::new("."));
//! let context = ExecutionContext::new(Arc::new(container));
//!
//! if let Err(e) = context
//!     .container()
//!     .create_provision_controller()
//!     .execute("test-env")
//!     .await
//! {
//!     eprintln!("Provision failed: {e}");
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
//! use torrust_tracker_deployer_lib::presentation::controllers::provision::handler::ProvisionCommandController;
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
//! if let Err(e) = ProvisionCommandController::new(repository, clock, output).execute("test-env").await {
//!     eprintln!("Provision failed: {e}");
//!     eprintln!("\n{}", e.help());
//! }
//! # }
//! ```

pub mod errors;
pub mod handler;
pub use handler::ProvisionCommandController;

#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use errors::ProvisionSubcommandError;
