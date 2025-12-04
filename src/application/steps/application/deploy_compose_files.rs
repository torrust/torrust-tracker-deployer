//! Deploy Docker Compose files step
//!
//! This module provides the `DeployComposeFilesStep` which handles the deployment
//! of Docker Compose files and related assets to a remote host via Ansible.
//!
//! ## Key Features
//!
//! - Synchronizes local docker-compose build folder to remote host
//! - Uses Ansible's synchronize module for reliable file transfer
//! - Creates necessary directories on the remote host
//! - Verifies successful deployment
//!
//! ## Deployment Process
//!
//! The step executes the "deploy-compose-files" Ansible playbook which:
//! - Creates the `/opt/torrust` directory on the remote host
//! - Synchronizes all files from the local compose build directory
//! - Verifies that `docker-compose.yml` exists on the remote host
//! - Reports the list of deployed files
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `DeployComposeFilesStep` handles file deployment
//! - **Remote Action** (Level 3): Ansible playbook executes on the remote host
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use std::path::PathBuf;
//! use crate::adapters::ansible::AnsibleClient;
//! use crate::application::steps::application::DeployComposeFilesStep;
//!
//! let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("/path/to/ansible/build")));
//! let compose_build_dir = PathBuf::from("/path/to/compose/build");
//!
//! let step = DeployComposeFilesStep::new(ansible_client, compose_build_dir);
//! step.execute()?;
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;
use crate::shared::{ErrorKind, Traceable};

/// Default deployment directory on the remote host
pub const DEFAULT_REMOTE_DEPLOY_DIR: &str = "/opt/torrust";

/// Step that deploys Docker Compose files to a remote host via Ansible
///
/// This step handles the transfer of Docker Compose files and related assets
/// to the remote instance using Ansible's synchronize module, which provides
/// reliable rsync-based file synchronization.
pub struct DeployComposeFilesStep {
    ansible_client: Arc<AnsibleClient>,
    compose_build_dir: PathBuf,
    remote_deploy_dir: String,
}

impl DeployComposeFilesStep {
    /// Creates a new `DeployComposeFilesStep`
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - The Ansible client for executing playbooks
    /// * `compose_build_dir` - Local directory containing Docker Compose files to deploy
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>, compose_build_dir: PathBuf) -> Self {
        Self {
            ansible_client,
            compose_build_dir,
            remote_deploy_dir: DEFAULT_REMOTE_DEPLOY_DIR.to_string(),
        }
    }

    /// Sets a custom remote deployment directory
    ///
    /// By default, files are deployed to `/opt/torrust` on the remote host.
    #[must_use]
    pub fn with_remote_deploy_dir(mut self, dir: impl Into<String>) -> Self {
        self.remote_deploy_dir = dir.into();
        self
    }

    /// Execute the deployment step
    ///
    /// This will run the "deploy-compose-files" Ansible playbook to synchronize
    /// the local compose build directory to the remote host.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The compose build directory does not exist
    /// * The Ansible playbook execution fails
    /// * File synchronization fails
    #[instrument(
        name = "deploy_compose_files",
        skip_all,
        fields(
            step_type = "application",
            operation = "deploy_compose_files",
            compose_build_dir = %self.compose_build_dir.display(),
            remote_deploy_dir = %self.remote_deploy_dir
        )
    )]
    pub fn execute(&self) -> Result<(), DeployComposeFilesStepError> {
        info!(
            step = "deploy_compose_files",
            compose_build_dir = %self.compose_build_dir.display(),
            remote_deploy_dir = %self.remote_deploy_dir,
            "Deploying Docker Compose files to remote host"
        );

        // Validate that the compose build directory exists
        if !self.compose_build_dir.exists() {
            return Err(DeployComposeFilesStepError::ComposeBuildDirNotFound {
                path: self.compose_build_dir.display().to_string(),
            });
        }

        // Canonicalize the path to get an absolute path that Ansible can resolve correctly.
        // The copy module needs an absolute path because it resolves relative paths
        // from the playbook's directory, not the current working directory.
        let absolute_compose_dir = self.compose_build_dir.canonicalize().map_err(|_| {
            DeployComposeFilesStepError::ComposeBuildDirNotFound {
                path: self.compose_build_dir.display().to_string(),
            }
        })?;

        // Execute the Ansible playbook with extra variables
        let compose_dir_str = absolute_compose_dir.display().to_string();
        let extra_var = format!("compose_files_source_dir={compose_dir_str}");

        self.ansible_client
            .run_playbook("deploy-compose-files", &["-e", &extra_var])
            .map_err(
                |source| DeployComposeFilesStepError::AnsiblePlaybookFailed {
                    message: source.to_string(),
                    source,
                },
            )?;

        info!(
            step = "deploy_compose_files",
            status = "success",
            remote_deploy_dir = %self.remote_deploy_dir,
            "Docker Compose files deployed successfully"
        );

        Ok(())
    }
}

/// Errors that can occur during the deploy compose files step
#[derive(Debug, Error)]
pub enum DeployComposeFilesStepError {
    /// The compose build directory does not exist
    #[error("Compose build directory not found: '{path}'")]
    ComposeBuildDirNotFound { path: String },

    /// Ansible playbook execution failed
    #[error("Ansible playbook 'deploy-compose-files' failed: {message}")]
    AnsiblePlaybookFailed {
        message: String,
        #[source]
        source: CommandError,
    },
}

impl DeployComposeFilesStepError {
    /// Returns troubleshooting help for this error
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::ComposeBuildDirNotFound { .. } => {
                "The Docker Compose build directory was not found. Please check:\n\
                 1. The release step was executed before this step\n\
                 2. Docker Compose templates were rendered successfully\n\
                 3. The build directory path is correct"
            }
            Self::AnsiblePlaybookFailed { .. } => {
                "Ansible playbook execution failed. Please check:\n\
                 1. SSH connectivity to the remote host\n\
                 2. Ansible is properly configured with valid inventory\n\
                 3. The remote host has sufficient disk space\n\
                 4. File permissions allow the deployment"
            }
        }
    }
}

impl Traceable for DeployComposeFilesStepError {
    fn trace_format(&self) -> String {
        match self {
            Self::ComposeBuildDirNotFound { path } => {
                format!("DeployComposeFilesStep::ComposeBuildDirNotFound - {path}")
            }
            Self::AnsiblePlaybookFailed { message, .. } => {
                format!("DeployComposeFilesStep::AnsiblePlaybookFailed - {message}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::AnsiblePlaybookFailed { source, .. } => Some(source),
            Self::ComposeBuildDirNotFound { .. } => None,
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::ComposeBuildDirNotFound { .. } => ErrorKind::Configuration,
            Self::AnsiblePlaybookFailed { .. } => ErrorKind::InfrastructureOperation,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;
    use crate::adapters::ansible::AnsibleClient;

    #[test]
    fn it_should_create_deploy_compose_files_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let compose_build_dir = PathBuf::from("/tmp/compose");

        let step = DeployComposeFilesStep::new(ansible_client, compose_build_dir.clone());

        assert_eq!(step.compose_build_dir, compose_build_dir);
        assert_eq!(step.remote_deploy_dir, DEFAULT_REMOTE_DEPLOY_DIR);
    }

    #[test]
    fn it_should_allow_custom_remote_deploy_dir() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let compose_build_dir = PathBuf::from("/tmp/compose");

        let step = DeployComposeFilesStep::new(ansible_client, compose_build_dir)
            .with_remote_deploy_dir("/custom/path");

        assert_eq!(step.remote_deploy_dir, "/custom/path");
    }

    #[test]
    fn it_should_fail_when_compose_build_dir_not_found() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let compose_build_dir = PathBuf::from("/nonexistent/path");

        let step = DeployComposeFilesStep::new(ansible_client, compose_build_dir);
        let result = step.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            DeployComposeFilesStepError::ComposeBuildDirNotFound { path } => {
                assert!(path.contains("nonexistent"));
            }
            DeployComposeFilesStepError::AnsiblePlaybookFailed { message, .. } => {
                panic!("Expected ComposeBuildDirNotFound, got AnsiblePlaybookFailed: {message}")
            }
        }
    }

    #[test]
    fn errors_should_provide_help() {
        let error = DeployComposeFilesStepError::ComposeBuildDirNotFound {
            path: "/tmp/missing".to_string(),
        };
        assert!(error.help().contains("Docker Compose build directory"));

        let cmd_error = CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        };
        let error = DeployComposeFilesStepError::AnsiblePlaybookFailed {
            message: "test".to_string(),
            source: cmd_error,
        };
        assert!(error.help().contains("Ansible playbook"));
    }

    #[test]
    fn errors_should_implement_traceable() {
        let error = DeployComposeFilesStepError::ComposeBuildDirNotFound {
            path: "/tmp/test".to_string(),
        };

        assert!(error.trace_format().contains("ComposeBuildDirNotFound"));
        assert!(error.trace_source().is_none());
        assert!(matches!(error.error_kind(), ErrorKind::Configuration));
    }
}
