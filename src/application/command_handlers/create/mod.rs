//! Create Command Module
//!
//! This module implements the delivery-agnostic `CreateCommandHandler` for orchestrating
//! environment creation business logic. The command is synchronous and follows
//! existing patterns from `ProvisionCommandHandler`.
//!
//! ## Architecture
//!
//! The `CreateCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Persists environment state via `EnvironmentRepository`
//! - **Clock Abstraction**: Provides deterministic time for testing via `Clock` trait
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Synchronous**: Follows existing patterns (no async/await)
//! - **Repository Responsibility**: Lets repository handle directory creation atomically
//! - **Explicit Errors**: All errors implement `.help()` with actionable guidance
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::application::command_handlers::create::CreateCommandHandler;
//! use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
//!     EnvironmentCreationConfig, EnvironmentSection, LxdProviderSection, ProviderSection,
//!     SshCredentialsConfig,
//! };
//! use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
//! use torrust_tracker_deployer_lib::shared::{SystemClock, Clock};
//!
//! // Setup dependencies
//! let repository_factory = RepositoryFactory::new(std::time::Duration::from_secs(30));
//! let repository = repository_factory.create(std::path::PathBuf::from("."));
//! let clock: Arc<dyn Clock> = Arc::new(SystemClock);
//!
//! // Create command
//! let command = CreateCommandHandler::new(repository, clock);
//!
//! // Prepare configuration
//! let config = EnvironmentCreationConfig::new(
//!     EnvironmentSection {
//!         name: "production".to_string(),
//!         instance_name: None, // Auto-generate from environment name
//!     },
//!     SshCredentialsConfig::new(
//!         "keys/prod_key".to_string(),
//!         "keys/prod_key.pub".to_string(),
//!         "torrust".to_string(),
//!         22,
//!     ),
//!     ProviderSection::Lxd(LxdProviderSection {
//!         profile_name: "lxd-production".to_string(),
//!     }),
//! );
//!
//! // Execute command with working directory
//! let working_dir = std::path::Path::new(".");
//! match command.execute(config, working_dir) {
//!     Ok(environment) => {
//!         println!("Created environment: {}", environment.name());
//!     }
//!     Err(error) => {
//!         eprintln!("Error: {}", error);
//!         eprintln!("\n{}", error.help());
//!     }
//! }
//! ```

pub mod config;
pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::CreateCommandHandlerError;
pub use handler::CreateCommandHandler;
