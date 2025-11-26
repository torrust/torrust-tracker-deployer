//! Validate deployment task for black-box E2E testing.
//!
//! This module provides functionality to validate a deployment
//! using CLI commands in a black-box testing context.

use anyhow::Result;
use tracing::{error, info};

use crate::testing::e2e::ProcessRunner;

/// Validates the deployment by running the test command.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `environment_name` - The name of the environment to validate
///
/// # Errors
///
/// Returns an error if the test command fails.
pub fn validate_deployment(runner: &ProcessRunner, environment_name: &str) -> Result<()> {
    info!(
        step = "test",
        environment = environment_name,
        "Validating deployment"
    );

    let test_result = runner
        .run_test_command(environment_name)
        .map_err(|e| anyhow::anyhow!("Failed to execute test command: {e}"))?;

    if !test_result.success() {
        error!(
            step = "test",
            exit_code = ?test_result.exit_code(),
            stderr = %test_result.stderr(),
            "Test command failed"
        );
        return Err(anyhow::anyhow!(
            "Test failed with exit code {:?}",
            test_result.exit_code()
        ));
    }

    info!(
        step = "test",
        status = "success",
        "Deployment validated successfully"
    );

    Ok(())
}
