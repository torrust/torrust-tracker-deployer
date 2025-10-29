//! Command Handler Factory
//!
//! Provides centralized creation of command handlers with consistent
//! dependency injection and configuration management.
//!
//! ## Purpose
//!
//! Previously, each presentation command handler manually created application
//! command handlers with explicit dependency setup:
//!
//! ```rust,ignore
//! // Duplicate code in every handler:
//! let command_handler = CreateCommandHandler::new(
//!     ctx.repository().clone(),
//!     ctx.clock().clone()
//! );
//! ```
//!
//! `CommandHandlerFactory` eliminates this duplication by providing a single place to:
//! - Create application layer command handlers consistently
//! - Manage shared configuration (lock timeout)
//! - Support testing with custom factory configuration
//!
//! ## Benefits
//!
//! - **Consistency**: All command handlers created with same configuration
//! - **Maintainability**: Changes to handler creation logic in one place
//! - **Testability**: Easy to inject test configuration via `new_for_testing()`
//! - **Simplicity**: Presentation handlers focus on workflow, not setup
//!
//! ## Usage Example
//!
//! ```rust
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
//!
//! fn handle_command(working_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
//!     // Create factory with default configuration
//!     let factory = CommandHandlerFactory::new();
//!     
//!     // Create command context
//!     let context = factory.create_context(working_dir.to_path_buf());
//!     
//!     // Create command handler
//!     let handler = factory.create_create_handler(&context);
//!     
//!     // Use handler...
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;

use crate::application::command_handlers::{CreateCommandHandler, DestroyCommandHandler};
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;

use super::constants::DEFAULT_LOCK_TIMEOUT;
use super::context::CommandContext;

/// Factory for creating command handlers with consistent configuration
///
/// This factory centralizes the creation of application layer command handlers,
/// ensuring consistent dependency injection and configuration across all commands.
///
/// The factory uses `RepositoryFactory` to configure repository lock timeouts,
/// and delegates context creation to `CommandContext` for managing output,
/// repository, and clock dependencies.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
///
/// let factory = CommandHandlerFactory::new();
/// let context = factory.create_context(PathBuf::from("."));
/// let handler = factory.create_create_handler(&context);
/// ```
pub struct CommandHandlerFactory {
    /// Repository factory for creating environment repositories
    repository_factory: RepositoryFactory,
}

impl CommandHandlerFactory {
    /// Create a new factory with default configuration
    ///
    /// This constructor initializes the factory with production defaults:
    /// - Repository lock timeout from `DEFAULT_LOCK_TIMEOUT`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
    ///
    /// let factory = CommandHandlerFactory::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        Self { repository_factory }
    }

    /// Create a command context for the given working directory
    ///
    /// This method creates a `CommandContext` with all shared dependencies:
    /// - Repository configured with the factory's settings
    /// - System clock
    /// - User output with default verbosity
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Root directory for environment data storage
    ///
    /// # Returns
    ///
    /// A `CommandContext` ready for use with command handlers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
    ///
    /// let factory = CommandHandlerFactory::new();
    /// let context = factory.create_context(PathBuf::from("./data"));
    /// ```
    #[must_use]
    pub fn create_context(&self, working_dir: PathBuf) -> CommandContext {
        CommandContext::new_with_factory(&self.repository_factory, working_dir)
    }

    /// Create a create command handler
    ///
    /// This method creates a `CreateCommandHandler` with dependencies from the context.
    ///
    /// # Arguments
    ///
    /// * `context` - Command context containing repository and clock
    ///
    /// # Returns
    ///
    /// A `CreateCommandHandler` ready to execute create operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
    ///
    /// let factory = CommandHandlerFactory::new();
    /// let context = factory.create_context(PathBuf::from("."));
    /// let handler = factory.create_create_handler(&context);
    /// ```
    #[must_use]
    pub fn create_create_handler(&self, context: &CommandContext) -> CreateCommandHandler {
        CreateCommandHandler::new(context.repository().clone(), context.clock().clone())
    }

    /// Create a destroy command handler
    ///
    /// This method creates a `DestroyCommandHandler` with dependencies from the context.
    ///
    /// # Arguments
    ///
    /// * `context` - Command context containing repository and clock
    ///
    /// # Returns
    ///
    /// A `DestroyCommandHandler` ready to execute destroy operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
    ///
    /// let factory = CommandHandlerFactory::new();
    /// let context = factory.create_context(PathBuf::from("."));
    /// let handler = factory.create_destroy_handler(&context);
    /// ```
    #[must_use]
    pub fn create_destroy_handler(&self, context: &CommandContext) -> DestroyCommandHandler {
        DestroyCommandHandler::new(context.repository().clone(), context.clock().clone())
    }
}

impl Default for CommandHandlerFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl CommandHandlerFactory {
    /// Create a factory for testing with custom repository factory
    ///
    /// This constructor allows tests to inject custom configuration, such as
    /// different lock timeouts for testing timeout scenarios.
    ///
    /// # Arguments
    ///
    /// * `repository_factory` - Custom repository factory for testing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    ///
    /// // Create factory with custom lock timeout for testing
    /// let repository_factory = RepositoryFactory::new(Duration::from_millis(100));
    /// let factory = CommandHandlerFactory::new_for_testing(repository_factory);
    /// ```
    #[must_use]
    pub fn new_for_testing(repository_factory: RepositoryFactory) -> Self {
        Self { repository_factory }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_factory_with_default_configuration() {
        let factory = CommandHandlerFactory::new();

        // Verify factory is created (basic structure test)
        // The internal repository_factory is private, so we just verify
        // the factory can be created
        let _ = factory;
    }

    #[test]
    fn it_should_create_factory_via_default_trait() {
        let factory = CommandHandlerFactory::default();

        // Verify default trait works
        let _ = factory;
    }

    #[test]
    fn it_should_create_context_with_factory() {
        let factory = CommandHandlerFactory::new();
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let context = factory.create_context(working_dir);

        // Verify context is created with dependencies
        let _ = context.repository();
        let _ = context.clock();
    }

    #[test]
    fn it_should_create_create_handler() {
        let factory = CommandHandlerFactory::new();
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let context = factory.create_context(working_dir);
        let _handler = factory.create_create_handler(&context);

        // Verify handler is created (basic structure test)
    }

    #[test]
    fn it_should_create_destroy_handler() {
        let factory = CommandHandlerFactory::new();
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let context = factory.create_context(working_dir);
        let _handler = factory.create_destroy_handler(&context);

        // Verify handler is created (basic structure test)
    }

    #[test]
    fn it_should_create_multiple_contexts_from_same_factory() {
        let factory = CommandHandlerFactory::new();
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        // Should be able to create multiple contexts
        let context1 = factory.create_context(working_dir.clone());
        let context2 = factory.create_context(working_dir);

        // Both contexts should be functional
        let _ = context1.repository();
        let _ = context2.repository();
    }

    #[test]
    fn it_should_create_multiple_handlers_from_same_context() {
        let factory = CommandHandlerFactory::new();
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let context = factory.create_context(working_dir);

        // Should be able to create multiple handlers from same context
        let _create_handler = factory.create_create_handler(&context);
        let _destroy_handler = factory.create_destroy_handler(&context);

        // Both handlers should be functional
    }

    #[test]
    fn it_should_create_factory_for_testing() {
        use std::time::Duration;

        let repository_factory = RepositoryFactory::new(Duration::from_millis(100));
        let factory = CommandHandlerFactory::new_for_testing(repository_factory);

        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        // Should be able to create context with custom factory
        let context = factory.create_context(working_dir);
        let _ = context.repository();
    }
}
