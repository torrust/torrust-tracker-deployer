//! Infrastructure provisioning task for E2E testing
//!
//! This module provides the E2E testing task for provisioning infrastructure using
//! `OpenTofu`. It orchestrates the complete provisioning workflow through the
//! `ProvisionCommand` and returns the IP address of the provisioned instance.
//!
//! ## Key Operations
//!
//! - Execute infrastructure provisioning via `ProvisionCommand`
//! - Initialize and apply `OpenTofu` configurations
//! - Render dynamic templates with runtime variables
//! - Retrieve instance IP addresses from `OpenTofu` outputs
//! - Prepare infrastructure for configuration phase
//!
//! ## Return Value
//!
//! Returns the IP address of the provisioned instance, which is essential
//! for subsequent E2E testing phases (configuration, validation, etc.).
//!
//! ## Integration
//!
//! This task is a critical early step in the E2E testing workflow, providing
//! the foundation infrastructure for all subsequent testing operations.

use anyhow::{Context, Result};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::info;

use crate::application::commands::ProvisionCommand;
use crate::e2e::context::TestContext;

/// Provision infrastructure using `OpenTofu` and prepare for configuration
///
/// # Errors
///
/// Returns an error if:
/// - `ProvisionCommand` execution fails
/// - Infrastructure provisioning fails
/// - IP address cannot be obtained from `OpenTofu` outputs
pub async fn run_provision_command(test_context: &TestContext) -> Result<IpAddr> {
    info!("Provisioning test infrastructure");

    // Use the new ProvisionCommand to handle all infrastructure provisioning steps
    let provision_command = ProvisionCommand::new(
        Arc::clone(&test_context.services.tofu_template_renderer),
        Arc::clone(&test_context.services.ansible_template_renderer),
        Arc::clone(&test_context.services.ansible_client),
        Arc::clone(&test_context.services.opentofu_client),
    );

    let opentofu_instance_ip = provision_command
        .execute(test_context.environment.ssh_credentials())
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to provision infrastructure")?;

    info!(
        status = "complete",
        opentofu_ip = %opentofu_instance_ip,
        "Infrastructure provisioned successfully"
    );

    // Return the IP from OpenTofu as it's our preferred source
    Ok(opentofu_instance_ip)
}
