//! Virtual machine-specific preflight cleanup functionality
//!
//! This module provides preflight cleanup functionality specifically designed
//! for VM-based E2E testing using LXD and `OpenTofu`. It handles cleanup of
//! infrastructure resources including `OpenTofu` state and LXD instances.

use std::path::PathBuf;

use crate::adapters::lxd::client::LxdClient;
use crate::adapters::tofu;
use crate::domain::{EnvironmentName, InstanceName, ProfileName};
use crate::infrastructure::external_tools::tofu::OPENTOFU_SUBFOLDER;
use crate::testing::e2e::tasks::preflight_cleanup::PreflightCleanupError;
use tracing::{info, warn};

/// Minimal context required for preflight cleanup operations
///
/// This type captures only the essential information needed to clean up
/// artifacts from previous test runs, without requiring a fully initialized
/// `TestContext`. This allows cleanup to run before environment initialization.
pub struct PreflightCleanupContext {
    /// Build directory path where `OpenTofu` and Ansible files are generated
    pub build_dir: PathBuf,
    /// Templates directory path where embedded templates are extracted
    pub templates_dir: PathBuf,
    /// Environment name used to locate data directory (`data/{environment_name}`)
    pub environment_name: EnvironmentName,
    /// Instance name for LXD resource cleanup
    pub instance_name: InstanceName,
    /// Profile name for LXD resource cleanup
    pub profile_name: ProfileName,
}

impl PreflightCleanupContext {
    /// Creates a new preflight cleanup context with the minimum required information
    ///
    /// # Arguments
    ///
    /// * `build_dir` - Path to the build directory
    /// * `templates_dir` - Path to the templates directory
    /// * `environment_name` - Name of the environment to clean up
    /// * `instance_name` - LXD instance name to clean up
    /// * `profile_name` - LXD profile name to clean up
    #[must_use]
    pub fn new(
        build_dir: PathBuf,
        templates_dir: PathBuf,
        environment_name: EnvironmentName,
        instance_name: InstanceName,
        profile_name: ProfileName,
    ) -> Self {
        Self {
            build_dir,
            templates_dir,
            environment_name,
            instance_name,
            profile_name,
        }
    }
}

/// Performs comprehensive pre-flight cleanup for VM-based E2E tests
///
/// This function cleans up any artifacts remaining from previous test runs that may have
/// been interrupted before cleanup, including directories, `OpenTofu` infrastructure,
/// and LXD resources. This ensures a clean slate for new test execution.
///
/// # Arguments
///
/// * `context` - The preflight cleanup context containing paths and names
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if there were no resources to clean up.
///
/// # Errors
///
/// Returns an error if cleanup fails and would prevent new test runs.
pub fn preflight_cleanup_previous_resources(
    context: &PreflightCleanupContext,
) -> Result<(), PreflightCleanupError> {
    info!(
        operation = "preflight_cleanup",
        "Starting pre-flight cleanup of any lingering test resources"
    );

    // Clean the build directory to ensure fresh template state for E2E tests
    cleanup_build_directory(context)?;

    // Clean the templates directory to ensure fresh embedded template extraction for E2E tests
    cleanup_templates_directory(context)?;

    // Clean the data directory to ensure fresh environment state for E2E tests
    cleanup_data_environment(context)?;

    // Clean any existing OpenTofu infrastructure from previous test runs
    cleanup_opentofu_infrastructure(context)?;

    // Clean any existing LXD resources that might conflict with new test runs
    cleanup_lxd_resources(context);

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
/// * `context` - The preflight cleanup context containing paths
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if the build directory doesn't exist.
///
/// # Errors
///
/// Returns a `PreflightCleanupError::ResourceConflicts` error if the build directory
/// cannot be removed due to permission issues or file locks.
fn cleanup_build_directory(context: &PreflightCleanupContext) -> Result<(), PreflightCleanupError> {
    let build_dir = &context.build_dir;

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

/// Cleans the templates directory to ensure fresh embedded template extraction for E2E tests
///
/// This function removes the templates directory if it exists, ensuring that
/// E2E tests start with fresh embedded templates and don't use stale cached template files.
///
/// # Arguments
///
/// * `context` - The preflight cleanup context containing paths
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if the templates directory doesn't exist.
///
/// # Errors
///
/// Returns a `PreflightCleanupError::ResourceConflicts` error if the templates directory
/// cannot be removed due to permission issues or file locks.
fn cleanup_templates_directory(
    context: &PreflightCleanupContext,
) -> Result<(), PreflightCleanupError> {
    let templates_dir = &context.templates_dir;

    if !templates_dir.exists() {
        info!(
            operation = "templates_directory_cleanup",
            status = "clean",
            path = %templates_dir.display(),
            "Templates directory doesn't exist, skipping cleanup"
        );
        return Ok(());
    }

    info!(
        operation = "templates_directory_cleanup",
        path = %templates_dir.display(),
        "Cleaning templates directory to ensure fresh embedded template extraction"
    );

    match std::fs::remove_dir_all(templates_dir) {
        Ok(()) => {
            info!(
                operation = "templates_directory_cleanup",
                status = "success",
                path = %templates_dir.display(),
                "Templates directory cleaned successfully"
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                operation = "templates_directory_cleanup",
                status = "failed",
                path = %templates_dir.display(),
                error = %e,
                "Failed to clean templates directory"
            );
            Err(PreflightCleanupError::ResourceConflicts {
                details: format!(
                    "Failed to clean templates directory '{}': {}",
                    templates_dir.display(),
                    e
                ),
            })
        }
    }
}

/// Cleans the data directory for the test environment to ensure fresh state for E2E tests
///
/// This function removes the environment's data directory if it exists, ensuring that
/// E2E tests start with a clean state and don't encounter conflicts with stale
/// environment data from previous test runs.
///
/// # Arguments
///
/// * `context` - The preflight cleanup context containing the environment name
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if the data directory doesn't exist.
///
/// # Errors
///
/// Returns a `PreflightCleanupError::ResourceConflicts` error if the data directory
/// cannot be removed due to permission issues or file locks.
fn cleanup_data_environment(
    context: &PreflightCleanupContext,
) -> Result<(), PreflightCleanupError> {
    use std::path::Path;

    // Construct the data directory path: data/{environment_name}
    let data_dir = Path::new("data").join(context.environment_name.as_str());

    if !data_dir.exists() {
        info!(
            operation = "data_directory_cleanup",
            status = "clean",
            path = %data_dir.display(),
            "Data directory doesn't exist, skipping cleanup"
        );
        return Ok(());
    }

    info!(
        operation = "data_directory_cleanup",
        path = %data_dir.display(),
        "Cleaning data directory for previous test environment"
    );

    match std::fs::remove_dir_all(&data_dir) {
        Ok(()) => {
            info!(
                operation = "data_directory_cleanup",
                status = "success",
                path = %data_dir.display(),
                "Data directory cleaned successfully"
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                operation = "data_directory_cleanup",
                status = "failed",
                path = %data_dir.display(),
                error = %e,
                "Failed to clean data directory"
            );
            Err(PreflightCleanupError::ResourceConflicts {
                details: format!(
                    "Failed to clean data directory '{}': {}",
                    data_dir.display(),
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
/// * `context` - The preflight cleanup context containing paths
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
    context: &PreflightCleanupContext,
) -> Result<(), PreflightCleanupError> {
    let tofu_dir = context.build_dir.join(OPENTOFU_SUBFOLDER);

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
    match tofu::emergency_destroy(&tofu_dir) {
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
/// * `context` - The preflight cleanup context containing instance and profile names
fn cleanup_lxd_resources(context: &PreflightCleanupContext) {
    info!(
        operation = "lxd_resources_cleanup",
        "Cleaning existing LXD resources that might conflict with new test runs"
    );

    let lxd_client = LxdClient::new();

    // Clean up test instance if it exists
    match lxd_client.delete_instance(&context.instance_name, true) {
        Ok(()) => {
            info!(
                operation = "lxd_resources_cleanup",
                resource = "instance",
                name = %context.instance_name.as_str(),
                status = "success",
                "LXD instance cleanup completed successfully"
            );
        }
        Err(e) => {
            warn!(
                operation = "lxd_resources_cleanup",
                resource = "instance",
                name = %context.instance_name.as_str(),
                error = %e,
                "Failed to clean LXD instance"
            );
        }
    }

    // Clean up test profile if it exists
    match lxd_client.delete_profile(context.profile_name.as_str()) {
        Ok(()) => {
            info!(
                operation = "lxd_resources_cleanup",
                resource = "profile",
                name = %context.profile_name.as_str(),
                status = "success",
                "LXD profile cleanup completed successfully"
            );
        }
        Err(e) => {
            warn!(
                operation = "lxd_resources_cleanup",
                resource = "profile",
                name = %context.profile_name.as_str(),
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
