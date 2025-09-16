//! Infrastructure configuration task for E2E testing
//!
//! This module provides the E2E testing task for configuring infrastructure using
//! Ansible playbooks. It orchestrates the complete configuration workflow through
//! the `ConfigureCommand` to ensure deployed infrastructure is properly set up.
//!
//! ## Key Operations
//!
//! - Execute infrastructure configuration via `ConfigureCommand`
//! - Apply Ansible playbooks to configure system settings
//! - Install and configure software packages (Docker, Docker Compose, etc.)
//! - Validate configuration completion
//!
//! ## Integration
//!
//! This task is part of the E2E testing workflow and runs after infrastructure
//! provisioning to prepare the environment for application deployment and testing.

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::info;

use crate::commands::ConfigureCommand;
use crate::e2e::environment::TestEnvironment;

/// Configure infrastructure using Ansible playbooks
///
/// # Errors
///
/// Returns an error if:
/// - `ConfigureCommand` execution fails
/// - Infrastructure configuration fails
pub fn configure_infrastructure(env: &TestEnvironment) -> Result<()> {
    info!("Configuring test infrastructure");

    // Use the new ConfigureCommand to handle all infrastructure configuration steps
    let configure_command = ConfigureCommand::new(Arc::clone(&env.services.ansible_client));

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
