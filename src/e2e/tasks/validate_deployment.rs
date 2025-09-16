use anyhow::{Context, Result};
use std::net::IpAddr;
use tracing::info;

use crate::commands::TestCommand;
use crate::e2e::environment::TestEnvironment;

/// Validate deployment by running infrastructure validation tests
///
/// # Errors
///
/// Returns an error if:
/// - `TestCommand` execution fails
/// - Any validation check fails
pub async fn validate_deployment(env: &TestEnvironment, instance_ip: &IpAddr) -> Result<()> {
    info!(stage = "validation", "Starting deployment validation");

    // Use the new TestCommand to handle all infrastructure validation steps
    let test_command = TestCommand::new(env.config.ssh_credentials.clone(), *instance_ip);

    test_command
        .execute()
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to validate deployment")?;

    info!(
        stage = "validation",
        status = "success",
        "All deployment validations passed"
    );
    Ok(())
}
