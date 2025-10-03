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
#[allow(unused_imports)]
use crate::domain::{InstanceName, ProfileName};
use crate::infrastructure::external_tools::ansible::adapter::AnsibleClient;
use crate::infrastructure::external_tools::ansible::AnsibleTemplateRenderer;
use crate::infrastructure::external_tools::tofu::adapter::client::{InstanceInfo, OpenTofuError};
use crate::infrastructure::external_tools::tofu::{ProvisionTemplateError, TofuTemplateRenderer};
use crate::shared::command::CommandError;
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
/// # Refactoring Note (Phase 5 preparation)
///
/// The command structure has been refactored to extract logical steps:
/// 1. Render `OpenTofu` templates
/// 2. Create infrastructure via `OpenTofu`
/// 3. Get instance information
/// 4. Render Ansible templates with runtime IP
/// 5. Wait for system readiness
///
/// This makes it easier to integrate state management with clear persistence points.
///
/// # TODO(Phase 5): State Management Integration
///
/// When implementing state management in Phase 5:
/// 1. Add `repository: Arc<dyn EnvironmentRepository>` field
/// 2. Update `new()` to accept repository parameter
/// 3. Change `execute()` to accept `Environment<Created>` instead of `&SshCredentials`
/// 4. Return `Environment<Provisioned>` instead of `IpAddr`
/// 5. Add state transitions and persistence calls at marked points in `execute()`
pub struct ProvisionCommand {
    tofu_template_renderer: Arc<TofuTemplateRenderer>,
    ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    ansible_client: Arc<AnsibleClient>,
    opentofu_client:
        Arc<crate::infrastructure::external_tools::tofu::adapter::client::OpenTofuClient>,
    // TODO(Phase 5): Add repository field here
    // repository: Arc<dyn EnvironmentRepository>,
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
    ) -> Self {
        Self {
            tofu_template_renderer,
            ansible_template_renderer,
            ansible_client,
            opentofu_client,
        }
    }

    /// Execute the complete provisioning workflow
    ///
    /// # Arguments
    ///
    /// * `ssh_credentials` - SSH credentials for connecting to the instance
    ///
    /// # Returns
    ///
    /// Returns the IP address of the provisioned instance
    ///
    /// # Errors
    ///
    /// Returns an error if any step in the provisioning workflow fails:
    /// * Template rendering fails
    /// * `OpenTofu` initialization, planning, or apply fails
    /// * Unable to retrieve instance information
    /// * SSH connectivity cannot be established
    /// * Cloud-init does not complete successfully
    #[instrument(
        name = "provision_command",
        skip_all,
        fields(command_type = "provision")
    )]
    pub async fn execute(
        &self,
        ssh_credentials: &SshCredentials,
    ) -> Result<IpAddr, ProvisionCommandError> {
        info!(
            command = "provision",
            "Starting complete infrastructure provisioning workflow"
        );

        // TODO(Phase 5): Transition to Provisioning state and persist
        // let environment = environment.start_provisioning();
        // self.persist_state(&environment)?;

        self.render_opentofu_templates().await?;

        self.create_instance()?;

        // TODO(Phase 5): Consider persisting state after infrastructure creation

        let instance_info = self.get_instance_info()?;
        let instance_ip = instance_info.ip_address;

        self.render_ansible_templates(ssh_credentials, instance_ip)
            .await?;

        self.wait_for_readiness(ssh_credentials, instance_ip)
            .await?;

        // TODO(Phase 5): Transition to Provisioned state and persist
        // let provisioned_env = environment.complete_provisioning(instance_ip);
        // self.persist_state(&provisioned_env)?;
        // return Ok(provisioned_env);

        info!(
            command = "provision",
            instance_ip = %instance_ip,
            "Infrastructure provisioning completed successfully"
        );

        Ok(instance_ip)
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

    /// Extract the failed step name from a provisioning error
    ///
    /// This helper method provides context about which step failed during provisioning.
    /// It will be used in Phase 5 to track failed steps in error states.
    ///
    /// # Arguments
    ///
    /// * `error` - The provisioning error to extract step information from
    ///
    /// # Returns
    ///
    /// A string identifying the failed step
    #[allow(dead_code)] // Will be used in Phase 5 Subtask 3
    fn extract_failed_step(&self, error: &ProvisionCommandError) -> String {
        match error {
            ProvisionCommandError::OpenTofuTemplateRendering(_) => {
                "render_opentofu_templates".to_string()
            }
            ProvisionCommandError::OpenTofu(e) => {
                format!("opentofu_{}", self.extract_opentofu_step(e))
            }
            ProvisionCommandError::AnsibleTemplateRendering(_) => {
                "render_ansible_templates".to_string()
            }
            ProvisionCommandError::SshConnectivity(_) => "wait_ssh_connectivity".to_string(),
            ProvisionCommandError::Command(_) => "cloud_init_wait".to_string(),
        }
    }

    /// Extract the specific `OpenTofu` operation that failed
    ///
    /// This helper method provides more granular context about `OpenTofu` failures.
    ///
    /// # Arguments
    ///
    /// * `_error` - The `OpenTofu` error (currently unused, but could be used for more sophisticated extraction)
    ///
    /// # Returns
    ///
    /// A string identifying the `OpenTofu` operation (currently returns generic "operation")
    #[allow(dead_code)] // Will be used in Phase 5 Subtask 3
    #[allow(clippy::unused_self)] // Will use self in future implementations
    fn extract_opentofu_step(&self, _error: &OpenTofuError) -> String {
        // Could be more sophisticated based on error variant in the future
        "operation".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::shared::Username;

    // Helper function to create mock dependencies for testing
    fn create_mock_dependencies() -> (
        Arc<TofuTemplateRenderer>,
        Arc<AnsibleTemplateRenderer>,
        Arc<AnsibleClient>,
        Arc<crate::infrastructure::external_tools::tofu::adapter::client::OpenTofuClient>,
        SshCredentials,
        TempDir,
    ) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_manager = Arc::new(crate::domain::template::TemplateManager::new(
            temp_dir.path(),
        ));

        let ssh_credentials = SshCredentials::new(
            "dummy_key".into(),
            "dummy_key.pub".into(),
            Username::new("testuser").unwrap(),
        );

        let tofu_renderer = Arc::new(TofuTemplateRenderer::new(
            template_manager.clone(),
            temp_dir.path(),
            ssh_credentials.clone(),
            InstanceName::new("torrust-tracker-vm".to_string())
                .expect("Valid hardcoded instance name"), // TODO: Make this configurable in Phase 3
            ProfileName::new("default-profile".to_string()).expect("Valid hardcoded profile name"), // TODO: Make this configurable in Phase 3
        ));

        let ansible_renderer = Arc::new(AnsibleTemplateRenderer::new(
            temp_dir.path(),
            template_manager,
        ));

        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path()));

        let opentofu_client = Arc::new(
            crate::infrastructure::external_tools::tofu::adapter::client::OpenTofuClient::new(
                temp_dir.path(),
            ),
        );

        let ssh_key_path = temp_dir.path().join("test_key");
        let ssh_pub_key_path = temp_dir.path().join("test_key.pub");
        let ssh_credentials = SshCredentials::new(
            ssh_key_path,
            ssh_pub_key_path,
            Username::new("test_user").unwrap(),
        );

        (
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
            ssh_credentials,
            temp_dir,
        )
    }

    #[test]
    fn it_should_create_provision_command_with_all_dependencies() {
        let (
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
            _ssh_credentials,
            _temp_dir,
        ) = create_mock_dependencies();

        let command = ProvisionCommand::new(
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
        );

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
    fn it_should_extract_failed_step_from_opentofu_template_error() {
        let (
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
            _ssh_credentials,
            _temp_dir,
        ) = create_mock_dependencies();

        let command = ProvisionCommand::new(
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
        );

        let error = ProvisionCommandError::OpenTofuTemplateRendering(
            ProvisionTemplateError::DirectoryCreationFailed {
                directory: "/test".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
            },
        );

        let step_name = command.extract_failed_step(&error);
        assert_eq!(step_name, "render_opentofu_templates");
    }

    // Note: We don't test AnsibleTemplateRendering errors directly as the error types are complex
    // and deeply nested. The extract_failed_step method handles them by matching on the
    // ProvisionCommandError::AnsibleTemplateRendering variant, which is sufficient for
    // Phase 5 integration.

    #[test]
    fn it_should_extract_failed_step_from_ssh_connectivity_error() {
        let (
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
            _ssh_credentials,
            _temp_dir,
        ) = create_mock_dependencies();

        let command = ProvisionCommand::new(
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
        );

        let error = ProvisionCommandError::SshConnectivity(SshError::ConnectivityTimeout {
            host_ip: "127.0.0.1".to_string(),
            attempts: 5,
            timeout_seconds: 30,
        });

        let step_name = command.extract_failed_step(&error);
        assert_eq!(step_name, "wait_ssh_connectivity");
    }

    #[test]
    fn it_should_extract_failed_step_from_command_error() {
        let (
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
            _ssh_credentials,
            _temp_dir,
        ) = create_mock_dependencies();

        let command = ProvisionCommand::new(
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
        );

        let error = ProvisionCommandError::Command(CommandError::ExecutionFailed {
            command: "test".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "test error".to_string(),
        });

        let step_name = command.extract_failed_step(&error);
        assert_eq!(step_name, "cloud_init_wait");
    }

    #[test]
    fn it_should_extract_failed_step_from_opentofu_error() {
        let (
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
            _ssh_credentials,
            _temp_dir,
        ) = create_mock_dependencies();

        let command = ProvisionCommand::new(
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
        );

        let opentofu_error = OpenTofuError::CommandError(CommandError::ExecutionFailed {
            command: "tofu init".to_string(),
            exit_code: "1".to_string(),
            stdout: String::new(),
            stderr: "init failed".to_string(),
        });

        let error = ProvisionCommandError::OpenTofu(opentofu_error);

        let step_name = command.extract_failed_step(&error);
        assert_eq!(step_name, "opentofu_operation");
    }
}
