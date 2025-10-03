use std::sync::Arc;

use tracing::{info, instrument};

use crate::application::steps::{InstallDockerComposeStep, InstallDockerStep};
use crate::infrastructure::external_tools::ansible::adapter::AnsibleClient;
use crate::shared::command::CommandError;

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
///
/// # TODO(Phase 5): State Management Integration
///
/// When implementing state management in Phase 5:
/// 1. Add `repository: Arc<dyn EnvironmentRepository>` field
/// 2. Update `new()` to accept repository parameter
/// 3. Change `execute()` to accept `Environment<Provisioned>` instead of no parameters
/// 4. Return `Environment<Configured>` instead of `()`
/// 5. Add state transitions and persistence calls at marked points in `execute()`
pub struct ConfigureCommand {
    ansible_client: Arc<AnsibleClient>,
    // TODO(Phase 5): Add repository field here
    // repository: Arc<dyn EnvironmentRepository>,
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

        // TODO(Phase 5): Transition to Configuring state and persist
        // let environment = environment.start_configuring();
        // self.persist_state(&environment)?;

        InstallDockerStep::new(Arc::clone(&self.ansible_client)).execute()?;

        InstallDockerComposeStep::new(Arc::clone(&self.ansible_client)).execute()?;

        // TODO(Phase 5): Transition to Configured state and persist
        // let configured_env = environment.complete_configuring();
        // self.persist_state(&configured_env)?;
        // return Ok(configured_env);

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
