use std::sync::Arc;

use tracing::{info, instrument, warn};

use crate::adapters::ansible::AnsibleClient;
use crate::application::steps::{InstallDockerComposeStep, InstallDockerStep};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::{
    BaseFailureContext, ConfigureFailureContext, ConfigureStep,
};
use crate::domain::environment::{Configured, Configuring, Environment, Provisioned, TraceId};
use crate::infrastructure::trace::ConfigureTraceWriter;
use crate::shared::command::CommandError;
use crate::shared::error::Traceable;

/// Comprehensive error type for the `ConfigureCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum ConfigureCommandHandlerError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),
}

impl crate::shared::Traceable for ConfigureCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::Command(e) => {
                format!("ConfigureCommandHandlerError: Command execution failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("ConfigureCommandHandlerError: Failed to persist environment state - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::Command(e) => Some(e),
            Self::StatePersistence(_) => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StatePersistence(_) => crate::shared::ErrorKind::StatePersistence,
        }
    }
}

/// `ConfigureCommandHandler` orchestrates the complete infrastructure configuration workflow
///
/// The `ConfigureCommandHandler` orchestrates the complete infrastructure configuration workflow.
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
pub struct ConfigureCommandHandler {
    ansible_client: Arc<AnsibleClient>,
    clock: Arc<dyn crate::shared::Clock>,
    repository: Arc<dyn EnvironmentRepository>,
}

impl ConfigureCommandHandler {
    /// Create a new `ConfigureCommandHandler`
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
    ) -> Result<Environment<Configured>, ConfigureCommandHandlerError> {
        info!(
            command = "configure",
            environment = %environment.name(),
            "Starting complete infrastructure configuration workflow"
        );

        // Capture start time before transitioning to Configuring state
        let started_at = self.clock.now();

        // Transition to Configuring state
        let environment = environment.start_configuring();

        // Persist intermediate state
        self.repository.save(&environment.clone().into_any())?;

        // Execute configuration steps with explicit step tracking
        match self.execute_configuration_with_tracking(&environment) {
            Ok(configured_env) => {
                // Persist final state
                self.repository.save(&configured_env.clone().into_any())?;

                info!(
                    command = "configure",
                    environment = %configured_env.name(),
                    "Infrastructure configuration completed successfully"
                );

                Ok(configured_env)
            }
            Err((e, current_step)) => {
                // Transition to error state with structured context
                // current_step contains the step that was executing when the error occurred
                let context =
                    self.build_failure_context(&environment, &e, current_step, started_at);
                let failed = environment.configure_failed(context);

                // Persist error state
                self.repository.save(&failed.clone().into_any())?;

                Err(e)
            }
        }
    }

    /// Execute the configuration steps with step tracking
    ///
    /// This method executes all configuration steps while tracking which step is currently
    /// being executed. If an error occurs, it returns both the error and the step that
    /// was being executed, enabling accurate failure context generation.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any configuration step fails
    fn execute_configuration_with_tracking(
        &self,
        environment: &Environment<Configuring>,
    ) -> Result<Environment<Configured>, (ConfigureCommandHandlerError, ConfigureStep)> {
        // Track current step and execute each step
        // If an error occurs, we return it along with the current step

        let current_step = ConfigureStep::InstallDocker;
        InstallDockerStep::new(Arc::clone(&self.ansible_client))
            .execute()
            .map_err(|e| (e.into(), current_step))?;

        let current_step = ConfigureStep::InstallDockerCompose;
        InstallDockerComposeStep::new(Arc::clone(&self.ansible_client))
            .execute()
            .map_err(|e| (e.into(), current_step))?;

        // Transition to Configured state
        let configured = environment.clone().configured();

        Ok(configured)
    }

    /// Persist configuring state
    /// Build failure context for a configuration error and generate trace file
    ///
    /// This helper method builds structured error context including the failed step,
    /// error classification, timing information, and generates a trace file for
    /// post-mortem analysis.
    ///
    /// The trace file is written to `{environment.data_dir()}/traces/{trace_id}.txt`
    /// and contains a formatted representation of the entire error chain.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment being configured (for trace directory path)
    /// * `error` - The configuration error that occurred
    /// * `current_step` - The step that was executing when the error occurred
    /// * `started_at` - The timestamp when configuration execution started
    ///
    /// # Returns
    ///
    /// A structured `ConfigureFailureContext` with timing, error details, and trace file path
    fn build_failure_context(
        &self,
        environment: &Environment<Configuring>,
        error: &ConfigureCommandHandlerError,
        current_step: ConfigureStep,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> ConfigureFailureContext {
        // Step that failed is directly provided - no reverse engineering needed
        let failed_step = current_step;

        // Get error kind from the error itself (errors are self-describing)
        let error_kind = error.error_kind();

        let now = self.clock.now();
        let trace_id = TraceId::new();

        // Calculate actual execution duration
        let execution_duration = now
            .signed_duration_since(started_at)
            .to_std()
            .unwrap_or_default();

        let mut context = ConfigureFailureContext {
            failed_step,
            error_kind,
            base: BaseFailureContext {
                error_summary: error.to_string(),
                failed_at: now,
                execution_started_at: started_at,
                execution_duration,
                trace_id,
                trace_file_path: None,
            },
        };

        // Generate trace file (logging handled by trace writer)
        let traces_dir = environment.traces_dir();
        let trace_writer = ConfigureTraceWriter::new(traces_dir, Arc::clone(&self.clock));

        if let Ok(trace_file_path) = trace_writer.write_trace(&context, error) {
            context.base.trace_file_path = Some(trace_file_path);
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    /// Test builder for `ConfigureCommand` that manages dependencies and lifecycle
    ///
    /// This builder simplifies test setup by:
    /// - Managing `TempDir` lifecycle
    /// - Providing sensible defaults for all dependencies
    /// - Returning only the command and necessary test artifacts
    pub struct ConfigureCommandHandlerTestBuilder {
        temp_dir: TempDir,
    }

    impl ConfigureCommandHandlerTestBuilder {
        /// Create a new test builder with default configuration
        pub fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            Self { temp_dir }
        }

        /// Build the `ConfigureCommandHandler` with all dependencies
        ///
        /// Returns: (`command`, `temp_dir`)
        /// The `temp_dir` must be kept alive for the duration of the test.
        pub fn build(self) -> (ConfigureCommandHandler, TempDir) {
            let ansible_client = Arc::new(AnsibleClient::new(self.temp_dir.path()));
            let clock: Arc<dyn crate::shared::Clock> = Arc::new(crate::shared::SystemClock);

            let repository_factory =
                crate::infrastructure::persistence::repository_factory::RepositoryFactory::new(
                    std::time::Duration::from_secs(30),
                );
            let repository = repository_factory.create(self.temp_dir.path().to_path_buf());

            let command_handler = ConfigureCommandHandler::new(ansible_client, clock, repository);

            (command_handler, self.temp_dir)
        }
    }

    #[test]
    fn it_should_create_configure_command_handler_with_all_dependencies() {
        let (command, _temp_dir) = ConfigureCommandHandlerTestBuilder::new().build();

        // Verify the command was created (basic structure test)
        // This test just verifies that the command can be created with the dependencies
        assert_eq!(Arc::strong_count(&command_handler.ansible_client), 1);
    }

    #[test]
    fn it_should_have_correct_error_type_conversions() {
        // Test that all error types can convert to ConfigureCommandHandlerError
        let command_error = CommandError::StartupFailed {
            command: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        };
        let configure_error: ConfigureCommandHandlerError = command_error.into();
        drop(configure_error);
    }

    #[test]
    fn it_should_build_failure_context_from_command_error() {
        use chrono::{TimeZone, Utc};

        let (command, temp_dir) = ConfigureCommandHandlerTestBuilder::new().build();

        // Create test environment for trace generation
        let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

        let error = ConfigureCommandHandlerError::Command(CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        });

        let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let current_step = ConfigureStep::InstallDocker;
        let context = command.build_failure_context(&environment, &error, current_step, started_at);
        assert_eq!(context.failed_step, ConfigureStep::InstallDocker);
        assert_eq!(
            context.error_kind,
            crate::shared::ErrorKind::CommandExecution
        );
        assert_eq!(context.base.execution_started_at, started_at);
    }
}
