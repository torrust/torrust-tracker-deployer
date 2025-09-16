//! Docker Compose installation step
//!
//! This module provides the `InstallDockerComposeStep` which handles Docker Compose
//! installation on remote hosts via Ansible playbooks. This step ensures that
//! the container orchestration tool is properly installed and configured.
//!
//! ## Key Features
//!
//! - Docker Compose installation via Ansible playbook execution
//! - Version management and compatibility checking
//! - Integration with existing Docker installations
//! - Integration with the step-based deployment architecture
//!
//! ## Installation Process
//!
//! The step executes the "install-docker-compose" Ansible playbook which handles:
//! - Docker Compose binary download and installation
//! - Executable permissions and path configuration
//! - Version verification and compatibility checking
//!
//! This step typically runs after Docker engine installation to provide
//! complete container orchestration capabilities.

use std::sync::Arc;
use tracing::{info, instrument};

use crate::command::CommandError;
use crate::command_wrappers::ansible::AnsibleClient;

/// Step that installs Docker Compose on a remote host via Ansible
pub struct InstallDockerComposeStep {
    ansible_client: Arc<AnsibleClient>,
}

impl InstallDockerComposeStep {
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the Docker Compose installation step
    ///
    /// This will run the "install-docker-compose" Ansible playbook to install
    /// Docker Compose on the remote host.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The Ansible client fails to execute the playbook
    /// * Docker Compose installation fails
    /// * The playbook execution fails for any other reason
    #[instrument(
        name = "install_docker_compose",
        skip_all,
        fields(
            step_type = "software",
            component = "docker_compose",
            method = "ansible"
        )
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "install_docker_compose",
            action = "install_docker_compose",
            "Installing Docker Compose via Ansible"
        );

        self.ansible_client.run_playbook("install-docker-compose")?;

        info!(
            step = "install_docker_compose",
            status = "success",
            "Docker Compose installation completed"
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
    fn it_should_create_install_docker_compose_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = InstallDockerComposeStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }
}
