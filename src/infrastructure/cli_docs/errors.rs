//! CLI Documentation Generation Errors
//!
//! Error types for CLI JSON documentation generation failures.

use thiserror::Error;

/// Errors that can occur during CLI documentation generation
#[derive(Debug, Error)]
pub enum CliDocsGenerationError {
    /// Failed to serialize documentation to JSON
    #[error("Failed to serialize CLI documentation to JSON")]
    SerializationFailed {
        /// The underlying serialization error
        #[source]
        source: serde_json::Error,
    },
}

impl CliDocsGenerationError {
    /// Returns actionable help text for resolving this error
    ///
    /// Following the project's tiered help system pattern.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::SerializationFailed { .. } => {
                "CLI documentation serialization failed. This is likely a bug in the documentation generator.\n\
                 \n\
                 What to do:\n\
                 1. Check if the CLI structure is valid\n\
                 2. Verify all metadata can be extracted from Clap\n\
                 3. Report this as a bug if the error persists\n\
                 4. Include the full error message in your bug report"
                    .to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_text_for_serialization_error() {
        let error = CliDocsGenerationError::SerializationFailed {
            source: serde_json::Error::io(std::io::Error::other("test")),
        };

        let help = error.help();
        assert!(help.contains("What to do:"));
        assert!(help.contains("bug"));
    }
}
