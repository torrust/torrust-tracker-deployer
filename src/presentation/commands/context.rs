//! Command Context Module
//!
//! This module provides a unified context for command execution that encapsulates
//! shared dependencies used across all command handlers.
//!
//! ## Purpose
//!
//! Previously, each command handler manually created the same dependencies:
//!
//! ```rust,ignore
//! // Duplicate code in every handler:
//! let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
//! let repository = repository_factory.create(working_dir.to_path_buf());
//! let clock = Arc::new(SystemClock);
//! let mut output = UserOutput::new(VerbosityLevel::Normal);
//! ```
//!
//! `CommandContext` eliminates this duplication by providing a single place to:
//! - Initialize shared dependencies with consistent configuration
//! - Access dependencies through a clean interface
//! - Support testing with mock implementations
//!
//! ## Benefits
//!
//! - **Consistency**: All commands use the same dependency initialization
//! - **Maintainability**: Changes to dependency setup only need to be made once
//! - **Testability**: Easy to inject test doubles via `new_for_testing()`
//! - **Simplicity**: Command handlers focus on their logic, not setup
//!
//! ## Usage Example
//!
//! ```rust
//! use std::path::Path;
//! use std::sync::{Arc, Mutex};
//! use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
//! use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
//!
//! fn handle_command(working_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
//!     let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
//!     let mut ctx = CommandContext::new(working_dir.to_path_buf(), output.clone());
//!     
//!     output.lock().unwrap().progress("Starting operation...");
//!     
//!     // Use repository and clock through context
//!     let repo = ctx.repository();
//!     let clock = ctx.clock();
//!     
//!     output.lock().unwrap().success("Operation completed");
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::environment::repository::EnvironmentRepository;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::user_output::UserOutput;
use crate::shared::{Clock, SystemClock};

use super::constants::DEFAULT_LOCK_TIMEOUT;

/// Command execution context containing shared dependencies
///
/// This struct encapsulates all the common dependencies that command handlers need:
/// - Repository for persistent environment state
/// - Clock for timing operations
/// - User output for progress and results
///
/// By centralizing these dependencies, we eliminate duplicate initialization code
/// and ensure consistent configuration across all commands.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use std::sync::{Arc, Mutex};
/// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
/// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
///
/// let working_dir = PathBuf::from(".");
/// let user_output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
/// let ctx = CommandContext::new(working_dir, user_output.clone());
///
/// // Access repository
/// let repo = ctx.repository();
///
/// // Access clock
/// let clock = ctx.clock();
///
/// // Access user output
/// let mut output = ctx.user_output().lock().unwrap();
/// output.progress("Processing...");
/// output.success("Complete!");
/// ```
pub struct CommandContext {
    repository: Arc<dyn EnvironmentRepository>,
    clock: Arc<dyn Clock>,
    user_output: Arc<std::sync::Mutex<UserOutput>>,
}

impl CommandContext {
    /// Create a new command context with production dependencies
    ///
    /// This constructor initializes all dependencies using production implementations
    /// and default configuration from constants:
    /// - Repository with default lock timeout
    /// - System clock
    /// - Injected user output service
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Root directory for environment data storage
    /// * `user_output` - Shared user output service for consistent output formatting
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let working_dir = PathBuf::from("./data");
    /// let user_output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let ctx = CommandContext::new(working_dir, user_output);
    /// ```
    #[must_use]
    pub fn new(working_dir: PathBuf, user_output: Arc<std::sync::Mutex<UserOutput>>) -> Self {
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = repository_factory.create(working_dir);
        let clock = Arc::new(SystemClock);

        Self {
            repository,
            clock,
            user_output,
        }
    }

    /// Create a command context using an existing repository factory
    ///
    /// This constructor allows creating a context with a pre-configured repository factory,
    /// useful when consistent repository configuration (like lock timeout) needs to be
    /// shared across multiple contexts.
    ///
    /// # Arguments
    ///
    /// * `repository_factory` - Pre-configured repository factory
    /// * `working_dir` - Root directory for environment data storage
    /// * `user_output` - Shared user output service for consistent output formatting
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let factory = RepositoryFactory::new(Duration::from_secs(30));
    /// let working_dir = PathBuf::from("./data");
    /// let user_output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let ctx = CommandContext::new_with_factory(&factory, working_dir, user_output);
    /// ```
    #[must_use]
    pub fn new_with_factory(
        repository_factory: &RepositoryFactory,
        working_dir: PathBuf,
        user_output: Arc<std::sync::Mutex<UserOutput>>,
    ) -> Self {
        let repository = repository_factory.create(working_dir);
        let clock = Arc::new(SystemClock);

        Self {
            repository,
            clock,
            user_output,
        }
    }

    /// Create a command context for testing with injected dependencies
    ///
    /// This constructor allows tests to inject mock implementations for better isolation
    /// and control over behavior.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository implementation (can be a mock)
    /// * `clock` - Clock implementation (can be a mock)
    /// * `user_output` - Shared user output service for consistent output formatting
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::{Arc, Mutex};
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    /// use torrust_tracker_deployer_lib::shared::{Clock, SystemClock};
    /// use std::time::Duration;
    ///
    /// // Create test dependencies
    /// let factory = RepositoryFactory::new(Duration::from_secs(5));
    /// let repository = factory.create(PathBuf::from("/tmp/test"));
    /// let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    /// let user_output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Quiet)));
    ///
    /// let ctx = CommandContext::new_for_testing(repository, clock, user_output);
    /// ```
    #[must_use]
    pub fn new_for_testing(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn Clock>,
        user_output: Arc<std::sync::Mutex<UserOutput>>,
    ) -> Self {
        Self {
            repository,
            clock,
            user_output,
        }
    }

    /// Get reference to the environment repository
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let ctx = CommandContext::new(PathBuf::from("."), output);
    /// let repo = ctx.repository();
    /// ```
    #[must_use]
    pub fn repository(&self) -> &Arc<dyn EnvironmentRepository> {
        &self.repository
    }

    /// Get reference to the clock
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let ctx = CommandContext::new(PathBuf::from("."), output);
    /// let clock = ctx.clock();
    /// ```
    #[must_use]
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Get reference to the shared user output
    ///
    /// Returns the Arc-wrapped Mutex-protected `UserOutput` instance, allowing
    /// multiple components to share access to the same output sink.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let user_output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let ctx = CommandContext::new(PathBuf::from("."), user_output);
    /// let output_ref = ctx.user_output();
    /// output_ref.lock().unwrap().progress("Working...");
    /// output_ref.lock().unwrap().success("Done!");
    /// ```
    #[must_use]
    pub fn user_output(&self) -> &Arc<std::sync::Mutex<UserOutput>> {
        &self.user_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::presentation::user_output::VerbosityLevel;

    /// Test helper to create a test user output
    fn create_test_user_output() -> Arc<std::sync::Mutex<UserOutput>> {
        Arc::new(std::sync::Mutex::new(UserOutput::new(
            VerbosityLevel::Normal,
        )))
    }

    /// Test helper to create a test context with temporary directory
    ///
    /// Returns a tuple of (`TempDir`, `PathBuf`, `Arc<Mutex<UserOutput>>`)
    /// The `TempDir` must be kept alive for the duration of the test.
    fn create_test_setup() -> (TempDir, PathBuf, Arc<std::sync::Mutex<UserOutput>>) {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();
        let user_output = create_test_user_output();
        (temp_dir, working_dir, user_output)
    }

    #[test]
    fn it_should_create_context_with_production_dependencies() {
        let (_temp_dir, working_dir, user_output) = create_test_setup();

        let ctx = CommandContext::new(working_dir, user_output);

        // Verify dependencies are present and accessible (we can call methods on them)
        let _ = ctx.repository();
        let _ = ctx.clock();
    }

    #[test]
    fn it_should_provide_access_to_repository() {
        let (_temp_dir, working_dir, user_output) = create_test_setup();

        let ctx = CommandContext::new(working_dir, user_output);

        // Should be able to access repository
        let _repo = ctx.repository();
    }

    #[test]
    fn it_should_provide_access_to_clock() {
        let (_temp_dir, working_dir, user_output) = create_test_setup();

        let ctx = CommandContext::new(working_dir, user_output);

        // Should be able to access clock
        let _clock = ctx.clock();
    }

    #[test]
    fn it_should_provide_access_to_user_output() {
        let (_temp_dir, working_dir, user_output) = create_test_setup();

        let ctx = CommandContext::new(working_dir, user_output);

        // Should be able to use output methods through Arc<Mutex<>>
        let output_ref = ctx.user_output();
        output_ref.lock().unwrap().progress("Test progress");
        output_ref.lock().unwrap().success("Test success");
    }

    #[test]
    fn it_should_create_context_with_factory() {
        let (_temp_dir, working_dir, user_output) = create_test_setup();

        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let ctx = CommandContext::new_with_factory(&repository_factory, working_dir, user_output);

        // Verify we can access all dependencies
        let _repo = ctx.repository();
        let _clock = ctx.clock();
    }

    #[test]
    fn it_should_allow_creating_context_for_testing() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        // Create test dependencies
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = repository_factory.create(working_dir);
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let user_output = Arc::new(std::sync::Mutex::new(UserOutput::new(
            VerbosityLevel::Normal,
        )));

        // Create context with test dependencies
        let ctx = CommandContext::new_for_testing(repository, clock, user_output);

        // Verify we can access all dependencies
        let _repo = ctx.repository();
        let _clock = ctx.clock();
    }

    #[test]
    fn it_should_allow_accessing_output_multiple_times() {
        let (_temp_dir, working_dir, user_output) = create_test_setup();

        let ctx = CommandContext::new(working_dir, user_output);

        // Should be able to call user_output() multiple times
        let output_ref = ctx.user_output();
        output_ref.lock().unwrap().progress("First message");
        output_ref.lock().unwrap().success("Second message");
        output_ref.lock().unwrap().error("Third message");
    }

    #[test]
    fn it_should_use_default_constants() {
        let (_temp_dir, working_dir, user_output) = create_test_setup();

        // Creating context should use DEFAULT_LOCK_TIMEOUT
        let _ctx = CommandContext::new(working_dir, user_output);

        // This test verifies that the code compiles with the constants
        // The actual values are tested in the constants module
    }
}
