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
use crate::infrastructure::adapters::ansible::AnsibleClient;
#[allow(unused_imports)]
use crate::infrastructure::adapters::lxd::InstanceName;
use crate::infrastructure::adapters::opentofu::client::{InstanceInfo, OpenTofuError};
use crate::infrastructure::ansible::AnsibleTemplateRenderer;
use crate::infrastructure::tofu::{ProvisionTemplateError, TofuTemplateRenderer};
use crate::shared::executor::CommandError;
use crate::shared::ssh::{credentials::SshCredentials, SshError};

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
pub struct ProvisionCommand {
    tofu_template_renderer: Arc<TofuTemplateRenderer>,
    ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    ansible_client: Arc<AnsibleClient>,
    opentofu_client: Arc<crate::infrastructure::adapters::opentofu::client::OpenTofuClient>,
    ssh_credentials: SshCredentials,
}

impl ProvisionCommand {
    /// Create a new `ProvisionCommand`
    #[must_use]
    pub fn new(
        tofu_template_renderer: Arc<TofuTemplateRenderer>,
        ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
        ansible_client: Arc<AnsibleClient>,
        opentofu_client: Arc<crate::infrastructure::adapters::opentofu::client::OpenTofuClient>,
        ssh_credentials: SshCredentials,
    ) -> Self {
        Self {
            tofu_template_renderer,
            ansible_template_renderer,
            ansible_client,
            opentofu_client,
            ssh_credentials,
        }
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

    /// Execute the complete provisioning workflow
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
    pub async fn execute(&self) -> Result<IpAddr, ProvisionCommandError> {
        info!(
            command = "provision",
            "Starting complete infrastructure provisioning workflow"
        );

        RenderOpenTofuTemplatesStep::new(Arc::clone(&self.tofu_template_renderer))
            .execute()
            .await?;

        self.create_instance()?;

        let instance_info: InstanceInfo =
            GetInstanceInfoStep::new(Arc::clone(&self.opentofu_client)).execute()?;
        let instance_ip = instance_info.ip_address;

        let socket_addr = std::net::SocketAddr::new(instance_ip, 22); // Default SSH port for VMs
        RenderAnsibleTemplatesStep::new(
            Arc::clone(&self.ansible_template_renderer),
            self.ssh_credentials.clone(),
            socket_addr,
        )
        .execute()
        .await?;

        let ssh_connection = self.ssh_credentials.clone().with_host(instance_ip);
        WaitForSSHConnectivityStep::new(ssh_connection)
            .execute()
            .await?;

        WaitForCloudInitStep::new(Arc::clone(&self.ansible_client)).execute()?;

        info!(
            command = "provision",
            instance_ip = %instance_ip,
            "Infrastructure provisioning completed successfully"
        );

        Ok(instance_ip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper function to create mock dependencies for testing
    fn create_mock_dependencies() -> (
        Arc<TofuTemplateRenderer>,
        Arc<AnsibleTemplateRenderer>,
        Arc<AnsibleClient>,
        Arc<crate::infrastructure::adapters::opentofu::client::OpenTofuClient>,
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
            "testuser".to_string(),
        );

        let tofu_renderer = Arc::new(TofuTemplateRenderer::new(
            template_manager.clone(),
            temp_dir.path(),
            ssh_credentials.clone(),
            InstanceName::new("torrust-tracker-vm".to_string())
                .expect("Valid hardcoded instance name"), // TODO: Make this configurable in Phase 3
        ));

        let ansible_renderer = Arc::new(AnsibleTemplateRenderer::new(
            temp_dir.path(),
            template_manager,
        ));

        let ansible_client = Arc::new(AnsibleClient::new(temp_dir.path()));

        let opentofu_client = Arc::new(
            crate::infrastructure::adapters::opentofu::client::OpenTofuClient::new(temp_dir.path()),
        );

        let ssh_key_path = temp_dir.path().join("test_key");
        let ssh_pub_key_path = temp_dir.path().join("test_key.pub");
        let ssh_credentials =
            SshCredentials::new(ssh_key_path, ssh_pub_key_path, "test_user".to_string());

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
            ssh_credentials,
            _temp_dir,
        ) = create_mock_dependencies();

        let command = ProvisionCommand::new(
            tofu_renderer,
            ansible_renderer,
            ansible_client,
            opentofu_client,
            ssh_credentials,
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
}
