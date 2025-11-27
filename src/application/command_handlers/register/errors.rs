//! Error types for the Register command handler

use std::net::IpAddr;

use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::RepositoryError;
use crate::domain::environment::state::StateTypeError;
use crate::shared::error::{ErrorKind, Traceable};

/// Comprehensive error type for the `RegisterCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum RegisterCommandHandlerError {
    /// Environment was not found in repository
    #[error("Environment '{name}' not found")]
    EnvironmentNotFound {
        /// The name of the environment that was not found
        name: EnvironmentName,
    },

    /// Environment is not in the expected Created state
    #[error("Environment '{name}' is not in Created state (current: {current_state})")]
    InvalidState {
        /// The name of the environment
        name: EnvironmentName,
        /// The actual state of the environment
        current_state: String,
    },

    /// Failed to connect to the instance via SSH
    #[error("Failed to connect to instance at {address}: {reason}")]
    ConnectivityFailed {
        /// The IP address that failed to connect
        address: IpAddr,
        /// Description of why the connection failed
        reason: String,
    },

    /// Invalid IP address provided
    #[error("Invalid IP address: {value}")]
    InvalidIpAddress {
        /// The invalid IP address string
        value: String,
    },

    /// Failed to persist environment state
    #[error("Failed to save environment: {0}")]
    RepositorySave(#[from] RepositoryError),

    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    StateTransition(#[from] StateTypeError),

    /// Failed to render Ansible templates
    #[error("Failed to render Ansible templates: {reason}")]
    TemplateRenderingFailed {
        /// Description of why template rendering failed
        reason: String,
    },
}

impl Traceable for RegisterCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::EnvironmentNotFound { name } => {
                format!("RegisterCommandHandlerError: Environment '{name}' not found")
            }
            Self::InvalidState {
                name,
                current_state,
            } => {
                format!(
                    "RegisterCommandHandlerError: Environment '{name}' is not in Created state (current: {current_state})"
                )
            }
            Self::ConnectivityFailed { address, reason } => {
                format!(
                    "RegisterCommandHandlerError: Failed to connect to instance at {address} - {reason}"
                )
            }
            Self::InvalidIpAddress { value } => {
                format!("RegisterCommandHandlerError: Invalid IP address: {value}")
            }
            Self::RepositorySave(e) => {
                format!("RegisterCommandHandlerError: Failed to save environment - {e}")
            }
            Self::StateTransition(e) => {
                format!("RegisterCommandHandlerError: Invalid state transition - {e}")
            }
            Self::TemplateRenderingFailed { reason } => {
                format!(
                    "RegisterCommandHandlerError: Failed to render Ansible templates - {reason}"
                )
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::ConnectivityFailed { .. }
            | Self::EnvironmentNotFound { .. }
            | Self::InvalidState { .. }
            | Self::InvalidIpAddress { .. }
            | Self::RepositorySave(_)
            | Self::StateTransition(_)
            | Self::TemplateRenderingFailed { .. } => None,
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::EnvironmentNotFound { .. } | Self::InvalidState { .. } => {
                ErrorKind::StatePersistence
            }
            Self::ConnectivityFailed { .. } => ErrorKind::NetworkConnectivity,
            Self::InvalidIpAddress { .. } => ErrorKind::Configuration,
            Self::RepositorySave(_) | Self::StateTransition(_) => ErrorKind::StatePersistence,
            Self::TemplateRenderingFailed { .. } => ErrorKind::TemplateRendering,
        }
    }
}

impl RegisterCommandHandlerError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::register::RegisterCommandHandlerError;
    /// use torrust_tracker_deployer_lib::domain::environment::name::EnvironmentName;
    ///
    /// let name = EnvironmentName::new("test-env".to_string()).unwrap();
    /// let error = RegisterCommandHandlerError::EnvironmentNotFound { name };
    ///
    /// let help = error.help();
    /// assert!(help.contains("create environment"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::EnvironmentNotFound { .. } => {
                "Environment Not Found - Troubleshooting:

1. Verify the environment name is correct
2. Check that 'create environment' was run first:
   torrust-tracker-deployer create environment --env-file config.json

3. Verify you're in the correct working directory
4. List existing environments in the data directory

The register command requires an existing environment in 'Created' state.
Use 'create environment' to create one first."
            }
            Self::InvalidState { .. } => {
                "Invalid Environment State - Troubleshooting:

The environment must be in 'Created' state to use the register command.

1. If the environment is already provisioned:
   - The instance IP is already set
   - Continue with 'configure' command instead

2. If you want to re-register with a different IP:
   - Destroy the environment first: torrust-tracker-deployer destroy <name>
   - Create a new environment: torrust-tracker-deployer create environment --env-file config.json
   - Register the new instance: torrust-tracker-deployer register <name> --instance-ip <IP>

3. Valid states for register: Created
   Invalid states: Provisioning, Provisioned, Configuring, Configured, etc."
            }
            Self::ConnectivityFailed { .. } => {
                "SSH Connectivity Failed - Troubleshooting:

1. Verify the instance is running and reachable:
   ping <instance-ip>

2. Verify SSH is running on the instance:
   nc -zv <instance-ip> 22

3. Test SSH connectivity manually:
   ssh -i <key-path> <user>@<instance-ip>

4. Common SSH issues:
   - SSH key permissions: chmod 600 <key-path>
   - SSH service not running on the instance
   - Firewall blocking port 22
   - Wrong SSH user in environment configuration
   - SSH key not installed on the instance

5. Verify the SSH key is installed on the instance:
   - Copy the public key to ~/.ssh/authorized_keys on the instance

For SSH troubleshooting, see docs/debugging.md"
            }
            Self::InvalidIpAddress { .. } => {
                "Invalid IP Address - Troubleshooting:

1. Verify the IP address format:
   - IPv4: 192.168.1.100
   - IPv6: 2001:db8::1

2. Common issues:
   - Including port number (use just the IP, not IP:port)
   - Extra whitespace or characters
   - Using hostname instead of IP address

Example usage:
  torrust-tracker-deployer register my-env --instance-ip 192.168.1.100"
            }
            Self::RepositorySave(_) => {
                "State Persistence Failed - Troubleshooting:

1. Check file system permissions for the data directory
2. Verify available disk space: df -h
3. Check if another process has locked the environment file
4. Try running the command again

If the problem persists, check the data directory for corruption."
            }
            Self::StateTransition(_) => {
                "State Transition Failed - Troubleshooting:

This is an internal error indicating an invalid state machine transition.

1. Check the environment state in the data directory
2. Try destroying and recreating the environment

If this error persists, please report it as a bug."
            }
            Self::TemplateRenderingFailed { .. } => {
                "Template Rendering Failed - Troubleshooting:

The Ansible template rendering step failed during instance registration.

1. Check the template files exist in the templates/ansible directory
2. Verify the environment configuration is complete and valid
3. Check disk space and permissions in the build directory
4. Review the error message for specific template or variable issues

Common causes:
- Missing template files
- Invalid template variable references
- Disk full or permission denied

If the problem persists, please report it as a bug."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod error_construction {
        use super::*;

        #[test]
        fn it_should_create_environment_not_found_error() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let error = RegisterCommandHandlerError::EnvironmentNotFound { name };

            assert!(error.to_string().contains("test-env"));
            assert!(error.to_string().contains("not found"));
        }

        #[test]
        fn it_should_create_invalid_state_error() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let error = RegisterCommandHandlerError::InvalidState {
                name,
                current_state: "Provisioned".to_string(),
            };

            assert!(error.to_string().contains("test-env"));
            assert!(error.to_string().contains("Provisioned"));
            assert!(error.to_string().contains("Created"));
        }

        #[test]
        fn it_should_create_invalid_ip_address_error() {
            let error = RegisterCommandHandlerError::InvalidIpAddress {
                value: "not-an-ip".to_string(),
            };

            assert!(error.to_string().contains("not-an-ip"));
            assert!(error.to_string().contains("Invalid IP address"));
        }
    }

    mod help_methods {
        use super::*;

        #[test]
        fn it_should_provide_help_for_all_error_variants() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();

            let errors: Vec<RegisterCommandHandlerError> = vec![
                RegisterCommandHandlerError::EnvironmentNotFound { name: name.clone() },
                RegisterCommandHandlerError::InvalidState {
                    name,
                    current_state: "Provisioned".to_string(),
                },
                RegisterCommandHandlerError::InvalidIpAddress {
                    value: "bad".to_string(),
                },
                RegisterCommandHandlerError::RepositorySave(RepositoryError::NotFound),
                RegisterCommandHandlerError::StateTransition(StateTypeError::UnexpectedState {
                    expected: "Created",
                    actual: "Provisioned".to_string(),
                }),
            ];

            for error in errors {
                let help = error.help();
                assert!(!help.is_empty(), "Help should not be empty for: {error}");
                assert!(
                    help.contains("Troubleshooting"),
                    "Help should contain troubleshooting guidance for: {error}"
                );
            }
        }
    }

    mod traceable_implementation {
        use super::*;

        #[test]
        fn it_should_implement_traceable_trait() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let error = RegisterCommandHandlerError::EnvironmentNotFound { name };

            let trace = error.trace_format();
            assert!(trace.contains("RegisterCommandHandlerError"));
            assert!(trace.contains("test-env"));
        }

        #[test]
        fn it_should_return_correct_error_kinds() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();

            let error = RegisterCommandHandlerError::EnvironmentNotFound { name };
            assert!(matches!(error.error_kind(), ErrorKind::StatePersistence));

            let error = RegisterCommandHandlerError::InvalidIpAddress {
                value: "bad".to_string(),
            };
            assert!(matches!(error.error_kind(), ErrorKind::Configuration));
        }
    }
}
