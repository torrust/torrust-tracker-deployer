//! Provision command handler implementation

use std::net::IpAddr;
use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::ProvisionCommandHandlerError;
use crate::adapters::ansible::AnsibleClient;
use crate::adapters::ssh::{SshConfig, SshCredentials};
use crate::adapters::tofu::client::InstanceInfo;
use crate::application::steps::{
    ApplyInfrastructureStep, GetInstanceInfoStep, InitializeInfrastructureStep,
    PlanInfrastructureStep, RenderAnsibleTemplatesStep, RenderOpenTofuTemplatesStep,
    ValidateInfrastructureStep, WaitForCloudInitStep, WaitForSSHConnectivityStep,
};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::{ProvisionFailureContext, ProvisionStep};
use crate::domain::environment::{Created, Environment, Provisioned, Provisioning};
use crate::infrastructure::external_tools::ansible::AnsibleTemplateRenderer;
use crate::infrastructure::external_tools::tofu::TofuTemplateRenderer;
use crate::shared::error::Traceable;

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
    pub(crate) tofu_template_renderer: Arc<TofuTemplateRenderer>,
    pub(crate) ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    pub(crate) ansible_client: Arc<AnsibleClient>,
    pub(crate) opentofu_client: Arc<crate::adapters::tofu::client::OpenTofuClient>,
    pub(crate) clock: Arc<dyn crate::shared::Clock>,
    pub(crate) repository: Arc<dyn EnvironmentRepository>,
}

impl ProvisionCommandHandler {
    /// Create a new `ProvisionCommandHandler`
    #[must_use]
    pub fn new(
        tofu_template_renderer: Arc<TofuTemplateRenderer>,
        ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
        ansible_client: Arc<AnsibleClient>,
        opentofu_client: Arc<crate::adapters::tofu::client::OpenTofuClient>,
        clock: Arc<dyn crate::shared::Clock>,
        repository: Arc<dyn EnvironmentRepository>,
    ) -> Self {
        Self {
            tofu_template_renderer,
            ansible_template_renderer,
            ansible_client,
            opentofu_client,
            clock,
            repository,
        }
    }

    /// Execute the complete provisioning workflow
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in `Created` state to provision
    ///
    /// # Returns
    ///
    /// Returns a tuple of the provisioned environment and its IP address
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the provisioning workflow fails:
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
            environment = %environment.name()
        )
    )]
    pub async fn execute(
        &self,
        environment: Environment<Created>,
    ) -> Result<Environment<Provisioned>, ProvisionCommandHandlerError> {
        info!(
            command = "provision",
            environment = %environment.name(),
            "Starting complete infrastructure provisioning workflow"
        );

        // Capture start time before transitioning to Provisioning state
        let started_at = self.clock.now();

        // Transition to Provisioning state
        let environment = environment.start_provisioning();

        // Persist intermediate state
        self.repository.save(&environment.clone().into_any())?;

        // Execute provisioning steps with explicit step tracking
        // This allows us to know exactly which step failed if an error occurs
        match self.execute_provisioning_with_tracking(&environment).await {
            Ok((provisioned, instance_ip)) => {
                // Store instance IP in the environment context
                let provisioned = provisioned.with_instance_ip(instance_ip);

                // Persist final state
                self.repository.save(&provisioned.clone().into_any())?;

                info!(
                    command = "provision",
                    environment = %provisioned.name(),
                    instance_ip = ?provisioned.instance_ip(),
                    "Infrastructure provisioning completed successfully"
                );

                Ok(provisioned)
            }
            Err((e, current_step)) => {
                // Transition to error state with structured context
                // current_step contains the step that was executing when the error occurred
                let context =
                    self.build_failure_context(&environment, &e, current_step, started_at);
                let failed = environment.provision_failed(context);

                // Persist error state
                self.repository.save(&failed.clone().into_any())?;

                Err(e)
            }
        }
    }

    /// Execute the provisioning steps with step tracking
    ///
    /// This method executes all provisioning steps while tracking which step is currently
    /// being executed. If an error occurs, it returns both the error and the step that
    /// was being executed, enabling accurate failure context generation.
    ///
    /// # Errors
    ///
    /// Returns a tuple of (error, `current_step`) if any provisioning step fails
    ///
    /// # Returns
    ///
    /// Returns a tuple of:
    /// - The provisioned environment
    /// - The instance IP address
    async fn execute_provisioning_with_tracking(
        &self,
        environment: &Environment<Provisioning>,
    ) -> Result<(Environment<Provisioned>, IpAddr), (ProvisionCommandHandlerError, ProvisionStep)>
    {
        let ssh_credentials = environment.ssh_credentials();

        // Track current step and execute each step
        // If an error occurs, we return it along with the current step

        let current_step = ProvisionStep::RenderOpenTofuTemplates;
        self.render_opentofu_templates()
            .await
            .map_err(|e| (e, current_step))?;

        let current_step = ProvisionStep::OpenTofuInit;
        self.create_instance().map_err(|e| (e, current_step))?;

        let current_step = ProvisionStep::GetInstanceInfo;
        let instance_info = self.get_instance_info().map_err(|e| (e, current_step))?;
        let instance_ip = instance_info.ip_address;

        let current_step = ProvisionStep::RenderAnsibleTemplates;
        let ssh_port = environment.ssh_port();
        self.render_ansible_templates(ssh_credentials, instance_ip, ssh_port)
            .await
            .map_err(|e| (e, current_step))?;

        let current_step = ProvisionStep::WaitSshConnectivity;
        self.wait_for_readiness(ssh_credentials, instance_ip)
            .await
            .map_err(|e| (e, current_step))?;

        // Transition to Provisioned state
        let provisioned = environment.clone().provisioned();

        Ok((provisioned, instance_ip))
    }

    // Private helper methods - organized from higher to lower level of abstraction

    /// Render `OpenTofu` templates
    ///
    /// Generates `OpenTofu` configuration files from templates.
    ///
    /// # Errors
    ///
    /// Returns an error if template rendering fails
    async fn render_opentofu_templates(&self) -> Result<(), ProvisionCommandHandlerError> {
        RenderOpenTofuTemplatesStep::new(Arc::clone(&self.tofu_template_renderer))
            .execute()
            .await?;
        Ok(())
    }

    /// Create the infrastructure instance using `OpenTofu`
    ///
    /// This method handles the `OpenTofu` workflow:
    /// - Initialize `OpenTofu` configuration
    /// - Validate configuration syntax and consistency
    /// - Plan the infrastructure changes
    /// - Apply the infrastructure changes
    ///
    /// # Errors
    ///
    /// Returns an error if any `OpenTofu` operation fails
    fn create_instance(&self) -> Result<(), ProvisionCommandHandlerError> {
        InitializeInfrastructureStep::new(Arc::clone(&self.opentofu_client)).execute()?;
        ValidateInfrastructureStep::new(Arc::clone(&self.opentofu_client)).execute()?;
        PlanInfrastructureStep::new(Arc::clone(&self.opentofu_client)).execute()?;
        ApplyInfrastructureStep::new(Arc::clone(&self.opentofu_client)).execute()?;
        Ok(())
    }

    /// Get instance information from `OpenTofu`
    ///
    /// Retrieves information about the provisioned instance, including its IP address.
    ///
    /// # Errors
    ///
    /// Returns an error if instance information cannot be retrieved
    fn get_instance_info(&self) -> Result<InstanceInfo, ProvisionCommandHandlerError> {
        let instance_info =
            GetInstanceInfoStep::new(Arc::clone(&self.opentofu_client)).execute()?;
        Ok(instance_info)
    }

    /// Render Ansible templates with runtime IP
    ///
    /// Generates Ansible inventory and configuration files with the actual instance IP.
    ///
    /// # Arguments
    ///
    /// * `ssh_credentials` - SSH credentials for connecting to the instance
    /// * `instance_ip` - IP address of the provisioned instance
    ///
    /// # Errors
    ///
    /// Returns an error if template rendering fails
    async fn render_ansible_templates(
        &self,
        ssh_credentials: &SshCredentials,
        instance_ip: IpAddr,
        ssh_port: u16,
    ) -> Result<(), ProvisionCommandHandlerError> {
        let socket_addr = std::net::SocketAddr::new(instance_ip, ssh_port);
        RenderAnsibleTemplatesStep::new(
            Arc::clone(&self.ansible_template_renderer),
            ssh_credentials.clone(),
            socket_addr,
        )
        .execute()
        .await?;
        Ok(())
    }

    /// Wait for system readiness
    ///
    /// Waits for SSH connectivity and cloud-init completion.
    ///
    /// # Arguments
    ///
    /// * `ssh_credentials` - SSH credentials for connecting to the instance
    /// * `instance_ip` - IP address of the provisioned instance
    ///
    /// # Errors
    ///
    /// Returns an error if SSH connectivity fails or cloud-init does not complete
    async fn wait_for_readiness(
        &self,
        ssh_credentials: &SshCredentials,
        instance_ip: IpAddr,
    ) -> Result<(), ProvisionCommandHandlerError> {
        let ssh_config = SshConfig::with_default_port(ssh_credentials.clone(), instance_ip);
        WaitForSSHConnectivityStep::new(ssh_config)
            .execute()
            .await?;

        WaitForCloudInitStep::new(Arc::clone(&self.ansible_client)).execute()?;

        Ok(())
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
    pub(crate) fn build_failure_context(
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
}
