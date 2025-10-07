//! Infrastructure provisioning command
//!
//! This module contains the `ProvisionCommand` which orchestrates the complete infrastructure
//! provisioning workflow including:
//!
//! - Template rendering for `OpenTofu` configuration
//! - Infrastructure planning and application via `OpenTofu`
//! - Instance information retrieval
//! - Ansible template rendering with dynamic VM data
//! - System readiness validation (cloud-init, SSH connectivity)
//!
//! The command handles the complex interaction between different deployment tools
//! and ensures proper sequencing of provisioning steps.

use std::net::IpAddr;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::application::steps::{
    ApplyInfrastructureStep, GetInstanceInfoStep, InitializeInfrastructureStep,
    PlanInfrastructureStep, RenderAnsibleTemplatesError, RenderAnsibleTemplatesStep,
    RenderOpenTofuTemplatesStep, ValidateInfrastructureStep, WaitForCloudInitStep,
    WaitForSSHConnectivityStep,
};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::{
    BaseFailureContext, ProvisionFailureContext, ProvisionStep,
};
use crate::domain::environment::{Created, Environment, Provisioned, Provisioning, TraceId};
#[allow(unused_imports)]
use crate::domain::{InstanceName, ProfileName};
use crate::infrastructure::external_tools::ansible::adapter::AnsibleClient;
use crate::infrastructure::external_tools::ansible::AnsibleTemplateRenderer;
use crate::infrastructure::external_tools::tofu::adapter::client::{InstanceInfo, OpenTofuError};
use crate::infrastructure::external_tools::tofu::{ProvisionTemplateError, TofuTemplateRenderer};
use crate::shared::command::CommandError;
use crate::shared::error::Traceable;
use crate::shared::ssh::{SshConnection, SshCredentials, SshError};

/// Comprehensive error type for the `ProvisionCommand`
#[derive(Debug, thiserror::Error)]
pub enum ProvisionCommandError {
    #[error("OpenTofu template rendering failed: {0}")]
    OpenTofuTemplateRendering(#[from] ProvisionTemplateError),

    #[error("Ansible template rendering failed: {0}")]
    AnsibleTemplateRendering(#[from] RenderAnsibleTemplatesError),

    #[error("OpenTofu command failed: {0}")]
    OpenTofu(#[from] OpenTofuError),

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("SSH connectivity failed: {0}")]
    SshConnectivity(#[from] SshError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),
}

impl crate::shared::Traceable for ProvisionCommandError {
    fn trace_format(&self) -> String {
        match self {
            Self::OpenTofuTemplateRendering(e) => {
                format!("ProvisionCommandError: OpenTofu template rendering failed - {e}")
            }
            Self::AnsibleTemplateRendering(e) => {
                format!("ProvisionCommandError: Ansible template rendering failed - {e}")
            }
            Self::OpenTofu(e) => {
                format!("ProvisionCommandError: OpenTofu command failed - {e}")
            }
            Self::Command(e) => {
                format!("ProvisionCommandError: Command execution failed - {e}")
            }
            Self::SshConnectivity(e) => {
                format!("ProvisionCommandError: SSH connectivity failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("ProvisionCommandError: Failed to persist environment state - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::OpenTofuTemplateRendering(e) => Some(e),
            Self::AnsibleTemplateRendering(e) => Some(e),
            Self::OpenTofu(e) => Some(e),
            Self::Command(e) => Some(e),
            Self::SshConnectivity(e) => Some(e),
            Self::StatePersistence(_) => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::OpenTofuTemplateRendering(_) | Self::AnsibleTemplateRendering(_) => {
                crate::shared::ErrorKind::TemplateRendering
            }
            Self::OpenTofu(_) => crate::shared::ErrorKind::InfrastructureOperation,
            Self::SshConnectivity(_) => crate::shared::ErrorKind::NetworkConnectivity,
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StatePersistence(_) => crate::shared::ErrorKind::StatePersistence,
        }
    }
}

/// `ProvisionCommand` orchestrates the complete infrastructure provisioning workflow
///
/// The `ProvisionCommand` orchestrates the complete infrastructure provisioning workflow.
///
/// This command handles all steps required to provision infrastructure:
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
/// The command integrates with the type-state pattern for environment lifecycle:
/// - Accepts `Environment<Created>` as input
/// - Transitions to `Environment<Provisioning>` at start
/// - Returns `Environment<Provisioned>` on success
/// - Transitions to `Environment<ProvisionFailed>` on error
///
/// State is persisted after each transition using the injected repository.
/// Persistence failures are logged but don't fail the command (state remains valid in memory).
pub struct ProvisionCommand {
    tofu_template_renderer: Arc<TofuTemplateRenderer>,
    ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    ansible_client: Arc<AnsibleClient>,
    opentofu_client:
        Arc<crate::infrastructure::external_tools::tofu::adapter::client::OpenTofuClient>,
    clock: Arc<dyn crate::shared::Clock>,
    repository: Arc<dyn EnvironmentRepository>,
}

impl ProvisionCommand {
    /// Create a new `ProvisionCommand`
    #[must_use]
    pub fn new(
        tofu_template_renderer: Arc<TofuTemplateRenderer>,
        ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
        ansible_client: Arc<AnsibleClient>,
        opentofu_client: Arc<
            crate::infrastructure::external_tools::tofu::adapter::client::OpenTofuClient,
        >,
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
    ) -> Result<Environment<Provisioned>, ProvisionCommandError> {
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
    ) -> Result<(Environment<Provisioned>, IpAddr), (ProvisionCommandError, ProvisionStep)> {
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
        self.render_ansible_templates(ssh_credentials, instance_ip)
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
    async fn render_opentofu_templates(&self) -> Result<(), ProvisionCommandError> {
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
    fn create_instance(&self) -> Result<(), ProvisionCommandError> {
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
    fn get_instance_info(&self) -> Result<InstanceInfo, ProvisionCommandError> {
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
    ) -> Result<(), ProvisionCommandError> {
        let socket_addr = std::net::SocketAddr::new(instance_ip, 22); // Default SSH port for VMs
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
    ) -> Result<(), ProvisionCommandError> {
        let ssh_connection = SshConnection::with_default_port(ssh_credentials.clone(), instance_ip);
        WaitForSSHConnectivityStep::new(ssh_connection)
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
    fn build_failure_context(
        &self,
        environment: &Environment<Provisioning>,
        error: &ProvisionCommandError,
        current_step: ProvisionStep,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> ProvisionFailureContext {
        use crate::infrastructure::trace::ProvisionTraceWriter;

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

        // Build initial context without trace file path
        let mut context = ProvisionFailureContext {
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
        let writer = ProvisionTraceWriter::new(traces_dir, Arc::clone(&self.clock));

        if let Ok(trace_file) = writer.write_trace(&context, error) {
            context.base.trace_file_path = Some(trace_file);
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::shared::Username;

    fn create_test_environment(_temp_dir: &TempDir) -> (Environment<Provisioning>, TempDir) {
        use crate::domain::environment::testing::EnvironmentTestBuilder;

        let (env, _data_dir, _build_dir, env_temp_dir) = EnvironmentTestBuilder::new()
            .with_name("test-env")
            .build_with_custom_paths();

        // Environment is created with paths inside env_temp_dir
        // which will be automatically cleaned up when env_temp_dir is dropped

        (env.start_provisioning(), env_temp_dir)
    }

    /// Test builder for `ProvisionCommand` that manages dependencies and lifecycle
    ///
    /// This builder simplifies test setup by:
    /// - Managing `TempDir` lifecycle
    /// - Providing sensible defaults for all dependencies
    /// - Allowing selective customization of dependencies
    /// - Returning only the command and necessary test artifacts
    pub struct ProvisionCommandTestBuilder {
        temp_dir: TempDir,
        ssh_credentials: Option<SshCredentials>,
    }

    impl ProvisionCommandTestBuilder {
        /// Create a new test builder with default configuration
        pub fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            Self {
                temp_dir,
                ssh_credentials: None,
            }
        }

        /// Customize SSH credentials (optional - uses defaults if not called)
        #[allow(dead_code)]
        pub fn with_ssh_credentials(mut self, credentials: SshCredentials) -> Self {
            self.ssh_credentials = Some(credentials);
            self
        }

        /// Build the `ProvisionCommand` with all dependencies
        ///
        /// Returns: (`command`, `temp_dir`, `ssh_credentials`)
        /// The `temp_dir` must be kept alive for the duration of the test.
        pub fn build(self) -> (ProvisionCommand, TempDir, SshCredentials) {
            let template_manager = Arc::new(crate::domain::template::TemplateManager::new(
                self.temp_dir.path(),
            ));

            // Use provided SSH credentials or create defaults
            let ssh_credentials = self.ssh_credentials.unwrap_or_else(|| {
                let ssh_key_path = self.temp_dir.path().join("test_key");
                let ssh_pub_key_path = self.temp_dir.path().join("test_key.pub");
                SshCredentials::new(
                    ssh_key_path,
                    ssh_pub_key_path,
                    Username::new("test_user").unwrap(),
                )
            });

            let tofu_renderer = Arc::new(TofuTemplateRenderer::new(
                template_manager.clone(),
                self.temp_dir.path(),
                ssh_credentials.clone(),
                InstanceName::new("torrust-tracker-vm".to_string())
                    .expect("Valid hardcoded instance name"),
                ProfileName::new("default-profile".to_string())
                    .expect("Valid hardcoded profile name"),
            ));

            let ansible_renderer = Arc::new(AnsibleTemplateRenderer::new(
                self.temp_dir.path(),
                template_manager,
            ));

            let ansible_client = Arc::new(AnsibleClient::new(self.temp_dir.path()));

            let opentofu_client = Arc::new(
                crate::infrastructure::external_tools::tofu::adapter::client::OpenTofuClient::new(
                    self.temp_dir.path(),
                ),
            );

            let clock: Arc<dyn crate::shared::Clock> = Arc::new(crate::shared::SystemClock);

            let repository_factory =
                crate::infrastructure::persistence::repository_factory::RepositoryFactory::new(
                    std::time::Duration::from_secs(30),
                );
            let repository = repository_factory.create(self.temp_dir.path().to_path_buf());

            let command = ProvisionCommand::new(
                tofu_renderer,
                ansible_renderer,
                ansible_client,
                opentofu_client,
                clock,
                repository,
            );

            (command, self.temp_dir, ssh_credentials)
        }
    }

    #[test]
    fn it_should_create_provision_command_with_all_dependencies() {
        let (command, _temp_dir, _ssh_credentials) = ProvisionCommandTestBuilder::new().build();

        // Verify the command was created (basic structure test)
        // This test just verifies that the command can be created with the dependencies
        assert_eq!(Arc::strong_count(&command.tofu_template_renderer), 1);
        assert_eq!(Arc::strong_count(&command.ansible_template_renderer), 1);
    }

    #[test]
    fn it_should_have_correct_error_type_conversions() {
        // Test that all error types can convert to ProvisionCommandError
        let template_error = ProvisionTemplateError::DirectoryCreationFailed {
            directory: "/test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };
        let provision_error: ProvisionCommandError = template_error.into();
        drop(provision_error);

        let command_error = CommandError::StartupFailed {
            command: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        };
        let opentofu_error = OpenTofuError::CommandError(command_error);
        let provision_error: ProvisionCommandError = opentofu_error.into();
        drop(provision_error);

        let command_error_direct = CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        };
        let provision_error: ProvisionCommandError = command_error_direct.into();
        drop(provision_error);

        let ssh_error = SshError::ConnectivityTimeout {
            host_ip: "127.0.0.1".to_string(),
            attempts: 5,
            timeout_seconds: 30,
        };
        let provision_error: ProvisionCommandError = ssh_error.into();
        drop(provision_error);
    }

    #[test]
    fn it_should_build_failure_context_from_opentofu_template_error() {
        use chrono::{TimeZone, Utc};

        let (command, temp_dir, _ssh_credentials) = ProvisionCommandTestBuilder::new().build();

        let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

        let error = ProvisionCommandError::OpenTofuTemplateRendering(
            ProvisionTemplateError::DirectoryCreationFailed {
                directory: "/test".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
            },
        );

        let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let current_step = ProvisionStep::RenderOpenTofuTemplates;
        let context = command.build_failure_context(&environment, &error, current_step, started_at);
        assert_eq!(context.failed_step, ProvisionStep::RenderOpenTofuTemplates);
        assert_eq!(
            context.error_kind,
            crate::shared::ErrorKind::TemplateRendering
        );
        assert_eq!(context.base.execution_started_at, started_at);
    }

    // Note: We don't test AnsibleTemplateRendering errors directly as the error types are complex
    // and deeply nested. The build_failure_context method handles them by matching on the
    // ProvisionCommandError::AnsibleTemplateRendering variant, which is sufficient for
    // error context generation.

    #[test]
    fn it_should_build_failure_context_from_ssh_connectivity_error() {
        use chrono::{TimeZone, Utc};

        let (command, temp_dir, _ssh_credentials) = ProvisionCommandTestBuilder::new().build();

        let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

        let error = ProvisionCommandError::SshConnectivity(SshError::ConnectivityTimeout {
            host_ip: "127.0.0.1".to_string(),
            attempts: 5,
            timeout_seconds: 30,
        });

        let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let current_step = ProvisionStep::WaitSshConnectivity;
        let context = command.build_failure_context(&environment, &error, current_step, started_at);
        assert_eq!(context.failed_step, ProvisionStep::WaitSshConnectivity);
        assert_eq!(
            context.error_kind,
            crate::shared::ErrorKind::NetworkConnectivity
        );
        assert_eq!(context.base.execution_started_at, started_at);
    }

    #[test]
    fn it_should_build_failure_context_from_command_error() {
        use chrono::{TimeZone, Utc};

        let (command, temp_dir, _ssh_credentials) = ProvisionCommandTestBuilder::new().build();

        let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

        let error = ProvisionCommandError::Command(CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        });

        let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let current_step = ProvisionStep::CloudInitWait;
        let context = command.build_failure_context(&environment, &error, current_step, started_at);
        assert_eq!(context.failed_step, ProvisionStep::CloudInitWait);
        assert_eq!(
            context.error_kind,
            crate::shared::ErrorKind::CommandExecution
        );
        assert_eq!(context.base.execution_started_at, started_at);
    }

    #[test]
    fn it_should_build_failure_context_from_opentofu_error() {
        use chrono::{TimeZone, Utc};

        let (command, temp_dir, _ssh_credentials) = ProvisionCommandTestBuilder::new().build();

        let (environment, _env_temp_dir) = create_test_environment(&temp_dir);

        let opentofu_error = OpenTofuError::CommandError(CommandError::ExecutionFailed {
            command: "tofu init".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "init failed".to_string(),
        });

        let error = ProvisionCommandError::OpenTofu(opentofu_error);

        let started_at = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let current_step = ProvisionStep::OpenTofuInit;
        let context = command.build_failure_context(&environment, &error, current_step, started_at);
        assert_eq!(context.failed_step, ProvisionStep::OpenTofuInit);
        assert_eq!(
            context.error_kind,
            crate::shared::ErrorKind::InfrastructureOperation
        );
        assert_eq!(context.base.execution_started_at, started_at);
    }
}
