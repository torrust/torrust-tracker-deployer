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
    info!(
        stage = "infrastructure_configuration",
        "Configuring test infrastructure"
    );

    // Use the new ConfigureCommand to handle all infrastructure configuration steps
    let configure_command = ConfigureCommand::new(Arc::clone(&env.services.ansible_client));

    configure_command
        .execute()
        .map_err(anyhow::Error::from)
        .context("Failed to configure infrastructure")?;

    info!(
        stage = "infrastructure_configuration",
        status = "complete",
        "Infrastructure configuration completed successfully"
    );

    Ok(())
}
