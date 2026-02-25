//! Errors for Docs Command Controller (Presentation Layer)

use std::path::PathBuf;
use thiserror::Error;

use crate::infrastructure::cli_docs::CliDocsGenerationError;
use crate::presentation::cli::views::progress::ProgressReporterError;

/// Errors that can occur during CLI documentation creation in the presentation layer
#[derive(Debug, Error)]
pub enum DocsCommandError {
    /// CLI documentation generation failed (infrastructure layer)
    #[error("Failed to generate CLI documentation")]
    SchemaGenerationFailed {
        /// The underlying infrastructure error
        #[source]
        source: CliDocsGenerationError,
    },

    /// Failed to create parent directory for documentation file
    #[error("Failed to create parent directory: {path}")]
    DirectoryCreationFailed {
        /// Path that couldn't be created
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Failed to write documentation file
    #[error("Failed to write documentation file: {path}")]
    FileWriteFailed {
        /// Path that couldn't be written
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Progress reporter error
    #[error("Progress reporter error")]
    ProgressReporterFailed {
        /// The underlying progress reporter error
        #[source]
        source: ProgressReporterError,
    },
}

// Enable automatic conversion from ProgressReporterError
impl From<ProgressReporterError> for DocsCommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReporterFailed { source }
    }
}

impl DocsCommandError {
    /// Returns actionable help text for resolving this error
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::SchemaGenerationFailed { source } => {
                format!(
                    "CLI documentation generation failed.\n\
                     \n\
                     {}\n\
                     \n\
                     If you need further assistance, check the documentation or report an issue.",
                    source.help()
                )
            }
            Self::DirectoryCreationFailed { path, source } => {
                format!(
                    "Failed to create parent directory: {}\n\
                     \n\
                     Error: {}\n\
                     \n\
                     What to do:\n\
                     1. Check that you have write permissions to the parent directory\n\
                     2. Verify the path is valid for your operating system\n\
                     3. Ensure the disk is not full\n\
                     4. Try specifying a different output directory",
                    path.display(),
                    source
                )
            }
            Self::FileWriteFailed { path, source } => {
                format!(
                    "Failed to write CLI documentation file: {}\n\
                     \n\
                     Error: {}\n\
                     \n\
                     What to do:\n\
                     1. Check that you have write permissions to the directory\n\
                     2. Verify the path is valid\n\
                     3. Ensure the disk is not full\n\
                     4. Try a different output path",
                    path.display(),
                    source
                )
            }
            Self::ProgressReporterFailed { .. } => "Progress reporting failed.\n\
                 \n\
                 This is an internal error with the progress display system.\n\
                 \n\
                 What to do:\n\
                 1. The command may have still succeeded - check the output\n\
                 2. If the problem persists, report it as a bug"
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_text_for_schema_generation_failed() {
        let error = DocsCommandError::SchemaGenerationFailed {
            source: CliDocsGenerationError::SerializationFailed {
                source: serde_json::Error::io(std::io::Error::other("test")),
            },
        };

        let help = error.help();
        assert!(help.contains("What to do:"));
    }

    #[test]
    fn it_should_provide_help_text_for_directory_creation_failed() {
        let error = DocsCommandError::DirectoryCreationFailed {
            path: PathBuf::from("/invalid/path"),
            source: std::io::Error::other("test error"),
        };

        let help = error.help();
        assert!(help.contains("What to do:"));
        assert!(help.contains("write permissions"));
        assert!(help.contains("/invalid/path"));
    }

    #[test]
    fn it_should_provide_help_text_for_file_write_failed() {
        let error = DocsCommandError::FileWriteFailed {
            path: PathBuf::from("/test/schema.json"),
            source: std::io::Error::other("disk full"),
        };

        let help = error.help();
        assert!(help.contains("What to do:"));
        assert!(help.contains("write permissions"));
        assert!(help.contains("/test/schema.json"));
    }

    #[test]
    fn it_should_provide_help_text_for_progress_reporter_failed() {
        let error = DocsCommandError::ProgressReporterFailed {
            source: ProgressReporterError::UserOutputMutexPoisoned,
        };
        let help = error.help();
        assert!(help.contains("What to do:"));
    }

    #[test]
    fn it_should_convert_from_progress_reporter_error() {
        let progress_error = ProgressReporterError::UserOutputMutexPoisoned;

        let error: DocsCommandError = progress_error.into();
        match error {
            DocsCommandError::ProgressReporterFailed { .. } => {}
            _ => panic!("Expected ProgressReporterFailed variant"),
        }
    }

    #[test]
    fn it_should_implement_error_trait() {
        let error = DocsCommandError::SchemaGenerationFailed {
            source: CliDocsGenerationError::SerializationFailed {
                source: serde_json::Error::io(std::io::Error::other("test")),
            },
        };

        // Should be able to use as std::error::Error
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn it_should_implement_debug_trait() {
        let error = DocsCommandError::SchemaGenerationFailed {
            source: CliDocsGenerationError::SerializationFailed {
                source: serde_json::Error::io(std::io::Error::other("test")),
            },
        };

        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("SchemaGenerationFailed"));
    }
}
