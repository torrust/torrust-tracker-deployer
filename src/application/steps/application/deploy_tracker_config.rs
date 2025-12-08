//! Deploy Tracker configuration step
//!
//! This module provides the `DeployTrackerConfigStep` which handles the deployment
//! of Tracker configuration files to a remote host via Ansible.
//!
//! ## Key Features
//!
//! - Deploys tracker.toml configuration file to remote host
//! - Uses Ansible's copy module for reliable file transfer
//! - Verifies successful deployment
//! - Sets correct file permissions and ownership
//!
//! ## Deployment Process
//!
//! The step executes the "deploy-tracker-config" Ansible playbook which:
//! - Copies tracker.toml from the local build directory to the remote host
//! - Places it in `/opt/torrust/storage/tracker/etc/tracker.toml`
//! - Sets appropriate permissions (0644) and ownership
//! - Verifies the file was deployed successfully
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `DeployTrackerConfigStep` handles file deployment
//! - **Remote Action** (Level 3): Ansible playbook executes on the remote host
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use std::path::PathBuf;
//! use crate::adapters::ansible::AnsibleClient;
//! use crate::application::steps::application::DeployTrackerConfigStep;
//!
//! let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("/path/to/ansible/build")));
//! let tracker_build_dir = PathBuf::from("/path/to/tracker/build");
//!
//! let step = DeployTrackerConfigStep::new(ansible_client, tracker_build_dir);
//! step.execute()?;
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;
use crate::shared::{ErrorKind, Traceable};

/// Default remote configuration directory for tracker
pub const DEFAULT_TRACKER_CONFIG_DIR: &str = "/opt/torrust/storage/tracker/etc";

/// Step that deploys Tracker configuration file to a remote host via Ansible
///
/// This step handles the transfer of the tracker.toml configuration file
/// to the remote instance using Ansible's copy module.
pub struct DeployTrackerConfigStep {
    ansible_client: Arc<AnsibleClient>,
    tracker_build_dir: PathBuf,
}

impl DeployTrackerConfigStep {
    /// Creates a new `DeployTrackerConfigStep`
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - The Ansible client for executing playbooks
    /// * `tracker_build_dir` - Local directory containing rendered tracker.toml
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>, tracker_build_dir: PathBuf) -> Self {
        Self {
            ansible_client,
            tracker_build_dir,
        }
    }

    /// Execute the deployment step
    ///
    /// This will run the "deploy-tracker-config" Ansible playbook to copy
    /// the tracker.toml configuration file to the remote host.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The tracker build directory does not exist
    /// * The tracker.toml file does not exist in the build directory
    /// * The Ansible playbook execution fails
    /// * File copying fails
    #[instrument(
        name = "deploy_tracker_config",
        skip_all,
        fields(
            step_type = "application",
            operation = "deploy_tracker_config",
            tracker_build_dir = %self.tracker_build_dir.display()
        )
    )]
    pub fn execute(&self) -> Result<(), DeployTrackerConfigStepError> {
        info!(
            step = "deploy_tracker_config",
            tracker_build_dir = %self.tracker_build_dir.display(),
            "Deploying Tracker configuration to remote host"
        );

        // Validate that the tracker build directory exists
        if !self.tracker_build_dir.exists() {
            return Err(DeployTrackerConfigStepError::TrackerBuildDirNotFound {
                path: self.tracker_build_dir.display().to_string(),
            });
        }

        // Validate that tracker.toml exists
        let tracker_toml = self.tracker_build_dir.join("tracker.toml");
        if !tracker_toml.exists() {
            return Err(DeployTrackerConfigStepError::TrackerConfigNotFound {
                path: tracker_toml.display().to_string(),
            });
        }

        // Execute the Ansible playbook
        // Note: The playbook uses a relative path from playbook_dir to find tracker.toml
        // The Ansible build directory structure is: build/<env>/ansible/
        // The tracker build directory structure is: build/<env>/tracker/
        // So from ansible/ directory, tracker.toml is at: ../tracker/tracker.toml
        self.ansible_client
            .run_playbook("deploy-tracker-config", &[])
            .map_err(
                |source| DeployTrackerConfigStepError::AnsiblePlaybookFailed {
                    message: source.to_string(),
                    source,
                },
            )?;

        info!(
            step = "deploy_tracker_config",
            status = "success",
            "Tracker configuration deployed successfully to {DEFAULT_TRACKER_CONFIG_DIR}/tracker.toml"
        );

        Ok(())
    }
}

/// Errors that can occur during tracker configuration deployment
#[derive(Error, Debug)]
pub enum DeployTrackerConfigStepError {
    /// Tracker build directory not found
    #[error("Tracker build directory not found: {path}")]
    TrackerBuildDirNotFound { path: String },

    /// Tracker configuration file (tracker.toml) not found
    #[error("Tracker configuration file not found: {path}")]
    TrackerConfigNotFound { path: String },

    /// Ansible playbook execution failed
    #[error("Ansible playbook execution failed: {message}")]
    AnsiblePlaybookFailed {
        message: String,
        #[source]
        source: CommandError,
    },
}

impl Traceable for DeployTrackerConfigStepError {
    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::TrackerBuildDirNotFound { .. } | Self::TrackerConfigNotFound { .. } => {
                ErrorKind::Configuration
            }
            Self::AnsiblePlaybookFailed { source, .. } => source.error_kind(),
        }
    }

    fn trace_format(&self) -> String {
        match self {
            Self::TrackerBuildDirNotFound { path } => {
                format!("TrackerBuildDirNotFound {{ path: {path} }}")
            }
            Self::TrackerConfigNotFound { path } => {
                format!("TrackerConfigNotFound {{ path: {path} }}")
            }
            Self::AnsiblePlaybookFailed { message, source } => {
                format!("AnsiblePlaybookFailed {{ message: {message}, source: {source:?} }}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::AnsiblePlaybookFailed { .. } => None, // CommandError doesn't implement Traceable
            _ => None,
        }
    }
}

impl DeployTrackerConfigStepError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue.
    #[must_use]
    pub fn help(&self) -> Option<String> {
        match self {
            Self::TrackerBuildDirNotFound { path } => Some(format!(
                r"Tracker Build Directory Not Found - Troubleshooting:

1. The tracker build directory does not exist at: {path}

2. Ensure the RenderTrackerTemplatesStep ran successfully before this step

3. Check the build directory structure:
   ls -la build/<env-name>/tracker/

4. Verify the tracker template rendering completed:
   cat build/<env-name>/tracker/tracker.toml

5. If the build directory is missing:
   - Re-run the release command
   - Check logs for rendering errors

Common causes:
- Template rendering step was skipped
- Build directory was deleted
- Wrong build directory path configured

For more information, see docs/user-guide/commands.md
"
            )),

            Self::TrackerConfigNotFound { path } => Some(format!(
                r"Tracker Configuration File Not Found - Troubleshooting:

1. The tracker.toml file does not exist at: {path}

2. Ensure the RenderTrackerTemplatesStep completed successfully

3. Check if the file was rendered:
   ls -la build/<env-name>/tracker/
   cat build/<env-name>/tracker/tracker.toml

4. Verify the template file exists:
   ls templates/tracker/tracker.toml.tera

5. Check for rendering errors in the logs

Common causes:
- Template rendering failed silently
- Wrong build directory path
- Template file missing from templates/
- File permissions issue

For more information, see docs/user-guide/commands.md
"
            )),

            Self::AnsiblePlaybookFailed { source, .. } => {
                let base_help = format!(
                    r"Ansible Playbook Failed - Troubleshooting:

1. Check SSH connectivity to the remote host:
   ssh -i <ssh-key> <user>@<host>

2. Verify the Ansible playbook exists:
   ls templates/ansible/deploy-tracker-config.yml

3. Check Ansible execution permissions

4. Verify the tracker storage directories exist:
   ssh <user>@<host> 'ls -la /opt/torrust/storage/tracker/'

Common causes:
- Ansible playbook not found
- SSH connectivity issues
- Remote directory permissions
- Tracker storage not created (run create-tracker-storage.yml first)

Original Ansible error:
{source}

For more information, see docs/user-guide/commands.md
"
                );

                Some(base_help)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn it_should_return_error_when_build_dir_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_build_dir = temp_dir.path().join("build/ansible");
        let tracker_build_dir = temp_dir.path().join("build/tracker");

        fs::create_dir_all(&ansible_build_dir).expect("Failed to create ansible dir");

        let ansible_client = Arc::new(AnsibleClient::new(ansible_build_dir));
        let step = DeployTrackerConfigStep::new(ansible_client, tracker_build_dir.clone());

        let result = step.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            DeployTrackerConfigStepError::TrackerBuildDirNotFound { path } => {
                assert_eq!(path, tracker_build_dir.display().to_string());
            }
            other => panic!("Expected TrackerBuildDirNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_when_tracker_toml_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_build_dir = temp_dir.path().join("build/ansible");
        let tracker_build_dir = temp_dir.path().join("build/tracker");

        fs::create_dir_all(&ansible_build_dir).expect("Failed to create ansible dir");
        fs::create_dir_all(&tracker_build_dir).expect("Failed to create tracker dir");

        let ansible_client = Arc::new(AnsibleClient::new(ansible_build_dir));
        let step = DeployTrackerConfigStep::new(ansible_client, tracker_build_dir.clone());

        let result = step.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            DeployTrackerConfigStepError::TrackerConfigNotFound { path } => {
                assert_eq!(
                    path,
                    tracker_build_dir.join("tracker.toml").display().to_string()
                );
            }
            other => panic!("Expected TrackerConfigNotFound error, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_support_debug_formatting() {
        let error = DeployTrackerConfigStepError::TrackerBuildDirNotFound {
            path: "/path/to/build".to_string(),
        };

        let debug_output = format!("{error:?}");
        assert!(debug_output.contains("TrackerBuildDirNotFound"));
        assert!(debug_output.contains("/path/to/build"));
    }

    #[test]
    fn it_should_provide_help_for_build_dir_not_found() {
        let error = DeployTrackerConfigStepError::TrackerBuildDirNotFound {
            path: "/test/path".to_string(),
        };

        let help = error.help();
        assert!(help.is_some());
        let help_text = help.unwrap();
        assert!(help_text.contains("Tracker Build Directory Not Found"));
        assert!(help_text.contains("/test/path"));
        assert!(help_text.contains("RenderTrackerTemplatesStep"));
    }

    #[test]
    fn it_should_provide_help_for_config_not_found() {
        let error = DeployTrackerConfigStepError::TrackerConfigNotFound {
            path: "/test/tracker.toml".to_string(),
        };

        let help = error.help();
        assert!(help.is_some());
        let help_text = help.unwrap();
        assert!(help_text.contains("Tracker Configuration File Not Found"));
        assert!(help_text.contains("/test/tracker.toml"));
        assert!(help_text.contains("tracker.toml.tera"));
    }
}
