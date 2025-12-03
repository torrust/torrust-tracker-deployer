//! Error types for the Release command handler

use crate::shared::error::{ErrorKind, Traceable};

/// Comprehensive error type for the `ReleaseCommandHandler`
///
/// This error type captures all possible failures that can occur during
/// software release operations. Each variant provides detailed context
/// and actionable troubleshooting guidance.
#[derive(Debug, thiserror::Error)]
pub enum ReleaseCommandHandlerError {
    /// Environment was not found in the repository
    #[error("Environment not found: {name}")]
    EnvironmentNotFound {
        /// The name of the environment that was not found
        name: String,
    },

    /// Environment is in an invalid state for release
    #[error("Environment '{name}' is in an invalid state for release: expected Configured, got {actual_state}")]
    InvalidState {
        /// The name of the environment
        name: String,
        /// The actual state of the environment
        actual_state: String,
    },

    /// Failed to persist environment state
    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),

    /// Release operation failed
    #[error("Release operation failed for environment '{name}': {message}")]
    ReleaseOperationFailed {
        /// The name of the environment
        name: String,
        /// Description of the failure
        message: String,
    },
}

impl Traceable for ReleaseCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::EnvironmentNotFound { name } => {
                format!("ReleaseCommandHandlerError: Environment not found - {name}")
            }
            Self::InvalidState { name, actual_state } => {
                format!(
                    "ReleaseCommandHandlerError: Invalid state for environment '{name}' - expected Configured, got {actual_state}"
                )
            }
            Self::StatePersistence(e) => {
                format!("ReleaseCommandHandlerError: Failed to persist environment state - {e}")
            }
            Self::ReleaseOperationFailed { name, message } => {
                format!(
                    "ReleaseCommandHandlerError: Release operation failed for '{name}' - {message}"
                )
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::StatePersistence(_)
            | Self::EnvironmentNotFound { .. }
            | Self::InvalidState { .. }
            | Self::ReleaseOperationFailed { .. } => None,
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::EnvironmentNotFound { .. } | Self::InvalidState { .. } => {
                ErrorKind::Configuration
            }
            Self::StatePersistence(_) => ErrorKind::StatePersistence,
            Self::ReleaseOperationFailed { .. } => ErrorKind::InfrastructureOperation,
        }
    }
}

impl ReleaseCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::release::ReleaseCommandHandlerError;
    ///
    /// let error = ReleaseCommandHandlerError::EnvironmentNotFound {
    ///     name: "my-env".to_string(),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("Environment Not Found"));
    /// assert!(help.contains("Troubleshooting"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::EnvironmentNotFound { .. } => {
                "Environment Not Found - Troubleshooting:

1. Verify the environment name is correct
2. Check if the environment was created:
   ls data/

3. If the environment doesn't exist, create it first:
   cargo run -- create environment --env-file <config.json>

4. If the environment was previously destroyed, recreate it

Common causes:
- Typo in environment name
- Environment was destroyed
- Working in the wrong directory

For more information, see docs/user-guide/commands.md"
            }
            Self::InvalidState { .. } => {
                "Invalid Environment State - Troubleshooting:

1. The release command requires the environment to be in Configured state
2. Check the current environment state:
   cat data/<env-name>/environment.json

3. If the environment is not configured, run:
   cargo run -- configure <env-name>

4. If the environment is in a failed state, resolve the issue first

State progression for release:
   Created → Provisioned → Configured → Released

For more information, see docs/user-guide/commands.md"
            }
            Self::StatePersistence(_) => {
                "State Persistence Failed - Troubleshooting:

1. Check file system permissions for the data directory
2. Verify available disk space: df -h
3. Ensure no other process is accessing the environment files
4. Check for file system errors: dmesg | tail
5. Verify the data directory is writable

State files are stored in: data/<env-name>/

If the problem persists, report it with full system details."
            }
            Self::ReleaseOperationFailed { .. } => {
                "Release Operation Failed - Troubleshooting:

1. Verify the target instance is reachable:
   ssh <user>@<instance-ip>

2. Check that required software is installed on the target

3. Review the error message above for specific details

4. Check network connectivity and firewall rules

5. Verify SSH credentials are correct

Common causes:
- Network connectivity issues
- SSH authentication failure
- Target instance not running
- Insufficient permissions on target

For more information, see docs/user-guide/commands.md"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::environment::repository::RepositoryError;

    #[test]
    fn it_should_provide_help_for_environment_not_found() {
        let error = ReleaseCommandHandlerError::EnvironmentNotFound {
            name: "test-env".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Environment Not Found"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_provide_help_for_invalid_state() {
        let error = ReleaseCommandHandlerError::InvalidState {
            name: "test-env".to_string(),
            actual_state: "Created".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Invalid Environment State"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_provide_help_for_state_persistence() {
        let error = ReleaseCommandHandlerError::StatePersistence(RepositoryError::NotFound);

        let help = error.help();
        assert!(help.contains("State Persistence"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_provide_help_for_release_operation_failed() {
        let error = ReleaseCommandHandlerError::ReleaseOperationFailed {
            name: "test-env".to_string(),
            message: "Connection refused".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Release Operation Failed"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        let errors = vec![
            ReleaseCommandHandlerError::EnvironmentNotFound {
                name: "test".to_string(),
            },
            ReleaseCommandHandlerError::InvalidState {
                name: "test".to_string(),
                actual_state: "Created".to_string(),
            },
            ReleaseCommandHandlerError::StatePersistence(RepositoryError::NotFound),
            ReleaseCommandHandlerError::ReleaseOperationFailed {
                name: "test".to_string(),
                message: "error".to_string(),
            },
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Troubleshooting"),
                "Help should contain troubleshooting guidance"
            );
            assert!(help.len() > 50, "Help should be detailed");
        }
    }
}
