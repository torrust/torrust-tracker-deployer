//! Provision command handler implementation

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use tracing::{error, info, instrument};

use super::errors::ProvisionCommandHandlerError;
use crate::adapters::ansible::AnsibleClient;
use crate::adapters::ssh::SshConfig;
use crate::adapters::tofu::client::InstanceInfo;
use crate::adapters::OpenTofuClient;
use crate::application::command_handlers::common::StepResult;
use crate::application::services::rendering::AnsibleTemplateRenderingService;
use crate::application::steps::{
    ApplyInfrastructureStep, GetInstanceInfoStep, InitializeInfrastructureStep,
    PlanInfrastructureStep, RenderOpenTofuTemplatesStep, ValidateInfrastructureStep,
    WaitForCloudInitStep, WaitForSSHConnectivityStep,
};
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::runtime_outputs::ProvisionMethod;
use crate::domain::environment::state::{ProvisionFailureContext, ProvisionStep};
use crate::domain::environment::{Environment, Provisioned, Provisioning};
use crate::domain::EnvironmentName;
use crate::infrastructure::templating::tofu::TofuProjectGenerator;
use crate::shared::clock::SystemClock;
use crate::shared::error::Traceable;

/// Total number of steps in the provisioning workflow.
///
/// This constant is used for progress reporting via `CommandProgressListener`
/// to display step progress like "[Step 1/9] Rendering `OpenTofu` templates...".
const TOTAL_PROVISION_STEPS: usize = 9;

/// `ProvisionCommandHandler` orchestrates the complete infrastructure provisioning workflow
///
/// The `ProvisionCommandHandler` orchestrates the complete infrastructure provisioning workflow.
///
/// This command handler handles all steps required to provision infrastructure:
/// 1. Render `OpenTofu` templates
/// 2. Initialize `OpenTofu`
/// 3. Validate configuration syntax and consistency
/// 4. Plan infrastructure
/// 5. Apply infrastructure
/// 6. Get instance information
/// 7. Render `Ansible` templates (with runtime IP address)
/// 8. Wait for SSH connectivity
/// 9. Wait for cloud-init completion
///
/// # State Management
///
/// The command handler integrates with the type-state pattern for environment lifecycle:
/// - Accepts `Environment<Created>` as input
/// - Transitions to `Environment<Provisioning>` at start
/// - Returns `Environment<Provisioned>` on success
/// - Transitions to `Environment<ProvisionFailed>` on error
///
/// State is persisted after each transition using the injected repository.
/// Persistence failures are logged but don't fail the command handler (state remains valid in memory).
pub struct ProvisionCommandHandler {
    clock: Arc<dyn crate::shared::Clock>,
    repository: TypedEnvironmentRepository,
}

impl ProvisionCommandHandler {
    /// Create a new `ProvisionCommandHandler`
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

    /// Execute the complete provisioning workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to provision
    /// * `listener` - Optional progress listener for reporting step-level progress.
    ///   When provided, the handler reports progress at each of the 9 provisioning steps.
    ///   When `None`, the handler executes silently (backward compatible).
    ///
    /// # Returns
    ///
    /// Returns the provisioned environment
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the provisioning workflow fails:
    /// * Environment not found or not in `Created` state
    /// * Template rendering fails
    /// * `OpenTofu` initialization, planning, or apply fails
    /// * Unable to retrieve instance information
    /// * SSH connectivity cannot be established
    /// * Cloud-init does not complete successfully
    ///
    /// On error, the environment transitions to `ProvisionFailed` state and is persisted.
    #[instrument(
        name = "provision_command",
        skip_all,
        fields(
            command_type = "provision",
            environment = %env_name
        )
    )]
    pub async fn execute(
        &self,
        env_name: &EnvironmentName,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<Environment<Provisioned>, ProvisionCommandHandlerError> {
        let environment = self.load_created_environment(env_name)?;

        let started_at = self.clock.now();

        let environment = environment.start_provisioning();

        self.repository.save_provisioning(&environment)?;

        // Execute provisioning workflow with explicit step tracking
        // This allows us to know exactly which step failed if an error occurs
        match self
            .execute_provisioning_workflow(&environment, listener)
            .await
        {
            Ok(provisioned) => {
                info!(
                    command = "provision",
                    environment = %provisioned.name(),
                    instance_ip = ?provisioned.instance_ip(),
                    "Infrastructure provisioning completed successfully"
                );

                self.repository.save_provisioned(&provisioned)?;

                Ok(provisioned)
            }
            Err((e, current_step)) => {
                error!(
                    command = "provision",
                    environment = %environment.name(),
                    error = %e,
                    step = ?current_step,
                    "Infrastructure provisioning failed"
                );

                let context =
                    self.build_failure_context(&environment, &e, current_step, started_at);
                let failed = environment.provision_failed(context);

                self.repository.save_provision_failed(&failed)?;

                Err(e)
            }
        }
    }

    /// Execute the provisioning workflow
    ///
    /// This method orchestrates the complete provisioning workflow across multiple phases:
    /// 1. Infrastructure provisioning (`OpenTofu`)
    /// 2. Configuration preparation (Ansible templates and system readiness)
    /// 3. State transition to Provisioned (with instance IP and provision method)
    ///
    /// If an error occurs, it returns both the error and the step that was being
    /// executed, enabling accurate failure context generation.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any provisioning step fails
    ///
    /// # Returns
    ///
    /// Returns the provisioned environment with instance IP and provision method set
    async fn execute_provisioning_workflow(
        &self,
        environment: &Environment<Provisioning>,
        listener: Option<&dyn CommandProgressListener>,
    ) -> StepResult<Environment<Provisioned>, ProvisionCommandHandlerError, ProvisionStep> {
        let instance_ip = self.provision_infrastructure(environment, listener).await?;

        self.prepare_for_configuration(environment, instance_ip, listener)
            .await?;

        self.wait_for_system_readiness(environment, instance_ip, listener)
            .await?;

        let provisioned = environment
            .clone()
            .provisioned(instance_ip, ProvisionMethod::Provisioned);

        Ok(provisioned)
    }

    // Private helper methods - organized from higher to lower level of abstraction

    /// Provision infrastructure using `OpenTofu`
    ///
    /// This method handles the complete `OpenTofu`-based infrastructure provisioning:
    /// - Render `OpenTofu` templates (step 1/9)
    /// - Initialize `OpenTofu` (step 2/9)
    /// - Validate configuration (step 3/9)
    /// - Plan infrastructure changes (step 4/9)
    /// - Apply infrastructure changes (step 5/9)
    /// - Retrieve instance information (step 6/9)
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Provisioning state
    /// * `listener` - Optional progress listener for step-level reporting
    ///
    /// # Returns
    ///
    /// Returns the IP address of the provisioned instance
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any provisioning step fails
    async fn provision_infrastructure(
        &self,
        environment: &Environment<Provisioning>,
        listener: Option<&dyn CommandProgressListener>,
    ) -> StepResult<IpAddr, ProvisionCommandHandlerError, ProvisionStep> {
        let (tofu_template_renderer, opentofu_client) =
            Self::build_infrastructure_dependencies(environment);

        // Step 1/9: Render OpenTofu templates
        let current_step = ProvisionStep::RenderOpenTofuTemplates;
        Self::notify_step_started(listener, 1, "Rendering OpenTofu templates");
        self.render_opentofu_templates(&tofu_template_renderer, listener)
            .await
            .map_err(|e| (e, current_step))?;

        // Step 2/9: Initialize OpenTofu
        let current_step = ProvisionStep::OpenTofuInit;
        Self::notify_step_started(listener, 2, "Initializing OpenTofu");
        InitializeInfrastructureStep::new(Arc::clone(&opentofu_client))
            .execute(listener)
            .map_err(|e| (ProvisionCommandHandlerError::from(e), current_step))?;

        // Step 3/9: Validate infrastructure configuration
        let current_step = ProvisionStep::OpenTofuValidate;
        Self::notify_step_started(listener, 3, "Validating infrastructure configuration");
        ValidateInfrastructureStep::new(Arc::clone(&opentofu_client))
            .execute(listener)
            .map_err(|e| (ProvisionCommandHandlerError::from(e), current_step))?;

        // Step 4/9: Plan infrastructure changes
        let current_step = ProvisionStep::OpenTofuPlan;
        Self::notify_step_started(listener, 4, "Planning infrastructure changes");
        PlanInfrastructureStep::new(Arc::clone(&opentofu_client))
            .execute(listener)
            .map_err(|e| (ProvisionCommandHandlerError::from(e), current_step))?;

        // Step 5/9: Apply infrastructure changes
        let current_step = ProvisionStep::OpenTofuApply;
        Self::notify_step_started(listener, 5, "Applying infrastructure changes");
        ApplyInfrastructureStep::new(Arc::clone(&opentofu_client))
            .execute(listener)
            .map_err(|e| (ProvisionCommandHandlerError::from(e), current_step))?;

        // Step 6/9: Get instance information
        let current_step = ProvisionStep::GetInstanceInfo;
        Self::notify_step_started(listener, 6, "Retrieving instance information");
        let instance_info =
            Self::get_instance_info(&opentofu_client, listener).map_err(|e| (e, current_step))?;
        let instance_ip = instance_info.ip_address;

        Ok(instance_ip)
    }

    /// Build dependencies for infrastructure provisioning
    ///
    /// Creates the template renderer and `OpenTofu` client needed for infrastructure provisioning.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Provisioning state
    ///
    /// # Returns
    ///
    /// Returns a tuple of:
    /// - `TofuProjectGenerator` - For rendering `OpenTofu` templates
    /// - `OpenTofuClient` - For executing `OpenTofu` operations
    fn build_infrastructure_dependencies(
        environment: &Environment<Provisioning>,
    ) -> (Arc<TofuProjectGenerator>, Arc<OpenTofuClient>) {
        let opentofu_client = Arc::new(OpenTofuClient::new(environment.tofu_build_dir()));

        let template_manager = Arc::new(crate::domain::TemplateManager::new(
            environment.templates_dir(),
        ));

        let clock = Arc::new(SystemClock);

        let tofu_template_renderer = Arc::new(TofuProjectGenerator::new(
            template_manager,
            environment.build_dir(),
            environment.ssh_credentials().clone(),
            environment.ssh_port(),
            environment.instance_name().clone(),
            environment.provider_config().clone(),
            clock,
        ));

        (tofu_template_renderer, opentofu_client)
    }

    /// Prepare for configuration stages
    ///
    /// This method handles preparation for future configuration stages:
    /// - Render Ansible templates with user inputs and runtime instance IP (step 7/9)
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Provisioning state
    /// * `instance_ip` - IP address of the provisioned instance
    /// * `listener` - Optional progress listener for step-level reporting
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any preparation step fails
    async fn prepare_for_configuration(
        &self,
        environment: &Environment<Provisioning>,
        instance_ip: IpAddr,
        listener: Option<&dyn CommandProgressListener>,
    ) -> StepResult<(), ProvisionCommandHandlerError, ProvisionStep> {
        // Step 7/9: Render Ansible templates
        let current_step = ProvisionStep::RenderAnsibleTemplates;
        Self::notify_step_started(listener, 7, "Rendering Ansible templates");

        if let Some(l) = listener {
            l.on_debug(&format!(
                "Template directory: {}",
                environment.templates_dir().display()
            ));
            l.on_debug(&format!(
                "Build directory: {}",
                environment.ansible_build_dir().display()
            ));
            l.on_debug(&format!("Instance IP: {instance_ip}"));
        }

        let ansible_template_service = AnsibleTemplateRenderingService::from_paths(
            environment.templates_dir(),
            environment.build_dir().clone(),
            self.clock.clone(),
        );

        ansible_template_service
            .render_templates(&environment.context().user_inputs, instance_ip, None)
            .await
            .map_err(|e| {
                (
                    ProvisionCommandHandlerError::TemplateRendering(e.to_string()),
                    current_step,
                )
            })?;

        if let Some(l) = listener {
            l.on_detail(&format!(
                "Template directory: {}",
                environment.ansible_build_dir().display()
            ));
            l.on_detail("Generated inventory and playbooks");
        }

        Ok(())
    }

    /// Wait for system readiness
    ///
    /// This method waits for the provisioned instance to be ready:
    /// - Wait for SSH connectivity on the configured port (step 8/9)
    /// - Wait for cloud-init completion (step 9/9)
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Provisioning state
    /// * `instance_ip` - IP address of the provisioned instance
    /// * `listener` - Optional progress listener for step-level reporting
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any readiness check fails
    async fn wait_for_system_readiness(
        &self,
        environment: &Environment<Provisioning>,
        instance_ip: IpAddr,
        listener: Option<&dyn CommandProgressListener>,
    ) -> StepResult<(), ProvisionCommandHandlerError, ProvisionStep> {
        let ansible_client = Self::build_ansible_client(environment);
        let ssh_credentials = environment.ssh_credentials();
        let ssh_port = environment.ssh_port();
        let ssh_socket_addr = SocketAddr::new(instance_ip, ssh_port);
        let ssh_config = SshConfig::new(ssh_credentials.clone(), ssh_socket_addr);

        // Step 8/9: Wait for SSH connectivity
        let current_step = ProvisionStep::WaitSshConnectivity;
        Self::notify_step_started(listener, 8, "Waiting for SSH connectivity");
        WaitForSSHConnectivityStep::new(ssh_config)
            .execute(listener)
            .await
            .map_err(|e| (ProvisionCommandHandlerError::from(e), current_step))?;

        // Step 9/9: Wait for cloud-init completion
        let current_step = ProvisionStep::CloudInitWait;
        Self::notify_step_started(listener, 9, "Waiting for cloud-init completion");
        WaitForCloudInitStep::new(Arc::clone(&ansible_client))
            .execute(listener)
            .map_err(|e| (ProvisionCommandHandlerError::from(e), current_step))?;

        Ok(())
    }

    /// Build Ansible client for playbook execution
    ///
    /// Creates the Ansible client needed for waiting on cloud-init completion.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Provisioning state
    ///
    /// # Returns
    ///
    /// Returns `AnsibleClient` for executing Ansible playbooks
    fn build_ansible_client(environment: &Environment<Provisioning>) -> Arc<AnsibleClient> {
        Arc::new(AnsibleClient::new(environment.ansible_build_dir()))
    }

    /// Render `OpenTofu` templates
    ///
    /// Generates `OpenTofu` configuration files from templates.
    ///
    /// # Arguments
    ///
    /// * `tofu_template_renderer` - The template renderer for generating `OpenTofu` configs
    /// * `listener` - Optional progress listener for reporting details
    ///
    /// # Errors
    ///
    /// Returns an error if template rendering fails
    async fn render_opentofu_templates(
        &self,
        tofu_template_renderer: &Arc<TofuProjectGenerator>,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<(), ProvisionCommandHandlerError> {
        RenderOpenTofuTemplatesStep::new(tofu_template_renderer.clone())
            .execute(listener)
            .await?;

        Ok(())
    }

    /// Get instance information from `OpenTofu`
    ///
    /// Retrieves information about the provisioned instance, including its IP address.
    ///
    /// # Arguments
    ///
    /// * `opentofu_client` - The `OpenTofu` client for executing commands
    /// * `listener` - Optional progress listener for reporting details
    ///
    /// # Errors
    ///
    /// Returns an error if instance information cannot be retrieved
    fn get_instance_info(
        opentofu_client: &Arc<OpenTofuClient>,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<InstanceInfo, ProvisionCommandHandlerError> {
        let instance_info =
            GetInstanceInfoStep::new(Arc::clone(opentofu_client)).execute(listener)?;
        Ok(instance_info)
    }

    /// Notify the progress listener that a step has started.
    ///
    /// This is a convenience helper that handles the `Option` check,
    /// keeping the step-reporting code in the workflow methods clean.
    fn notify_step_started(
        listener: Option<&dyn CommandProgressListener>,
        step_number: usize,
        description: &str,
    ) {
        if let Some(l) = listener {
            l.on_step_started(step_number, TOTAL_PROVISION_STEPS, description);
        }
    }

    /// Build failure context for a provisioning error and generate trace file
    ///
    /// This helper method builds structured error context including the failed step,
    /// error classification, timing information, and generates a trace file for
    /// post-mortem analysis.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment being provisioned (for trace directory path)
    /// * `error` - The provisioning error that occurred
    /// * `current_step` - The step that was executing when the error occurred
    /// * `started_at` - The timestamp when provisioning execution started
    ///
    /// # Returns
    ///
    /// A `ProvisionFailureContext` with all failure metadata and trace file path
    fn build_failure_context(
        &self,
        environment: &Environment<Provisioning>,
        error: &ProvisionCommandHandlerError,
        current_step: ProvisionStep,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> ProvisionFailureContext {
        use crate::application::command_handlers::common::failure_context::build_base_failure_context;
        use crate::infrastructure::trace::ProvisionTraceWriter;

        // Step that failed is directly provided - no reverse engineering needed
        let failed_step = current_step;

        // Get error kind from the error itself (errors are self-describing)
        let error_kind = error.error_kind();

        // Build base failure context using common helper
        let base = build_base_failure_context(&self.clock, started_at, error.to_string());

        // Build handler-specific context
        let mut context = ProvisionFailureContext {
            failed_step,
            error_kind,
            base,
        };

        // Generate trace file (logging handled by trace writer)
        let traces_dir = environment.traces_dir();
        let writer = ProvisionTraceWriter::new(traces_dir, Arc::clone(&self.clock));

        if let Ok(trace_file) = writer.write_trace(&context, error) {
            context.base.trace_file_path = Some(trace_file);
        }

        context
    }

    /// Load environment from storage and validate it is in `Created` state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Persistence error occurs during load
    /// * Environment does not exist
    /// * Environment is not in `Created` state
    fn load_created_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<crate::domain::environment::Created>, ProvisionCommandHandlerError>
    {
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(|e| ProvisionCommandHandlerError::StatePersistence(e.into()))?;

        let any_env = any_env.ok_or_else(|| ProvisionCommandHandlerError::EnvironmentNotFound {
            name: env_name.to_string(),
        })?;

        Ok(any_env.try_into_created()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{ProgressEvent, RecordingProgressListener};

    #[test]
    fn it_should_have_nine_total_provision_steps() {
        assert_eq!(TOTAL_PROVISION_STEPS, 9);
    }

    #[test]
    fn it_should_notify_listener_when_provided() {
        let listener = RecordingProgressListener::new();

        ProvisionCommandHandler::notify_step_started(Some(&listener), 1, "Test step");

        let events = listener.events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            ProgressEvent::StepStarted {
                step_number: 1,
                total_steps: TOTAL_PROVISION_STEPS,
                description: "Test step".to_string(),
            }
        );
    }

    #[test]
    fn it_should_not_panic_when_listener_is_none() {
        ProvisionCommandHandler::notify_step_started(None, 1, "Test step");
    }

    #[test]
    fn it_should_pass_correct_total_steps_to_listener() {
        let listener = RecordingProgressListener::new();

        ProvisionCommandHandler::notify_step_started(Some(&listener), 5, "Some step");

        let events = listener.events();
        assert_eq!(events.len(), 1);
        if let ProgressEvent::StepStarted { total_steps, .. } = &events[0] {
            assert_eq!(*total_steps, 9);
        } else {
            panic!("Expected StepStarted event");
        }
    }

    #[test]
    fn it_should_record_all_nine_step_descriptions_when_notified_sequentially() {
        let listener = RecordingProgressListener::new();

        let step_descriptions = [
            (1, "Rendering OpenTofu templates"),
            (2, "Initializing OpenTofu"),
            (3, "Validating infrastructure configuration"),
            (4, "Planning infrastructure changes"),
            (5, "Applying infrastructure changes"),
            (6, "Retrieving instance information"),
            (7, "Rendering Ansible templates"),
            (8, "Waiting for SSH connectivity"),
            (9, "Waiting for cloud-init completion"),
        ];

        for (step_number, description) in &step_descriptions {
            ProvisionCommandHandler::notify_step_started(
                Some(&listener),
                *step_number,
                description,
            );
        }

        let events = listener.step_started_events();
        assert_eq!(events.len(), 9);

        for (i, (expected_number, expected_desc)) in step_descriptions.iter().enumerate() {
            if let ProgressEvent::StepStarted {
                step_number,
                total_steps,
                description,
            } = &events[i]
            {
                assert_eq!(step_number, expected_number);
                assert_eq!(*total_steps, TOTAL_PROVISION_STEPS);
                assert_eq!(description, *expected_desc);
            } else {
                panic!("Expected StepStarted event at index {i}");
            }
        }
    }
}
