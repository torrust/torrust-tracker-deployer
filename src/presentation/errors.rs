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

use crate::presentation::commands::destroy::DestroyError;

/// Errors that can occur during CLI command execution
///
/// This enum provides a unified interface for all command-specific errors,
/// following the project's error handling conventions with structured error
/// types, source preservation, and tiered help system support.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Destroy command specific errors
    ///
    /// Encapsulates all errors that can occur during environment destruction.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Destroy command failed: {0}")]
    Destroy(Box<DestroyError>),
}

impl From<DestroyError> for CommandError {
    fn from(error: DestroyError) -> Self {
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
    /// use torrust_tracker_deployer_lib::presentation::commands::destroy::DestroyError;
    /// use torrust_tracker_deployer_lib::application::command_handlers::destroy::DestroyCommandHandlerError;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create error for demonstration
    /// let destroy_error = DestroyError::DestroyOperationFailed {
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
            Self::Destroy(e) => e.help(),
        }
    }
}
