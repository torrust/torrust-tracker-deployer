//! Container-specific preflight cleanup functionality
//!
//! This module provides preflight cleanup functionality specifically designed
//! for Docker container-based E2E testing. Since containers are managed by
//! testcontainers and automatically cleaned up, this module only handles
//! directory cleanup operations.

use crate::e2e::environment::TestEnvironment;
use crate::e2e::tasks::preflight_cleanup::PreflightCleanupError;
use crate::e2e::tasks::preflight_cleanup_common::{
    cleanup_build_directory, cleanup_templates_directory,
};
use tracing::info;

/// Performs pre-flight cleanup for Docker-based E2E tests
///
/// This function is specifically designed for Docker-based E2E tests that use
/// testcontainers for container lifecycle management. It only cleans directories
/// since Docker containers are automatically cleaned up when testcontainer objects
/// are dropped.
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
pub fn cleanup_lingering_resources_docker(
    env: &TestEnvironment,
) -> Result<(), PreflightCleanupError> {
    info!(
        operation = "preflight_cleanup_docker",
        "Starting pre-flight cleanup for Docker-based E2E tests"
    );

    // Clean the build directory to ensure fresh template state for E2E tests
    cleanup_build_directory(env)?;

    // Clean the templates directory to ensure fresh embedded template extraction for E2E tests
    cleanup_templates_directory(env)?;

    // Note: Docker containers are automatically cleaned up by testcontainers when objects are dropped
    // No need for explicit container cleanup like with LXD/OpenTofu

    info!(
        operation = "preflight_cleanup_docker",
        status = "success",
        "Pre-flight cleanup for Docker-based E2E tests completed successfully"
    );
    Ok(())
}
