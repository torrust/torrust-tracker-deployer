//! Error types for the Create Subcommand
//!
//! This module defines error types that can occur during CLI create command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use std::path::PathBuf;
use thiserror::Error;

use crate::application::command_handlers::create::CreateCommandHandlerError;

/// Format of configuration file
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    /// JSON format
    Json,
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
        }
    }
}

/// Errors that can occur during create subcommand execution
///
/// These errors represent failures in the CLI presentation layer when
/// handling the create command. They provide structured context for
/// troubleshooting and user feedback.
#[derive(Debug, Error)]
pub enum CreateSubcommandError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound {
        /// Path to the missing configuration file
        path: PathBuf,
    },

    /// Failed to parse configuration file
    #[error("Failed to parse configuration file '{path}' as {format}")]
    ConfigParsingFailed {
        /// Path to the configuration file
        path: PathBuf,
        /// Expected format of the file
        format: ConfigFormat,
        /// Underlying parsing error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Configuration validation failed
    #[error("Configuration validation failed")]
    ConfigValidationFailed(
        /// Underlying validation error from domain layer
        #[source]
        crate::domain::config::CreateConfigError,
    ),

    /// Command execution failed
    #[error("Create command execution failed")]
    CommandFailed(
        /// Underlying command handler error
        #[source]
        CreateCommandHandlerError,
    ),

    /// Template generation failed
    #[error("Template generation failed")]
    TemplateGenerationFailed(
        /// Underlying template generation error from domain layer
        #[source]
        crate::domain::config::CreateConfigError,
    ),
}

impl CreateSubcommandError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::commands::create::CreateSubcommandError;
    /// use std::path::PathBuf;
    ///
    /// let error = CreateSubcommandError::ConfigFileNotFound {
    ///     path: PathBuf::from("config.json"),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("File Not Found"));
    /// assert!(help.contains("Check that the file path"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::ConfigFileNotFound { .. } => {
                "Configuration File Not Found - Troubleshooting:

1. Check that the file path is correct in your --env-file argument
2. Verify the file exists: ls -la <path>
3. Ensure you have read permissions on the file
4. Use absolute paths or paths relative to current directory

Example:
  torrust-tracker-deployer create environment --env-file ./config/environment.json

For more information about configuration format, see the documentation."
            }
            Self::ConfigParsingFailed { format, .. } => match format {
                ConfigFormat::Json => {
                    "JSON Configuration Parsing Failed - Troubleshooting:

1. Validate JSON syntax using a JSON validator:
   - Online: jsonlint.com
   - Command line: jq . < your-config.json

2. Common JSON syntax errors:
   - Missing or extra commas
   - Missing quotes around strings
   - Unclosed braces or brackets
   - Invalid escape sequences

3. Verify required fields are present:
   - environment.name
   - ssh_credentials.private_key_path
   - ssh_credentials.public_key_path

4. Check field types match expectations:
   - Strings must be in quotes
   - Numbers should not have quotes
   - Booleans are true/false (lowercase)

Example valid configuration:
{
  \"environment\": {
    \"name\": \"dev\"
  },
  \"ssh_credentials\": {
    \"private_key_path\": \"fixtures/testing_rsa\",
    \"public_key_path\": \"fixtures/testing_rsa.pub\"
  }
}

For more information, see the configuration documentation."
                }
            },
            Self::ConfigValidationFailed(inner) | Self::TemplateGenerationFailed(inner) => {
                inner.help()
            }
            Self::CommandFailed(inner) => inner.help(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_for_config_file_not_found() {
        let error = CreateSubcommandError::ConfigFileNotFound {
            path: PathBuf::from("missing.json"),
        };

        let help = error.help();
        assert!(help.contains("File Not Found"));
        assert!(help.contains("Check that the file path"));
        assert!(help.contains("ls -la"));
    }

    #[test]
    fn it_should_provide_help_for_json_parsing_failed() {
        let source = std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json");
        let error = CreateSubcommandError::ConfigParsingFailed {
            path: PathBuf::from("config.json"),
            format: ConfigFormat::Json,
            source: Box::new(source),
        };

        let help = error.help();
        assert!(help.contains("JSON"));
        assert!(help.contains("syntax"));
        assert!(help.contains("jq"));
    }

    #[test]
    fn it_should_display_config_file_path_in_error() {
        let error = CreateSubcommandError::ConfigFileNotFound {
            path: PathBuf::from("/path/to/config.json"),
        };

        let message = error.to_string();
        assert!(message.contains("/path/to/config.json"));
        assert!(message.contains("not found"));
    }

    #[test]
    fn it_should_display_format_in_parsing_error() {
        let source = std::io::Error::new(std::io::ErrorKind::InvalidData, "test");
        let error = CreateSubcommandError::ConfigParsingFailed {
            path: PathBuf::from("config.json"),
            format: ConfigFormat::Json,
            source: Box::new(source),
        };

        let message = error.to_string();
        assert!(message.contains("JSON"));
        assert!(message.contains("config.json"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        use crate::domain::config::CreateConfigError;
        use crate::domain::EnvironmentNameError;

        let errors: Vec<CreateSubcommandError> = vec![
            CreateSubcommandError::ConfigFileNotFound {
                path: PathBuf::from("test.json"),
            },
            CreateSubcommandError::ConfigParsingFailed {
                path: PathBuf::from("test.json"),
                format: ConfigFormat::Json,
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "test")),
            },
            CreateSubcommandError::ConfigValidationFailed(
                CreateConfigError::InvalidEnvironmentName(EnvironmentNameError::InvalidFormat {
                    attempted_name: "test".to_string(),
                    reason: "invalid".to_string(),
                    valid_examples: vec!["dev".to_string()],
                }),
            ),
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Troubleshooting") || help.contains("Fix") || help.len() > 50,
                "Help should contain actionable guidance"
            );
        }
    }
}
