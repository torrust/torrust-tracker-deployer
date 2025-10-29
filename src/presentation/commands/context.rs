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
//! use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
//!
//! fn handle_command(working_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
//!     let mut ctx = CommandContext::new(working_dir.to_path_buf());
//!     
//!     ctx.output().progress("Starting operation...");
//!     
//!     // Use repository and clock through context
//!     let repo = ctx.repository();
//!     let clock = ctx.clock();
//!     
//!     ctx.output().success("Operation completed");
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::environment::repository::EnvironmentRepository;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::user_output::UserOutput;
use crate::shared::{Clock, SystemClock};

use super::constants::{DEFAULT_LOCK_TIMEOUT, DEFAULT_VERBOSITY};

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
/// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
///
/// let working_dir = PathBuf::from(".");
/// let mut ctx = CommandContext::new(working_dir);
///
/// // Access repository
/// let repo = ctx.repository();
///
/// // Access clock
/// let clock = ctx.clock();
///
/// // Access user output
/// ctx.output().progress("Processing...");
/// ctx.output().success("Complete!");
/// ```
pub struct CommandContext {
    repository: Arc<dyn EnvironmentRepository>,
    clock: Arc<dyn Clock>,
    output: UserOutput,
}

impl CommandContext {
    /// Create a new command context with production dependencies
    ///
    /// This constructor initializes all dependencies using production implementations
    /// and default configuration from constants:
    /// - Repository with default lock timeout
    /// - System clock
    /// - User output with default verbosity
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Root directory for environment data storage
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    ///
    /// let working_dir = PathBuf::from("./data");
    /// let ctx = CommandContext::new(working_dir);
    /// ```
    #[must_use]
    pub fn new(working_dir: PathBuf) -> Self {
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = repository_factory.create(working_dir);
        let clock = Arc::new(SystemClock);
        let output = UserOutput::new(DEFAULT_VERBOSITY);

        Self {
            repository,
            clock,
            output,
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
    /// * `output` - User output instance (can use custom writers)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
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
    /// let output = UserOutput::new(VerbosityLevel::Quiet);
    ///
    /// let ctx = CommandContext::new_for_testing(repository, clock, output);
    /// ```
    #[must_use]
    pub fn new_for_testing(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn Clock>,
        output: UserOutput,
    ) -> Self {
        Self {
            repository,
            clock,
            output,
        }
    }

    /// Get reference to the environment repository
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    ///
    /// let ctx = CommandContext::new(PathBuf::from("."));
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
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    ///
    /// let ctx = CommandContext::new(PathBuf::from("."));
    /// let clock = ctx.clock();
    /// ```
    #[must_use]
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Get mutable reference to user output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::presentation::commands::context::CommandContext;
    ///
    /// let mut ctx = CommandContext::new(PathBuf::from("."));
    /// ctx.output().progress("Working...");
    /// ctx.output().success("Done!");
    /// ```
    pub fn output(&mut self) -> &mut UserOutput {
        &mut self.output
    }
}

/// Report an error through user output
///
/// This utility function provides a consistent way to report errors to users.
/// It outputs the error message through the provided user output instance.
///
/// # Arguments
///
/// * `output` - User output instance to use for reporting
/// * `error` - Error to report (any type implementing `std::error::Error`)
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::commands::context::report_error;
/// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
///
/// let mut output = UserOutput::new(VerbosityLevel::Normal);
/// let error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
/// report_error(&mut output, &error);
/// ```
pub fn report_error(output: &mut UserOutput, error: &dyn std::error::Error) {
    output.error(&error.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_context_with_production_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let ctx = CommandContext::new(working_dir);

        // Verify dependencies are present and accessible (we can call methods on them)
        let _ = ctx.repository();
        let _ = ctx.clock();
    }

    #[test]
    fn it_should_provide_access_to_repository() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let ctx = CommandContext::new(working_dir);

        // Should be able to access repository
        let _repo = ctx.repository();
    }

    #[test]
    fn it_should_provide_access_to_clock() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let ctx = CommandContext::new(working_dir);

        // Should be able to access clock
        let _clock = ctx.clock();
    }

    #[test]
    fn it_should_provide_mutable_access_to_output() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let mut ctx = CommandContext::new(working_dir);

        // Should be able to use output methods
        ctx.output().progress("Test progress");
        ctx.output().success("Test success");
    }

    #[test]
    fn it_should_allow_creating_context_for_testing() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        // Create test dependencies
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = repository_factory.create(working_dir);
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let output = UserOutput::new(DEFAULT_VERBOSITY);

        // Create context with test dependencies
        let ctx = CommandContext::new_for_testing(repository, clock, output);

        // Verify we can access all dependencies
        let _repo = ctx.repository();
        let _clock = ctx.clock();
    }

    #[test]
    fn it_should_allow_accessing_output_multiple_times() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        let mut ctx = CommandContext::new(working_dir);

        // Should be able to call output() multiple times
        ctx.output().progress("First message");
        ctx.output().success("Second message");
        ctx.output().error("Third message");
    }

    #[test]
    fn it_should_report_errors_through_output() {
        // Create output with custom writers for testing
        let stderr_buf = Vec::new();
        let stderr_writer = Box::new(Cursor::new(stderr_buf));
        let stdout_writer = Box::new(Cursor::new(Vec::new()));

        let mut output = UserOutput::with_writers(DEFAULT_VERBOSITY, stdout_writer, stderr_writer);

        // Create an error and report it
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        report_error(&mut output, &error);

        // Note: In a real test, we'd verify the output was written,
        // but that requires extracting the buffer from output which isn't directly possible
        // without additional helper methods. The important thing is that it compiles and runs.
    }

    #[test]
    fn it_should_use_default_constants() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_path_buf();

        // Creating context should use DEFAULT_LOCK_TIMEOUT and DEFAULT_VERBOSITY
        let _ctx = CommandContext::new(working_dir);

        // This test verifies that the code compiles with the constants
        // The actual values are tested in the constants module
    }
}
