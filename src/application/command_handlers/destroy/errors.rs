//! Error types for the Destroy command handler

use std::path::PathBuf;

use crate::adapters::tofu::client::OpenTofuError;
use crate::domain::environment::state::StateTypeError;
use crate::shared::command::CommandError;

/// Comprehensive error type for the `DestroyCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum DestroyCommandHandlerError {
    #[error("OpenTofu command failed: {0}")]
    OpenTofu(#[from] OpenTofuError),

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),

    #[error("Invalid state transition: {0}")]
    StateTransition(#[from] StateTypeError),

    #[error("Failed to clean up state files at '{path}': {source}")]
    StateCleanupFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl crate::shared::Traceable for DestroyCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::OpenTofu(e) => {
                format!("DestroyCommandHandlerError: OpenTofu command failed - {e}")
            }
            Self::Command(e) => {
                format!("DestroyCommandHandlerError: Command execution failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("DestroyCommandHandlerError: Failed to persist environment state - {e}")
            }
            Self::StateTransition(e) => {
                format!("DestroyCommandHandlerError: Invalid state transition - {e}")
            }
            Self::StateCleanupFailed { path, source } => {
                format!(
                    "DestroyCommandHandlerError: Failed to clean up state files at '{}' - {source}",
                    path.display()
                )
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::OpenTofu(e) => Some(e),
            Self::Command(e) => Some(e),
            Self::StatePersistence(_)
            | Self::StateTransition(_)
            | Self::StateCleanupFailed { .. } => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::OpenTofu(_) => crate::shared::ErrorKind::InfrastructureOperation,
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StateTransition(_) => crate::shared::ErrorKind::Configuration,
            Self::StatePersistence(_) | Self::StateCleanupFailed { .. } => {
                crate::shared::ErrorKind::StatePersistence
            }
        }
    }
}

impl DestroyCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::destroy::DestroyCommandHandlerError;
    /// use torrust_tracker_deployer_lib::adapters::tofu::client::OpenTofuError;
    /// use torrust_tracker_deployer_lib::shared::command::CommandError;
    ///
    /// let error = DestroyCommandHandlerError::OpenTofu(
    ///     OpenTofuError::CommandError(CommandError::ExecutionFailed {
    ///         command: "tofu".to_string(),
    ///         exit_code: "1".to_string(),
    ///         stdout: String::new(),
    ///         stderr: "error".to_string(),
    ///     })
    /// );
    ///
    /// let help = error.help();
    /// assert!(help.contains("OpenTofu"));
    /// assert!(help.contains("Troubleshooting"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::OpenTofu(_) => {
                "OpenTofu Destroy Failed - Troubleshooting:

1. Check OpenTofu is installed: tofu version
2. Verify LXD is running: lxc version
3. Check if instance still exists: lxc list
4. Review OpenTofu error output above for specific issues
5. Try manually running:
   cd build/<env-name> && tofu destroy

6. Common issues:
   - Instance already deleted: Normal, destroy succeeds
   - LXD not running: Start LXD service
   - Permission denied: Check LXD group membership
   - State file locked: Wait or remove .terraform.lock.hcl

7. Force removal if needed:
   lxc delete <instance-name> --force

For LXD troubleshooting, see docs/vm-providers.md"
            }
            Self::Command(_) => {
                "Command Execution Failed - Troubleshooting:

1. Check that required tools are installed (tofu)
2. Verify PATH environment variable includes tool locations
3. Check command permissions and executability
4. Review stderr output above for specific error details
5. Try running the command manually to diagnose issues

Common issues:
- Tool not in PATH: which tofu
- Permission denied: Check execute permissions
- Command not found: Install OpenTofu

For tool installation, see the setup documentation."
            }
            Self::StatePersistence(_) => {
                "State Persistence Failed - Troubleshooting:

1. Check file system permissions for the data directory
2. Verify available disk space: df -h
3. Ensure no other process is accessing the environment files
4. Check for file system errors: dmesg | tail
5. Verify the data directory is writable

State files are stored in: data/<env-name>/

The repository handles directory creation atomically during save.
If partially created files exist, remove them and retry.

If the problem persists, report it with full system details."
            }
            Self::StateTransition(_) => {
                "Invalid State Transition - Troubleshooting:

1. This error indicates an internal state machine issue
2. Check the current environment state: cat data/<env-name>/environment.json
3. The environment may be in an unexpected state

Possible causes:
- Manual modification of state files
- Interrupted previous command
- Bug in state transition logic

To recover:
1. Check environment status to understand current state
2. If necessary, manually edit state file (not recommended)
3. Or destroy and recreate the environment

If this persists, please report it with full details."
            }
            Self::StateCleanupFailed { .. } => {
                "State Cleanup Failed - Troubleshooting:

1. Check file permissions for the directory:
   ls -la <parent-directory>

2. Verify the path exists and is accessible

3. Check for read-only filesystems:
   mount | grep <filesystem>

4. Look for process locks:
   lsof | grep <path>

5. Manual cleanup if needed:
   rm -rf data/<env-name>
   rm -rf build/<env-name>

Common causes:
- Permission denied: Check ownership and permissions
- Directory in use: Close applications using the directory
- Read-only filesystem: Remount as read-write
- File system errors: Check dmesg for errors

If the problem persists, report it with full system details."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::tofu::client::OpenTofuError;
    use crate::domain::environment::repository::RepositoryError;
    use crate::domain::environment::state::StateTypeError;
    use crate::shared::command::CommandError;
    use std::path::PathBuf;

    #[test]
    fn it_should_provide_help_for_opentofu_error() {
        use crate::shared::command::CommandError;

        let error = DestroyCommandHandlerError::OpenTofu(OpenTofuError::CommandError(
            CommandError::ExecutionFailed {
                command: "tofu".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "destroy failed".to_string(),
            },
        ));

        let help = error.help();
        assert!(help.contains("OpenTofu Destroy"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("tofu version"));
        assert!(help.contains("lxc list"));
    }

    #[test]
    fn it_should_provide_help_for_command_execution() {
        let error = DestroyCommandHandlerError::Command(CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "error".to_string(),
        });

        let help = error.help();
        assert!(help.contains("Command Execution"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("PATH"));
    }

    #[test]
    fn it_should_provide_help_for_state_persistence() {
        let error = DestroyCommandHandlerError::StatePersistence(RepositoryError::NotFound);

        let help = error.help();
        assert!(help.contains("State Persistence"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("data/<env-name>/"));
    }

    #[test]
    fn it_should_provide_help_for_state_transition() {
        let error = DestroyCommandHandlerError::StateTransition(StateTypeError::UnexpectedState {
            expected: "Provisioned",
            actual: "Created".to_string(),
        });

        let help = error.help();
        assert!(help.contains("State Transition"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("state machine"));
    }

    #[test]
    fn it_should_provide_help_for_state_cleanup_failed() {
        let error = DestroyCommandHandlerError::StateCleanupFailed {
            path: PathBuf::from("/test/path"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };

        let help = error.help();
        assert!(help.contains("State Cleanup"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("permissions"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        let errors = vec![
            DestroyCommandHandlerError::OpenTofu(OpenTofuError::CommandError(
                CommandError::ExecutionFailed {
                    command: "tofu".to_string(),
                    exit_code: "1".to_string(),
                    stdout: String::new(),
                    stderr: "error".to_string(),
                },
            )),
            DestroyCommandHandlerError::Command(CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            }),
            DestroyCommandHandlerError::StatePersistence(RepositoryError::NotFound),
            DestroyCommandHandlerError::StateTransition(StateTypeError::UnexpectedState {
                expected: "Provisioned",
                actual: "Created".to_string(),
            }),
            DestroyCommandHandlerError::StateCleanupFailed {
                path: PathBuf::from("/test"),
                source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
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
