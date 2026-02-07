//! Error types for Validate Command Handler
//!
//! This module defines error types that can occur during validation
//! at the application layer.

use std::path::PathBuf;
use thiserror::Error;

use crate::application::command_handlers::create::config::CreateConfigError;

/// Errors that can occur during validation
///
/// These errors represent different stages of validation failure:
/// - File system errors (file not found, permission denied)
/// - JSON parsing errors (syntax errors, type mismatches)
/// - Domain validation errors (SSH keys missing, invalid values)
#[derive(Debug, Error)]
pub enum ValidateCommandHandlerError {
    /// Failed to read configuration file
    ///
    /// This error occurs when the file cannot be read, for example:
    /// - File does not exist
    /// - Permission denied
    /// - I/O error
    #[error("Failed to read configuration file: {path}")]
    FileReadFailed {
        /// Path to the file that failed to read
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// JSON parsing failed
    ///
    /// This error occurs when the file is not valid JSON or doesn't
    /// match the expected structure.
    #[error("JSON parsing failed for file: {path}")]
    JsonParsingFailed {
        /// Path to the file with invalid JSON
        path: PathBuf,
        /// Underlying JSON parsing error
        #[source]
        source: serde_json::Error,
    },

    /// Domain validation failed
    ///
    /// This error occurs when the configuration violates domain rules:
    /// - SSH key files don't exist
    /// - Invalid port numbers
    /// - Malformed domain names
    /// - Business rule violations
    #[error("Domain validation failed")]
    DomainValidationFailed(#[source] CreateConfigError),
}

impl ValidateCommandHandlerError {
    /// Provides context-specific help for troubleshooting
    ///
    /// Returns detailed guidance based on the specific error type,
    /// helping users understand what went wrong and how to fix it.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::FileReadFailed { path, source } => {
                format!(
                    "Failed to read configuration file at '{}'.\n\n\
                    Possible causes:\n\
                    - File does not exist: Run 'create template' to generate a valid configuration file\n\
                    - Permission denied: Check file permissions with 'ls -l {}'\n\
                    - I/O error: {}\n\n\
                    For more information, see: docs/user-guide/commands/validate.md",
                    path.display(),
                    path.display(),
                    source
                )
            }
            Self::JsonParsingFailed { path, source } => {
                format!(
                    "JSON parsing failed for file '{}'.\n\n\
                    Error details:\n{}\n\n\
                    Common issues:\n\
                    - Missing or extra commas\n\
                    - Unmatched braces or brackets\n\
                    - Invalid escape sequences\n\
                    - Comments (not allowed in JSON)\n\n\
                    Tips:\n\
                    - Use a JSON validator or editor with syntax highlighting\n\
                    - Compare with a template: 'create template --provider lxd'\n\
                    - Check the JSON schema in schemas/environment-config.json\n\n\
                    For more information, see: docs/user-guide/commands/validate.md",
                    path.display(),
                    source
                )
            }
            Self::DomainValidationFailed(source) => {
                format!(
                    "Configuration validation failed.\n\n\
                    Error: {source}\n\n\
                    This means the configuration file has valid JSON syntax but violates\n\
                    domain constraints or business rules.\n\n\
                    Common issues:\n\
                    - SSH key files don't exist at specified paths\n\
                    - Invalid environment name (must be lowercase with dashes)\n\
                    - Invalid port numbers or IP addresses\n\
                    - Missing required fields\n\
                    - HTTPS configured but no services have TLS enabled\n\n\
                    For more information, see: docs/user-guide/commands/validate.md"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_text_when_file_read_fails() {
        let error = ValidateCommandHandlerError::FileReadFailed {
            path: PathBuf::from("/tmp/missing.json"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        };

        let help = error.help();

        assert!(help.contains("File does not exist"));
        assert!(help.contains("create template"));
    }

    #[test]
    fn it_should_provide_help_text_when_json_parsing_fails() {
        let json_error = serde_json::from_str::<serde_json::Value>("{ invalid }").unwrap_err();
        let error = ValidateCommandHandlerError::JsonParsingFailed {
            path: PathBuf::from("config.json"),
            source: json_error,
        };

        let help = error.help();

        assert!(help.contains("JSON parsing failed"));
        assert!(help.contains("Common issues"));
        assert!(help.contains("JSON validator"));
    }

    #[test]
    fn it_should_provide_help_text_when_domain_validation_fails() {
        let config_error = CreateConfigError::TemplateSerializationFailed {
            source: serde_json::Error::io(std::io::Error::other("test error")),
        };
        let error = ValidateCommandHandlerError::DomainValidationFailed(config_error);

        let help = error.help();

        assert!(help.contains("domain constraints"));
        assert!(help.contains("Common issues"));
    }
}
