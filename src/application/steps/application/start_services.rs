//! Start Docker Compose services step
//!
//! This module provides the `StartServicesStep` which handles starting the
//! Docker Compose application stack on a remote host via Ansible.
//!
//! ## Key Features
//!
//! - Executes `docker compose up -d` on the remote host
//! - Pulls container images before starting
//! - Waits for services to become healthy
//! - Reports container status
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `RunCommandHandler` orchestrates the run workflow
//! - **Step** (Level 2): This `StartServicesStep` handles service startup
//! - **Remote Action** (Level 3): Ansible playbook executes on the remote host
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use std::path::PathBuf;
//! use crate::adapters::ansible::AnsibleClient;
//! use crate::application::steps::application::StartServicesStep;
//!
//! let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("/path/to/ansible/build")));
//!
//! let step = StartServicesStep::new(ansible_client);
//! step.execute()?;
//! ```

use std::sync::Arc;

use thiserror::Error;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;
use crate::shared::{ErrorKind, Traceable};

/// Step that starts Docker Compose services on a remote host via Ansible
///
/// This step handles the execution of `docker compose up -d` on the remote
/// instance, bringing up all application containers defined in the compose file.
pub struct StartServicesStep {
    ansible_client: Arc<AnsibleClient>,
}

impl StartServicesStep {
    /// Creates a new `StartServicesStep`
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - The Ansible client for executing playbooks
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the service startup step
    ///
    /// This will run the "run-compose-services" Ansible playbook to start
    /// all Docker Compose services on the remote host.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The Ansible playbook execution fails
    /// * Docker Compose services fail to start
    /// * Container health checks fail
    #[instrument(
        name = "start_services",
        skip_all,
        fields(step_type = "application", operation = "start_services")
    )]
    pub fn execute(&self) -> Result<(), StartServicesStepError> {
        info!(
            step = "start_services",
            status = "starting",
            "Starting Docker Compose services on remote host"
        );

        self.ansible_client
            .run_playbook("run-compose-services", &[])
            .map_err(|source| StartServicesStepError::AnsiblePlaybookFailed {
                message: source.to_string(),
                source,
            })?;

        info!(
            step = "start_services",
            status = "success",
            "Docker Compose services started successfully"
        );

        Ok(())
    }
}

/// Errors that can occur during the start services step
#[derive(Debug, Error)]
pub enum StartServicesStepError {
    /// Ansible playbook execution failed
    #[error("Ansible playbook 'run-compose-services' failed: {message}")]
    AnsiblePlaybookFailed {
        message: String,
        #[source]
        source: CommandError,
    },
}

impl StartServicesStepError {
    /// Returns troubleshooting help for this error
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::AnsiblePlaybookFailed { .. } => {
                "Failed to start Docker Compose services. Please check:\n\
                 1. Docker daemon is running on the remote host\n\
                 2. Docker Compose files were deployed via 'release' command\n\
                 3. Container images can be pulled (network connectivity)\n\
                 4. No port conflicts with existing services\n\
                 5. Sufficient disk space and memory on the remote host\n\
                 6. SSH connectivity to the remote host is working"
            }
        }
    }
}

impl Traceable for StartServicesStepError {
    fn trace_format(&self) -> String {
        match self {
            Self::AnsiblePlaybookFailed { message, .. } => {
                format!("StartServicesStep::AnsiblePlaybookFailed - {message}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::AnsiblePlaybookFailed { source, .. } => Some(source),
        }
    }

    fn error_kind(&self) -> ErrorKind {
        ErrorKind::InfrastructureOperation
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;
    use crate::adapters::ansible::AnsibleClient;

    #[test]
    fn it_should_create_start_services_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));

        let step = StartServicesStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }

    #[test]
    fn errors_should_provide_help() {
        let cmd_error = CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        };
        let error = StartServicesStepError::AnsiblePlaybookFailed {
            message: "test".to_string(),
            source: cmd_error,
        };

        let help = error.help();
        assert!(help.contains("Docker daemon"));
        assert!(help.contains("release"));
        assert!(help.contains("port conflicts"));
    }

    #[test]
    fn errors_should_implement_traceable() {
        let cmd_error = CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        };
        let error = StartServicesStepError::AnsiblePlaybookFailed {
            message: "test error".to_string(),
            source: cmd_error,
        };

        assert!(error.trace_format().contains("AnsiblePlaybookFailed"));
        assert!(error.trace_source().is_some());
        assert!(matches!(
            error.error_kind(),
            ErrorKind::InfrastructureOperation
        ));
    }
}
