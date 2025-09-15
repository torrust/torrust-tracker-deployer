use std::sync::Arc;
use tracing::info;

use crate::command::CommandError;
use crate::command_wrappers::ansible::AnsibleClient;

/// Step that waits for cloud-init completion on a remote host
pub struct WaitForCloudInitStep {
    ansible_client: Arc<AnsibleClient>,
}

impl WaitForCloudInitStep {
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the cloud-init wait step
    ///
    /// This will run the "wait-cloud-init" Ansible playbook to ensure
    /// cloud-init has completed on the remote host before proceeding.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The Ansible client fails to execute the playbook
    /// * Cloud-init has not completed within the timeout period
    /// * The playbook execution fails for any other reason
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "wait_cloud_init",
            action = "wait_cloud_init",
            "Waiting for cloud-init completion"
        );

        self.ansible_client.run_playbook("wait-cloud-init")?;

        info!(
            step = "wait_cloud_init",
            status = "success",
            "Cloud-init completion confirmed"
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
    fn it_should_create_wait_for_cloud_init_step() {
        let ansible_client = Arc::new(AnsibleClient::new(PathBuf::from("test_inventory.yml")));
        let step = WaitForCloudInitStep::new(ansible_client);

        // Test that the step can be created successfully
        assert_eq!(
            std::mem::size_of_val(&step),
            std::mem::size_of::<Arc<AnsibleClient>>()
        );
    }
}
