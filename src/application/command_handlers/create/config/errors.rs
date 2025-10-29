//! Configuration validation errors with actionable help messages
//!
//! This module defines error types for configuration validation failures.
//! All errors follow the project's error handling principles by providing
//! clear, contextual, and actionable error messages with `.help()` methods.

use std::path::PathBuf;
use thiserror::Error;

use crate::domain::EnvironmentNameError;
use crate::shared::UsernameError;

/// Errors that can occur during configuration validation
///
/// These errors follow the project's error handling principles by providing
/// clear, contextual, and actionable error messages through the `.help()` method.
#[derive(Debug, Error)]
pub enum CreateConfigError {
    /// Invalid environment name format
    #[error("Invalid environment name: {0}")]
    InvalidEnvironmentName(#[from] EnvironmentNameError),

    /// Invalid SSH username format
    #[error("Invalid SSH username: {0}")]
    InvalidUsername(#[from] UsernameError),

    /// SSH private key file not found
    #[error("SSH private key file not found: {path}")]
    PrivateKeyNotFound { path: PathBuf },

    /// SSH public key file not found
    #[error("SSH public key file not found: {path}")]
    PublicKeyNotFound { path: PathBuf },

    /// Invalid SSH port (must be 1-65535)
    #[error("Invalid SSH port: {port} (must be between 1 and 65535)")]
    InvalidPort { port: u16 },

    /// Failed to serialize configuration template to JSON
    #[error("Failed to serialize configuration template to JSON")]
    TemplateSerializationFailed {
        #[source]
        source: serde_json::Error,
    },

    /// Failed to create parent directory for template file
    #[error("Failed to create directory: {path}")]
    TemplateDirectoryCreationFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to write template file
    #[error("Failed to write template file: {path}")]
    TemplateFileWriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl CreateConfigError {
    /// Provides detailed troubleshooting guidance for configuration errors
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the configuration issue. This implements the project's tiered help
    /// system pattern for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::CreateConfigError;
    /// use std::path::PathBuf;
    ///
    /// let error = CreateConfigError::PrivateKeyNotFound {
    ///     path: PathBuf::from("/home/user/.ssh/missing_key"),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("private key file"));
    /// assert!(help.contains("Check that the file path is correct"));
    /// ```
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEnvironmentName(_) => {
                "Environment name validation failed.\n\
                 \n\
                 Valid environment names must:\n\
                 - Contain only lowercase letters (a-z) and numbers (0-9)\n\
                 - Use dashes (-) as word separators\n\
                 - Not start or end with separators\n\
                 - Not start with numbers\n\
                 \n\
                 Examples: 'dev', 'staging', 'e2e-config', 'production'\n\
                 \n\
                 Fix: Update the environment name in your configuration to follow these rules."
            }
            Self::InvalidUsername(_) => {
                "SSH username validation failed.\n\
                 \n\
                 Valid usernames must:\n\
                 - Be 1-32 characters long\n\
                 - Start with a letter (a-z, A-Z) or underscore (_)\n\
                 - Contain only letters, digits, underscores, and hyphens\n\
                 \n\
                 Common usernames: 'ubuntu', 'torrust', 'deploy', 'admin'\n\
                 \n\
                 Fix: Update the SSH username in your configuration to follow Linux username requirements."
            }
            Self::PrivateKeyNotFound { .. } => {
                "SSH private key file not found.\n\
                 \n\
                 The specified private key file does not exist or is not accessible.\n\
                 \n\
                 Common causes:\n\
                 - Incorrect file path in configuration\n\
                 - File was moved or deleted\n\
                 - Insufficient permissions to access the file\n\
                 \n\
                 Fix:\n\
                 1. Check that the file path is correct in your configuration\n\
                 2. Verify the file exists: ls -la <path>\n\
                 3. Ensure you have read permissions on the file\n\
                 4. Generate a new SSH key pair if needed: ssh-keygen -t rsa -b 4096"
            }
            Self::PublicKeyNotFound { .. } => {
                "SSH public key file not found.\n\
                 \n\
                 The specified public key file does not exist or is not accessible.\n\
                 \n\
                 Common causes:\n\
                 - Incorrect file path in configuration\n\
                 - File was moved or deleted\n\
                 - Insufficient permissions to access the file\n\
                 \n\
                 Fix:\n\
                 1. Check that the file path is correct in your configuration\n\
                 2. Verify the file exists: ls -la <path>\n\
                 3. Ensure you have read permissions on the file\n\
                 4. Generate public key from private key if needed: ssh-keygen -y -f <private_key> > <public_key>"
            }
            Self::InvalidPort { .. } => {
                "Invalid SSH port number.\n\
                 \n\
                 SSH port must be between 1 and 65535.\n\
                 \n\
                 Common SSH ports:\n\
                 - 22 (standard SSH port)\n\
                 - 2222 (common alternative)\n\
                 \n\
                 Fix: Update the SSH port in your configuration to a valid port number (1-65535)."
            }
            Self::TemplateSerializationFailed { .. } => {
                "Template serialization failed.\n\
                 \n\
                 This indicates an internal error in template generation.\n\
                 \n\
                 Common causes:\n\
                 - Software bug in template generation logic\n\
                 - Invalid data structure for JSON serialization\n\
                 \n\
                 Fix:\n\
                 1. Report this issue with full error details\n\
                 2. Check for application updates\n\
                 \n\
                 This is likely a software bug that needs to be reported."
            }
            Self::TemplateDirectoryCreationFailed { .. } => {
                "Failed to create directory for template file.\n\
                 \n\
                 Common causes:\n\
                 - Insufficient permissions to create directory\n\
                 - No disk space available\n\
                 - A file exists with the same name as the directory\n\
                 - Path length exceeds system limits\n\
                 \n\
                 Fix:\n\
                 1. Check write permissions for the parent directory\n\
                 2. Verify disk space is available: df -h\n\
                 3. Ensure no file exists with the same name as the directory\n\
                 4. Try using a shorter path"
            }
            Self::TemplateFileWriteFailed { .. } => {
                "Failed to write template file.\n\
                 \n\
                 Common causes:\n\
                 - Insufficient permissions to write file\n\
                 - No disk space available\n\
                 - File is open in another application\n\
                 - Antivirus software blocking file creation\n\
                 \n\
                 Fix:\n\
                 1. Check write permissions for the target file and directory\n\
                 2. Verify disk space is available: df -h\n\
                 3. Ensure the file is not open in another application\n\
                 4. Check if antivirus software is blocking file creation"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EnvironmentName;
    use crate::shared::Username;

    #[test]
    fn test_invalid_environment_name_error() {
        let result = EnvironmentName::new("Invalid_Name");
        assert!(result.is_err());

        let error = CreateConfigError::from(result.unwrap_err());
        assert!(error.to_string().contains("Invalid environment name"));
        assert!(error.help().contains("lowercase letters"));
        assert!(error.help().contains("dashes"));
    }

    #[test]
    fn test_invalid_username_error() {
        let result = Username::new("123invalid");
        assert!(result.is_err());

        let error = CreateConfigError::from(result.unwrap_err());
        assert!(error.to_string().contains("Invalid SSH username"));
        assert!(error.help().contains("Start with a letter"));
        assert!(error.help().contains("1-32 characters"));
    }

    #[test]
    fn test_private_key_not_found_error() {
        let error = CreateConfigError::PrivateKeyNotFound {
            path: PathBuf::from("/nonexistent/key"),
        };
        assert!(error.to_string().contains("private key file not found"));
        assert!(error.to_string().contains("/nonexistent/key"));
        assert!(error.help().contains("Check that the file path is correct"));
        assert!(error.help().contains("ssh-keygen"));
    }

    #[test]
    fn test_public_key_not_found_error() {
        let error = CreateConfigError::PublicKeyNotFound {
            path: PathBuf::from("/nonexistent/key.pub"),
        };
        assert!(error.to_string().contains("public key file not found"));
        assert!(error.to_string().contains("/nonexistent/key.pub"));
        assert!(error.help().contains("Check that the file path is correct"));
        assert!(error.help().contains("ssh-keygen -y"));
    }

    #[test]
    fn test_invalid_port_error() {
        let error = CreateConfigError::InvalidPort { port: 0 };
        assert!(error.to_string().contains("Invalid SSH port"));
        assert!(error.to_string().contains("must be between 1 and 65535"));
        assert!(error.help().contains("22"));
        assert!(error.help().contains("2222"));
    }

    #[test]
    fn test_all_errors_have_help() {
        // Verify all error variants have help text
        let errors = vec![
            CreateConfigError::PrivateKeyNotFound {
                path: PathBuf::from("/test"),
            },
            CreateConfigError::PublicKeyNotFound {
                path: PathBuf::from("/test"),
            },
            CreateConfigError::InvalidPort { port: 0 },
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Fix:") || help.contains("Common"),
                "Help should contain actionable guidance"
            );
        }
    }

    #[test]
    fn test_template_serialization_failed_error() {
        // Simulate serialization error (hard to create naturally)
        let json_error = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let error = CreateConfigError::TemplateSerializationFailed { source: json_error };

        assert!(error
            .to_string()
            .contains("serialize configuration template"));
        assert!(error.help().contains("internal error"));
        assert!(error.help().contains("Report this issue"));
    }

    #[test]
    fn test_template_directory_creation_failed_error() {
        let error = CreateConfigError::TemplateDirectoryCreationFailed {
            path: PathBuf::from("/test/path"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };

        assert!(error.to_string().contains("Failed to create directory"));
        assert!(error.to_string().contains("/test/path"));
        assert!(error.help().contains("permissions"));
        assert!(error.help().contains("df -h"));
    }

    #[test]
    fn test_template_file_write_failed_error() {
        let error = CreateConfigError::TemplateFileWriteFailed {
            path: PathBuf::from("/test/file.json"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };

        assert!(error.to_string().contains("Failed to write template file"));
        assert!(error.to_string().contains("/test/file.json"));
        assert!(error.help().contains("permissions"));
        assert!(error.help().contains("disk space"));
    }
}
