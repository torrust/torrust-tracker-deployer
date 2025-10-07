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
use std::sync::Arc;
use tracing::info;

use crate::application::commands::ProvisionCommand;
use crate::e2e::context::TestContext;

/// Provision infrastructure using `OpenTofu` and prepare for configuration
///
/// Returns the provisioned environment with the instance IP stored in its context.
/// Callers can extract the IP address using `environment.instance_ip()`.
///
/// # Errors
///
/// Returns an error if:
/// - `ProvisionCommand` execution fails
/// - Infrastructure provisioning fails
/// - IP address cannot be obtained from `OpenTofu` outputs
pub async fn run_provision_command(
    test_context: &TestContext,
) -> Result<crate::domain::Environment<crate::domain::environment::Provisioned>> {
    info!("Provisioning test infrastructure");

    // Create repository for this environment
    let repository = test_context.create_repository();

    // Use the new ProvisionCommand to handle all infrastructure provisioning steps
    let provision_command = ProvisionCommand::new(
        Arc::clone(&test_context.services.tofu_template_renderer),
        Arc::clone(&test_context.services.ansible_template_renderer),
        Arc::clone(&test_context.services.ansible_client),
        Arc::clone(&test_context.services.opentofu_client),
        Arc::clone(&test_context.services.clock),
        repository,
    );

    // Execute provisioning with environment in Created state
    let provisioned_env = provision_command
        .execute(test_context.environment.clone())
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to provision infrastructure")?;

    info!(
        status = "complete",
        environment = %provisioned_env.name(),
        instance_ip = ?provisioned_env.instance_ip(),
        "Instance provisioned successfully"
    );

    Ok(provisioned_env)
}
