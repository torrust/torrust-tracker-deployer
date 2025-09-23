//! Docker infrastructure provisioning task for E2E testing
//!
//! This module provides the E2E testing task for simulating infrastructure provisioning
//! when using Docker containers instead of VMs. It performs only the necessary steps
//! that would normally happen after instance creation in the full provision workflow.
//!
//! ## Key Operations
//!
//! - Render Ansible templates with the container's IP and SSH port
//! - Note: SSH connectivity is already established by the container setup
//!
//! ## Integration
//!
//! This task replaces the full `ProvisionCommand` workflow for container-based testing,
//! skipping `OpenTofu` operations and only performing the post-instance-creation steps
//! needed for Ansible configuration.

use std::net::IpAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::info;

use crate::application::steps::RenderAnsibleTemplatesStep;
use crate::config::SshCredentials;
use crate::infrastructure::ansible::AnsibleTemplateRenderer;

/// Provision Docker-based infrastructure for E2E testing
///
/// This function simulates the post-instance-creation steps of the provision workflow
/// specifically for Docker containers. It renders Ansible templates with the container's
/// connection details. SSH connectivity is assumed to be already established by the
/// container startup process.
///
/// # Arguments
///
/// * `ansible_template_renderer` - Renderer for creating Ansible inventory and configuration
/// * `ssh_credentials` - SSH credentials for connecting to the container
/// * `container_ip` - IP address of the container (typically 127.0.0.1)
/// * `ssh_port` - SSH port mapped from the container
///
/// # Errors
///
/// Returns an error if:
/// - Ansible template rendering fails
pub async fn provision_docker_infrastructure(
    ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    ssh_credentials: SshCredentials,
    container_ip: IpAddr,
    ssh_port: u16,
) -> Result<()> {
    info!(
        container_ip = %container_ip,
        ssh_port = ssh_port,
        "Starting Docker infrastructure provisioning simulation"
    );

    // Step 1: Render Ansible templates with container connection details
    info!("Rendering Ansible templates for container");
    RenderAnsibleTemplatesStep::new(
        ansible_template_renderer,
        ssh_credentials,
        container_ip,
        ssh_port,
    )
    .execute()
    .await
    .context("Failed to render Ansible templates for container")?;

    // Note: SSH connectivity check is skipped for Docker containers since
    // the container setup process already ensures SSH is ready and accessible

    info!(
        container_ip = %container_ip,
        ssh_port = ssh_port,
        "Docker infrastructure provisioning simulation completed successfully"
    );

    Ok(())
}
