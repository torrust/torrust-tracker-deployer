//! Error types for test command handler

use crate::domain::environment::repository::RepositoryError;
use crate::domain::environment::state::StateTypeError;
use crate::infrastructure::remote_actions::RemoteActionError;
use crate::shared::command::CommandError;

/// Comprehensive error type for the `TestCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum TestCommandHandlerError {
    #[error("Environment not found: '{name}'")]
    EnvironmentNotFound { name: String },

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Remote action failed: {0}")]
    RemoteAction(#[from] RemoteActionError),

    #[error("Environment '{environment_name}' does not have an instance IP set. The environment must be provisioned before running tests.")]
    MissingInstanceIp { environment_name: String },

    #[error("Invalid tracker configuration: {message}")]
    InvalidTrackerConfiguration { message: String },

    #[error("Invalid state transition: {0}")]
    StateTransition(#[from] StateTypeError),

    #[error("State persistence error: {0}")]
    StatePersistence(#[from] RepositoryError),
}

impl crate::shared::Traceable for TestCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::EnvironmentNotFound { name } => {
                format!("TestCommandHandlerError: Environment not found - '{name}'")
            }
            Self::Command(e) => {
                format!("TestCommandHandlerError: Command execution failed - {e}")
            }
            Self::RemoteAction(e) => {
                format!("TestCommandHandlerError: Remote action failed - {e}")
            }
            Self::MissingInstanceIp { environment_name } => {
                format!(
                    "TestCommandHandlerError: Missing instance IP for environment '{environment_name}'"
                )
            }
            Self::InvalidTrackerConfiguration { message } => {
                format!("TestCommandHandlerError: Invalid tracker configuration - {message}")
            }
            Self::StateTransition(e) => {
                format!("TestCommandHandlerError: Invalid state transition - {e}")
            }
            Self::StatePersistence(e) => {
                format!("TestCommandHandlerError: State persistence error - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::Command(e) => Some(e),
            Self::EnvironmentNotFound { .. }
            | Self::RemoteAction(_)
            | Self::MissingInstanceIp { .. }
            | Self::InvalidTrackerConfiguration { .. }
            | Self::StateTransition(_)
            | Self::StatePersistence(_) => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::EnvironmentNotFound { .. }
            | Self::MissingInstanceIp { .. }
            | Self::InvalidTrackerConfiguration { .. } => crate::shared::ErrorKind::Configuration,
            Self::Command(_) | Self::RemoteAction(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StateTransition(_) | Self::StatePersistence(_) => {
                crate::shared::ErrorKind::StatePersistence
            }
        }
    }
}

impl TestCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
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
            Self::Command(_) => {
                "Command Execution Failed - Troubleshooting:

1. Check that required tools are installed
2. Verify PATH environment variable includes tool locations
3. Check command permissions and executability
4. Review stderr output above for specific error details

For tool installation, see the setup documentation."
            }
            Self::RemoteAction(_) => {
                "Remote Action Failed - Troubleshooting:

1. Verify the instance is running
2. Check SSH connectivity to the instance
3. Ensure the remote command is available on the instance
4. Review the error message for specific details

For SSH troubleshooting, see docs/contributing/debugging.md"
            }
            Self::MissingInstanceIp { .. } => {
                "Missing Instance IP - Troubleshooting:

The environment does not have an instance IP address set.
This typically means the environment was created but not provisioned.

1. Check the environment state:
   cat data/<env-name>/environment.json

2. Provision the environment first:
   cargo run -- provision <env-name>

3. Then run the test command

For workflow details, see docs/deployment-overview.md"
            }
            Self::InvalidTrackerConfiguration { .. } => {
                "Invalid Tracker Configuration - Troubleshooting:

The tracker configuration in the environment is invalid or incomplete.

1. Check the tracker configuration in your environment file:
   cat data/<env-name>/environment.json

2. Verify the HTTP API bind_address format:
   Expected: \"0.0.0.0:1212\" (host:port)

3. If needed, recreate the environment with correct configuration:
   cargo run -- create template my-config.json
   # Edit my-config.json with correct tracker settings
   cargo run -- create environment --env-file my-config.json

For tracker configuration details, see docs/user-guide/configuration.md"
            }
            Self::StateTransition(_) => {
                "Invalid State Transition - Troubleshooting:

The environment is not in the expected state for this operation.

1. Check current environment state
2. Verify the workflow sequence
3. If environment is in wrong state, destroy and recreate if needed

For workflow details, see docs/deployment-overview.md"
            }
            Self::StatePersistence(_) => {
                "State Persistence Failed - Troubleshooting:

1. Check file system permissions for the data directory
2. Verify available disk space: df -h
3. Ensure no other process is accessing the environment files
4. Check for file system errors: dmesg | tail

State files are stored in: data/<env-name>/

If the problem persists, report it with full system details."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_for_environment_not_found() {
        let error = TestCommandHandlerError::EnvironmentNotFound {
            name: "test-env".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Environment Not Found"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("environment name"));
    }

    #[test]
    fn it_should_provide_help_for_missing_instance_ip() {
        let error = TestCommandHandlerError::MissingInstanceIp {
            environment_name: "test-env".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Missing Instance IP"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("provision"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        use crate::domain::environment::repository::RepositoryError;
        use crate::domain::environment::state::StateTypeError;
        use crate::shared::command::CommandError;

        let errors: Vec<TestCommandHandlerError> = vec![
            TestCommandHandlerError::EnvironmentNotFound {
                name: "test-env".to_string(),
            },
            TestCommandHandlerError::Command(CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            }),
            TestCommandHandlerError::MissingInstanceIp {
                environment_name: "test-env".to_string(),
            },
            TestCommandHandlerError::InvalidTrackerConfiguration {
                message: "Invalid bind address".to_string(),
            },
            TestCommandHandlerError::StateTransition(StateTypeError::UnexpectedState {
                expected: "Provisioned",
                actual: "Created".to_string(),
            }),
            TestCommandHandlerError::StatePersistence(RepositoryError::NotFound),
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
