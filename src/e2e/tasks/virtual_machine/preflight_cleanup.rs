//! Virtual machine-specific preflight cleanup functionality
//!
//! This module provides preflight cleanup functionality specifically designed
//! for VM-based E2E testing using LXD and `OpenTofu`. It handles cleanup of
//! infrastructure resources including `OpenTofu` state and LXD instances.

use crate::e2e::context::TestContext;
use crate::e2e::tasks::preflight_cleanup::{
    cleanup_build_directory, cleanup_templates_directory, PreflightCleanupError,
};
use crate::infrastructure::adapters::lxd::client::LxdClient;
use crate::infrastructure::adapters::opentofu::{self};
use tracing::{info, warn};

/// Performs comprehensive pre-flight cleanup for VM-based E2E tests
///
/// This function cleans up any lingering resources from previous test runs
/// that may have been interrupted before cleanup, including directories,
/// `OpenTofu` infrastructure, and LXD resources.
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
/// Returns an error if cleanup fails and would prevent new test runs.
pub fn cleanup_lingering_resources(
    test_context: &TestContext,
) -> Result<(), PreflightCleanupError> {
    info!(
        operation = "preflight_cleanup",
        "Starting pre-flight cleanup of any lingering test resources"
    );

    // Clean the build directory to ensure fresh template state for E2E tests
    cleanup_build_directory(test_context)?;

    // Clean the templates directory to ensure fresh embedded template extraction for E2E tests
    cleanup_templates_directory(test_context)?;

    // Clean any existing OpenTofu infrastructure from previous test runs
    cleanup_opentofu_infrastructure(test_context)?;

    // Clean any existing LXD resources that might conflict with new test runs
    cleanup_lxd_resources(test_context);

    info!(
        operation = "preflight_cleanup",
        status = "success",
        "Pre-flight cleanup completed successfully"
    );
    Ok(())
}

/// Cleans any existing `OpenTofu` infrastructure from previous test runs
///
/// This function attempts to destroy `OpenTofu` infrastructure that might remain from
/// previous interrupted test runs. It uses `emergency_destroy` which is designed to
/// handle cases where resources may not exist.
///
/// # Safety
///
/// This function is only intended for E2E test environments and should never
/// be called in production code paths. It's designed to clean up test infrastructure
/// to ensure test isolation.
///
/// # Arguments
///
/// * `env` - The test environment containing configuration paths
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if no `OpenTofu` infrastructure exists.
///
/// # Errors
///
/// Returns a `PreflightCleanupError` if infrastructure cleanup fails and resources
/// are still present that would prevent new test runs.
fn cleanup_opentofu_infrastructure(
    test_context: &TestContext,
) -> Result<(), PreflightCleanupError> {
    let tofu_dir = test_context
        .config
        .build_dir
        .join(&test_context.config.opentofu_subfolder);

    if !tofu_dir.exists() {
        info!(
            operation = "opentofu_infrastructure_cleanup",
            status = "clean",
            path = %tofu_dir.display(),
            "No OpenTofu directory found, skipping infrastructure cleanup"
        );
        return Ok(());
    }

    info!(
        operation = "opentofu_infrastructure_cleanup",
        path = %tofu_dir.display(),
        "Cleaning existing OpenTofu infrastructure from previous test runs"
    );

    // Use emergency_destroy which is designed to handle cases where resources may not exist
    match opentofu::emergency_destroy(&tofu_dir) {
        Ok(()) => {
            info!(
                operation = "opentofu_infrastructure_cleanup",
                status = "success",
                path = %tofu_dir.display(),
                "OpenTofu infrastructure cleaned successfully"
            );
            Ok(())
        }
        Err(e) => {
            // Log as warning rather than error since this is pre-flight cleanup
            // and resources may legitimately not exist
            warn!(
                operation = "opentofu_infrastructure_cleanup",
                status = "partial_failure",
                path = %tofu_dir.display(),
                error = %e,
                "OpenTofu infrastructure cleanup encountered issues (this may be normal if no resources existed)"
            );

            // Don't return an error for pre-flight cleanup failures unless they indicate
            // actual resource conflicts that would prevent new test runs
            let error_message = e.to_string().to_lowercase();
            if error_message.contains("already exists") || error_message.contains("in use") {
                return Err(PreflightCleanupError::ResourceConflicts {
                    details: format!(
                        "Failed to clean OpenTofu infrastructure in '{}': {}",
                        tofu_dir.display(),
                        e
                    ),
                });
            }
            Err(PreflightCleanupError::EmergencyDestroyFailed { source: e })
        }
    }
}

/// Cleans any existing LXD resources that might conflict with new test runs
///
/// This function attempts to clean up LXD instances and profiles that might remain from
/// previous interrupted test runs. It uses direct LXD commands to ensure resources
/// are properly removed before starting new tests.
///
/// # Safety
///
/// This function is only intended for E2E test environments and should never
/// be called in production code paths. It's designed to clean up test infrastructure
/// to ensure test isolation.
///
/// # Arguments
///
/// * `env` - The test environment containing configuration paths
fn cleanup_lxd_resources(test_context: &TestContext) {
    info!(
        operation = "lxd_resources_cleanup",
        "Cleaning existing LXD resources that might conflict with new test runs"
    );

    let lxd_client = LxdClient::new();

    // Clean up test instance if it exists
    match lxd_client.delete_instance(
        &test_context.config.instance_name, // Phase 3: Use instance_name from config instead of hardcoded value
        true,
    ) {
        Ok(()) => {
            info!(
                operation = "lxd_resources_cleanup",
                resource = "instance",
                name = %test_context.config.instance_name.as_str(),
                status = "success",
                "LXD instance cleanup completed successfully"
            );
        }
        Err(e) => {
            warn!(
                operation = "lxd_resources_cleanup",
                resource = "instance",
                name = %test_context.config.instance_name.as_str(),
                error = %e,
                "Failed to clean LXD instance"
            );
        }
    }

    // Clean up test profile if it exists
    match lxd_client.delete_profile(test_context.config.profile_name.as_str()) {
        Ok(()) => {
            info!(
                operation = "lxd_resources_cleanup",
                resource = "profile",
                name = %test_context.config.profile_name.as_str(),
                status = "success",
                "LXD profile cleanup completed successfully"
            );
        }
        Err(e) => {
            warn!(
                operation = "lxd_resources_cleanup",
                resource = "profile",
                name = %test_context.config.profile_name.as_str(),
                error = %e,
                "Failed to clean LXD profile"
            );
        }
    }

    info!(
        operation = "lxd_resources_cleanup",
        status = "success",
        "LXD resources cleanup completed"
    );
}
