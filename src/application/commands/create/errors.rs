//! Error types for the Create Command
//!
//! This module defines error types that can occur during environment creation
//! command execution. All errors follow the project's error handling principles
//! by providing clear, contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::domain::config::CreateConfigError;
use crate::domain::environment::repository::RepositoryError;

/// Errors that can occur during environment creation command execution
///
/// These errors represent failures in the business logic orchestration
/// and provide structured context for troubleshooting and user feedback.
#[derive(Debug, Error)]
pub enum CreateCommandError {
    /// Configuration validation failed
    #[error("Configuration validation failed")]
    InvalidConfiguration(#[source] CreateConfigError),

    /// Environment with the given name already exists
    #[error("Environment '{name}' already exists")]
    EnvironmentAlreadyExists { name: String },

    /// Repository operation failed
    #[error("Repository operation failed")]
    RepositoryError(#[source] RepositoryError),
}

impl CreateCommandError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::commands::create::CreateCommandError;
    ///
    /// let error = CreateCommandError::EnvironmentAlreadyExists {
    ///     name: "production".to_string(),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("already exists"));
    /// assert!(help.contains("different name"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidConfiguration(_) => {
                "Configuration Validation Failed - Troubleshooting:

1. Check JSON syntax and format
2. Verify all required fields are present
3. Ensure SSH key files exist and are readable
4. Verify environment name follows naming rules (lowercase, dashes, no leading/trailing dashes)
5. Check that SSH username follows Linux username requirements

Run with --generate-template to see a valid configuration example.

For more details, see the configuration documentation."
            }
            Self::EnvironmentAlreadyExists { .. } => {
                "Environment Already Exists - Troubleshooting:

1. List existing environments:
   torrust-tracker-deployer list

2. Choose a different environment name in your configuration

3. Or destroy the existing environment first:
   torrust-tracker-deployer destroy --env-name <name>

4. Or work with the existing environment (no need to recreate)

Note: Environment names must be unique across the system.

For more information, see the environment management documentation."
            }
            Self::RepositoryError(_) => {
                "Repository Operation Failed - Troubleshooting:

1. Check file system permissions for the data directory
2. Verify available disk space: df -h
3. Ensure no other process is accessing the environment files
4. Check for file system errors: dmesg | tail
5. Verify the data directory is writable

The repository handles directory creation atomically during save.
If partially created files exist, remove them and retry.

If the problem persists, report it with full system details."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_for_environment_already_exists() {
        let error = CreateCommandError::EnvironmentAlreadyExists {
            name: "test-env".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Already Exists"));
        assert!(help.contains("Choose a different"));
        assert!(help.contains("destroy"));
    }

    #[test]
    fn it_should_display_environment_name_in_error() {
        let error = CreateCommandError::EnvironmentAlreadyExists {
            name: "production".to_string(),
        };

        let message = error.to_string();
        assert!(message.contains("production"));
        assert!(message.contains("already exists"));
    }

    #[test]
    fn it_should_provide_help_for_invalid_configuration() {
        use crate::domain::EnvironmentNameError;

        let config_error =
            CreateConfigError::InvalidEnvironmentName(EnvironmentNameError::InvalidFormat {
                attempted_name: "Invalid_Name".to_string(),
                reason: "contains invalid characters".to_string(),
                valid_examples: vec!["dev".to_string(), "staging".to_string()],
            });
        let error = CreateCommandError::InvalidConfiguration(config_error);

        let help = error.help();
        assert!(help.contains("Configuration"));
        assert!(help.contains("JSON"));
        assert!(help.contains("SSH key"));
    }

    #[test]
    fn it_should_provide_help_for_repository_error() {
        let repo_error = RepositoryError::NotFound;
        let error = CreateCommandError::RepositoryError(repo_error);

        let help = error.help();
        assert!(help.contains("Repository"));
        assert!(help.contains("permissions"));
        assert!(help.contains("disk space"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        use crate::domain::EnvironmentNameError;

        let errors = vec![
            CreateCommandError::InvalidConfiguration(CreateConfigError::InvalidEnvironmentName(
                EnvironmentNameError::InvalidFormat {
                    attempted_name: "test".to_string(),
                    reason: "invalid".to_string(),
                    valid_examples: vec!["dev".to_string()],
                },
            )),
            CreateCommandError::EnvironmentAlreadyExists {
                name: "test".to_string(),
            },
            CreateCommandError::RepositoryError(RepositoryError::NotFound),
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Troubleshooting") || help.len() > 50,
                "Help should contain actionable guidance"
            );
        }
    }
}
