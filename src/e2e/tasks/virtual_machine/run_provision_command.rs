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
/// This function updates the `TestContext`'s internal environment to reflect the
/// provisioned state, ensuring consistency throughout the test lifecycle. Callers
/// can access the provisioned environment and its instance IP through the `TestContext`.
///
/// # Errors
///
/// Returns an error if:
/// - `ProvisionCommand` execution fails
/// - Infrastructure provisioning fails
/// - IP address cannot be obtained from `OpenTofu` outputs
pub async fn run_provision_command(test_context: &mut TestContext) -> Result<()> {
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
    // Extract the Created environment from AnyEnvironmentState
    let created_env = test_context
        .environment
        .clone()
        .try_into_created()
        .context("Environment must be in Created state to provision")?;

    let provisioned_env = provision_command
        .execute(created_env)
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to provision infrastructure")?;

    info!(
        status = "complete",
        environment = %provisioned_env.name(),
        instance_ip = ?provisioned_env.instance_ip(),
        "Instance provisioned successfully"
    );

    // Update the test context with the provisioned environment state
    test_context.update_from_provisioned(provisioned_env);

    Ok(())
}
