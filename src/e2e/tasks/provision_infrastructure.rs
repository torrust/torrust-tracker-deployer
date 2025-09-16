use anyhow::{Context, Result};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::info;

use crate::commands::ProvisionCommand;
use crate::e2e::environment::TestEnvironment;

/// Provision infrastructure using `OpenTofu` and prepare for configuration
///
/// # Errors
///
/// Returns an error if:
/// - `ProvisionCommand` execution fails
/// - Infrastructure provisioning fails
/// - IP address cannot be obtained from `OpenTofu` outputs
pub async fn provision_infrastructure(env: &TestEnvironment) -> Result<IpAddr> {
    info!("Provisioning test infrastructure");

    // Use the new ProvisionCommand to handle all infrastructure provisioning steps
    let provision_command = ProvisionCommand::new(
        Arc::clone(&env.services.tofu_template_renderer),
        Arc::clone(&env.services.ansible_template_renderer),
        Arc::clone(&env.services.ansible_client),
        Arc::clone(&env.services.opentofu_client),
        env.config.ssh_credentials.clone(),
    );

    let opentofu_instance_ip = provision_command
        .execute()
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
