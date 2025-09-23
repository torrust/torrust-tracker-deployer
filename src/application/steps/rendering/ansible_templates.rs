//! Ansible template rendering step
//!
//! This module provides the `RenderAnsibleTemplatesStep` which handles rendering
//! of Ansible configuration templates with runtime variables like IP addresses
//! and SSH keys. This step prepares Ansible inventory and playbook files for
//! configuration management operations.
//!
//! ## Key Features
//!
//! - Dynamic template rendering with runtime variables (IP addresses, SSH keys)
//! - Ansible inventory generation with host information
//! - SSH key path processing and validation
//! - Comprehensive error handling with detailed context
//!
//! ## Usage Context
//!
//! This step is executed after infrastructure provisioning when instance IP
//! addresses are known, allowing for the generation of dynamic Ansible
//! configurations for remote host management.

use std::net::IpAddr;
use std::sync::Arc;

use thiserror::Error;
use tracing::{info, instrument};

use crate::infrastructure::adapters::ssh::credentials::SshCredentials;
use crate::infrastructure::ansible::template::renderer::ConfigurationTemplateError;
use crate::infrastructure::ansible::AnsibleTemplateRenderer;
use crate::infrastructure::template::wrappers::ansible::inventory::{
    AnsibleHost, AnsiblePort, AnsiblePortError, InventoryContext, InventoryContextError,
    SshPrivateKeyFile, SshPrivateKeyFileError,
};

/// Errors that can occur during Ansible template rendering step execution
#[derive(Error, Debug)]
pub enum RenderAnsibleTemplatesError {
    /// SSH key path parsing failed
    #[error("SSH key path parsing failed: {0}")]
    SshKeyPathError(#[from] SshPrivateKeyFileError),

    /// SSH port parsing failed
    #[error("SSH port parsing failed: {0}")]
    SshPortError(#[from] AnsiblePortError),

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
    ssh_credentials: SshCredentials,
    instance_ip: IpAddr,
    ssh_port: u16,
}

impl RenderAnsibleTemplatesStep {
    #[must_use]
    pub fn new(
        ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
        ssh_credentials: SshCredentials,
        instance_ip: IpAddr,
        ssh_port: u16,
    ) -> Self {
        Self {
            ansible_template_renderer,
            ssh_credentials,
            instance_ip,
            ssh_port,
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
        let ssh_key = SshPrivateKeyFile::new(&self.ssh_credentials.ssh_priv_key_path)?;
        let ssh_port = AnsiblePort::new(self.ssh_port)?;

        InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .with_ssh_port(ssh_port)
            .build()
            .map_err(RenderAnsibleTemplatesError::from)
    }
}
