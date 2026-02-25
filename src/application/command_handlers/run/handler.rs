//! Run command handler implementation

use std::net::IpAddr;
use std::sync::Arc;

use tracing::{error, info, instrument};

use super::errors::RunCommandHandlerError;
use crate::adapters::ansible::AnsibleClient;
use crate::application::command_handlers::common::StepResult;
use crate::application::steps::application::StartServicesStep;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::runtime_outputs::ServiceEndpoints;
use crate::domain::environment::state::{RunFailureContext, RunStep};
use crate::domain::environment::{Environment, Released, Running};
use crate::domain::EnvironmentName;
use crate::shared::error::Traceable;

/// `RunCommandHandler` orchestrates the stack execution workflow
///
/// The `RunCommandHandler` orchestrates the execution of the deployed software
/// stack on the target environment.
///
/// This command handler handles all steps required to run the stack:
/// 1. Load the environment from storage
/// 2. Validate the environment is in the correct state
/// 3. Start services via Ansible playbook
/// 4. Transition environment to `Running` state
///
/// # Architecture
///
/// Follows the three-level architecture:
/// - **Command** (Level 1): This handler orchestrates the run workflow
/// - **Step** (Level 2): `StartServicesStep`
/// - **Remote Action** (Level 3): Ansible playbook executes on remote host
///
/// # State Management
///
/// The command handler integrates with the type-state pattern for environment lifecycle:
/// - Accepts environment in `Released` state
/// - Transitions to `Environment<Running>` on success
/// - Transitions to `Environment<RunFailed>` on error
///
/// State is persisted after each transition using the injected repository.
pub struct RunCommandHandler {
    pub(crate) clock: Arc<dyn crate::shared::Clock>,
    pub(crate) repository: TypedEnvironmentRepository,
}

impl RunCommandHandler {
    /// Create a new `RunCommandHandler`
    #[must_use]
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn crate::shared::Clock>,
    ) -> Self {
        Self {
            clock,
            repository: TypedEnvironmentRepository::new(repository),
        }
    }

    /// Execute the run workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to run
    ///
    /// # Returns
    ///
    /// Returns `Ok(Environment<Running>)` on success
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found
    /// * Environment is not in `Released` state
    /// * Instance IP is not available
    /// * Starting services fails
    /// * State persistence fails
    #[allow(clippy::result_large_err)]
    #[instrument(
        name = "run_command",
        skip_all,
        fields(
            command_type = "run",
            environment = %env_name
        )
    )]
    pub fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Running>, RunCommandHandlerError> {
        let environment = self.load_released_environment(env_name)?;

        let instance_ip =
            environment
                .instance_ip()
                .ok_or_else(|| RunCommandHandlerError::MissingInstanceIp {
                    name: env_name.to_string(),
                })?;

        let started_at = self.clock.now();

        info!(
            command = "run",
            environment = %env_name,
            instance_ip = %instance_ip,
            current_state = "released",
            target_state = "running",
            "Environment loaded and validated. Executing run steps."
        );

        match self.execute_run_workflow(&environment, instance_ip) {
            Ok(running) => {
                info!(
                    command = "run",
                    environment = %running.name(),
                    final_state = "running",
                    "Stack execution completed successfully"
                );

                self.repository.save_running(&running)?;

                Ok(running)
            }
            Err((e, current_step)) => {
                error!(
                    command = "run",
                    environment = %environment.name(),
                    error = %e,
                    step = ?current_step,
                    "Stack execution failed"
                );

                let context =
                    self.build_failure_context(&environment, &e, current_step, started_at);

                let failed = environment.start_running().run_failed(context);

                self.repository.save_run_failed(&failed)?;

                Err(e)
            }
        }
    }

    /// Execute the run workflow with step tracking
    ///
    /// This method orchestrates the complete run workflow:
    /// 1. Start Docker Compose services on the remote host
    /// 2. Build service endpoints for display
    ///
    /// If an error occurs, it returns both the error and the step that was being
    /// executed, enabling accurate failure context generation.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Released state
    /// * `instance_ip` - The validated instance IP address (precondition checked by caller)
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any run step fails
    #[allow(clippy::result_large_err)]
    fn execute_run_workflow(
        &self,
        environment: &Environment<Released>,
        instance_ip: IpAddr,
    ) -> StepResult<Environment<Running>, RunCommandHandlerError, RunStep> {
        // Step 1: Start Docker Compose services
        self.start_services(environment, instance_ip)?;

        // Build service endpoints from tracker config and instance IP
        let service_endpoints =
            ServiceEndpoints::from_tracker_config(environment.tracker_config(), instance_ip);

        // Transition to running state with service endpoints
        let running = environment
            .clone()
            .start_running_with_endpoints(service_endpoints);

        Ok(running)
    }

    /// Start Docker Compose services on the remote host via Ansible
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `RunStep::StartServices`) if starting services fails
    #[allow(clippy::result_large_err, clippy::unused_self)]
    fn start_services(
        &self,
        environment: &Environment<Released>,
        instance_ip: IpAddr,
    ) -> StepResult<(), RunCommandHandlerError, RunStep> {
        let current_step = RunStep::StartServices;

        let ansible_client = Arc::new(AnsibleClient::new(environment.ansible_build_dir()));
        let step = StartServicesStep::new(ansible_client);

        step.execute().map_err(|e| {
            (
                RunCommandHandlerError::StartServicesFailed {
                    message: e.to_string(),
                    source: e,
                },
                current_step,
            )
        })?;

        info!(
            command = "run",
            instance_ip = %instance_ip,
            "Docker Compose services started successfully"
        );

        Ok(())
    }

    /// Build failure context for a run error and generate trace file
    ///
    /// This helper method builds structured error context including the failed step,
    /// error classification, timing information, and generates a trace file for
    /// post-mortem analysis.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment being run (for trace directory path)
    /// * `error` - The run error that occurred
    /// * `current_step` - The step that was executing when the error occurred
    /// * `started_at` - The timestamp when run execution started
    ///
    /// # Returns
    ///
    /// A `RunFailureContext` with all failure metadata and trace file path
    fn build_failure_context(
        &self,
        environment: &Environment<Released>,
        error: &RunCommandHandlerError,
        current_step: RunStep,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> RunFailureContext {
        use crate::application::command_handlers::common::failure_context::build_base_failure_context;
        use crate::infrastructure::trace::RunTraceWriter;

        // Step that failed is directly provided - no reverse engineering needed
        let failed_step = current_step;

        // Get error kind from the error itself (errors are self-describing)
        let error_kind = error.error_kind();

        // Build base failure context using common helper
        let base = build_base_failure_context(&self.clock, started_at, error.to_string());

        // Build handler-specific context
        let mut context = RunFailureContext {
            failed_step,
            error_kind,
            base,
        };

        // Generate trace file (logging handled by trace writer)
        let traces_dir = environment.traces_dir();
        let writer = RunTraceWriter::new(traces_dir, Arc::clone(&self.clock));

        if let Ok(trace_file) = writer.write_trace(&context, error) {
            context.base.trace_file_path = Some(trace_file);
        }

        context
    }

    /// Load environment from storage and validate it is in `Released` state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Persistence error occurs during load
    /// * Environment does not exist
    /// * Environment is not in `Released` state
    #[allow(clippy::result_large_err)]
    fn load_released_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Released>, RunCommandHandlerError> {
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(|e| RunCommandHandlerError::StatePersistence(e.into()))?;

        let any_env = any_env.ok_or_else(|| RunCommandHandlerError::EnvironmentNotFound {
            name: env_name.to_string(),
        })?;

        Ok(any_env.try_into_released()?)
    }
}
