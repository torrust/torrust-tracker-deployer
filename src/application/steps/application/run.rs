//! Application run step
//!
//! This module provides the `RunStep` which handles starting the application
//! stack on the remote host. The run step executes Docker Compose to bring
//! up all application services.
//!
//! ## Key Features
//!
//! - Docker Compose stack execution
//! - Service startup management
//! - Container orchestration via Ansible
//! - Integration with the step-based deployment architecture
//!
//! ## Run Process
//!
//! The step handles the run phase which typically includes:
//! - Executing `docker compose up -d` on the remote host
//! - Starting all application services in detached mode
//! - Verifying service startup (future enhancement)
//! - Managing container lifecycle
//!
//! This step is designed to be executed after the release step has
//! deployed all necessary configuration and compose files.

use std::sync::Arc;

use thiserror::Error;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::{ErrorKind, Traceable};

/// Step that runs the application stack on a remote host
///
/// This step handles starting Docker Compose services on the remote instance,
/// bringing up all application containers.
pub struct RunStep {
    ansible_client: Arc<AnsibleClient>,
}

impl RunStep {
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the run step
    ///
    /// This will start the Docker Compose application stack on the remote host.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Docker Compose execution fails
    /// * Service startup fails
    /// * Container creation fails
    #[instrument(
        name = "run_application",
        skip_all,
        fields(step_type = "application", operation = "run")
    )]
    pub fn execute(&self) -> Result<(), RunStepError> {
        info!(
            step = "run_application",
            status = "starting",
            "Starting application stack"
        );

        // TODO: Implement actual run logic
        // This will include:
        // 1. Execute docker compose up -d via Ansible
        // 2. Verify services started successfully
        // 3. Report container status
        let _ = &self.ansible_client; // Suppress unused warning for now

        info!(
            step = "run_application",
            status = "success",
            "Application stack started (placeholder)"
        );

        Ok(())
    }
}

/// Errors that can occur during the run step
#[derive(Debug, Error)]
pub enum RunStepError {
    /// Failed to execute Docker Compose
    #[error("Failed to execute Docker Compose: {message}")]
    DockerComposeExecutionFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Failed to start services
    #[error("Failed to start services: {message}")]
    ServiceStartupFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Failed to create containers
    #[error("Failed to create containers: {message}")]
    ContainerCreationFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl RunStepError {
    /// Returns troubleshooting help for this error
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DockerComposeExecutionFailed { .. } => {
                "Docker Compose execution failed. Please check:\n\
                 1. Docker Compose files are present on the remote host\n\
                 2. Docker Compose syntax is valid\n\
                 3. Docker daemon is running on the remote host\n\
                 4. User has permissions to run Docker commands"
            }
            Self::ServiceStartupFailed { .. } => {
                "Service startup failed. Please check:\n\
                 1. Container images are available or can be pulled\n\
                 2. Port conflicts with existing services\n\
                 3. Volume mounts are accessible\n\
                 4. Environment variables are properly configured"
            }
            Self::ContainerCreationFailed { .. } => {
                "Container creation failed. Please check:\n\
                 1. Docker daemon is running and healthy\n\
                 2. Sufficient disk space for containers\n\
                 3. Network configuration is valid\n\
                 4. Container images exist and are valid"
            }
        }
    }
}

impl Traceable for RunStepError {
    fn trace_format(&self) -> String {
        match self {
            Self::DockerComposeExecutionFailed { message, .. } => {
                format!("RunStep::DockerComposeExecutionFailed - {message}")
            }
            Self::ServiceStartupFailed { message, .. } => {
                format!("RunStep::ServiceStartupFailed - {message}")
            }
            Self::ContainerCreationFailed { message, .. } => {
                format!("RunStep::ContainerCreationFailed - {message}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        // These errors don't wrap Traceable sources
        None
    }

    fn error_kind(&self) -> ErrorKind {
        // All run step errors are infrastructure-related
        ErrorKind::InfrastructureOperation
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;

    #[test]
    fn it_should_create_run_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = RunStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }

    #[test]
    fn it_should_execute_run_step_placeholder() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = RunStep::new(ansible_client);

        // Placeholder should succeed
        let result = step.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn docker_compose_error_should_provide_help() {
        let error = RunStepError::DockerComposeExecutionFailed {
            message: "Command not found".to_string(),
            source: None,
        };

        let help = error.help();
        assert!(help.contains("Docker Compose"));
        assert!(help.contains("Docker daemon"));
    }

    #[test]
    fn service_startup_error_should_provide_help() {
        let error = RunStepError::ServiceStartupFailed {
            message: "Port already in use".to_string(),
            source: None,
        };

        let help = error.help();
        assert!(help.contains("Port conflicts"));
        assert!(help.contains("Container images"));
    }

    #[test]
    fn container_creation_error_should_provide_help() {
        let error = RunStepError::ContainerCreationFailed {
            message: "No space left on device".to_string(),
            source: None,
        };

        let help = error.help();
        assert!(help.contains("disk space"));
        assert!(help.contains("Docker daemon"));
    }

    #[test]
    fn errors_should_implement_traceable() {
        let error = RunStepError::DockerComposeExecutionFailed {
            message: "test error".to_string(),
            source: None,
        };

        assert!(error
            .trace_format()
            .contains("DockerComposeExecutionFailed"));
        assert!(error.trace_source().is_none());
        assert!(matches!(
            error.error_kind(),
            ErrorKind::InfrastructureOperation
        ));
    }
}
