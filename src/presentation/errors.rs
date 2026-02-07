//! Presentation Layer Error Types
//!
//! This module defines unified error handling for CLI commands following the error
//! handling conventions documented in docs/contributing/error-handling.md.
//!
//! ## Design Principles
//!
//! - **Clarity**: Unambiguous error messages with specific context
//! - **Traceability**: Full error chains preserved for debugging  
//! - **Actionability**: Clear instructions for resolution
//! - **Unified Structure**: Single `CommandError` enum containing all command-specific errors
//!
//! ## Error Hierarchy
//!
//! ```text
//! CommandError
//! └── Destroy(DestroyError)       # Destroy command errors
//! ```

use thiserror::Error;

use crate::presentation::controllers::{
    configure::ConfigureSubcommandError, create::CreateCommandError,
    destroy::DestroySubcommandError, list::ListSubcommandError,
    provision::ProvisionSubcommandError, purge::PurgeSubcommandError,
    register::errors::RegisterSubcommandError, release::ReleaseSubcommandError,
    run::RunSubcommandError, show::ShowSubcommandError, test::TestSubcommandError,
    validate::errors::ValidateSubcommandError,
};

/// Errors that can occur during CLI command execution
///
/// This enum provides a unified interface for all command-specific errors,
/// following the project's error handling conventions with structured error
/// types, source preservation, and tiered help system support.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Create command specific errors
    ///
    /// Encapsulates all errors that can occur during create operations (environment or template).
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Create command failed: {0}")]
    Create(Box<CreateCommandError>),

    /// Destroy command specific errors
    ///
    /// Encapsulates all errors that can occur during environment destruction.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Destroy command failed: {0}")]
    Destroy(Box<DestroySubcommandError>),

    /// Provision command specific errors
    ///
    /// Encapsulates all errors that can occur during infrastructure provisioning.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Provision command failed: {0}")]
    Provision(Box<ProvisionSubcommandError>),

    /// Configure command specific errors
    ///
    /// Encapsulates all errors that can occur during environment configuration.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Configure command failed: {0}")]
    Configure(Box<ConfigureSubcommandError>),

    /// Test command specific errors
    ///
    /// Encapsulates all errors that can occur during infrastructure validation.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Test command failed: {0}")]
    Test(Box<TestSubcommandError>),

    /// Register command specific errors
    ///
    /// Encapsulates all errors that can occur during instance registration.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Register command failed: {0}")]
    Register(Box<RegisterSubcommandError>),

    /// Release command specific errors
    ///
    /// Encapsulates all errors that can occur during software release operations.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Release command failed: {0}")]
    Release(Box<ReleaseSubcommandError>),

    /// Run command specific errors
    ///
    /// Encapsulates all errors that can occur during stack execution.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Run command failed: {0}")]
    Run(Box<RunSubcommandError>),

    /// Show command specific errors
    ///
    /// Encapsulates all errors that can occur during environment information display.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Show command failed: {0}")]
    Show(Box<ShowSubcommandError>),

    /// List command specific errors
    ///
    /// Encapsulates all errors that can occur during environment listing.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("List command failed: {0}")]
    List(Box<ListSubcommandError>),

    /// Purge command specific errors
    ///
    /// Encapsulates all errors that can occur during local environment data removal.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Purge command failed: {0}")]
    Purge(Box<PurgeSubcommandError>),

    /// Validate command specific errors
    ///
    /// Encapsulates all errors that can occur during configuration validation.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Validate command failed: {0}")]
    Validate(Box<ValidateSubcommandError>),

    /// User output lock acquisition failed
    ///
    /// Failed to acquire the mutex lock for user output. This typically indicates
    /// a panic occurred in another thread while holding the lock.
    #[error("Failed to acquire user output lock - a panic occurred in another thread while displaying output")]
    UserOutputLockFailed,
}

impl From<CreateCommandError> for CommandError {
    fn from(error: CreateCommandError) -> Self {
        Self::Create(Box::new(error))
    }
}

impl From<DestroySubcommandError> for CommandError {
    fn from(error: DestroySubcommandError) -> Self {
        Self::Destroy(Box::new(error))
    }
}

impl From<ProvisionSubcommandError> for CommandError {
    fn from(error: ProvisionSubcommandError) -> Self {
        Self::Provision(Box::new(error))
    }
}

impl From<ConfigureSubcommandError> for CommandError {
    fn from(error: ConfigureSubcommandError) -> Self {
        Self::Configure(Box::new(error))
    }
}

impl From<RegisterSubcommandError> for CommandError {
    fn from(error: RegisterSubcommandError) -> Self {
        Self::Register(Box::new(error))
    }
}

impl From<TestSubcommandError> for CommandError {
    fn from(error: TestSubcommandError) -> Self {
        Self::Test(Box::new(error))
    }
}

impl From<ReleaseSubcommandError> for CommandError {
    fn from(error: ReleaseSubcommandError) -> Self {
        Self::Release(Box::new(error))
    }
}

impl From<RunSubcommandError> for CommandError {
    fn from(error: RunSubcommandError) -> Self {
        Self::Run(Box::new(error))
    }
}

impl From<ShowSubcommandError> for CommandError {
    fn from(error: ShowSubcommandError) -> Self {
        Self::Show(Box::new(error))
    }
}

impl From<ListSubcommandError> for CommandError {
    fn from(error: ListSubcommandError) -> Self {
        Self::List(Box::new(error))
    }
}

impl From<PurgeSubcommandError> for CommandError {
    fn from(error: PurgeSubcommandError) -> Self {
        Self::Purge(Box::new(error))
    }
}

impl From<ValidateSubcommandError> for CommandError {
    fn from(error: ValidateSubcommandError) -> Self {
        Self::Validate(Box::new(error))
    }
}

impl CommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    /// It delegates to the specific command error's help method.
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap::Parser;
    /// use torrust_tracker_deployer_lib::presentation::{input::cli, errors};
    /// use torrust_tracker_deployer_lib::presentation::controllers::destroy::DestroySubcommandError;
    /// use torrust_tracker_deployer_lib::application::command_handlers::destroy::DestroyCommandHandlerError;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create error for demonstration
    /// let destroy_error = DestroySubcommandError::DestroyOperationFailed {
    ///     name: "test-env".to_string(),
    ///     source: DestroyCommandHandlerError::StateCleanupFailed {
    ///         path: PathBuf::from("/tmp/test"),
    ///         source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied"),
    ///     },
    /// };
    /// let error = errors::CommandError::Destroy(Box::new(destroy_error));
    ///
    /// // Get help text
    /// let help_text = error.help();
    /// println!("{}", help_text);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::Create(e) => e.help(),
            Self::Destroy(e) => e.help().to_string(),
            Self::Provision(e) => e.help().to_string(),
            Self::Configure(e) => e.help().to_string(),
            Self::Register(e) => e.help().to_string(),
            Self::Test(e) => e.as_ref().help().to_string(),
            Self::Release(e) => e.help().to_string(),
            Self::Run(e) => e.help().to_string(),
            Self::Show(e) => e.help().to_string(),
            Self::List(e) => e.help().to_string(),
            Self::Purge(e) => e.help().to_string(),
            Self::Validate(e) => e
                .help()
                .unwrap_or_else(|| "No additional help available".to_string()),
            Self::UserOutputLockFailed => "User Output Lock Failed - Detailed Troubleshooting:

This error indicates that a panic occurred in another thread while it was using
the user output system, leaving the mutex in a \"poisoned\" state.

1. Check for any error messages that appeared before this one
   - The original panic message should appear earlier in the output
   - This will indicate what caused the initial failure

2. This is typically caused by:
   - A bug in the application code that caused a panic
   - An unhandled error condition that triggered a panic
   - Resource exhaustion (memory, file handles, etc.)

3. If you can reproduce this issue:
   - Run with --verbose to see more detailed logging
   - Report the issue with the full error output and steps to reproduce

This is a serious application error that indicates a bug. Please report it to the developers."
                .to_string(),
        }
    }
}
