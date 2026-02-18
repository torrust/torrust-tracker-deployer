//! Release command handler implementation

use std::sync::Arc;

use tracing::{error, info, instrument};

use super::errors::ReleaseCommandHandlerError;
use super::workflow;
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::state::{ReleaseFailureContext, ReleaseStep};
use crate::domain::environment::{Configured, Environment, Released, Releasing};
use crate::domain::EnvironmentName;
use crate::shared::error::Traceable;

/// Total number of steps in the release workflow.
///
/// This constant is used for progress reporting via `CommandProgressListener`
/// to display step progress like "[Step 1/7] Releasing Tracker service...".
pub(super) const TOTAL_RELEASE_STEPS: usize = 7;

/// `ReleaseCommandHandler` orchestrates the software release workflow
///
/// The `ReleaseCommandHandler` orchestrates the software release workflow to
/// deploy software to a configured environment.
///
/// This command handler handles all steps required to release software:
/// 1. Load the environment from storage
/// 2. Validate the environment is in the correct state
/// 3. Render Docker Compose templates to the build directory
/// 4. Deploy compose files to the remote host via Ansible
/// 5. Transition environment to `Released` state
///
/// # Architecture
///
/// Follows the three-level architecture:
/// - **Command** (Level 1): This handler orchestrates the release workflow
/// - **Step** (Level 2): `RenderDockerComposeTemplatesStep`, `DeployComposeFilesStep`
/// - **Remote Action** (Level 3): Ansible playbook executes on remote host
///
/// # State Management
///
/// The command handler integrates with the type-state pattern for environment lifecycle:
/// - Accepts environment in `Configured` state
/// - Transitions to `Environment<Releasing>` at start
/// - Returns `Environment<Released>` on success
/// - Transitions to `Environment<ReleaseFailed>` on error
///
/// State is persisted after each transition using the injected repository.
pub struct ReleaseCommandHandler {
    clock: Arc<dyn crate::shared::Clock>,
    repository: TypedEnvironmentRepository,
}

impl ReleaseCommandHandler {
    /// Create a new `ReleaseCommandHandler`
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

    /// Execute the release workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to release to
    /// * `listener` - Optional progress listener for step-level reporting
    ///
    /// # Returns
    ///
    /// Returns `Ok(Environment<Released>)` on success
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found
    /// * Environment is not in `Configured` state
    /// * Docker Compose template rendering fails
    /// * File deployment to VM fails
    /// * State persistence fails
    #[instrument(
        name = "release_command",
        skip_all,
        fields(
            command_type = "release",
            environment = %env_name
        )
    )]
    pub async fn execute(
        &self,
        env_name: &EnvironmentName,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<Environment<Released>, ReleaseCommandHandlerError> {
        let environment = self.load_configured_environment(env_name)?;

        // Validate instance IP exists before proceeding (fail early)
        let instance_ip = environment.instance_ip().ok_or_else(|| {
            ReleaseCommandHandlerError::MissingInstanceIp {
                name: env_name.to_string(),
            }
        })?;

        let started_at = self.clock.now();

        info!(
            command = "release",
            environment = %env_name,
            instance_ip = %instance_ip,
            current_state = "configured",
            target_state = "releasing",
            "Environment loaded and validated. Transitioning to Releasing state."
        );

        let releasing_env = environment.start_releasing();

        self.repository.save_releasing(&releasing_env)?;

        info!(
            command = "release",
            environment = %env_name,
            current_state = "releasing",
            "Releasing state persisted. Executing release steps."
        );

        match workflow::execute(&releasing_env, listener).await {
            Ok(released) => {
                info!(
                    command = "release",
                    environment = %released.name(),
                    final_state = "released",
                    "Software release completed successfully"
                );

                self.repository.save_released(&released)?;

                Ok(released)
            }
            Err((e, current_step)) => {
                error!(
                    command = "release",
                    environment = %releasing_env.name(),
                    error = %e,
                    step = ?current_step,
                    "Software release failed"
                );

                let context =
                    self.build_failure_context(&releasing_env, &e, current_step, started_at);
                let failed = releasing_env.release_failed(context);

                self.repository.save_release_failed(&failed)?;

                Err(e)
            }
        }
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    /// Build failure context for a release error and generate trace file
    ///
    /// This helper method builds structured error context including the failed step,
    /// error classification, timing information, and generates a trace file for
    /// post-mortem analysis.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment being released (for trace directory path)
    /// * `error` - The release error that occurred
    /// * `current_step` - The step that was executing when the error occurred
    /// * `started_at` - The timestamp when release execution started
    ///
    /// # Returns
    ///
    /// A `ReleaseFailureContext` with all failure metadata and trace file path
    fn build_failure_context(
        &self,
        environment: &Environment<Releasing>,
        error: &ReleaseCommandHandlerError,
        current_step: ReleaseStep,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> ReleaseFailureContext {
        use crate::application::command_handlers::common::failure_context::build_base_failure_context;
        use crate::infrastructure::trace::ReleaseTraceWriter;

        // Step that failed is directly provided - no reverse engineering needed
        let failed_step = current_step;

        // Get error kind from the error itself (errors are self-describing)
        let error_kind = error.error_kind();

        // Build base failure context using common helper
        let base = build_base_failure_context(&self.clock, started_at, error.to_string());

        // Build handler-specific context
        let mut context = ReleaseFailureContext {
            failed_step,
            error_kind,
            base,
        };

        // Generate trace file (logging handled by trace writer)
        let traces_dir = environment.traces_dir();
        let writer = ReleaseTraceWriter::new(traces_dir, Arc::clone(&self.clock));

        if let Ok(trace_file) = writer.write_trace(&context, error) {
            context.base.trace_file_path = Some(trace_file);
        }

        context
    }

    /// Load environment from storage and validate it is in `Configured` state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Persistence error occurs during load
    /// * Environment does not exist
    /// * Environment is not in `Configured` state
    #[allow(clippy::result_large_err)]
    fn load_configured_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Configured>, ReleaseCommandHandlerError> {
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(ReleaseCommandHandlerError::StatePersistence)?;

        let any_env = any_env.ok_or_else(|| ReleaseCommandHandlerError::EnvironmentNotFound {
            name: env_name.to_string(),
        })?;

        Ok(any_env.try_into_configured()?)
    }
}
