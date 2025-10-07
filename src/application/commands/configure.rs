use std::sync::Arc;

use tracing::{info, instrument, warn};

use crate::application::steps::{InstallDockerComposeStep, InstallDockerStep};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::{
    BaseFailureContext, ConfigureErrorKind, ConfigureFailureContext, ConfigureStep,
};
use crate::domain::environment::{
    ConfigureFailed, Configured, Configuring, Environment, Provisioned, TraceId,
};
use crate::infrastructure::external_tools::ansible::adapter::AnsibleClient;
use crate::infrastructure::trace::ConfigureTraceWriter;
use crate::shared::command::CommandError;

/// Comprehensive error type for the `ConfigureCommand`
#[derive(Debug, thiserror::Error)]
pub enum ConfigureCommandError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),
}

impl crate::shared::Traceable for ConfigureCommandError {
    fn trace_format(&self) -> String {
        match self {
            Self::Command(e) => {
                format!("ConfigureCommandError: Command execution failed - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::Command(e) => Some(e),
        }
    }
}

/// `ConfigureCommand` orchestrates the complete infrastructure configuration workflow
///
/// The `ConfigureCommand` orchestrates the complete infrastructure configuration workflow.
///
/// This command handles all steps required to configure infrastructure:
/// 1. Install Docker
/// 2. Install Docker Compose
///
/// # State Management
///
/// The command integrates with the type-state pattern for environment lifecycle:
/// - Accepts `Environment<Provisioned>` as input
/// - Transitions to `Environment<Configuring>` at start
/// - Returns `Environment<Configured>` on success
/// - Transitions to `Environment<ConfigureFailed>` on error
///
/// State is persisted after each transition using the injected repository.
/// Persistence failures are logged but don't fail the command (state remains valid in memory).
pub struct ConfigureCommand {
    ansible_client: Arc<AnsibleClient>,
    clock: Arc<dyn crate::shared::Clock>,
    repository: Arc<dyn EnvironmentRepository>,
}

impl ConfigureCommand {
    /// Create a new `ConfigureCommand`
    #[must_use]
    pub fn new(
        ansible_client: Arc<AnsibleClient>,
        clock: Arc<dyn crate::shared::Clock>,
        repository: Arc<dyn EnvironmentRepository>,
    ) -> Self {
        Self {
            ansible_client,
            clock,
            repository,
        }
    }

    /// Execute the complete configuration workflow
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in `Provisioned` state to configure
    ///
    /// # Returns
    ///
    /// Returns the configured environment
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the configuration workflow fails:
    /// * Docker installation fails
    /// * Docker Compose installation fails
    ///
    /// On error, the environment transitions to `ConfigureFailed` state and is persisted.
    #[instrument(
        name = "configure_command",
        skip_all,
        fields(
            command_type = "configure",
            environment = %environment.name()
        )
    )]
    pub fn execute(
        &self,
        environment: Environment<Provisioned>,
    ) -> Result<Environment<Configured>, ConfigureCommandError> {
        info!(
            command = "configure",
            environment = %environment.name(),
            "Starting complete infrastructure configuration workflow"
        );

        // Transition to Configuring state
        let environment = environment.start_configuring();

        // Persist intermediate state
        self.persist_configuring_state(&environment);

        // Execute configuration steps
        match self.execute_configuration_steps(&environment) {
            Ok(configured_env) => {
                // Persist final state
                self.persist_configured_state(&configured_env);

                info!(
                    command = "configure",
                    environment = %configured_env.name(),
                    "Infrastructure configuration completed successfully"
                );

                Ok(configured_env)
            }
            Err(e) => {
                // Transition to error state with structured context
                let context = self.build_failure_context(&environment, &e);
                let failed = environment.configure_failed(context);

                // Persist error state
                self.persist_configure_failed_state(&failed);

                Err(e)
            }
        }
    }

    /// Execute the configuration steps on an environment in `Configuring` state
    ///
    /// This internal method performs all the configuration work without state management.
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration step fails
    fn execute_configuration_steps(
        &self,
        environment: &Environment<Configuring>,
    ) -> Result<Environment<Configured>, ConfigureCommandError> {
        InstallDockerStep::new(Arc::clone(&self.ansible_client)).execute()?;

        InstallDockerComposeStep::new(Arc::clone(&self.ansible_client)).execute()?;

        // Transition to Configured state
        let configured = environment.clone().configured();

        Ok(configured)
    }

    /// Persist configuring state
    fn persist_configuring_state(&self, environment: &Environment<Configuring>) {
        let any_state = environment.clone().into_any();

        if let Err(e) = self.repository.save(&any_state) {
            warn!(
                environment = %environment.name(),
                error = %e,
                "Failed to persist configuring state. Command execution continues."
            );
        }
    }

    /// Persist configured state
    fn persist_configured_state(&self, environment: &Environment<Configured>) {
        let any_state = environment.clone().into_any();

        if let Err(e) = self.repository.save(&any_state) {
            warn!(
                environment = %environment.name(),
                error = %e,
                "Failed to persist configured state. Command execution continues."
            );
        }
    }

    /// Persist configure failed state
    fn persist_configure_failed_state(&self, environment: &Environment<ConfigureFailed>) {
        let any_state = environment.clone().into_any();

        if let Err(e) = self.repository.save(&any_state) {
            warn!(
                environment = %environment.name(),
                error = %e,
                "Failed to persist configure failed state. Command execution continues."
            );
        }
    }

    /// Extract the failed step name from a configuration error
    ///
    /// This helper method provides context about which step failed during configuration
    /// and generates a detailed trace file with the complete error chain.
    ///
    /// The trace file is written to `{environment.data_dir()}/traces/{trace_id}.txt`
    /// and contains a formatted representation of the entire error chain.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment being configured (for determining trace file location)
    /// * `error` - The configuration error to build context from
    ///
    /// # Returns
    ///
    /// A structured `ConfigureFailureContext` with timing, error details, and trace file path
    #[allow(clippy::unused_self)] // Method kept as instance method for consistency with ProvisionCommand
    fn build_failure_context(
        &self,
        environment: &Environment<Configuring>,
        error: &ConfigureCommandError,
    ) -> ConfigureFailureContext {
        use std::time::Duration;

        let (failed_step, error_kind) = match error {
            ConfigureCommandError::Command(_) => {
                // For now, we can't distinguish between Docker and Docker Compose installation
                // from a CommandError alone. Default to Docker as it's the first step.
                // In the future, we could add more context to CommandError to distinguish.
                (
                    ConfigureStep::InstallDocker,
                    ConfigureErrorKind::InstallationFailed,
                )
            }
        };

        let now = self.clock.now();
        let trace_id = TraceId::new();

        let mut context = ConfigureFailureContext {
            failed_step,
            error_kind,
            base: BaseFailureContext {
                error_summary: error.to_string(),
                failed_at: now,
                execution_started_at: now, // TODO: Track actual start time
                execution_duration: Duration::from_secs(0), // TODO: Calculate actual duration
                trace_id,
                trace_file_path: None,
            },
        };

        // Generate trace file with complete error chain
        let traces_dir = environment.data_dir().join("traces");
        let trace_writer = ConfigureTraceWriter::new(traces_dir);

        match trace_writer.write_trace(&context, error) {
            Ok(trace_file_path) => {
                info!(
                    command = "configure",
                    trace_id = %context.base.trace_id,
                    trace_file = ?trace_file_path,
                    "Trace file generated successfully"
                );
                context.base.trace_file_path = Some(trace_file_path);
            }
            Err(e) => {
                warn!(
                    command = "configure",
                    trace_id = %context.base.trace_id,
                    error = %e,
                    "Failed to generate trace file"
                );
            }
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper function to create mock dependencies for testing
    #[allow(clippy::type_complexity)]
    fn create_mock_dependencies() -> (
        Arc<AnsibleClient>,
        Arc<dyn crate::shared::Clock>,
        Arc<dyn EnvironmentRepository>,
        TempDir,
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path()));

        // Create clock
        let clock: Arc<dyn crate::shared::Clock> = Arc::new(crate::shared::SystemClock);

        // Create repository
        let repository_factory =
            crate::infrastructure::persistence::repository_factory::RepositoryFactory::new(
                std::time::Duration::from_secs(30),
            );
        let repository = repository_factory.create(temp_dir.path().to_path_buf());

        (ansible_client, clock, repository, temp_dir)
    }

    // Helper function to create a test environment in Configuring state
    fn create_test_environment(_temp_dir: &TempDir) -> (Environment<Configuring>, TempDir) {
        use crate::domain::environment::testing::EnvironmentTestBuilder;

        let (env, _data_dir, _build_dir, env_temp_dir) = EnvironmentTestBuilder::new()
            .with_name("test-env")
            .build_with_custom_paths();

        // Environment is created with paths inside env_temp_dir
        // which will be automatically cleaned up when env_temp_dir is dropped

        // Transition Created -> Provisioning -> Provisioned -> Configuring
        (
            env.start_provisioning().provisioned().start_configuring(),
            env_temp_dir,
        )
    }

    #[test]
    fn it_should_create_configure_command_with_all_dependencies() {
        let (ansible_client, clock, repository, _temp_dir) = create_mock_dependencies();

        let command = ConfigureCommand::new(ansible_client, clock, repository);

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

    #[test]
    fn it_should_build_failure_context_from_command_error() {
        let (ansible_client, clock, repository, temp_dir) = create_mock_dependencies();

        let command = ConfigureCommand::new(ansible_client, clock, repository);

        // Create test environment for trace generation
        let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

        let error = ConfigureCommandError::Command(CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        });

        let context = command.build_failure_context(&environment, &error);
        assert_eq!(context.failed_step, ConfigureStep::InstallDocker);
        assert_eq!(context.error_kind, ConfigureErrorKind::InstallationFailed);
    }
}
