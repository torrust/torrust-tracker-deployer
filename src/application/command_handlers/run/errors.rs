//! Error types for the Run command handler

use crate::application::errors::{InvalidStateError, PersistenceError};
use crate::application::steps::application::StartServicesStepError;
use crate::shared::error::{ErrorKind, Traceable};

/// Comprehensive error type for the `RunCommandHandler`
///
/// This error type captures all possible failures that can occur during
/// stack execution operations. Each variant provides detailed context
/// and actionable troubleshooting guidance.
#[derive(Debug, thiserror::Error)]
pub enum RunCommandHandlerError {
    /// Environment was not found in the repository
    #[error("Environment not found: {name}")]
    EnvironmentNotFound {
        /// The name of the environment that was not found
        name: String,
    },

    /// Instance IP address is not available (required for running services)
    ///
    /// The run command requires the instance IP address to start services
    /// on the remote host. This IP should be available after provisioning.
    #[error("Instance IP address is not available for environment '{name}'. The provision step should have set this value.")]
    MissingInstanceIp {
        /// The name of the environment missing the instance IP
        name: String,
    },

    /// Environment is in an invalid state for running
    #[error("Environment is in an invalid state for running: {0}")]
    InvalidState(#[from] InvalidStateError),

    /// Failed to persist environment state
    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] PersistenceError),

    /// Starting services on remote host failed
    #[error("Starting services failed: {message}")]
    StartServicesFailed {
        /// Description of the failure
        message: String,
        /// The underlying step error
        #[source]
        source: StartServicesStepError,
    },

    /// Run operation failed
    #[error("Run operation failed for environment '{name}': {message}")]
    RunOperationFailed {
        /// The name of the environment
        name: String,
        /// Description of the failure
        message: String,
    },
}

impl From<crate::domain::environment::repository::RepositoryError> for RunCommandHandlerError {
    fn from(e: crate::domain::environment::repository::RepositoryError) -> Self {
        Self::StatePersistence(e.into())
    }
}

impl From<crate::domain::environment::state::StateTypeError> for RunCommandHandlerError {
    fn from(e: crate::domain::environment::state::StateTypeError) -> Self {
        Self::InvalidState(e.into())
    }
}

impl Traceable for RunCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::EnvironmentNotFound { name } => {
                format!("RunCommandHandlerError: Environment not found - {name}")
            }
            Self::MissingInstanceIp { name } => {
                format!(
                    "RunCommandHandlerError: Instance IP not available for environment '{name}'"
                )
            }
            Self::InvalidState(e) => {
                format!("RunCommandHandlerError: Invalid state for run - {e}")
            }
            Self::StatePersistence(e) => {
                format!("RunCommandHandlerError: Failed to persist environment state - {e}")
            }
            Self::StartServicesFailed { message, .. } => {
                format!("RunCommandHandlerError: Start services failed - {message}")
            }
            Self::RunOperationFailed { name, message } => {
                format!("RunCommandHandlerError: Run operation failed for '{name}' - {message}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::StartServicesFailed { source, .. } => Some(source),
            Self::StatePersistence(_)
            | Self::EnvironmentNotFound { .. }
            | Self::MissingInstanceIp { .. }
            | Self::InvalidState(_)
            | Self::RunOperationFailed { .. } => None,
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::EnvironmentNotFound { .. }
            | Self::MissingInstanceIp { .. }
            | Self::InvalidState(_) => ErrorKind::Configuration,
            Self::StatePersistence(_) => ErrorKind::StatePersistence,
            Self::StartServicesFailed { source, .. } => source.error_kind(),
            Self::RunOperationFailed { .. } => ErrorKind::InfrastructureOperation,
        }
    }
}

impl RunCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::run::RunCommandHandlerError;
    ///
    /// let error = RunCommandHandlerError::EnvironmentNotFound {
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
            Self::MissingInstanceIp { .. } => {
                "Missing Instance IP Address - Troubleshooting:

The run command requires the instance IP address to start services on the
remote host. This IP should be automatically set during provisioning.

1. Check if the environment was provisioned correctly:
   cat data/<env-name>/environment.json
   Look for the 'instance_ip' field in runtime_outputs

2. If instance_ip is null, the provision step may have failed:
   cargo run -- provision <env-name>

3. For registered instances, ensure the IP was provided during registration

4. If using LXD, verify the VM is running and has an IP:
   lxc list

Common causes:
- Provision step failed or was interrupted
- VM/container has no network connectivity
- DHCP lease not yet assigned
- Registration was incomplete

For more information, see docs/user-guide/commands.md"
            }
            Self::InvalidState { .. } => {
                "Invalid Environment State - Troubleshooting:

1. The run command requires the environment to be in Released state
2. Check the current environment state:
   cat data/<env-name>/environment.json

3. If the environment is not released, run:
   cargo run -- release <env-name>

4. If the environment is in a failed state, resolve the issue first

State progression for run:
   Created → Provisioned → Configured → Released → Running

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
            Self::StartServicesFailed { source, .. } => source.help(),
            Self::RunOperationFailed { .. } => {
                "Run Operation Failed - Troubleshooting:

1. Verify the target instance is reachable:
   ssh <user>@<instance-ip>

2. Check that the software was properly released:
   cargo run -- release <env-name>

3. Review the error message above for specific details

4. Check service logs on the target instance

5. Verify network connectivity and firewall rules

Common causes:
- Service failed to start
- Port already in use
- Configuration errors
- Missing dependencies on target

For more information, see docs/user-guide/commands.md"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::command::CommandError;

    #[test]
    fn it_should_provide_help_for_environment_not_found() {
        let error = RunCommandHandlerError::EnvironmentNotFound {
            name: "test-env".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Environment Not Found"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_provide_help_for_missing_instance_ip() {
        let error = RunCommandHandlerError::MissingInstanceIp {
            name: "test-env".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Missing Instance IP"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_provide_help_for_invalid_state() {
        let error = RunCommandHandlerError::InvalidState(InvalidStateError {
            expected: "Released".to_string(),
            actual: "Configured".to_string(),
        });

        let help = error.help();
        assert!(help.contains("Invalid Environment State"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_provide_help_for_state_persistence() {
        let error = RunCommandHandlerError::StatePersistence(PersistenceError::NotFound);

        let help = error.help();
        assert!(help.contains("State Persistence"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_provide_help_for_start_services_failed() {
        let cmd_error = CommandError::ExecutionFailed {
            command: "ansible-playbook".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "connection refused".to_string(),
        };
        let error = RunCommandHandlerError::StartServicesFailed {
            message: "Ansible playbook failed".to_string(),
            source: StartServicesStepError::AnsiblePlaybookFailed {
                message: "connection refused".to_string(),
                source: cmd_error,
            },
        };

        let help = error.help();
        assert!(help.contains("Docker daemon"));
        assert!(help.contains("release"));
    }

    #[test]
    fn it_should_provide_help_for_run_operation_failed() {
        let error = RunCommandHandlerError::RunOperationFailed {
            name: "test-env".to_string(),
            message: "Service failed to start".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Run Operation Failed"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        let cmd_error = CommandError::ExecutionFailed {
            command: "ansible-playbook".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test".to_string(),
        };
        let errors: Vec<RunCommandHandlerError> = vec![
            RunCommandHandlerError::EnvironmentNotFound {
                name: "test".to_string(),
            },
            RunCommandHandlerError::MissingInstanceIp {
                name: "test".to_string(),
            },
            RunCommandHandlerError::InvalidState(InvalidStateError {
                expected: "Released".to_string(),
                actual: "Configured".to_string(),
            }),
            RunCommandHandlerError::StatePersistence(PersistenceError::NotFound),
            RunCommandHandlerError::StartServicesFailed {
                message: "test".to_string(),
                source: StartServicesStepError::AnsiblePlaybookFailed {
                    message: "test".to_string(),
                    source: cmd_error,
                },
            },
            RunCommandHandlerError::RunOperationFailed {
                name: "test".to_string(),
                message: "error".to_string(),
            },
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(help.len() > 50, "Help should be detailed");
        }
    }
}
