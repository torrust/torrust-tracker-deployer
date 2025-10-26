//! Generic preflight cleanup functionality
//!
//! This module provides directory cleanup functions that are used by both
//! container-based and VM-based E2E testing workflows. These functions handle
//! the cleanup of build and template directories to ensure test isolation.

use std::fmt;

use crate::adapters::tofu::EmergencyDestroyError;
use crate::testing::e2e::context::TestContext;
use tracing::{info, warn};

// Re-export functions from the new modular structure for backward compatibility
pub use crate::testing::e2e::tasks::container::preflight_cleanup::preflight_cleanup_previous_resources;

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
pub fn cleanup_build_directory(test_context: &TestContext) -> Result<(), PreflightCleanupError> {
    let build_dir = &test_context.config.build_dir;

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
/// This is critical for testing template changes and instance name parameterization.
///
/// # Safety
///
/// This function is only intended for E2E test environments and should never
/// be called in production code paths. It's designed to provide test isolation
/// by ensuring fresh template extraction for each test run.
///
/// # Arguments
///
/// * `env` - The test environment containing configuration paths
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if the templates directory doesn't exist.
///
/// # Errors
///
/// Returns a `PreflightCleanupError::ResourceConflicts` error if the templates directory
/// cannot be removed due to permission issues or file locks.
pub fn cleanup_templates_directory(
    test_context: &TestContext,
) -> Result<(), PreflightCleanupError> {
    let templates_dir = std::path::Path::new(&test_context.config.templates_dir);

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
/// environment data from previous test runs. This prevents "environment already exists"
/// errors and ensures proper test isolation.
///
/// # Safety
///
/// This function is only intended for E2E test environments and should never
/// be called in production code paths. It's designed to provide test isolation
/// by ensuring fresh environment state for each test run.
///
/// # Arguments
///
/// * `test_context` - The test context containing the environment configuration
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if the data directory doesn't exist.
///
/// # Errors
///
/// Returns a `PreflightCleanupError::ResourceConflicts` error if the data directory
/// cannot be removed due to permission issues or file locks.
pub fn cleanup_data_environment(test_context: &TestContext) -> Result<(), PreflightCleanupError> {
    use std::path::Path;

    // Construct the data directory path: data/{environment_name}
    let data_dir = Path::new("data").join(test_context.environment.name().as_str());

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
