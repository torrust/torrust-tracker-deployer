use std::net::IpAddr;
use tracing::{error, info};

use crate::command::CommandError;
use crate::command_wrappers::ssh::credentials::SshCredentials;
use crate::remote_actions::RemoteActionError;
use crate::steps::{
    ValidateCloudInitCompletionStep, ValidateDockerComposeInstallationStep,
    ValidateDockerInstallationStep,
};

/// Comprehensive error type for the `TestCommand`
#[derive(Debug, thiserror::Error)]
pub enum TestCommandError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Remote action failed: {0}")]
    RemoteAction(#[from] RemoteActionError),
}

/// `TestCommand` orchestrates the complete infrastructure testing and validation workflow
///
/// The `TestCommand` orchestrates the complete infrastructure testing and validation workflow.
///
/// This command handles all steps required to validate infrastructure:
/// 1. Validate cloud-init completion
/// 2. Validate Docker installation
/// 3. Validate Docker Compose installation
pub struct TestCommand {
    ssh_credentials: SshCredentials,
    instance_ip: IpAddr,
}

impl TestCommand {
    /// Create a new `TestCommand`
    #[must_use]
    pub fn new(ssh_credentials: SshCredentials, instance_ip: IpAddr) -> Self {
        Self {
            ssh_credentials,
            instance_ip,
        }
    }

    /// Execute the complete testing and validation workflow
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the validation workflow fails:
    /// * Cloud-init completion validation fails
    /// * Docker installation validation fails
    /// * Docker Compose installation validation fails
    pub async fn execute(&self) -> Result<(), TestCommandError> {
        info!(
            command = "test",
            stage = "starting",
            instance_ip = %self.instance_ip,
            "Starting complete infrastructure testing workflow"
        );

        let ssh_connection = self.ssh_credentials.clone().with_host(self.instance_ip);

        ValidateCloudInitCompletionStep::new(ssh_connection.clone())
            .execute()
            .await?;

        ValidateDockerInstallationStep::new(ssh_connection.clone())
            .execute()
            .await?;

        ValidateDockerComposeInstallationStep::new(ssh_connection)
            .execute()
            .await?;

        info!(
            command = "test",
            stage = "completed",
            instance_ip = %self.instance_ip,
            "Infrastructure testing workflow completed successfully"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use tempfile::TempDir;

    // Helper function to create mock dependencies for testing
    fn create_mock_dependencies() -> (SshCredentials, IpAddr, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ssh_key_path = temp_dir.path().join("test_key");
        let ssh_pub_key_path = temp_dir.path().join("test_key.pub");
        let ssh_credentials =
            SshCredentials::new(ssh_key_path, ssh_pub_key_path, "test_user".to_string());
        let instance_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));

        (ssh_credentials, instance_ip, temp_dir)
    }

    #[test]
    fn it_should_create_test_command_with_ssh_credentials_and_ip() {
        let (ssh_credentials, instance_ip, _temp_dir) = create_mock_dependencies();

        let command = TestCommand::new(ssh_credentials.clone(), instance_ip);

        // Verify the command was created (basic structure test)
        // This test just verifies that the command can be created with the dependencies
        assert_eq!(command.instance_ip, instance_ip);
        assert_eq!(
            command.ssh_credentials.ssh_username,
            ssh_credentials.ssh_username
        );
    }

    #[test]
    fn it_should_have_correct_error_type_conversions() {
        // Test that all error types can convert to TestCommandError
        let command_error = CommandError::StartupFailed {
            command: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        };
        let test_error: TestCommandError = command_error.into();
        drop(test_error);

        let remote_action_error = RemoteActionError::ValidationFailed {
            action_name: "test".to_string(),
            message: "test error".to_string(),
        };
        let test_error: TestCommandError = remote_action_error.into();
        drop(test_error);
    }
}
