use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::{info, instrument};

use crate::ansible::template_renderer::ConfigurationTemplateError;
use crate::ansible::AnsibleTemplateRenderer;
use crate::template::wrappers::ansible::inventory::{
    AnsibleHost, InventoryContext, InventoryContextError, SshPrivateKeyFile, SshPrivateKeyFileError,
};

/// Errors that can occur during Ansible template rendering step execution
#[derive(Error, Debug)]
pub enum RenderAnsibleTemplatesError {
    /// SSH key path parsing failed
    #[error("SSH key path parsing failed: {0}")]
    SshKeyPathError(#[from] SshPrivateKeyFileError),

    /// Inventory context creation failed
    #[error("Inventory context creation failed: {0}")]
    InventoryContextError(#[from] InventoryContextError),

    /// Template rendering failed
    #[error("Template rendering failed: {0}")]
    TemplateRenderingError(#[from] ConfigurationTemplateError),
}

/// Simple step that renders `Ansible` templates to the build directory with runtime variables
pub struct RenderAnsibleTemplatesStep {
    ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    ssh_key_path: PathBuf,
    instance_ip: IpAddr,
}

impl RenderAnsibleTemplatesStep {
    #[must_use]
    pub fn new(
        ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
        ssh_key_path: PathBuf,
        instance_ip: IpAddr,
    ) -> Self {
        Self {
            ansible_template_renderer,
            ssh_key_path,
            instance_ip,
        }
    }

    /// Execute the template rendering step
    ///
    /// # Errors
    ///
    /// Returns an error if the template rendering fails or if there are issues
    /// with the template manager or renderer.
    #[instrument(
        name = "render_ansible_templates",
        skip_all,
        fields(step_type = "rendering", template_type = "ansible")
    )]
    pub async fn execute(&self) -> Result<(), RenderAnsibleTemplatesError> {
        info!(
            step = "render_ansible_templates",
            "Rendering Ansible templates with runtime variables"
        );

        // Create inventory context with runtime variables
        let inventory_context = self.create_inventory_context()?;

        // Use the configuration renderer to handle all template rendering
        self.ansible_template_renderer
            .render(&inventory_context)
            .await?;

        info!(
            step = "render_ansible_templates",
            status = "success",
            "Ansible templates rendered successfully"
        );

        Ok(())
    }

    /// Create inventory context with runtime variables from instance data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SSH key path parsing fails
    /// - Inventory context creation fails
    fn create_inventory_context(&self) -> Result<InventoryContext, RenderAnsibleTemplatesError> {
        let host = AnsibleHost::from(self.instance_ip);
        let ssh_key = SshPrivateKeyFile::new(&self.ssh_key_path)?;

        InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .map_err(RenderAnsibleTemplatesError::from)
    }
}
