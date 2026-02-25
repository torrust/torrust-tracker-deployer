//! Configure command handler implementation

use std::sync::Arc;

use tracing::{error, info, instrument};

use super::errors::ConfigureCommandHandlerError;
use crate::adapters::ansible::AnsibleClient;
use crate::application::command_handlers::common::StepResult;
use crate::application::steps::{
    ConfigureFirewallStep, ConfigureSecurityUpdatesStep, InstallDockerComposeStep,
    InstallDockerStep,
};
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::state::{ConfigureFailureContext, ConfigureStep};
use crate::domain::environment::{Configured, Configuring, Environment};
use crate::domain::EnvironmentName;
use crate::infrastructure::trace::ConfigureTraceWriter;
use crate::shared::error::Traceable;

/// Total number of steps in the configuration workflow.
///
/// This constant is used for progress reporting via `CommandProgressListener`
/// to display step progress like "[Step 1/4] Installing Docker...".
const TOTAL_CONFIGURE_STEPS: usize = 4;

/// `ConfigureCommandHandler` orchestrates the complete infrastructure configuration workflow
///
/// The `ConfigureCommandHandler` orchestrates the complete infrastructure configuration workflow.
///
/// This command handles all steps required to configure infrastructure:
/// 1. Install Docker
/// 2. Install Docker Compose
/// 3. Configure automatic security updates
/// 4. Configure UFW firewall
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
    pub(crate) clock: Arc<dyn crate::shared::Clock>,
    pub(crate) repository: TypedEnvironmentRepository,
}

impl ConfigureCommandHandler {
    /// Create a new `ConfigureCommandHandler`
    #[must_use]
    pub fn new(
        clock: Arc<dyn crate::shared::Clock>,
        repository: Arc<dyn EnvironmentRepository>,
    ) -> Self {
        Self {
            clock,
            repository: TypedEnvironmentRepository::new(repository),
        }
    }

    /// Execute the complete configuration workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to configure
    /// * `listener` - Optional progress listener for reporting step-level progress.
    ///   When provided, the handler reports progress at each of the 4 configuration steps.
    ///   When `None`, the handler executes silently (backward compatible).
    ///
    /// # Returns
    ///
    /// Returns the configured environment
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the configuration workflow fails:
    /// * Environment not found or not in `Provisioned` state
    /// * Docker installation fails
    /// * Docker Compose installation fails
    /// * Security updates configuration fails
    /// * Firewall configuration fails
    ///
    /// On error, the environment transitions to `ConfigureFailed` state and is persisted.
    #[instrument(
        name = "configure_command",
        skip_all,
        fields(
            command_type = "configure",
            environment = %env_name
        )
    )]
    pub fn execute(
        &self,
        env_name: &EnvironmentName,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<Environment<Configured>, ConfigureCommandHandlerError> {
        let environment = self.load_provisioned_environment(env_name)?;

        let started_at = self.clock.now();

        let environment = environment.start_configuring();

        self.repository.save_configuring(&environment)?;

        match Self::execute_configuration_with_tracking(&environment, listener) {
            Ok(configured_env) => {
                info!(
                    command = "configure",
                    environment = %configured_env.name(),
                    "Infrastructure configuration completed successfully"
                );

                self.repository.save_configured(&configured_env)?;

                Ok(configured_env)
            }
            Err((e, current_step)) => {
                error!(
                    command = "configure",
                    environment = %environment.name(),
                    failed_step = ?current_step,
                    error = %e,
                    "Infrastructure configuration failed"
                );

                let context =
                    self.build_failure_context(&environment, &e, current_step, started_at);

                let failed = environment.configure_failed(context);

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
    /// # Arguments
    ///
    /// * `environment` - The environment in Configuring state
    /// * `listener` - Optional progress listener for step-level reporting
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any configuration step fails
    fn execute_configuration_with_tracking(
        environment: &Environment<Configuring>,
        listener: Option<&dyn CommandProgressListener>,
    ) -> StepResult<Environment<Configured>, ConfigureCommandHandlerError, ConfigureStep> {
        let ansible_client = Arc::new(AnsibleClient::new(environment.ansible_build_dir()));

        // Allow tests or CI to skip Docker installation
        // (useful for container-based tests where Docker is already installed via Dockerfile)
        let skip_docker =
            std::env::var("TORRUST_TD_SKIP_DOCKER_INSTALL_IN_CONTAINER").is_ok_and(|v| v == "true");

        // Step 1/4: Install Docker
        let current_step = ConfigureStep::InstallDocker;
        Self::notify_step_started(listener, 1, "Installing Docker");
        if skip_docker {
            info!(
                command = "configure",
                step = "install_docker",
                status = "skipped",
                "Skipping Docker installation due to TORRUST_TD_SKIP_DOCKER_INSTALL_IN_CONTAINER (Docker pre-installed)"
            );
        } else {
            InstallDockerStep::new(Arc::clone(&ansible_client))
                .execute(listener)
                .map_err(|e| (e.into(), current_step))?;
        }

        // Step 2/4: Install Docker Compose
        let current_step = ConfigureStep::InstallDockerCompose;
        Self::notify_step_started(listener, 2, "Installing Docker Compose");
        if skip_docker {
            info!(
                command = "configure",
                step = "install_docker_compose",
                status = "skipped",
                "Skipping Docker Compose installation due to TORRUST_TD_SKIP_DOCKER_INSTALL_IN_CONTAINER (Docker Compose pre-installed)"
            );
        } else {
            InstallDockerComposeStep::new(Arc::clone(&ansible_client))
                .execute(listener)
                .map_err(|e| (e.into(), current_step))?;
        }

        // Step 3/4: Configure automatic security updates
        let current_step = ConfigureStep::ConfigureSecurityUpdates;
        Self::notify_step_started(listener, 3, "Configuring automatic security updates");
        ConfigureSecurityUpdatesStep::new(Arc::clone(&ansible_client))
            .execute(listener)
            .map_err(|e| (e.into(), current_step))?;

        // Step 4/4: Configure firewall (UFW)
        let current_step = ConfigureStep::ConfigureFirewall;
        Self::notify_step_started(listener, 4, "Configuring firewall (UFW)");
        // Allow tests or CI to explicitly skip the firewall configuration step
        // (useful for container-based test runs where iptables/ufw require
        // elevated kernel capabilities not available in unprivileged containers).
        let skip_firewall =
            std::env::var("TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER").is_ok_and(|v| v == "true");

        if skip_firewall {
            info!(
                command = "configure",
                step = "configure_firewall",
                status = "skipped",
                "Skipping UFW firewall configuration due to TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER"
            );
        } else {
            ConfigureFirewallStep::new(Arc::clone(&ansible_client))
                .execute(listener)
                .map_err(|e| (e.into(), current_step))?;
        }

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
    fn build_failure_context(
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

    /// Load environment from storage and validate it is in `Provisioned` state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Persistence error occurs during load
    /// * Environment does not exist
    /// * Environment is not in `Provisioned` state
    fn load_provisioned_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<
        crate::domain::environment::Environment<crate::domain::environment::Provisioned>,
        ConfigureCommandHandlerError,
    > {
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(|e| ConfigureCommandHandlerError::StatePersistence(e.into()))?;

        let any_env = any_env.ok_or_else(|| ConfigureCommandHandlerError::EnvironmentNotFound {
            name: env_name.to_string(),
        })?;

        Ok(any_env.try_into_provisioned()?)
    }

    /// Notify progress listener that a step has started
    ///
    /// Helper method to notify the listener when a configuration step begins.
    /// If no listener is provided, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `listener` - Optional progress listener
    /// * `step_number` - The current step number (1-based)
    /// * `description` - User-facing description of the step
    fn notify_step_started(
        listener: Option<&dyn CommandProgressListener>,
        step_number: usize,
        description: &str,
    ) {
        if let Some(l) = listener {
            l.on_step_started(step_number, TOTAL_CONFIGURE_STEPS, description);
        }
    }
}
