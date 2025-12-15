//! Release command handler implementation

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{error, info, instrument};

use super::errors::ReleaseCommandHandlerError;
use crate::adapters::ansible::AnsibleClient;
use crate::application::command_handlers::common::StepResult;
use crate::application::steps::{
    application::{CreateTrackerStorageStep, DeployTrackerConfigStep, InitTrackerDatabaseStep},
    rendering::{RenderPrometheusTemplatesStep, RenderTrackerTemplatesStep},
    DeployComposeFilesStep, RenderDockerComposeTemplatesStep,
};
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::state::{ReleaseFailureContext, ReleaseStep};
use crate::domain::environment::{Configured, Environment, Released, Releasing};
use crate::domain::template::TemplateManager;
use crate::domain::EnvironmentName;
use crate::shared::error::Traceable;

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
    ) -> Result<Environment<Released>, ReleaseCommandHandlerError> {
        let environment = self.load_configured_environment(env_name)?;

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

        match self
            .execute_release_workflow(&releasing_env, instance_ip)
            .await
        {
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

    /// Execute the release workflow with step tracking
    ///
    /// This method orchestrates the complete release workflow:
    /// 1. Create tracker storage directories
    /// 2. Initialize tracker `SQLite` database
    /// 3. Render Docker Compose templates to the build directory
    /// 4. Deploy compose files to the remote host via Ansible
    ///
    /// If an error occurs, it returns both the error and the step that was being
    /// executed, enabling accurate failure context generation.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Releasing state
    /// * `instance_ip` - The validated instance IP address (precondition checked by caller)
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any release step fails
    async fn execute_release_workflow(
        &self,
        environment: &Environment<Releasing>,
        instance_ip: IpAddr,
    ) -> StepResult<Environment<Released>, ReleaseCommandHandlerError, ReleaseStep> {
        // Step 1: Create tracker storage directories
        Self::create_tracker_storage(environment, instance_ip)?;

        // Step 2: Initialize tracker database
        Self::init_tracker_database(environment, instance_ip)?;

        // Step 3: Render tracker configuration templates
        let tracker_build_dir = Self::render_tracker_templates(environment)?;

        // Step 4: Deploy tracker configuration to remote
        self.deploy_tracker_config_to_remote(environment, &tracker_build_dir, instance_ip)?;

        // Step 5: Render Prometheus configuration templates (if enabled)
        Self::render_prometheus_templates(environment)?;

        // Step 6: Render Docker Compose templates
        let compose_build_dir = self.render_docker_compose_templates(environment).await?;

        // Step 7: Deploy compose files to remote
        self.deploy_compose_files_to_remote(environment, &compose_build_dir, instance_ip)?;

        let released = environment.clone().released();

        Ok(released)
    }

    /// Create tracker storage directories on the remote host
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::CreateTrackerStorage`) if creation fails
    #[allow(clippy::result_large_err)]
    fn create_tracker_storage(
        environment: &Environment<Releasing>,
        _instance_ip: IpAddr,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::CreateTrackerStorage;

        let ansible_client = Arc::new(AnsibleClient::new(environment.build_dir().join("ansible")));

        CreateTrackerStorageStep::new(ansible_client)
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::TrackerStorageCreation(e.to_string()),
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "Tracker storage directories created successfully"
        );

        Ok(())
    }

    /// Initialize tracker database on the remote host
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::InitTrackerDatabase`) if initialization fails
    #[allow(clippy::result_large_err)]
    fn init_tracker_database(
        environment: &Environment<Releasing>,
        _instance_ip: IpAddr,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::InitTrackerDatabase;

        let ansible_client = Arc::new(AnsibleClient::new(environment.build_dir().join("ansible")));

        InitTrackerDatabaseStep::new(ansible_client)
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::TrackerDatabaseInit(e.to_string()),
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "Tracker database initialized successfully"
        );

        Ok(())
    }

    /// Render Tracker configuration templates to the build directory
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::RenderTrackerTemplates`) if rendering fails
    #[allow(clippy::result_large_err)]
    fn render_tracker_templates(
        environment: &Environment<Releasing>,
    ) -> StepResult<PathBuf, ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::RenderTrackerTemplates;

        let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
        let step = RenderTrackerTemplatesStep::new(
            Arc::new(environment.clone()),
            template_manager,
            environment.build_dir().clone(),
        );

        let tracker_build_dir = step.execute().map_err(|e| {
            (
                ReleaseCommandHandlerError::TemplateRendering(e.to_string()),
                current_step,
            )
        })?;

        info!(
            command = "release",
            tracker_build_dir = %tracker_build_dir.display(),
            "Tracker configuration templates rendered successfully"
        );

        Ok(tracker_build_dir)
    }

    /// Render Prometheus configuration templates to the build directory (if enabled)
    ///
    /// This step is optional and only executes if Prometheus is configured in the environment.
    /// If Prometheus is not configured, the step is skipped without error.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::RenderPrometheusTemplates`) if rendering fails
    #[allow(clippy::result_large_err)]
    fn render_prometheus_templates(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::RenderPrometheusTemplates;

        // Check if Prometheus is configured
        if environment.context().user_inputs.prometheus.is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "Prometheus not configured - skipping template rendering"
            );
            return Ok(());
        }

        let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
        let step = RenderPrometheusTemplatesStep::new(
            Arc::new(environment.clone()),
            template_manager,
            environment.build_dir().clone(),
        );

        step.execute().map_err(|e| {
            (
                ReleaseCommandHandlerError::TemplateRendering(e.to_string()),
                current_step,
            )
        })?;

        info!(
            command = "release",
            step = %current_step,
            "Prometheus configuration templates rendered successfully"
        );

        Ok(())
    }

    /// Deploy tracker configuration to the remote host via Ansible
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Releasing state
    /// * `tracker_build_dir` - Path to the rendered tracker configuration
    /// * `instance_ip` - The target instance IP address
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::DeployTrackerConfigToRemote`) if deployment fails
    #[allow(clippy::result_large_err, clippy::unused_self)]
    fn deploy_tracker_config_to_remote(
        &self,
        environment: &Environment<Releasing>,
        tracker_build_dir: &Path,
        _instance_ip: IpAddr,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::DeployTrackerConfigToRemote;

        let ansible_client = Arc::new(AnsibleClient::new(environment.build_dir().join("ansible")));

        DeployTrackerConfigStep::new(ansible_client, tracker_build_dir.to_path_buf())
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::Deployment {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "Tracker configuration deployed successfully"
        );

        Ok(())
    }

    /// Render Docker Compose templates to the build directory
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::RenderDockerComposeTemplates`) if rendering fails
    async fn render_docker_compose_templates(
        &self,
        environment: &Environment<Releasing>,
    ) -> StepResult<PathBuf, ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::RenderDockerComposeTemplates;

        let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
        let step = RenderDockerComposeTemplatesStep::new(
            Arc::new(environment.clone()),
            template_manager,
            environment.build_dir().clone(),
        );

        let compose_build_dir = step.execute().await.map_err(|e| {
            (
                ReleaseCommandHandlerError::TemplateRendering(e.to_string()),
                current_step,
            )
        })?;

        info!(
            command = "release",
            compose_build_dir = %compose_build_dir.display(),
            "Docker Compose templates rendered successfully"
        );

        Ok(compose_build_dir)
    }

    /// Deploy compose files to the remote host via Ansible
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Releasing state
    /// * `compose_build_dir` - Path to the rendered compose files
    /// * `instance_ip` - The target instance IP address
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::DeployComposeFilesToRemote`) if deployment fails
    #[allow(clippy::result_large_err, clippy::unused_self)]
    fn deploy_compose_files_to_remote(
        &self,
        environment: &Environment<Releasing>,
        compose_build_dir: &Path,
        instance_ip: IpAddr,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::DeployComposeFilesToRemote;

        let ansible_client = Arc::new(AnsibleClient::new(environment.ansible_build_dir()));
        let step = DeployComposeFilesStep::new(ansible_client, compose_build_dir.to_path_buf());

        step.execute().map_err(|e| {
            (
                ReleaseCommandHandlerError::DeploymentFailed {
                    message: e.to_string(),
                    source: e,
                },
                current_step,
            )
        })?;

        info!(
            command = "release",
            compose_build_dir = %compose_build_dir.display(),
            instance_ip = %instance_ip,
            "Compose files deployed to remote host successfully"
        );

        Ok(())
    }

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
