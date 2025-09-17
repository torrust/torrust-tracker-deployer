//! Pre-flight cleanup module for E2E tests
//!
//! This module provides functionality to clean up any lingering resources
//! from previous test runs that may have been interrupted before cleanup.

use std::fmt;
use tracing::{info, warn};

#[allow(unused_imports)]
use crate::command_wrappers::lxd::{client::LxdClient, InstanceName};
use crate::command_wrappers::opentofu::{self, EmergencyDestroyError};
use crate::e2e::environment::TestEnvironment;

/// Errors that can occur during pre-flight cleanup operations
#[derive(Debug)]
pub enum PreflightCleanupError {
    /// Emergency destroy operation failed
    EmergencyDestroyFailed { source: EmergencyDestroyError },

    /// Resource conflicts detected that would prevent new test runs
    ResourceConflicts { details: String },
}

impl fmt::Display for PreflightCleanupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmergencyDestroyFailed { source } => {
                write!(f, "Emergency destroy operation failed: {source}")
            }
            Self::ResourceConflicts { details } => {
                write!(
                    f,
                    "Resource conflicts detected that would prevent new test runs: {details}"
                )
            }
        }
    }
}

impl std::error::Error for PreflightCleanupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EmergencyDestroyFailed { source } => Some(source),
            Self::ResourceConflicts { .. } => None,
        }
    }
}

/// Performs pre-flight cleanup of any lingering test resources
///
/// This function attempts to clean up any resources that might remain from
/// previous interrupted test runs. It's designed to be safe to run even when
/// no resources exist.
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
/// Returns an error if cleanup fails and resources are still present that would
/// prevent new test runs from starting successfully.
///
/// # Examples
///
/// ```no_run
/// use torrust_tracker_deploy::e2e::{environment::TestEnvironment, tasks::preflight_cleanup};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let env = TestEnvironment::new(false, "./templates")?;
/// preflight_cleanup::cleanup_lingering_resources(&env)?;
/// # Ok(())
/// # }
/// ```
pub fn cleanup_lingering_resources(env: &TestEnvironment) -> Result<(), PreflightCleanupError> {
    info!(
        operation = "preflight_cleanup",
        "Starting pre-flight cleanup of any lingering test resources"
    );

    // Clean the build directory to ensure fresh template state for E2E tests
    cleanup_build_directory(env)?;

    // Clean any existing OpenTofu infrastructure from previous test runs
    cleanup_opentofu_infrastructure(env)?;

    // Clean any existing LXD resources that might conflict with new test runs
    cleanup_lxd_resources(env);

    info!(
        operation = "preflight_cleanup",
        status = "success",
        "Pre-flight cleanup completed successfully"
    );
    Ok(())
}

/// Cleans the build directory to ensure fresh template state for E2E tests
///
/// This function removes the build directory if it exists, ensuring that
/// E2E tests start with a clean state and don't use stale cached template files.
///
/// # Safety
///
/// This function is only intended for E2E test environments and should never
/// be called in production code paths. It's designed to provide test isolation
/// by ensuring fresh template rendering for each test run.
///
/// # Arguments
///
/// * `env` - The test environment containing configuration paths
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if the build directory doesn't exist.
///
/// # Errors
///
/// Returns a `PreflightCleanupError::ResourceConflicts` error if the build directory
/// cannot be removed due to permission issues or file locks.
fn cleanup_build_directory(env: &TestEnvironment) -> Result<(), PreflightCleanupError> {
    let build_dir = &env.config.build_dir;

    if !build_dir.exists() {
        info!(
            operation = "build_directory_cleanup",
            status = "clean",
            path = %build_dir.display(),
            "Build directory doesn't exist, skipping cleanup"
        );
        return Ok(());
    }

    info!(
        operation = "build_directory_cleanup",
        path = %build_dir.display(),
        "Cleaning build directory to ensure fresh template state"
    );

    match std::fs::remove_dir_all(build_dir) {
        Ok(()) => {
            info!(
                operation = "build_directory_cleanup",
                status = "success",
                path = %build_dir.display(),
                "Build directory cleaned successfully"
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                operation = "build_directory_cleanup",
                status = "failed",
                path = %build_dir.display(),
                error = %e,
                "Failed to clean build directory"
            );
            Err(PreflightCleanupError::ResourceConflicts {
                details: format!(
                    "Failed to clean build directory '{}': {}",
                    build_dir.display(),
                    e
                ),
            })
        }
    }
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
fn cleanup_opentofu_infrastructure(env: &TestEnvironment) -> Result<(), PreflightCleanupError> {
    let tofu_dir = env.config.build_dir.join(&env.config.opentofu_subfolder);

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
fn cleanup_lxd_resources(_env: &TestEnvironment) {
    info!(
        operation = "lxd_resources_cleanup",
        "Cleaning existing LXD resources that might conflict with new test runs"
    );

    let lxd_client = LxdClient::new();

    // Clean up test instance if it exists
    match lxd_client.delete_instance(
        &InstanceName::new("torrust-vm".to_string()).expect("Valid hardcoded instance name"),
        true,
    ) {
        Ok(()) => {
            info!(
                operation = "lxd_resources_cleanup",
                resource = "instance",
                name = "torrust-vm",
                status = "success",
                "LXD instance cleanup completed successfully"
            );
        }
        Err(e) => {
            warn!(
                operation = "lxd_resources_cleanup",
                resource = "instance",
                name = "torrust-vm",
                error = %e,
                "Failed to clean LXD instance"
            );
        }
    }

    // Clean up test profile if it exists
    match lxd_client.delete_profile("torrust-profile") {
        Ok(()) => {
            info!(
                operation = "lxd_resources_cleanup",
                resource = "profile",
                name = "torrust-profile",
                status = "success",
                "LXD profile cleanup completed successfully"
            );
        }
        Err(e) => {
            warn!(
                operation = "lxd_resources_cleanup",
                resource = "profile",
                name = "torrust-profile",
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
