//! Destroy infrastructure task for black-box E2E testing.
//!
//! This module provides functionality to destroy infrastructure
//! using CLI commands in a black-box testing context.

use anyhow::Result;
use tracing::{error, info};

use crate::testing::e2e::ProcessRunner;

/// Destroys the infrastructure for the environment.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `environment_name` - The name of the environment to destroy
///
/// # Errors
///
/// Returns an error if the destroy command fails.
pub fn destroy_infrastructure(runner: &ProcessRunner, environment_name: &str) -> Result<()> {
    info!(
        step = "destroy",
        environment = environment_name,
        "Destroying infrastructure"
    );

    let destroy_result = runner
        .run_destroy_command(environment_name)
        .map_err(|e| anyhow::anyhow!("Failed to execute destroy command: {e}"))?;

    if !destroy_result.success() {
        error!(
            step = "destroy",
            exit_code = ?destroy_result.exit_code(),
            stderr = %destroy_result.stderr(),
            "Destroy command failed"
        );
        return Err(anyhow::anyhow!(
            "Destroy failed with exit code {:?}",
            destroy_result.exit_code()
        ));
    }

    info!(
        step = "destroy",
        status = "success",
        "Infrastructure destroyed successfully"
    );

    Ok(())
}
