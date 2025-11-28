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

// TODO: Refactor TestContext to eliminate the need for this workaround function
//
// Current issue: TestContext requires an Environment, but we need to clean up data
// directories BEFORE creating the Environment (because CreateCommandHandler checks
// if the environment already exists in the repository).
//
// Proposed solutions:
// 1. Make Environment optional in TestContext (TestContext { environment: Option<Environment> })
// 2. Move Environment out of TestContext (preferred - better separation of concerns)
//
// The second option is better because:
// - TestContext should manage test infrastructure (services, config, temp directories)
// - Environment is a domain entity that represents deployment state
// - Separating them provides clearer responsibilities and easier testing
//
// After refactoring, we could eliminate this standalone function and have all cleanup
// go through a single preflight_cleanup_previous_resources() that doesn't require
// a fully initialized TestContext with an Environment.

/// Cleans the data directory for a specific environment name before `TestContext` creation
///
/// This helper function removes the `data/{environment_name}` directory to prevent
/// "environment already exists" errors when `CreateCommandHandler` checks the repository.
/// Unlike `cleanup_data_environment`, this function works without a `TestContext` and is
/// intended to be called early in the test setup before environment creation.
///
/// # Safety
///
/// This function is only intended for E2E test environments and should never
/// be called in production code paths. It's designed to provide test isolation
/// by ensuring environments from previous test runs don't interfere.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to clean up
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if the directory doesn't exist.
///
/// # Errors
///
/// Returns a `PreflightCleanupError::ResourceConflicts` error if the data directory
/// cannot be removed due to permission issues or file locks.
pub fn cleanup_previous_test_data(environment_name: &str) -> Result<(), PreflightCleanupError> {
    use std::path::Path;

    let data_dir = Path::new("data").join(environment_name);

    if !data_dir.exists() {
        info!(
            operation = "preflight_data_cleanup",
            status = "clean",
            path = %data_dir.display(),
            "No previous data directory found, skipping cleanup"
        );
        return Ok(());
    }

    info!(
        operation = "preflight_data_cleanup",
        path = %data_dir.display(),
        "Cleaning data directory from previous test run"
    );

    match std::fs::remove_dir_all(&data_dir) {
        Ok(()) => {
            info!(
                operation = "preflight_data_cleanup",
                status = "success",
                path = %data_dir.display(),
                "Data directory cleaned successfully"
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                operation = "preflight_data_cleanup",
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

/// Generic directory cleanup function for E2E test preflight operations
///
/// This is the core cleanup function used by all directory cleanup operations.
/// It removes the specified directory if it exists, with proper logging.
///
/// # Safety
///
/// This function is only intended for E2E test environments and should never
/// be called in production code paths.
///
/// # Arguments
///
/// * `dir_path` - The path to the directory to clean
/// * `operation_name` - A descriptive name for the operation (used in logs)
/// * `description` - A human-readable description of what's being cleaned (used in logs)
///
/// # Returns
///
/// Returns `Ok(())` if cleanup succeeds or if the directory doesn't exist.
///
/// # Errors
///
/// Returns a `PreflightCleanupError::ResourceConflicts` error if the directory
/// cannot be removed due to permission issues or file locks.
pub fn cleanup_directory(
    dir_path: &std::path::Path,
    operation_name: &str,
    description: &str,
) -> Result<(), PreflightCleanupError> {
    if !dir_path.exists() {
        info!(
            operation = operation_name,
            status = "clean",
            path = %dir_path.display(),
            "{} doesn't exist, skipping cleanup", description
        );
        return Ok(());
    }

    info!(
        operation = operation_name,
        path = %dir_path.display(),
        "Cleaning {} to ensure fresh state", description
    );

    match std::fs::remove_dir_all(dir_path) {
        Ok(()) => {
            info!(
                operation = operation_name,
                status = "success",
                path = %dir_path.display(),
                "{} cleaned successfully", description
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                operation = operation_name,
                status = "failed",
                path = %dir_path.display(),
                error = %e,
                "Failed to clean {}", description
            );
            Err(PreflightCleanupError::ResourceConflicts {
                details: format!(
                    "Failed to clean {} '{}': {}",
                    description,
                    dir_path.display(),
                    e
                ),
            })
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
    cleanup_directory(
        &test_context.config.build_dir,
        "build_directory_cleanup",
        "build directory",
    )
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
    cleanup_directory(
        templates_dir,
        "templates_directory_cleanup",
        "templates directory",
    )
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
    cleanup_directory(&data_dir, "data_directory_cleanup", "data directory")
}
