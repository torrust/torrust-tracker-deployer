//! Error types for the Release command handler

use crate::application::steps::application::DeployComposeFilesStepError;
use crate::domain::environment::state::StateTypeError;
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

    /// Instance IP address is not available (required for deployment)
    ///
    /// The release command requires the instance IP address to deploy files
    /// to the remote host. This IP should be available after provisioning.
    #[error("Instance IP address is not available for environment '{name}'. The provision step should have set this value.")]
    MissingInstanceIp {
        /// The name of the environment missing the instance IP
        name: String,
    },

    /// Environment is in an invalid state for release
    #[error("Environment is in an invalid state for release: {0}")]
    InvalidState(#[from] StateTypeError),

    /// Failed to persist environment state
    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),

    /// Template rendering failed
    #[error("Template rendering failed: {0}")]
    TemplateRendering(String),

    /// Tracker storage directory creation failed
    #[error("Tracker storage creation failed: {0}")]
    TrackerStorageCreation(String),

    /// Tracker database initialization failed
    #[error("Tracker database initialization failed: {0}")]
    TrackerDatabaseInit(String),

    /// Prometheus storage directory creation failed
    #[error("Prometheus storage creation failed: {0}")]
    PrometheusStorageCreation(String),

    /// Caddy configuration deployment failed
    #[error("Caddy configuration deployment failed: {0}")]
    CaddyConfigDeployment(String),

    /// General deployment operation failed
    #[error("Deployment failed: {message}")]
    Deployment {
        /// The error message
        message: String,
        /// The underlying error source
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Deployment to remote host failed
    #[error("Deployment to remote host failed: {message}")]
    DeploymentFailed {
        /// Description of the failure
        message: String,
        /// The underlying deployment step error
        #[source]
        source: DeployComposeFilesStepError,
    },

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
            Self::MissingInstanceIp { name } => {
                format!("ReleaseCommandHandlerError: Instance IP not available for environment '{name}'")
            }
            Self::InvalidState(e) => {
                format!("ReleaseCommandHandlerError: Invalid state for release - {e}")
            }
            Self::StatePersistence(e) => {
                format!("ReleaseCommandHandlerError: Failed to persist environment state - {e}")
            }
            Self::TemplateRendering(message) => {
                format!("ReleaseCommandHandlerError: Template rendering failed - {message}")
            }
            Self::TrackerStorageCreation(message) => {
                format!("ReleaseCommandHandlerError: Tracker storage creation failed - {message}")
            }
            Self::TrackerDatabaseInit(message) => {
                format!("ReleaseCommandHandlerError: Tracker database initialization failed - {message}")
            }
            Self::PrometheusStorageCreation(message) => {
                format!(
                    "ReleaseCommandHandlerError: Prometheus storage creation failed - {message}"
                )
            }
            Self::CaddyConfigDeployment(message) => {
                format!(
                    "ReleaseCommandHandlerError: Caddy configuration deployment failed - {message}"
                )
            }
            Self::Deployment { message, .. } | Self::DeploymentFailed { message, .. } => {
                format!("ReleaseCommandHandlerError: Deployment failed - {message}")
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
            // Box<dyn Error> doesn't implement Traceable
            Self::DeploymentFailed { source, .. } => Some(source),
            Self::Deployment { .. }
            | Self::StatePersistence(_)
            | Self::EnvironmentNotFound { .. }
            | Self::MissingInstanceIp { .. }
            | Self::InvalidState(_)
            | Self::TemplateRendering(_)
            | Self::TrackerStorageCreation(_)
            | Self::TrackerDatabaseInit(_)
            | Self::PrometheusStorageCreation(_)
            | Self::CaddyConfigDeployment(_)
            | Self::ReleaseOperationFailed { .. } => None,
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::EnvironmentNotFound { .. }
            | Self::MissingInstanceIp { .. }
            | Self::InvalidState(_) => ErrorKind::Configuration,
            Self::StatePersistence(_) => ErrorKind::StatePersistence,
            Self::TemplateRendering(_)
            | Self::TrackerStorageCreation(_)
            | Self::TrackerDatabaseInit(_)
            | Self::PrometheusStorageCreation(_)
            | Self::CaddyConfigDeployment(_) => ErrorKind::TemplateRendering,
            Self::Deployment { .. } | Self::ReleaseOperationFailed { .. } => {
                ErrorKind::InfrastructureOperation
            }
            Self::DeploymentFailed { source, .. } => source.error_kind(),
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
    #[allow(clippy::too_many_lines)]
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
            Self::MissingInstanceIp { .. } => {
                "Missing Instance IP Address - Troubleshooting:

The release command requires the instance IP address to deploy files to the
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
            Self::TemplateRendering(_) => {
                "Template Rendering Failed - Troubleshooting:

1. Check that template files exist in the templates directory
2. Verify template syntax is valid
3. Check that all required template variables are provided
4. Verify file system permissions for the build directory

Common causes:
- Missing template files
- Invalid template syntax
- Insufficient disk space
- Permission denied on build directory

For more information, see docs/user-guide/commands.md"
            }
            Self::TrackerStorageCreation(_) => {
                "Tracker Storage Creation Failed - Troubleshooting:

1. Verify the target instance is reachable:
   ssh <user>@<instance-ip>

2. Check that the instance has sufficient disk space:
   df -h

3. Verify the Ansible playbook exists:
   ls templates/ansible/create-tracker-storage.yml

4. Check Ansible execution permissions

5. Review the error message above for specific details

Common causes:
- Insufficient disk space on target instance
- Permission denied on target directories
- Ansible playbook not found
- Network connectivity issues

For more information, see docs/user-guide/commands.md"
            }
            Self::TrackerDatabaseInit(_) => {
                "Tracker Database Initialization Failed - Troubleshooting:

1. Verify the tracker storage directories were created:
   ssh <user>@<instance-ip> 'ls -la /opt/torrust/storage/tracker/lib/database'

2. Check that the instance has sufficient disk space:
   df -h

3. Verify the Ansible playbook exists:
   ls templates/ansible/init-tracker-database.yml

4. Check file permissions on the database directory

5. Review the error message above for specific details

Common causes:
- Storage directories don't exist (run CreateTrackerStorage step first)
- Insufficient disk space on target instance
- Permission denied on database directory
- Ansible playbook not found
- Network connectivity issues

For more information, see docs/user-guide/commands.md"
            }
            Self::PrometheusStorageCreation(_) => {
                "Prometheus Storage Creation Failed - Troubleshooting:

1. Verify the target instance is reachable:
   ssh <user>@<instance-ip>

2. Check that the instance has sufficient disk space:
   df -h

3. Verify the Ansible playbook exists:
   ls templates/ansible/create-prometheus-storage.yml

4. Check Ansible execution permissions

5. Review the error message above for specific details

Common causes:
- Insufficient disk space on target instance
- Permission denied on target directories
- Ansible playbook not found
- Network connectivity issues

For more information, see docs/user-guide/commands.md"
            }
            Self::CaddyConfigDeployment(_) => {
                "Caddy Configuration Deployment Failed - Troubleshooting:

1. Verify the target instance is reachable:
   ssh <user>@<instance-ip>

2. Check that the Caddyfile was generated in the build directory:
   ls build/<env-name>/caddy/Caddyfile

3. Verify the Ansible playbook exists:
   ls templates/ansible/deploy-caddy-config.yml

4. Check that the instance has sufficient disk space:
   df -h

5. Review the error message above for specific details

Common causes:
- Caddyfile not generated (HTTPS not configured)
- Insufficient disk space on target instance
- Permission denied on target directories
- Ansible playbook not found
- Network connectivity issues

For more information, see docs/user-guide/commands.md"
            }
            Self::Deployment { .. } => {
                "Deployment Failed - Troubleshooting:

1. Verify the build directory exists and contains expected files

2. Check that the target instance is reachable:
   ssh <user>@<instance-ip>

3. Ensure Ansible playbook executed successfully

4. Review the error message above for specific details

5. Check file permissions and disk space on target

Common causes:
- Build directory not found or incomplete
- Network connectivity issues
- SSH authentication failure
- Insufficient permissions on target
- Disk space issues on target instance

For more information, see docs/user-guide/commands.md"
            }
            Self::DeploymentFailed { source, .. } => source.help(),
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
    use crate::domain::environment::state::StateTypeError;

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
        let error = ReleaseCommandHandlerError::InvalidState(StateTypeError::UnexpectedState {
            expected: "configured",
            actual: "created".to_string(),
        });

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
    fn it_should_provide_help_for_template_rendering() {
        let error = ReleaseCommandHandlerError::TemplateRendering("Test error".to_string());

        let help = error.help();
        assert!(help.contains("Template Rendering"));
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
    fn it_should_provide_help_for_missing_instance_ip() {
        let error = ReleaseCommandHandlerError::MissingInstanceIp {
            name: "test-env".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Missing Instance IP"));
        assert!(help.contains("Troubleshooting"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        let errors: Vec<ReleaseCommandHandlerError> = vec![
            ReleaseCommandHandlerError::EnvironmentNotFound {
                name: "test".to_string(),
            },
            ReleaseCommandHandlerError::MissingInstanceIp {
                name: "test".to_string(),
            },
            ReleaseCommandHandlerError::InvalidState(StateTypeError::UnexpectedState {
                expected: "configured",
                actual: "created".to_string(),
            }),
            ReleaseCommandHandlerError::StatePersistence(RepositoryError::NotFound),
            ReleaseCommandHandlerError::TemplateRendering("test".to_string()),
            ReleaseCommandHandlerError::DeploymentFailed {
                message: "test".to_string(),
                source: DeployComposeFilesStepError::ComposeBuildDirNotFound {
                    path: "/tmp/test".to_string(),
                },
            },
            ReleaseCommandHandlerError::ReleaseOperationFailed {
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
