use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use tracing::info;

use crate::ansible::template_renderer::ConfigurationTemplateError;
use crate::ansible::AnsibleTemplateRenderer;
use crate::template::wrappers::ansible::inventory::{
    AnsibleHost, InventoryContext, SshPrivateKeyFile,
};

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
    pub async fn execute(&self) -> Result<(), anyhow::Error> {
        info!(
            step = "render_ansible_templates",
            stage = 3,
            "Rendering Ansible templates with runtime variables"
        );

        // Create inventory context with runtime variables
        let inventory_context = {
            let host = AnsibleHost::from(self.instance_ip);
            let ssh_key = SshPrivateKeyFile::new(&self.ssh_key_path)
                .context("Failed to parse SSH key path")?;

            InventoryContext::builder()
                .with_host(host)
                .with_ssh_priv_key_path(ssh_key)
                .build()
                .context("Failed to create InventoryContext")?
        };

        // Use the configuration renderer to handle all template rendering
        self.ansible_template_renderer
            .render(&inventory_context)
            .await
            .map_err(|e: ConfigurationTemplateError| anyhow::anyhow!(e))?;

        info!(
            step = "render_ansible_templates",
            stage = 3,
            status = "success",
            "Ansible templates rendered successfully"
        );

        Ok(())
    }
}
