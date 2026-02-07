//! Validate Command Errors
//!
//! Domain-specific error types for the validate command presentation layer.

use std::path::PathBuf;
use thiserror::Error;

use crate::application::command_handlers::validate::ValidateCommandHandlerError;
use crate::presentation::views::progress::ProgressReporterError;

/// Errors that can occur during validate command execution
#[derive(Error, Debug)]
pub enum ValidateSubcommandError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound {
        /// Path to the missing file
        path: PathBuf,
    },

    /// Path exists but is not a file
    #[error("Path is not a file: {path}")]
    ConfigPathNotFile {
        /// Path that is not a file
        path: PathBuf,
    },

    /// File is not readable
    #[error("Cannot read configuration file: {path}")]
    ConfigFileNotReadable {
        /// Path to the unreadable file
        path: PathBuf,
    },

    /// Validation failed (delegated from application layer)
    #[error("Validation failed for configuration file: {path}")]
    ValidationFailed {
        /// Path to the invalid configuration file
        path: PathBuf,
        /// Underlying application layer error
        #[source]
        source: ValidateCommandHandlerError,
    },

    /// Progress reporter error
    #[error("Progress display error: {0}")]
    ProgressError(String),
}

impl ValidateSubcommandError {
    /// Provide troubleshooting guidance for the error
    ///
    /// Returns context-sensitive help text to guide users toward resolution.
    #[must_use]
    pub fn help(&self) -> Option<String> {
        match self {
            Self::ConfigFileNotFound { path } => Some(format!(
                "Verify the file path is correct: {}\n\
                Use 'create template' to generate a valid configuration file.",
                path.display()
            )),
            Self::ConfigPathNotFile { path } => Some(format!(
                "The path '{}' points to a directory, not a file.\n\
                Provide a path to a JSON configuration file.",
                path.display()
            )),
            Self::ConfigFileNotReadable { path } => Some(format!(
                "Check file permissions for '{}':\n\
                Ensure the file is readable by your user account.",
                path.display()
            )),
            Self::ValidationFailed { source, .. } => Some(source.help()),
            Self::ProgressError(_) => None,
        }
    }
}

impl From<ProgressReporterError> for ValidateSubcommandError {
    fn from(err: ProgressReporterError) -> Self {
        Self::ProgressError(err.to_string())
    }
}
