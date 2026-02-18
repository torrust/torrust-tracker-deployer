//! Execution Context
//!
//! This module provides the `ExecutionContext` wrapper around the Container for
//! dependency injection in command handlers. It offers a clean interface for
//! accessing services needed during command execution.
//!
//! ## Purpose
//!
//! The `ExecutionContext` serves as an abstraction layer between the Container
//! (which holds raw services) and command handlers (which need typed access).
//! This separation provides:
//!
//! - **Clean Interface**: Command handlers get strongly-typed service access
//! - **Thread Safety**: All services are properly wrapped for concurrent access
//! - **Future-Proofing**: Easy to add new services without changing handler signatures
//! - **Testing Support**: Easy to inject test doubles through Container
//!
//! ## Design
//!
//! ```text
//! Container (bootstrap) → ExecutionContext (dispatch) → Command Handlers
//!
//! Raw services        Clean typed access      Business logic
//! ```
//!
//! ## Usage Example
//!
//! ```ignore
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//! use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
//! use std::sync::Arc;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create execution context from container
//! let container = Container::new(VerbosityLevel::Normal, Path::new("."));
//! let context = ExecutionContext::new(Arc::new(container), global_args);
//!
//! // Command handlers access services through context
//! let user_output = context.user_output();
//! user_output.lock().borrow_mut().progress("Processing...");
//! # Ok(())
//! # }
//! ```

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::bootstrap::Container;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::input::cli::args::GlobalArgs;
use crate::presentation::input::cli::OutputFormat;
use crate::presentation::views::UserOutput;
use crate::shared::clock::Clock;

/// ### Design Consideration: Shared State Access
///
/// Currently, there is no shared mutable state in the system that requires `Arc<Mutex<T>>`
/// patterns. However, if shared state is needed in the future, it can be added to the
/// Container and accessed through standard Rust concurrency patterns:
///
/// # Examples
///
/// ```ignore
/// use std::sync::Arc;
/// use std::path::Path;
/// use torrust_tracker_deployer_lib::bootstrap::Container;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
/// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
///
/// let container = Arc::new(Container::new(VerbosityLevel::Normal, Path::new(".")));
/// let context = ExecutionContext::new(container, global_args);
///
/// // Access user output service
/// let user_output = context.user_output();
/// user_output.lock().borrow_mut().success("Operation completed");
/// ```
#[derive(Clone)]
pub struct ExecutionContext {
    container: Arc<Container>,
    global_args: GlobalArgs,
}

impl ExecutionContext {
    /// Create a new execution context from a container
    ///
    /// # Arguments
    ///
    /// * `container` - Application service container with initialized services
    /// * `global_args` - Global CLI arguments (logging config, output format, etc.)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use torrust_tracker_deployer_lib::presentation::input::cli::args::GlobalArgs;
    /// use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput};
    /// use torrust_tracker_deployer_lib::presentation::input::cli::OutputFormat;
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container = Container::new(VerbosityLevel::Normal, &PathBuf::from("."));
    /// let global_args = GlobalArgs {
    ///     log_file_format: LogFormat::Compact,
    ///     log_stderr_format: LogFormat::Pretty,
    ///     log_output: LogOutput::FileOnly,
    ///     log_dir: PathBuf::from("./data/logs"),
    ///     working_dir: PathBuf::from("."),
    ///     output_format: OutputFormat::Text,
    ///     verbosity: 0,
    /// };
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new(container: Arc<Container>, global_args: GlobalArgs) -> Self {
        Self {
            container,
            global_args,
        }
    }

    /// Get reference to the underlying container
    ///
    /// Provides access to the raw container for cases where direct access
    /// to container methods is needed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use std::sync::Arc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    ///
    /// let container_ref = context.container();
    /// // Use container_ref as needed
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn container(&self) -> &Arc<Container> {
        &self.container
    }

    /// Get shared reference to user output service
    ///
    /// Returns the user output service for displaying messages, progress,
    /// and results to users. The service is wrapped in `Arc<Mutex<T>>` for
    /// thread-safe shared access.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use std::sync::Arc;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    ///
    /// let user_output = context.user_output();
    /// user_output.lock().borrow_mut().success("Operation completed");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn user_output(&self) -> Arc<ReentrantMutex<RefCell<UserOutput>>> {
        self.container.user_output()
    }

    /// Get shared reference to repository factory service
    ///
    /// Returns the repository factory service for creating environment
    /// repositories. The service is wrapped in `Arc<T>` for shared access.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use std::sync::Arc;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    ///
    /// let repository_factory = context.repository_factory();
    /// // Use repository_factory to create repositories
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn repository_factory(&self) -> Arc<RepositoryFactory> {
        self.container.repository_factory()
    }

    /// Get shared reference to environment repository
    ///
    /// Returns the environment repository for persistence operations.
    /// The repository is wrapped in `Arc<dyn EnvironmentRepository>` for shared access.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use std::sync::Arc;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    ///
    /// let repository = context.repository();
    /// // Use repository for environment persistence
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn repository(
        &self,
    ) -> Arc<dyn crate::domain::environment::repository::EnvironmentRepository + Send + Sync> {
        self.container.repository()
    }

    /// Get shared reference to clock service
    ///
    /// Returns the clock service for time-related operations.
    /// The service is wrapped in `Arc<dyn Clock>` for shared access.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use std::sync::Arc;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    ///
    /// let clock = context.clock();
    /// // Use clock for time operations
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn clock(&self) -> Arc<dyn Clock> {
        self.container.clock()
    }

    /// Get the output format from global CLI arguments
    ///
    /// Returns the user-specified output format (Text or Json) for command results.
    /// This allows controllers to format their output appropriately based on user preference.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use torrust_tracker_deployer_lib::presentation::input::cli::args::GlobalArgs;
    /// use torrust_tracker_deployer_lib::presentation::input::cli::OutputFormat;
    /// use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput};
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container = Container::new(VerbosityLevel::Normal, &PathBuf::from("."));
    /// let global_args = GlobalArgs {
    ///     log_file_format: LogFormat::Compact,
    ///     log_stderr_format: LogFormat::Pretty,
    ///     log_output: LogOutput::FileOnly,
    ///     log_dir: PathBuf::from("./data/logs"),
    ///     working_dir: PathBuf::from("."),
    ///     output_format: OutputFormat::Json,
    ///     verbosity: 0,
    /// };
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    ///
    /// let format = context.output_format();
    /// match format {
    ///     OutputFormat::Text => println!("Human-readable text"),
    ///     OutputFormat::Json => println!("{{\"result\": \"json\"}}"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn output_format(&self) -> OutputFormat {
        self.global_args.output_format
    }

    /// Get the working directory from global CLI arguments
    ///
    /// Returns the working directory path specified by the user (or default ".").
    /// This is where environment data will be stored (data/ and build/ subdirectories).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use torrust_tracker_deployer_lib::presentation::input::cli::args::GlobalArgs;
    /// use torrust_tracker_deployer_lib::presentation::input::cli::OutputFormat;
    /// use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput};
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let container = Container::new(VerbosityLevel::Normal, &PathBuf::from("."));
    /// let global_args = GlobalArgs {
    ///     log_file_format: LogFormat::Compact,
    ///     log_stderr_format: LogFormat::Pretty,
    ///     log_output: LogOutput::FileOnly,
    ///     log_dir: PathBuf::from("./data/logs"),
    ///     working_dir: PathBuf::from("/tmp/test-workspace"),
    ///     output_format: OutputFormat::Text,
    ///     verbosity: 0,
    /// };
    /// let context = ExecutionContext::new(Arc::new(container), global_args);
    ///
    /// let working_dir = context.working_dir();
    /// println!("Working directory: {}", working_dir.display());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn working_dir(&self) -> &std::path::Path {
        &self.global_args.working_dir
    }
}
