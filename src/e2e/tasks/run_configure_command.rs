//! Ansible configuration task for E2E testing
//!
//! This module provides the E2E testing task for running Ansible configuration
//! on target instances. It executes Ansible playbooks to configure services
//! and applications on the deployed infrastructure.
//!
//! ## Key Operations
//!
//! - Executes Ansible playbooks using the `ConfigureCommand`
//! - Handles configuration workflow for both containers and VMs
//! - Provides structured error handling and reporting
//!
//! ## Integration
//!
//! This is a generic task that works with infrastructure-agnostic configuration:
//! - Uses rendered Ansible inventories from provision simulation
//! - Works with both container and VM-based infrastructure
//! - Integrates with the existing `ConfigureCommand` workflow
//!
//! ## E2E Config Tests Integration
//!
//! In E2E config tests, this module works seamlessly with provision simulation.
//! The provision simulation ensures that Ansible config files are generated with
//! the correct configuration even without executing the actual provision phase,
//! allowing the configuration command to run successfully on simulated infrastructure.

use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::info;

use crate::application::commands::ConfigureCommand;

use crate::e2e::context::TestContext;

/// Configure infrastructure using Ansible playbooks
///
/// This function executes Ansible configuration using the `ConfigureCommand` for E2E tests.
/// It works with both VM and container-based infrastructure, utilizing rendered Ansible
/// inventories and configuration files generated during the provision simulation phase.
///
/// # Errors
///
/// Returns an error if:
/// - `ConfigureCommand` execution fails
/// - Infrastructure configuration fails
pub fn run_configure_command(test_context: &TestContext) -> Result<()> {
    info!("Configuring test infrastructure");

    // Use the new ConfigureCommand to handle all infrastructure configuration steps
    let configure_command =
        ConfigureCommand::new(Arc::clone(&test_context.services.ansible_client));

    configure_command
        .execute()
        .map_err(anyhow::Error::from)
        .context("Failed to configure infrastructure")?;

    info!(
        status = "complete",
        "Infrastructure configuration completed successfully"
    );

    Ok(())
}
