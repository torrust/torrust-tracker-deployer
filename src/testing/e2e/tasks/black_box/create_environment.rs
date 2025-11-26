//! Create environment task for black-box E2E testing.
//!
//! This module provides functionality to create an environment from a configuration file
//! using CLI commands in a black-box testing context.

use anyhow::Result;
use tracing::{error, info};

use crate::testing::e2e::ProcessRunner;

/// Creates the environment from the configuration file.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `config_path` - Path to the environment configuration file
///
/// # Errors
///
/// Returns an error if the create command fails.
///
/// # Panics
///
/// Panics if the config path contains invalid UTF-8.
pub fn create_environment(runner: &ProcessRunner, config_path: &std::path::Path) -> Result<()> {
    info!(
        step = "create_environment",
        config_path = %config_path.display(),
        "Creating environment from config file"
    );

    let create_result = runner
        .run_create_command(config_path.to_str().expect("Valid UTF-8 path"))
        .map_err(|e| anyhow::anyhow!("Failed to execute create command: {e}"))?;

    if !create_result.success() {
        error!(
            step = "create_environment",
            exit_code = ?create_result.exit_code(),
            stderr = %create_result.stderr(),
            "Create environment command failed"
        );
        return Err(anyhow::anyhow!(
            "Create environment failed with exit code {:?}",
            create_result.exit_code()
        ));
    }

    info!(
        step = "create_environment",
        status = "success",
        "Environment created successfully"
    );

    Ok(())
}
