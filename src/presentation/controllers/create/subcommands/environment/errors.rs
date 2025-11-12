//! Error types for Environment Creation Subcommand
//!
//! This module defines error types that can occur during environment creation
//! command execution. All errors follow the project's error handling principles
//! by providing clear, contextual, and actionable error messages with `.help()` methods.

use std::path::PathBuf;
use thiserror::Error;

use crate::application::command_handlers::create::CreateCommandHandlerError;
use crate::presentation::views::progress::ProgressReporterError;

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

/// Errors that can occur during create environment command execution
///
/// These errors represent failures in the CLI presentation layer when
/// handling the create environment command. They provide structured context for
/// troubleshooting and user feedback.
#[derive(Debug, Error)]
pub enum CreateEnvironmentCommandError {
    // ===== Configuration File Errors =====
    /// Configuration file not found
    ///
    /// The specified configuration file does not exist or is not accessible.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Configuration file not found: {path}
Tip: Check that the file path is correct: ls -la {path}"
    )]
    ConfigFileNotFound {
        /// Path to the missing configuration file
        path: PathBuf,
    },

    /// Failed to parse configuration file
    ///
    /// The configuration file exists but could not be parsed in the expected format.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to parse configuration file '{path}' as {format}: {source}
Tip: Validate {format} syntax with: jq . < {path}"
    )]
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
    ///
    /// The configuration file was parsed successfully but contains invalid values.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Configuration validation failed: {source}
Tip: Review the validation error and fix the configuration file"
    )]
    ConfigValidationFailed {
        /// Underlying validation error from domain layer
        #[source]
        source: crate::application::command_handlers::create::config::CreateConfigError,
    },

    // ===== Command Execution Errors =====
    /// Command execution failed
    ///
    /// The create operation failed during execution after validation passed.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Create command execution failed: {source}
Tip: Check logs with --log-output file-and-stderr for detailed error information"
    )]
    CommandFailed {
        /// Underlying command handler error
        #[source]
        source: CreateCommandHandlerError,
    },

    // ===== Template Generation Errors =====
    /// Template generation failed
    ///
    /// Failed to generate template configuration or files.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Template generation failed: {source}
Tip: Check that you have write permissions in the target directory"
    )]
    TemplateGenerationFailed {
        /// Underlying template generation error from domain layer
        #[source]
        source: crate::application::command_handlers::create::config::CreateConfigError,
    },

    // ===== User Output Lock Errors =====
    /// User output lock acquisition failed
    ///
    /// Failed to acquire the mutex lock for user output. This indicates a panic
    /// occurred in another thread while holding the lock.
    #[error("Failed to acquire user output lock - a panic occurred in another thread")]
    UserOutputLockFailed,

    /// Progress reporting failed
    ///
    /// Failed to report progress to the user due to an internal error.
    /// This indicates a critical internal error.
    #[error(
        "Failed to report progress: {source}
Tip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    ProgressReportingFailed {
        #[source]
        source: ProgressReporterError,
    },
}

// ============================================================================
// ERROR CONVERSIONS
// ============================================================================

impl From<ProgressReporterError> for CreateEnvironmentCommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl CreateEnvironmentCommandError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::controllers::create::CreateEnvironmentCommandError;
    /// use std::path::PathBuf;
    ///
    /// let error = CreateEnvironmentCommandError::ConfigFileNotFound {
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
            Self::ConfigValidationFailed { source } | Self::TemplateGenerationFailed { source } => {
                source.help()
            }
            Self::CommandFailed { source } => source.help(),
            Self::UserOutputLockFailed => {
                "User Output Lock Failed - Troubleshooting:

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
            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Critical Internal Error:

This error indicates that the progress reporting system encountered a critical
internal error while trying to update the user interface. This is a BUG in the
application and should NOT occur under normal circumstances.

Immediate Actions:
1. Save any logs using: --log-output file-and-stderr
2. Note the operation that was in progress when this occurred
3. Record any error messages that appeared before this one
4. Document the current state of your environment

Report the Issue:
1. Include the full log output (--log-output file-and-stderr)
2. Provide steps to reproduce the error
3. Include your environment configuration file
4. Note your operating system and version
5. Report to: https://github.com/torrust/torrust-tracker-deployer/issues

Workaround:
1. Restart the application and retry the operation
2. Try the operation again with --verbose for more details
3. Check system resources (memory, disk space)
4. Check file system permissions

This error means the operation may have PARTIALLY completed or FAILED.
Verify the state of your environment before retrying."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn it_should_provide_help_for_config_file_not_found() {
        let error = CreateEnvironmentCommandError::ConfigFileNotFound {
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
        let error = CreateEnvironmentCommandError::ConfigParsingFailed {
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
        let error = CreateEnvironmentCommandError::ConfigFileNotFound {
            path: PathBuf::from("/path/to/config.json"),
        };

        let message = error.to_string();
        assert!(message.contains("/path/to/config.json"));
        assert!(message.contains("not found"));
    }

    #[test]
    fn it_should_display_format_in_parsing_error() {
        let source = std::io::Error::new(std::io::ErrorKind::InvalidData, "test");
        let error = CreateEnvironmentCommandError::ConfigParsingFailed {
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
        use crate::application::command_handlers::create::config::CreateConfigError;
        use crate::domain::EnvironmentNameError;

        let errors: Vec<CreateEnvironmentCommandError> = vec![
            CreateEnvironmentCommandError::ConfigFileNotFound {
                path: PathBuf::from("test.json"),
            },
            CreateEnvironmentCommandError::ConfigParsingFailed {
                path: PathBuf::from("test.json"),
                format: ConfigFormat::Json,
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "test")),
            },
            CreateEnvironmentCommandError::ConfigValidationFailed {
                source: CreateConfigError::InvalidEnvironmentName(
                    EnvironmentNameError::InvalidFormat {
                        attempted_name: "test".to_string(),
                        reason: "invalid".to_string(),
                        valid_examples: vec!["dev".to_string()],
                    },
                ),
            },
            CreateEnvironmentCommandError::UserOutputLockFailed,
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
