//! Error types for the Provision command handler

use crate::adapters::ssh::SshError;
use crate::adapters::tofu::client::OpenTofuError;
use crate::application::steps::RenderAnsibleTemplatesError;
use crate::infrastructure::external_tools::tofu::ProvisionTemplateError;
use crate::shared::command::CommandError;

/// Comprehensive error type for the `ProvisionCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum ProvisionCommandHandlerError {
    #[error("OpenTofu template rendering failed: {0}")]
    OpenTofuTemplateRendering(#[from] ProvisionTemplateError),

    #[error("Ansible template rendering failed: {0}")]
    AnsibleTemplateRendering(#[from] RenderAnsibleTemplatesError),

    #[error("OpenTofu command failed: {0}")]
    OpenTofu(#[from] OpenTofuError),

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("SSH connectivity failed: {0}")]
    SshConnectivity(#[from] SshError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),
}

impl crate::shared::Traceable for ProvisionCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::OpenTofuTemplateRendering(e) => {
                format!("ProvisionCommandHandlerError: OpenTofu template rendering failed - {e}")
            }
            Self::AnsibleTemplateRendering(e) => {
                format!("ProvisionCommandHandlerError: Ansible template rendering failed - {e}")
            }
            Self::OpenTofu(e) => {
                format!("ProvisionCommandHandlerError: OpenTofu command failed - {e}")
            }
            Self::Command(e) => {
                format!("ProvisionCommandHandlerError: Command execution failed - {e}")
            }
            Self::SshConnectivity(e) => {
                format!("ProvisionCommandHandlerError: SSH connectivity failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("ProvisionCommandHandlerError: Failed to persist environment state - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::OpenTofuTemplateRendering(e) => Some(e),
            Self::AnsibleTemplateRendering(e) => Some(e),
            Self::OpenTofu(e) => Some(e),
            Self::Command(e) => Some(e),
            Self::SshConnectivity(e) => Some(e),
            Self::StatePersistence(_) => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::OpenTofuTemplateRendering(_) | Self::AnsibleTemplateRendering(_) => {
                crate::shared::ErrorKind::TemplateRendering
            }
            Self::OpenTofu(_) => crate::shared::ErrorKind::InfrastructureOperation,
            Self::SshConnectivity(_) => crate::shared::ErrorKind::NetworkConnectivity,
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StatePersistence(_) => crate::shared::ErrorKind::StatePersistence,
        }
    }
}

impl ProvisionCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::provision::ProvisionCommandHandlerError;
    /// use torrust_tracker_deployer_lib::adapters::tofu::client::OpenTofuError;
    /// use torrust_tracker_deployer_lib::shared::command::CommandError;
    ///
    /// let error = ProvisionCommandHandlerError::OpenTofu(
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
            Self::OpenTofuTemplateRendering(_) => {
                "OpenTofu Template Rendering Failed - Troubleshooting:

1. Check that template source files exist in the templates directory
2. Verify template syntax is valid (Tera template syntax)
3. Ensure all required template variables are provided
4. Check file permissions on template directories
5. Verify the templates directory structure matches expected layout

Template files should be in: templates/tofu/

For template syntax issues, see the Tera template documentation."
            }
            Self::AnsibleTemplateRendering(_) => {
                "Ansible Template Rendering Failed - Troubleshooting:

1. Check that Ansible template files exist in the templates directory
2. Verify template syntax is valid (Tera template syntax)
3. Ensure runtime variables (IP address, SSH credentials) are provided
4. Check file permissions on template directories
5. Verify the templates directory structure matches expected layout

Template files should be in: templates/ansible/

For template syntax issues, see the Tera template documentation."
            }
            Self::OpenTofu(_) => {
                "OpenTofu Command Failed - Troubleshooting:

1. Check OpenTofu is installed: tofu version
2. Verify LXD is running: lxc version
3. Check LXD permissions: lxc list
4. Review OpenTofu error output above for specific issues
5. Try manually running:
   cd build/<env-name> && tofu init && tofu plan

6. Common LXD issues:
   - LXD not initialized: lxd init
   - User not in lxd group: sudo usermod -aG lxd $USER (requires logout)
   - LXD network not configured: lxc network list

For LXD setup issues, see docs/vm-providers.md"
            }
            Self::Command(_) => {
                "Command Execution Failed - Troubleshooting:

1. Check that required tools are installed (tofu, ansible, ssh)
2. Verify PATH environment variable includes tool locations
3. Check command permissions and executability
4. Review stderr output above for specific error details
5. Try running the command manually to diagnose issues

Common issues:
- Tool not in PATH: which <tool-name>
- Permission denied: Check execute permissions
- Command not found: Install the required tool

For tool installation, see the setup documentation."
            }
            Self::SshConnectivity(_) => {
                "SSH Connectivity Failed - Troubleshooting:

1. Verify the instance is running: lxc list
2. Check instance IP address: lxc list
3. Test SSH connectivity manually:
   ssh -i <key-path> <user>@<ip-address>

4. Common SSH issues:
   - SSH key permissions: chmod 600 <key-path>
   - SSH service not running: Check cloud-init status on instance
   - Firewall blocking SSH: Check UFW or iptables rules
   - Wrong SSH user: Verify username in configuration

5. Check cloud-init completion:
   lxc exec <instance-name> -- cloud-init status --wait

For SSH troubleshooting, see docs/debugging.md"
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
    use crate::adapters::tofu::client::OpenTofuError;
    use crate::domain::environment::repository::RepositoryError;

    #[test]
    fn it_should_provide_help_for_opentofu_template_rendering() {
        use crate::infrastructure::external_tools::tofu::ProvisionTemplateError;

        let error = ProvisionCommandHandlerError::OpenTofuTemplateRendering(
            ProvisionTemplateError::DirectoryCreationFailed {
                directory: "test".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
            },
        );

        let help = error.help();
        assert!(help.contains("OpenTofu Template"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("templates/tofu/"));
    }

    #[test]
    fn it_should_provide_help_for_ansible_template_rendering() {
        use crate::application::steps::RenderAnsibleTemplatesError;
        use crate::infrastructure::external_tools::ansible::template::wrappers::inventory::InventoryContextError;

        let error = ProvisionCommandHandlerError::AnsibleTemplateRendering(
            RenderAnsibleTemplatesError::InventoryContextError(
                InventoryContextError::MissingAnsibleHost,
            ),
        );

        let help = error.help();
        assert!(help.contains("Ansible Template"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("templates/ansible/"));
    }

    #[test]
    fn it_should_provide_help_for_opentofu_command() {
        use crate::shared::command::CommandError;

        let error = ProvisionCommandHandlerError::OpenTofu(OpenTofuError::CommandError(
            CommandError::ExecutionFailed {
                command: "tofu".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            },
        ));

        let help = error.help();
        assert!(help.contains("OpenTofu Command"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("tofu version"));
        assert!(help.contains("lxc list"));
    }

    #[test]
    fn it_should_provide_help_for_command_execution() {
        use crate::shared::command::CommandError;

        let error = ProvisionCommandHandlerError::Command(CommandError::ExecutionFailed {
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
    fn it_should_provide_help_for_ssh_connectivity() {
        use crate::adapters::ssh::SshError;

        let error = ProvisionCommandHandlerError::SshConnectivity(SshError::ConnectivityTimeout {
            host_ip: "10.0.0.1".to_string(),
            attempts: 5,
            timeout_seconds: 30,
        });

        let help = error.help();
        assert!(help.contains("SSH Connectivity"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("lxc list"));
        assert!(help.contains("cloud-init"));
    }

    #[test]
    fn it_should_provide_help_for_state_persistence() {
        let error = ProvisionCommandHandlerError::StatePersistence(RepositoryError::NotFound);

        let help = error.help();
        assert!(help.contains("State Persistence"));
        assert!(help.contains("Troubleshooting"));
        assert!(help.contains("data/<env-name>/"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        use crate::adapters::ssh::SshError;
        use crate::application::steps::RenderAnsibleTemplatesError;
        use crate::infrastructure::external_tools::ansible::template::wrappers::inventory::InventoryContextError;
        use crate::infrastructure::external_tools::tofu::ProvisionTemplateError;
        use crate::shared::command::CommandError;

        let errors = vec![
            ProvisionCommandHandlerError::OpenTofuTemplateRendering(
                ProvisionTemplateError::DirectoryCreationFailed {
                    directory: "test".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
                },
            ),
            ProvisionCommandHandlerError::AnsibleTemplateRendering(
                RenderAnsibleTemplatesError::InventoryContextError(
                    InventoryContextError::MissingAnsibleHost,
                ),
            ),
            ProvisionCommandHandlerError::OpenTofu(OpenTofuError::CommandError(
                CommandError::ExecutionFailed {
                    command: "tofu".to_string(),
                    exit_code: "1".to_string(),
                    stdout: String::new(),
                    stderr: "error".to_string(),
                },
            )),
            ProvisionCommandHandlerError::Command(CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            }),
            ProvisionCommandHandlerError::SshConnectivity(SshError::ConnectivityTimeout {
                host_ip: "10.0.0.1".to_string(),
                attempts: 5,
                timeout_seconds: 30,
            }),
            ProvisionCommandHandlerError::StatePersistence(RepositoryError::NotFound),
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
