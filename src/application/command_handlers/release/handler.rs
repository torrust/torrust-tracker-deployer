//! Release command handler implementation

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{error, info, instrument};

use super::errors::ReleaseCommandHandlerError;
use crate::adapters::ansible::AnsibleClient;
use crate::application::command_handlers::common::StepResult;
use crate::application::steps::{
    application::{
        CreateGrafanaStorageStep, CreateMysqlStorageStep, CreatePrometheusStorageStep,
        CreateTrackerStorageStep, DeployCaddyConfigStep, DeployGrafanaProvisioningStep,
        DeployPrometheusConfigStep, DeployTrackerConfigStep, InitTrackerDatabaseStep,
    },
    rendering::{
        RenderCaddyTemplatesStep, RenderGrafanaTemplatesStep, RenderPrometheusTemplatesStep,
        RenderTrackerTemplatesStep,
    },
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
    /// This method orchestrates the complete release workflow, organized by service:
    ///
    /// 1. **Tracker**: Storage creation, database init, config rendering, deployment
    /// 2. **Prometheus**: Storage creation, config rendering, deployment (if enabled)
    /// 3. **Grafana**: Storage creation, provisioning rendering, deployment (if enabled)
    /// 4. **`MySQL`**: Storage creation (if enabled)
    /// 5. **Caddy**: Config rendering, deployment (if HTTPS enabled)
    /// 6. **Docker Compose**: Template rendering, deployment
    ///
    /// If an error occurs, it returns both the error and the step that was being
    /// executed, enabling accurate failure context generation.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Releasing state
    /// * `instance_ip` - The validated instance IP address (used for Docker Compose deployment logging)
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any release step fails
    async fn execute_release_workflow(
        &self,
        environment: &Environment<Releasing>,
        instance_ip: IpAddr,
    ) -> StepResult<Environment<Released>, ReleaseCommandHandlerError, ReleaseStep> {
        // Tracker service steps
        Self::release_tracker_service(environment)?;

        // Prometheus service steps (if enabled)
        Self::release_prometheus_service(environment)?;

        // Grafana service steps (if enabled)
        Self::release_grafana_service(environment)?;

        // MySQL service steps (if enabled)
        Self::release_mysql_service(environment)?;

        // Caddy service steps (if HTTPS enabled)
        Self::release_caddy_service(environment)?;

        // Docker Compose deployment
        Self::release_docker_compose(environment, instance_ip).await?;

        let released = environment.clone().released();

        Ok(released)
    }

    // =========================================================================
    // Service-level release orchestration
    // =========================================================================

    /// Release the Tracker service
    ///
    /// Executes all steps required to release the Tracker:
    /// 1. Create storage directories
    /// 2. Initialize database
    /// 3. Render configuration templates
    /// 4. Deploy configuration to remote
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, step) if any tracker step fails
    #[allow(clippy::result_large_err)]
    fn release_tracker_service(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        Self::create_tracker_storage(environment)?;
        Self::init_tracker_database(environment)?;
        let tracker_build_dir = Self::render_tracker_templates(environment)?;
        Self::deploy_tracker_config_to_remote(environment, &tracker_build_dir)?;
        Ok(())
    }

    /// Release the Prometheus service (if enabled)
    ///
    /// Executes all steps required to release Prometheus:
    /// 1. Create storage directories
    /// 2. Render configuration templates
    /// 3. Deploy configuration to remote
    ///
    /// If Prometheus is not configured, all steps are skipped.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, step) if any Prometheus step fails
    #[allow(clippy::result_large_err)]
    fn release_prometheus_service(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        Self::create_prometheus_storage(environment)?;
        Self::render_prometheus_templates(environment)?;
        Self::deploy_prometheus_config_to_remote(environment)?;
        Ok(())
    }

    /// Release the Grafana service (if enabled)
    ///
    /// Executes all steps required to release Grafana:
    /// 1. Create storage directories
    /// 2. Render provisioning templates
    /// 3. Deploy provisioning to remote
    ///
    /// If Grafana is not configured, all steps are skipped.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, step) if any Grafana step fails
    #[allow(clippy::result_large_err)]
    fn release_grafana_service(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        Self::create_grafana_storage(environment)?;
        Self::render_grafana_templates(environment)?;
        Self::deploy_grafana_provisioning_to_remote(environment)?;
        Ok(())
    }

    /// Release the `MySQL` service (if enabled)
    ///
    /// Executes all steps required to release `MySQL`:
    /// 1. Create storage directories
    ///
    /// If `MySQL` is not configured as the tracker database, this step is skipped.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, step) if `MySQL` storage creation fails
    #[allow(clippy::result_large_err)]
    fn release_mysql_service(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        Self::create_mysql_storage(environment)?;
        Ok(())
    }

    /// Release the Caddy service (if HTTPS enabled)
    ///
    /// Executes all steps required to release Caddy:
    /// 1. Render configuration templates
    /// 2. Deploy configuration to remote
    ///
    /// If HTTPS is not configured, all steps are skipped.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, step) if any Caddy step fails
    #[allow(clippy::result_large_err)]
    fn release_caddy_service(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        Self::render_caddy_templates(environment)?;
        Self::deploy_caddy_config_to_remote(environment)?;
        Ok(())
    }

    /// Release Docker Compose configuration
    ///
    /// Executes all steps required to deploy Docker Compose:
    /// 1. Render Docker Compose templates
    /// 2. Deploy compose files to remote
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, step) if any Docker Compose step fails
    async fn release_docker_compose(
        environment: &Environment<Releasing>,
        instance_ip: IpAddr,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let compose_build_dir = Self::render_docker_compose_templates(environment).await?;
        Self::deploy_compose_files_to_remote(environment, &compose_build_dir, instance_ip)?;
        Ok(())
    }

    // =========================================================================
    // Individual step implementations
    // =========================================================================

    /// Create an Ansible client configured for the environment's build directory
    ///
    /// This is a helper method to reduce duplication across step implementations.
    fn ansible_client(environment: &Environment<Releasing>) -> Arc<AnsibleClient> {
        Arc::new(AnsibleClient::new(environment.build_dir().join("ansible")))
    }

    /// Create tracker storage directories on the remote host
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::CreateTrackerStorage`) if creation fails
    #[allow(clippy::result_large_err)]
    fn create_tracker_storage(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::CreateTrackerStorage;

        CreateTrackerStorageStep::new(Self::ansible_client(environment))
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::TrackerStorageCreation {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
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
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::InitTrackerDatabase;

        InitTrackerDatabaseStep::new(Self::ansible_client(environment))
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::TrackerDatabaseInit {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
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
                ReleaseCommandHandlerError::TemplateRendering {
                    message: e.to_string(),
                    source: Box::new(e),
                },
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
        if environment.context().user_inputs.prometheus().is_none() {
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
                ReleaseCommandHandlerError::TemplateRendering {
                    message: e.to_string(),
                    source: Box::new(e),
                },
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

    /// Create Prometheus storage directories on the remote host (if enabled)
    ///
    /// This step is optional and only executes if Prometheus is configured in the environment.
    /// If Prometheus is not configured, the step is skipped without error.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::CreatePrometheusStorage`) if creation fails
    #[allow(clippy::result_large_err)]
    fn create_prometheus_storage(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::CreatePrometheusStorage;

        // Check if Prometheus is configured
        if environment.context().user_inputs.prometheus().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "Prometheus not configured - skipping storage creation"
            );
            return Ok(());
        }

        CreatePrometheusStorageStep::new(Self::ansible_client(environment))
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::PrometheusStorageCreation {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "Prometheus storage directories created successfully"
        );

        Ok(())
    }

    /// Create Grafana storage directories on the remote host (if enabled)
    ///
    /// This step is optional and only executes if Grafana is configured in the environment.
    /// If Grafana is not configured, the step is skipped without error.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::CreateGrafanaStorage`) if creation fails
    #[allow(clippy::result_large_err)]
    fn create_grafana_storage(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::CreateGrafanaStorage;

        // Check if Grafana is configured
        if environment.context().user_inputs.grafana().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "Grafana not configured - skipping storage creation"
            );
            return Ok(());
        }

        CreateGrafanaStorageStep::new(Self::ansible_client(environment))
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::GrafanaStorageCreation {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "Grafana storage directories created successfully"
        );

        Ok(())
    }

    /// Create `MySQL` storage directories on the remote host (if enabled)
    ///
    /// This step is optional and only executes if `MySQL` is configured as the tracker database.
    /// If `MySQL` is not configured, the step is skipped without error.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::CreateMysqlStorage`) if creation fails
    #[allow(clippy::result_large_err)]
    fn create_mysql_storage(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::CreateMysqlStorage;

        // Check if MySQL is configured (via tracker database driver)
        if !environment.context().user_inputs.tracker().uses_mysql() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "MySQL not configured - skipping storage creation"
            );
            return Ok(());
        }

        CreateMysqlStorageStep::new(Self::ansible_client(environment))
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::MysqlStorageCreation {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "MySQL storage directories created successfully"
        );

        Ok(())
    }

    /// Deploy Prometheus configuration to the remote host via Ansible (if enabled)
    ///
    /// This step is optional and only executes if Prometheus is configured in the environment.
    /// If Prometheus is not configured, the step is skipped without error.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Releasing state
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::DeployPrometheusConfigToRemote`) if deployment fails
    #[allow(clippy::result_large_err)]
    fn deploy_prometheus_config_to_remote(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::DeployPrometheusConfigToRemote;

        // Check if Prometheus is configured
        if environment.context().user_inputs.prometheus().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "Prometheus not configured - skipping config deployment"
            );
            return Ok(());
        }

        DeployPrometheusConfigStep::new(Self::ansible_client(environment))
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::PrometheusConfigDeployment {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "Prometheus configuration deployed successfully"
        );

        Ok(())
    }

    /// Render Grafana provisioning templates (if enabled)
    ///
    /// This step is optional and only executes if Grafana is configured in the environment.
    /// If Grafana is not configured, the step is skipped without error.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::RenderGrafanaTemplates`) if rendering fails
    #[allow(clippy::result_large_err)]
    fn render_grafana_templates(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::RenderGrafanaTemplates;

        // Check if Grafana is configured
        if environment.context().user_inputs.grafana().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "Grafana not configured - skipping provisioning template rendering"
            );
            return Ok(());
        }

        // Check if Prometheus is configured (required for datasource)
        if environment.context().user_inputs.prometheus().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "Prometheus not configured - skipping Grafana provisioning (datasource requires Prometheus)"
            );
            return Ok(());
        }

        let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
        let step = RenderGrafanaTemplatesStep::new(
            Arc::new(environment.clone()),
            template_manager,
            environment.build_dir().clone(),
        );

        step.execute().map_err(|e| {
            (
                ReleaseCommandHandlerError::TemplateRendering {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

        info!(
            command = "release",
            step = %current_step,
            "Grafana provisioning templates rendered successfully"
        );

        Ok(())
    }

    /// Render Caddy configuration templates (if HTTPS enabled)
    ///
    /// This step is optional and only executes if HTTPS is configured in the environment.
    /// If HTTPS is not configured, the step is skipped without error.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::RenderCaddyTemplates`) if rendering fails
    #[allow(clippy::result_large_err)]
    fn render_caddy_templates(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::RenderCaddyTemplates;

        // Check if HTTPS is configured
        if environment.context().user_inputs.https().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "HTTPS not configured - skipping Caddy template rendering"
            );
            return Ok(());
        }

        let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
        let step = RenderCaddyTemplatesStep::new(
            Arc::new(environment.clone()),
            template_manager,
            environment.build_dir().clone(),
        );

        step.execute().map_err(|e| {
            (
                ReleaseCommandHandlerError::TemplateRendering {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

        info!(
            command = "release",
            step = %current_step,
            "Caddy configuration templates rendered successfully"
        );

        Ok(())
    }

    /// Deploy Caddy configuration to the remote host (if HTTPS enabled)
    ///
    /// This step is optional and only executes if HTTPS is configured in the environment.
    /// If HTTPS is not configured, the step is skipped without error.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::DeployCaddyConfigToRemote`) if deployment fails
    #[allow(clippy::result_large_err)]
    fn deploy_caddy_config_to_remote(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::DeployCaddyConfigToRemote;

        // Check if HTTPS is configured
        if environment.context().user_inputs.https().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "HTTPS not configured - skipping Caddy config deployment"
            );
            return Ok(());
        }

        DeployCaddyConfigStep::new(Self::ansible_client(environment))
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::CaddyConfigDeployment {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "Caddy configuration deployed to remote successfully"
        );

        Ok(())
    }

    /// Deploy Grafana provisioning configuration to the remote host (if enabled)
    ///
    /// This step is optional and only executes if Grafana is configured in the environment.
    /// If Grafana is not configured, the step is skipped without error.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::DeployGrafanaProvisioning`) if deployment fails
    #[allow(clippy::result_large_err)]
    fn deploy_grafana_provisioning_to_remote(
        environment: &Environment<Releasing>,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::DeployGrafanaProvisioning;

        // Check if Grafana is configured
        if environment.context().user_inputs.grafana().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "Grafana not configured - skipping provisioning deployment"
            );
            return Ok(());
        }

        // Check if Prometheus is configured (required for datasource)
        if environment.context().user_inputs.prometheus().is_none() {
            info!(
                command = "release",
                step = %current_step,
                status = "skipped",
                "Prometheus not configured - skipping Grafana provisioning deployment"
            );
            return Ok(());
        }

        DeployGrafanaProvisioningStep::new(Self::ansible_client(environment))
            .execute()
            .map_err(|e| {
                (
                    ReleaseCommandHandlerError::GrafanaProvisioningDeployment {
                        message: e.to_string(),
                        source: Box::new(e),
                    },
                    current_step,
                )
            })?;

        info!(
            command = "release",
            step = %current_step,
            "Grafana provisioning configuration deployed successfully"
        );

        Ok(())
    }

    /// Deploy tracker configuration to the remote host via Ansible
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Releasing state
    /// * `tracker_build_dir` - Path to the rendered tracker configuration
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `ReleaseStep::DeployTrackerConfigToRemote`) if deployment fails
    #[allow(clippy::result_large_err)]
    fn deploy_tracker_config_to_remote(
        environment: &Environment<Releasing>,
        tracker_build_dir: &Path,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::DeployTrackerConfigToRemote;

        DeployTrackerConfigStep::new(
            Self::ansible_client(environment),
            tracker_build_dir.to_path_buf(),
        )
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::TrackerConfigDeployment {
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
                ReleaseCommandHandlerError::TemplateRendering {
                    message: e.to_string(),
                    source: Box::new(e),
                },
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
    #[allow(clippy::result_large_err)]
    fn deploy_compose_files_to_remote(
        environment: &Environment<Releasing>,
        compose_build_dir: &Path,
        instance_ip: IpAddr,
    ) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
        let current_step = ReleaseStep::DeployComposeFilesToRemote;

        let ansible_client = Arc::new(AnsibleClient::new(environment.ansible_build_dir()));
        let step = DeployComposeFilesStep::new(ansible_client, compose_build_dir.to_path_buf());

        step.execute().map_err(|e| {
            (
                ReleaseCommandHandlerError::ComposeFilesDeployment {
                    message: e.to_string(),
                    source: Box::new(e),
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
