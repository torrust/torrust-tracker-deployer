//! Errors for Create Schema Command Handler

use std::path::PathBuf;

use thiserror::Error;

use crate::infrastructure::schema::SchemaGenerationError;

/// Errors that can occur during schema creation command handling
#[derive(Debug, Error)]
pub enum CreateSchemaCommandHandlerError {
    /// Failed to generate JSON schema
    #[error("Failed to generate JSON schema")]
    SchemaGenerationFailed {
        /// The underlying schema generation error
        #[source]
        source: SchemaGenerationError,
    },

    /// Failed to write schema to file
    #[error("Failed to write schema to file: {path}")]
    FileWriteFailed {
        /// Path where write failed
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Failed to create parent directories for output file
    #[error("Failed to create parent directories for: {path}")]
    DirectoryCreationFailed {
        /// Path where directory creation failed
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },
}

impl CreateSchemaCommandHandlerError {
    /// Returns actionable help text for resolving this error
    ///
    /// Following the project's tiered help system pattern.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::SchemaGenerationFailed { source } => {
                format!(
                    "Failed to generate JSON schema.\n\
                     \n\
                     {}\n\
                     \n\
                     What to do:\n\
                     1. Report this as a bug if the error persists\n\
                     2. Include the full error message in your bug report",
                    source.help()
                )
            }
            Self::FileWriteFailed { path, .. } => {
                format!(
                    "Failed to write schema to file: {}\n\
                     \n\
                     What to do:\n\
                     1. Check that you have write permissions for the output directory\n\
                     2. Ensure the disk is not full\n\
                     3. Verify the path is valid and accessible\n\
                     4. Try writing to a different location",
                    path.display()
                )
            }
            Self::DirectoryCreationFailed { path, .. } => {
                format!(
                    "Failed to create parent directories for: {}\n\
                     \n\
                     What to do:\n\
                     1. Check that you have write permissions for the parent directory\n\
                     2. Verify the path is valid\n\
                     3. Try using a path where you have permissions\n\
                     4. Consider using the current directory (output to stdout with no argument)",
                    path.display()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_text_for_schema_generation_error() {
        let inner_error = SchemaGenerationError::SerializationFailed {
            source: serde_json::Error::io(std::io::Error::other("test")),
        };
        let error = CreateSchemaCommandHandlerError::SchemaGenerationFailed {
            source: inner_error,
        };

        let help = error.help();
        assert!(help.contains("What to do:"));
        assert!(help.contains("bug"));
    }

    #[test]
    fn it_should_provide_help_text_for_file_write_error() {
        let error = CreateSchemaCommandHandlerError::FileWriteFailed {
            path: PathBuf::from("/test/schema.json"),
            source: std::io::Error::other("test"),
        };

        let help = error.help();
        assert!(help.contains("What to do:"));
        assert!(help.contains("permissions"));
        assert!(help.contains("/test/schema.json"));
    }

    #[test]
    fn it_should_provide_help_text_for_directory_creation_error() {
        let error = CreateSchemaCommandHandlerError::DirectoryCreationFailed {
            path: PathBuf::from("/test/nested/dir"),
            source: std::io::Error::other("test"),
        };

        let help = error.help();
        assert!(help.contains("What to do:"));
        assert!(help.contains("permissions"));
        assert!(help.contains("/test/nested/dir"));
    }
}
