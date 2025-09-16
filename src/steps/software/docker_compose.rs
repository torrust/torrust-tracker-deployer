use std::sync::Arc;
use tracing::info;

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
