//! Application release step
//!
//! This module provides the `ReleaseStep` which handles the release phase
//! of application deployment. The release step transfers configuration files,
//! Docker Compose manifests, and other assets to the remote host.
//!
//! ## Key Features
//!
//! - File transfer to remote hosts
//! - Docker Compose configuration deployment
//! - Application configuration management
//! - Integration with the step-based deployment architecture
//!
//! ## Release Process
//!
//! The step handles the release phase which typically includes:
//! - Transferring Docker Compose files to the remote host
//! - Deploying application configuration files
//! - Setting up necessary directories and permissions
//! - Preparing the environment for the run phase
//!
//! This step is designed to be executed after infrastructure provisioning
//! and software installation, but before the run step.

use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::{info, instrument};

use crate::domain::template::TemplateManager;
use crate::infrastructure::external_tools::docker_compose::{
    DockerComposeTemplateError, DockerComposeTemplateRenderer,
};
use crate::shared::{ErrorKind, Traceable};

/// Step that releases application files to a remote host
///
/// This step handles the deployment of Docker Compose files and application
/// configuration to the remote instance, preparing it for the run phase.
pub struct ReleaseStep {
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl ReleaseStep {
    #[must_use]
    pub fn new(templates_dir: PathBuf, build_dir: PathBuf) -> Self {
        let template_manager = Arc::new(TemplateManager::new(templates_dir));
        Self {
            template_manager,
            build_dir,
        }
    }

    /// Execute the release step
    ///
    /// This will prepare Docker Compose files in the build directory,
    /// ready for transfer to the remote host.
    ///
    /// # Returns
    ///
    /// Returns the path to the docker-compose build directory on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Docker Compose file preparation fails
    /// * Directory creation fails
    /// * File copying fails
    #[instrument(
        name = "release_application",
        skip_all,
        fields(step_type = "application", operation = "release")
    )]
    pub async fn execute(&self) -> Result<PathBuf, ReleaseStepError> {
        info!(
            step = "release_application",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            status = "starting",
            "Starting application release"
        );

        // Prepare Docker Compose files in build directory
        let renderer =
            DockerComposeTemplateRenderer::new(self.template_manager.clone(), &self.build_dir);

        let compose_build_dir = renderer.render().await.map_err(|source| {
            ReleaseStepError::DockerComposePrepFailed {
                message: source.to_string(),
                source,
            }
        })?;

        info!(
            step = "release_application",
            compose_build_dir = %compose_build_dir.display(),
            status = "success",
            "Docker Compose files prepared in build directory"
        );

        // TODO: Phase 8 will add file transfer to remote host here

        info!(
            step = "release_application",
            status = "success",
            "Application release completed"
        );

        Ok(compose_build_dir)
    }
}

/// Errors that can occur during the release step
#[derive(Debug, Error)]
pub enum ReleaseStepError {
    /// Failed to prepare Docker Compose files
    #[error("Failed to prepare Docker Compose files: {message}")]
    DockerComposePrepFailed {
        message: String,
        #[source]
        source: DockerComposeTemplateError,
    },

    /// Failed to transfer files to remote host
    #[error("Failed to transfer files to remote host: {message}")]
    FileTransferFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Failed to deploy configuration
    #[error("Failed to deploy configuration: {message}")]
    ConfigurationDeploymentFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Failed to set up remote directories
    #[error("Failed to set up remote directories: {message}")]
    DirectorySetupFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl ReleaseStepError {
    /// Returns troubleshooting help for this error
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DockerComposePrepFailed { source, .. } => source.help(),
            Self::FileTransferFailed { .. } => {
                "File transfer failed. Please check:\n\
                 1. SSH connectivity to the remote host\n\
                 2. Network connectivity and firewall rules\n\
                 3. Remote host disk space availability\n\
                 4. File permissions on source and destination"
            }
            Self::ConfigurationDeploymentFailed { .. } => {
                "Configuration deployment failed. Please check:\n\
                 1. Configuration files exist and are valid\n\
                 2. Remote host has necessary permissions\n\
                 3. Target directories exist or can be created"
            }
            Self::DirectorySetupFailed { .. } => {
                "Directory setup failed. Please check:\n\
                 1. Remote host disk space availability\n\
                 2. User permissions on the remote host\n\
                 3. Parent directories exist"
            }
        }
    }
}

impl Traceable for ReleaseStepError {
    fn trace_format(&self) -> String {
        match self {
            Self::DockerComposePrepFailed { message, .. } => {
                format!("ReleaseStep::DockerComposePrepFailed - {message}")
            }
            Self::FileTransferFailed { message, .. } => {
                format!("ReleaseStep::FileTransferFailed - {message}")
            }
            Self::ConfigurationDeploymentFailed { message, .. } => {
                format!("ReleaseStep::ConfigurationDeploymentFailed - {message}")
            }
            Self::DirectorySetupFailed { message, .. } => {
                format!("ReleaseStep::DirectorySetupFailed - {message}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::DockerComposePrepFailed { source, .. } => Some(source),
            _ => None,
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::DockerComposePrepFailed { source, .. } => source.error_kind(),
            Self::FileTransferFailed { .. } | Self::DirectorySetupFailed { .. } => {
                ErrorKind::InfrastructureOperation
            }
            Self::ConfigurationDeploymentFailed { .. } => ErrorKind::Configuration,
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::infrastructure::external_tools::docker_compose::DOCKER_COMPOSE_SUBFOLDER;

    #[tokio::test]
    async fn it_should_create_release_step() {
        let templates_dir = PathBuf::from("/templates");
        let build_dir = PathBuf::from("/build");
        let step = ReleaseStep::new(templates_dir.clone(), build_dir.clone());

        // The template_manager wraps the templates_dir internally
        assert_eq!(
            step.template_manager.templates_dir(),
            templates_dir.as_path()
        );
        assert_eq!(step.build_dir, build_dir);
    }

    #[tokio::test]
    async fn it_should_execute_release_step_with_embedded_templates() {
        // Use a temp dir as the templates directory - templates will be extracted from embedded
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let step = ReleaseStep::new(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
        );

        let result = step.execute().await;

        assert!(result.is_ok());
        let compose_build_dir = result.unwrap();
        assert!(compose_build_dir.join("docker-compose.yml").exists());

        // Verify the extracted template exists in templates_dir
        let extracted_template = templates_dir
            .path()
            .join(DOCKER_COMPOSE_SUBFOLDER)
            .join("docker-compose.yml");
        assert!(extracted_template.exists());
    }

    #[tokio::test]
    async fn it_should_copy_correct_content_from_embedded_templates() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let step = ReleaseStep::new(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
        );

        let result = step.execute().await;
        assert!(result.is_ok());

        // Read the output file
        let output_content = tokio::fs::read_to_string(
            build_dir
                .path()
                .join(DOCKER_COMPOSE_SUBFOLDER)
                .join("docker-compose.yml"),
        )
        .await
        .expect("Failed to read output");

        // Verify it contains expected content from embedded template
        assert!(output_content.contains("nginx:alpine"));
        assert!(output_content.contains("demo-app"));
    }

    #[test]
    fn file_transfer_error_should_provide_help() {
        let error = ReleaseStepError::FileTransferFailed {
            message: "Connection refused".to_string(),
            source: None,
        };

        let help = error.help();
        assert!(help.contains("SSH connectivity"));
        assert!(help.contains("Network connectivity"));
    }

    #[test]
    fn configuration_deployment_error_should_provide_help() {
        let error = ReleaseStepError::ConfigurationDeploymentFailed {
            message: "Permission denied".to_string(),
            source: None,
        };

        let help = error.help();
        assert!(help.contains("Configuration files"));
        assert!(help.contains("permissions"));
    }

    #[test]
    fn directory_setup_error_should_provide_help() {
        let error = ReleaseStepError::DirectorySetupFailed {
            message: "No space left on device".to_string(),
            source: None,
        };

        let help = error.help();
        assert!(help.contains("disk space"));
        assert!(help.contains("permissions"));
    }

    #[test]
    fn errors_should_implement_traceable() {
        let error = ReleaseStepError::FileTransferFailed {
            message: "test error".to_string(),
            source: None,
        };

        assert!(error.trace_format().contains("FileTransferFailed"));
        assert!(error.trace_source().is_none());
        assert!(matches!(
            error.error_kind(),
            ErrorKind::InfrastructureOperation
        ));
    }
}
