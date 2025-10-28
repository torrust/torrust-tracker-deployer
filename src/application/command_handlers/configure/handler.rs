//! Configure command handler implementation

use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::ConfigureCommandHandlerError;
use crate::adapters::ansible::AnsibleClient;
use crate::application::command_handlers::common::StepResult;
use crate::application::steps::{InstallDockerComposeStep, InstallDockerStep};
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::state::{ConfigureFailureContext, ConfigureStep};
use crate::domain::environment::{Configured, Configuring, Environment, Provisioned};
use crate::infrastructure::trace::ConfigureTraceWriter;
use crate::shared::error::Traceable;

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
    pub(crate) ansible_client: Arc<AnsibleClient>,
    pub(crate) clock: Arc<dyn crate::shared::Clock>,
    pub(crate) repository: TypedEnvironmentRepository,
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
            repository: TypedEnvironmentRepository::new(repository),
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
        self.repository.save_configuring(&environment)?;

        // Execute configuration steps with explicit step tracking
        match self.execute_configuration_with_tracking(&environment) {
            Ok(configured_env) => {
                // Persist final state
                self.repository.save_configured(&configured_env)?;

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
                self.repository.save_configure_failed(&failed)?;

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
    ) -> StepResult<Environment<Configured>, ConfigureCommandHandlerError, ConfigureStep> {
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
    pub(crate) fn build_failure_context(
        &self,
        environment: &Environment<Configuring>,
        error: &ConfigureCommandHandlerError,
        current_step: ConfigureStep,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> ConfigureFailureContext {
        use crate::application::command_handlers::common::failure_context::build_base_failure_context;

        // Step that failed is directly provided - no reverse engineering needed
        let failed_step = current_step;

        // Get error kind from the error itself (errors are self-describing)
        let error_kind = error.error_kind();

        // Build base failure context using common helper
        let base = build_base_failure_context(&self.clock, started_at, error.to_string());

        // Build handler-specific context
        let mut context = ConfigureFailureContext {
            failed_step,
            error_kind,
            base,
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
