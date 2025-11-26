//! Provision infrastructure task for black-box E2E testing.
//!
//! This module provides functionality to provision infrastructure
//! using CLI commands in a black-box testing context.

use anyhow::Result;
use tracing::{error, info, warn};

use crate::testing::e2e::ProcessRunner;

/// Provisions the infrastructure for the environment.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `environment_name` - The name of the environment to provision
/// * `destroy_on_failure` - If true, attempt to destroy infrastructure on failure
///
/// # Errors
///
/// Returns an error if the provision command fails.
pub fn provision_infrastructure(
    runner: &ProcessRunner,
    environment_name: &str,
    destroy_on_failure: bool,
) -> Result<()> {
    info!(
        step = "provision",
        environment = environment_name,
        "Provisioning infrastructure"
    );

    let provision_result = runner
        .run_provision_command(environment_name)
        .map_err(|e| anyhow::anyhow!("Failed to execute provision command: {e}"))?;

    if !provision_result.success() {
        error!(
            step = "provision",
            exit_code = ?provision_result.exit_code(),
            stderr = %provision_result.stderr(),
            "Provision command failed"
        );

        // Try to cleanup even if provision failed
        if destroy_on_failure {
            warn!(
                step = "cleanup_after_failure",
                "Attempting to destroy infrastructure after provision failure"
            );
            // Ignore destroy result - we're already in an error state
            drop(runner.run_destroy_command(environment_name));
        }

        return Err(anyhow::anyhow!(
            "Provision failed with exit code {:?}",
            provision_result.exit_code()
        ));
    }

    info!(
        step = "provision",
        status = "success",
        "Infrastructure provisioned successfully"
    );

    Ok(())
}
