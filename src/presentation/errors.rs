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

use crate::presentation::commands::create::CreateSubcommandError;
use crate::presentation::commands::destroy::DestroySubcommandError;

/// Errors that can occur during CLI command execution
///
/// This enum provides a unified interface for all command-specific errors,
/// following the project's error handling conventions with structured error
/// types, source preservation, and tiered help system support.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Create command specific errors
    ///
    /// Encapsulates all errors that can occur during environment creation.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Create command failed: {0}")]
    Create(Box<CreateSubcommandError>),

    /// Destroy command specific errors
    ///
    /// Encapsulates all errors that can occur during environment destruction.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Destroy command failed: {0}")]
    Destroy(Box<DestroySubcommandError>),

    /// User output lock acquisition failed
    ///
    /// Failed to acquire the mutex lock for user output. This typically indicates
    /// a panic occurred in another thread while holding the lock.
    #[error("Failed to acquire user output lock - a panic occurred in another thread while displaying output")]
    UserOutputLockFailed,
}

impl From<CreateSubcommandError> for CommandError {
    fn from(error: CreateSubcommandError) -> Self {
        Self::Create(Box::new(error))
    }
}

impl From<DestroySubcommandError> for CommandError {
    fn from(error: DestroySubcommandError) -> Self {
        Self::Destroy(Box::new(error))
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
    /// use torrust_tracker_deployer_lib::presentation::{cli, errors};
    /// use torrust_tracker_deployer_lib::presentation::commands::destroy::DestroySubcommandError;
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
    pub fn help(&self) -> &'static str {
        match self {
            Self::Create(e) => e.help(),
            Self::Destroy(e) => e.help(),
            Self::UserOutputLockFailed => {
                "User Output Lock Failed - Detailed Troubleshooting:

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
            }
        }
    }
}
