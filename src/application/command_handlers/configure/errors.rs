//! Error types for the Configure command handler

use crate::shared::command::CommandError;

/// Comprehensive error type for the `ConfigureCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum ConfigureCommandHandlerError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),
}

impl crate::shared::Traceable for ConfigureCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::Command(e) => {
                format!("ConfigureCommandHandlerError: Command execution failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("ConfigureCommandHandlerError: Failed to persist environment state - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::Command(e) => Some(e),
            Self::StatePersistence(_) => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StatePersistence(_) => crate::shared::ErrorKind::StatePersistence,
        }
    }
}

impl ConfigureCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::configure::ConfigureCommandHandlerError;
    /// use torrust_tracker_deployer_lib::shared::command::CommandError;
    ///
    /// let error = ConfigureCommandHandlerError::Command(
    ///     CommandError::ExecutionFailed {
    ///         command: "ansible-playbook".to_string(),
    ///         exit_code: "2".to_string(),
    ///         stdout: String::new(),
    ///         stderr: "connection failed".to_string(),
    ///     }
    /// );
    ///
    /// let help = error.help();
    /// assert!(help.contains("Command Execution"));
    /// assert!(help.contains("Troubleshooting"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::Command(_) => {
                "Command Execution Failed - Troubleshooting:

1. Verify Ansible is installed: ansible --version
2. Check instance connectivity:
   - Verify instance is running: lxc list
   - Test SSH access: ssh -i <key> <user>@<ip>
   - Check Ansible inventory file exists and is correct

3. Common Ansible issues:
   - Python not installed on target: Install python3
   - Wrong SSH user or key: Check inventory file
   - Permission denied: Verify SSH key permissions (chmod 600)
   - Connection refused: Check SSH service on instance

4. Check Ansible playbook syntax:
   ansible-playbook --syntax-check <playbook>.yml

5. Run with verbose output for more details:
   ansible-playbook -vvv <playbook>.yml

For Ansible troubleshooting, see the Ansible documentation."
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::environment::repository::RepositoryError;
    use crate::shared::command::CommandError;

    #[test]
    fn it_should_provide_help_for_command_execution() {
        let error = ConfigureCommandHandlerError::Command(CommandError::ExecutionFailed {
            command: "ansible-playbook".to_string(),
            exit_code: "2".to_string(),
            stdout: String::new(),
            stderr: "error".to_string(),
        });

        let help = error.help();
        assert!(help.contains("Command Execution"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("Ansible"));
    }

    #[test]
    fn it_should_provide_help_for_state_persistence() {
        let error = ConfigureCommandHandlerError::StatePersistence(RepositoryError::NotFound);

        let help = error.help();
        assert!(help.contains("State Persistence"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("data/<env-name>/"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        let errors = vec![
            ConfigureCommandHandlerError::Command(CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            }),
            ConfigureCommandHandlerError::StatePersistence(RepositoryError::NotFound),
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
