//! Preflight cleanup task for black-box E2E tests.
//!
//! This module provides a shared function to perform preflight cleanup
//! before running E2E tests, ensuring a clean slate by removing artifacts
//! from previous test runs.
//!
//! # Example
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::run_preflight_cleanup;
//!
//! // Clean up artifacts from previous "e2e-provision" test runs
//! run_preflight_cleanup("e2e-provision")?;
//! ```

use anyhow::Result;
use tracing::info;

use crate::domain::EnvironmentName;
use crate::testing::e2e::tasks::virtual_machine::preflight_cleanup::{
    preflight_cleanup_previous_resources, PreflightCleanupContext,
};

/// Performs preflight cleanup to remove artifacts from previous test runs.
///
/// This ensures a clean slate before starting new tests by removing:
/// - Build directory
/// - Templates directory
/// - Data directory for this environment
/// - LXD resources (instance and profile)
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to clean up
///
/// # Errors
///
/// Returns an error if cleanup fails.
///
/// # Panics
///
/// Panics if the environment name, instance name, or profile name is invalid.
/// This should not happen with valid E2E test environment names.
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::run_preflight_cleanup;
///
/// run_preflight_cleanup("e2e-provision")?;
/// run_preflight_cleanup("e2e-full")?;
/// ```
pub fn run_preflight_cleanup(environment_name: &str) -> Result<()> {
    info!(
        operation = "preflight_cleanup",
        environment = environment_name,
        "Running preflight cleanup"
    );

    // Create preflight cleanup context with paths for the environment
    let cleanup_context = PreflightCleanupContext::new(
        format!("./build/{environment_name}").into(),
        format!("./templates/{environment_name}").into(),
        EnvironmentName::new(environment_name).expect("Valid environment name"),
        format!("torrust-tracker-vm-{environment_name}")
            .try_into()
            .expect("Valid instance name"),
        format!("torrust-profile-{environment_name}")
            .try_into()
            .expect("Valid profile name"),
    );

    preflight_cleanup_previous_resources(&cleanup_context)?;

    info!(
        operation = "preflight_cleanup",
        status = "success",
        "Preflight cleanup completed"
    );

    Ok(())
}

/// Performs preflight cleanup for container-based E2E tests.
///
/// This is a specialized cleanup function for Docker container-based tests.
/// It cleans up:
/// - Build directory
/// - Templates directory
/// - Data directory for this environment
/// - Hanging Docker containers with the environment name
///
/// Unlike `run_preflight_cleanup`, this does NOT clean up LXD resources since
/// container tests don't use LXD.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to clean up
///
/// # Errors
///
/// Returns an error if directory cleanup fails.
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::run_container_preflight_cleanup;
///
/// run_container_preflight_cleanup("e2e-config-new")?;
/// ```
pub fn run_container_preflight_cleanup(environment_name: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;

    info!(
        operation = "container_preflight_cleanup",
        environment = environment_name,
        "Running container-based preflight cleanup"
    );

    // Clean build directory
    let build_dir = format!("./build/{environment_name}");
    if Path::new(&build_dir).exists() {
        fs::remove_dir_all(&build_dir)
            .map_err(|e| anyhow::anyhow!("Failed to remove build directory: {e}"))?;
        info!(
            operation = "container_preflight_cleanup",
            path = build_dir,
            "Removed build directory"
        );
    }

    // Clean templates directory
    let templates_dir = format!("./templates/{environment_name}");
    if Path::new(&templates_dir).exists() {
        fs::remove_dir_all(&templates_dir)
            .map_err(|e| anyhow::anyhow!("Failed to remove templates directory: {e}"))?;
        info!(
            operation = "container_preflight_cleanup",
            path = templates_dir,
            "Removed templates directory"
        );
    }

    // Clean data directory
    let data_dir = format!("data/{environment_name}");
    if Path::new(&data_dir).exists() {
        fs::remove_dir_all(&data_dir)
            .map_err(|e| anyhow::anyhow!("Failed to remove data directory: {e}"))?;
        info!(
            operation = "container_preflight_cleanup",
            path = data_dir,
            "Removed data directory"
        );
    }

    // Clean up hanging Docker containers
    cleanup_hanging_docker_container(environment_name);

    info!(
        operation = "container_preflight_cleanup",
        status = "success",
        "Container-based preflight cleanup completed"
    );

    Ok(())
}

/// Clean up a hanging Docker container by name
///
/// This handles containers that weren't properly cleaned up from previous
/// test runs (e.g., due to test crashes or interruptions).
fn cleanup_hanging_docker_container(container_name: &str) {
    use crate::shared::command::CommandExecutor;

    let command_executor = CommandExecutor::new();

    info!(
        operation = "hanging_container_cleanup",
        container_name = container_name,
        "Checking for hanging Docker containers"
    );

    // Check if container exists (running or stopped)
    let check_result = command_executor.run_command(
        "docker",
        &["ps", "-aq", "--filter", &format!("name=^{container_name}$")],
        None,
    );

    match check_result {
        Ok(output) => {
            if output.stdout_trimmed().is_empty() {
                info!(
                    operation = "hanging_container_cleanup",
                    container_name = container_name,
                    status = "clean",
                    "No hanging containers found"
                );
                return;
            }

            info!(
                operation = "hanging_container_cleanup",
                container_name = container_name,
                "Found hanging container, attempting cleanup"
            );

            // Try to stop the container (in case it's running)
            drop(command_executor.run_command("docker", &["stop", container_name], None));

            // Remove the container
            match command_executor.run_command("docker", &["rm", "-f", container_name], None) {
                Ok(_) => {
                    info!(
                        operation = "hanging_container_cleanup",
                        container_name = container_name,
                        status = "success",
                        "Hanging container removed"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        operation = "hanging_container_cleanup",
                        container_name = container_name,
                        status = "failed",
                        error = %e,
                        "Failed to remove hanging container"
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                operation = "hanging_container_cleanup",
                container_name = container_name,
                status = "check_failed",
                error = %e,
                "Could not check for hanging containers"
            );
        }
    }
}
