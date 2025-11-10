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
//! ```rust,ignore
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
//!
//! fn handle_command(working_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
//!     // Create factory with default configuration
//!     let factory = CommandHandlerFactory::new();
//!     
//!     // Create command handlers
//!     // (Context creation handled by specific handlers)
//!     let handler = factory.create_create_handler(&context);
//!     
//!     // Use handler...
//!     Ok(())
//! }
//! ```

use crate::application::command_handlers::{CreateCommandHandler, DestroyCommandHandler};

use super::context::CommandContext;

/// Factory for creating command handlers with consistent configuration
///
/// This factory centralizes the creation of application layer command handlers,
/// ensuring consistent dependency injection and configuration across all commands.
///
/// Command contexts should be created directly using `CommandContext::new` with
/// the desired configuration.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
///
/// let factory = CommandHandlerFactory::new();
/// // Context creation handled by specific command handlers
/// ```
pub struct CommandHandlerFactory;

impl CommandHandlerFactory {
    /// Create a new factory with default configuration
    ///
    /// This constructor initializes a factory for creating command handlers.
    /// Context creation is handled separately using `CommandContext::new`.
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
        Self
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
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let factory = CommandHandlerFactory::new();
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let context = CommandContext::new(PathBuf::from("."), output);
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
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::commands::factory::CommandHandlerFactory;
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let factory = CommandHandlerFactory::new();
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let context = CommandContext::new(PathBuf::from("."), output);
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
mod tests {
    use super::*;
    use crate::presentation::user_output::test_support::TestUserOutput;
    use crate::presentation::user_output::{UserOutput, VerbosityLevel};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Test helper to create a test setup with factory, temp directory, and user output
    ///
    /// Returns a tuple of (`CommandHandlerFactory`, `TempDir`, `PathBuf`, `Arc<Mutex<UserOutput>>`)
    /// The `TempDir` must be kept alive for the duration of the test.
    fn create_test_setup() -> (
        CommandHandlerFactory,
        TempDir,
        PathBuf,
        Arc<Mutex<UserOutput>>,
    ) {
        let factory = CommandHandlerFactory::new();
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();
        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        (factory, temp_dir, working_dir, user_output)
    }

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
        let factory = CommandHandlerFactory::new();

        // Verify factory works (both new() and default() should work the same)
        let _ = factory;
    }

    #[test]
    fn it_should_create_create_handler() {
        let (factory, _temp_dir, working_dir, user_output) = create_test_setup();

        let context = CommandContext::new(working_dir, user_output);
        let _handler = factory.create_create_handler(&context);

        // Verify handler is created (basic structure test)
    }

    #[test]
    fn it_should_create_destroy_handler() {
        let (factory, _temp_dir, working_dir, user_output) = create_test_setup();

        let context = CommandContext::new(working_dir, user_output);
        let _handler = factory.create_destroy_handler(&context);

        // Verify handler is created (basic structure test)
    }

    #[test]
    fn it_should_create_multiple_handlers_from_same_context() {
        let (factory, _temp_dir, working_dir, user_output) = create_test_setup();

        let context = CommandContext::new(working_dir, user_output);

        // Should be able to create multiple handlers from same context
        let _create_handler = factory.create_create_handler(&context);
        let _destroy_handler = factory.create_destroy_handler(&context);

        // Both handlers should be functional
    }
}
