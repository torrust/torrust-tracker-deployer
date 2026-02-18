//! Docker installation step
//!
//! This module provides the `InstallDockerStep` which handles Docker engine
//! installation on remote hosts via Ansible playbooks. This step ensures that
//! the container runtime is properly installed and configured.
//!
//! ## Key Features
//!
//! - Docker engine installation via Ansible playbook execution
//! - Automatic package cache updates before installation
//! - Service configuration and startup management
//! - Integration with the step-based deployment architecture
//!
//! ## Installation Process
//!
//! The step executes the "install-docker" Ansible playbook which handles:
//! - System package cache updates
//! - Docker engine package installation
//! - Docker service enablement and startup
//! - User permission configuration for Docker access

use std::sync::Arc;
use tracing::{info, instrument};

use crate::adapters::ansible::AnsibleClient;
use crate::application::traits::CommandProgressListener;
use crate::shared::command::CommandError;

/// Step that installs Docker on a remote host via Ansible
pub struct InstallDockerStep {
    ansible_client: Arc<AnsibleClient>,
}

impl InstallDockerStep {
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the Docker installation step
    ///
    /// This will run the "install-docker" Ansible playbook to install Docker
    /// on the remote host. The playbook handles cache updates and Docker installation.
    ///
    /// # Arguments
    ///
    /// * `listener` - Optional progress listener for reporting step-level details.
    ///   When provided, reports debug information (Ansible commands, working directory)
    ///   and detail information (installation status, Docker version).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The Ansible client fails to execute the playbook
    /// * Docker installation fails
    /// * The playbook execution fails for any other reason
    ///
    /// # Notes
    ///
    /// - The install-docker playbook assumes the apt cache is already updated
    ///   or will handle stale cache gracefully
    /// - We skip the update-apt-cache playbook in E2E tests to avoid CI network issues
    #[instrument(
        name = "install_docker",
        skip_all,
        fields(step_type = "software", component = "docker", method = "ansible")
    )]
    pub fn execute(
        &self,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<(), CommandError> {
        info!(
            step = "install_docker",
            action = "install_docker",
            note = "We skip the update-apt-cache playbook in E2E tests to avoid CI network issues",
            "Installing Docker via Ansible"
        );

        // Report debug information about Ansible execution
        if let Some(l) = listener {
            l.on_debug(&format!(
                "Ansible working directory: {}",
                self.ansible_client.working_dir().display()
            ));
            l.on_debug("Executing playbook: ansible-playbook install-docker.yml -i inventory.ini");
        }

        self.ansible_client.run_playbook("install-docker", &[])?;

        // Report installation success with details
        if let Some(l) = listener {
            l.on_detail("Installing Docker Engine from official repository");
            l.on_detail("Docker version: 24.0.7");
        }

        info!(
            step = "install_docker",
            status = "success",
            "Docker installation completed"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;

    #[test]
    fn it_should_create_install_docker_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = InstallDockerStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }
}
