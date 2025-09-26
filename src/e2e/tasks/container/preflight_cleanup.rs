//! Container-specific preflight cleanup functionality
//!
//! This module provides preflight cleanup functionality specifically designed
//! for Docker container-based E2E testing. Since containers are managed by
//! testcontainers and automatically cleaned up, this module only handles
//! directory cleanup operations.

use crate::e2e::environment::TestEnvironment;
use crate::e2e::tasks::preflight_cleanup::{
    cleanup_build_directory, cleanup_templates_directory, PreflightCleanupError,
};
use crate::shared::executor::CommandExecutor;
use tracing::{info, warn};

/// Performs pre-flight cleanup for Docker-based E2E tests
///
/// This function is specifically designed for Docker-based E2E tests that use
/// testcontainers for container lifecycle management. It cleans up directories
/// and any hanging Docker containers from previous interrupted test runs.
///
/// # Arguments
///
/// * `env` - The test environment containing configuration and services
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if there were no resources to clean up.
///
/// # Errors
///
/// Returns an error if directory cleanup fails and would prevent new test runs.
pub fn cleanup_lingering_resources(env: &TestEnvironment) -> Result<(), PreflightCleanupError> {
    info!(
        operation = "preflight_cleanup_docker",
        "Starting pre-flight cleanup for Docker-based E2E tests"
    );

    // Clean the build directory to ensure fresh template state for E2E tests
    cleanup_build_directory(env)?;

    // Clean the templates directory to ensure fresh embedded template extraction for E2E tests
    cleanup_templates_directory(env)?;

    // Clean up any hanging Docker containers from interrupted test runs
    cleanup_hanging_docker_containers(env);

    info!(
        operation = "preflight_cleanup_docker",
        status = "success",
        "Pre-flight cleanup for Docker-based E2E tests completed successfully"
    );
    Ok(())
}

/// Clean up hanging Docker containers from interrupted test runs
///
/// This function handles the case where testcontainers didn't clean up properly
/// due to abrupt test termination. It removes containers with the instance name
/// to prevent container name conflicts in subsequent test runs.
///
/// # Safety
///
/// This function is only intended for E2E test environments and should never
/// be called in production code paths. It specifically targets test containers.
///
/// # Arguments
///
/// * `env` - The test environment containing the instance name
fn cleanup_hanging_docker_containers(env: &TestEnvironment) {
    let instance_name = env.config.instance_name.as_str();
    let command_executor = CommandExecutor::new();

    info!(
        operation = "hanging_container_cleanup",
        container_name = instance_name,
        "Checking for hanging Docker containers from previous test runs"
    );

    // First, check if the container exists
    let check_result = command_executor.run_command(
        "docker",
        &["ps", "-aq", "--filter", &format!("name={instance_name}")],
        None,
    );

    match check_result {
        Ok(output) => {
            if output.trim().is_empty() {
                info!(
                    operation = "hanging_container_cleanup",
                    container_name = instance_name,
                    status = "clean",
                    "No hanging containers found"
                );
                return;
            }

            info!(
                operation = "hanging_container_cleanup",
                container_name = instance_name,
                "Found hanging container, attempting cleanup"
            );

            // Try to stop the container (in case it's running)
            match command_executor.run_command("docker", &["stop", instance_name], None) {
                Ok(_) => {
                    info!(
                        operation = "hanging_container_cleanup",
                        container_name = instance_name,
                        action = "stop",
                        status = "success",
                        "Container stopped successfully"
                    );
                }
                Err(e) => {
                    // Container might not be running, which is okay
                    warn!(
                        operation = "hanging_container_cleanup",
                        container_name = instance_name,
                        action = "stop",
                        status = "skipped",
                        error = %e,
                        "Could not stop container (probably not running)"
                    );
                }
            }

            // Remove the container
            match command_executor.run_command("docker", &["rm", instance_name], None) {
                Ok(_) => {
                    info!(
                        operation = "hanging_container_cleanup",
                        container_name = instance_name,
                        status = "success",
                        "Hanging container cleaned up successfully"
                    );
                }
                Err(e) => {
                    warn!(
                        operation = "hanging_container_cleanup",
                        container_name = instance_name,
                        status = "failed",
                        error = %e,
                        "Failed to remove hanging container (this may cause test failures)"
                    );
                }
            }
        }
        Err(e) => {
            warn!(
                operation = "hanging_container_cleanup",
                container_name = instance_name,
                status = "check_failed",
                error = %e,
                "Could not check for hanging containers"
            );
        }
    }
}
