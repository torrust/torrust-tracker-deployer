use std::sync::Arc;

use tracing::{info, instrument};

use crate::application::steps::{InstallDockerComposeStep, InstallDockerStep};
use crate::infrastructure::external_tools::ansible::adapter::AnsibleClient;
use crate::shared::executor::CommandError;

/// Comprehensive error type for the `ConfigureCommand`
#[derive(Debug, thiserror::Error)]
pub enum ConfigureCommandError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),
}

/// `ConfigureCommand` orchestrates the complete infrastructure configuration workflow
///
/// The `ConfigureCommand` orchestrates the complete infrastructure configuration workflow.
///
/// This command handles all steps required to configure infrastructure:
/// 1. Install Docker
/// 2. Install Docker Compose
pub struct ConfigureCommand {
    ansible_client: Arc<AnsibleClient>,
}

impl ConfigureCommand {
    /// Create a new `ConfigureCommand`
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the complete configuration workflow
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the configuration workflow fails:
    /// * Docker installation fails
    /// * Docker Compose installation fails
    #[instrument(
        name = "configure_command",
        skip_all,
        fields(command_type = "configure")
    )]
    pub fn execute(&self) -> Result<(), ConfigureCommandError> {
        info!(
            command = "configure",
            "Starting complete infrastructure configuration workflow"
        );

        InstallDockerStep::new(Arc::clone(&self.ansible_client)).execute()?;

        InstallDockerComposeStep::new(Arc::clone(&self.ansible_client)).execute()?;

        info!(
            command = "configure",
            "Infrastructure configuration completed successfully"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper function to create mock dependencies for testing
    fn create_mock_dependencies() -> (Arc<AnsibleClient>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path()));

        (ansible_client, temp_dir)
    }

    #[test]
    fn it_should_create_configure_command_with_ansible_client() {
        let (ansible_client, _temp_dir) = create_mock_dependencies();

        let command = ConfigureCommand::new(ansible_client);

        // Verify the command was created (basic structure test)
        // This test just verifies that the command can be created with the dependencies
        assert_eq!(Arc::strong_count(&command.ansible_client), 1);
    }

    #[test]
    fn it_should_have_correct_error_type_conversions() {
        // Test that all error types can convert to ConfigureCommandError
        let command_error = CommandError::StartupFailed {
            command: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        };
        let configure_error: ConfigureCommandError = command_error.into();
        drop(configure_error);
    }
}
