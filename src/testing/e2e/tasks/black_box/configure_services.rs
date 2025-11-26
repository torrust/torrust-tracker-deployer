//! Configure services task for black-box E2E testing.
//!
//! This module provides functionality to configure services on provisioned
//! infrastructure using CLI commands in a black-box testing context.

use anyhow::Result;
use tracing::{error, info, warn};

use crate::testing::e2e::ProcessRunner;

/// Configures services on the provisioned infrastructure.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `environment_name` - The name of the environment to configure
/// * `destroy_on_failure` - If true, attempt to destroy infrastructure on failure
///
/// # Errors
///
/// Returns an error if the configure command fails.
pub fn configure_services(
    runner: &ProcessRunner,
    environment_name: &str,
    destroy_on_failure: bool,
) -> Result<()> {
    info!(
        step = "configure",
        environment = environment_name,
        "Configuring services"
    );

    let configure_result = runner
        .run_configure_command(environment_name)
        .map_err(|e| anyhow::anyhow!("Failed to execute configure command: {e}"))?;

    if !configure_result.success() {
        error!(
            step = "configure",
            exit_code = ?configure_result.exit_code(),
            stderr = %configure_result.stderr(),
            "Configure command failed"
        );

        // Try to cleanup even if configure failed
        if destroy_on_failure {
            warn!(
                step = "cleanup_after_failure",
                "Attempting to destroy infrastructure after configure failure"
            );
            // Ignore destroy result - we're already in an error state
            drop(runner.run_destroy_command(environment_name));
        }

        return Err(anyhow::anyhow!(
            "Configure failed with exit code {:?}",
            configure_result.exit_code()
        ));
    }

    info!(
        step = "configure",
        status = "success",
        "Services configured successfully"
    );

    Ok(())
}
