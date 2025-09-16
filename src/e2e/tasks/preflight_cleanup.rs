//! Pre-flight cleanup module for E2E tests
//!
//! This module provides functionality to clean up any lingering resources
//! from previous test runs that may have been interrupted before cleanup.

use std::fmt;
use tracing::{info, warn};

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

    // Attempt to destroy any existing OpenTofu infrastructure
    let tofu_dir = env.config.build_dir.join(&env.config.opentofu_subfolder);

    if !tofu_dir.exists() {
        info!(
            operation = "preflight_cleanup",
            status = "clean",
            "No OpenTofu directory found, skipping cleanup"
        );
        return Ok(());
    }

    // Use emergency_destroy which is designed to handle cases where resources may not exist
    match opentofu::emergency_destroy(&tofu_dir) {
        Ok(()) => {
            info!(
                operation = "preflight_cleanup",
                status = "success",
                "Pre-flight cleanup completed successfully"
            );
        }
        Err(e) => {
            // Log as warning rather than error since this is pre-flight cleanup
            // and resources may legitimately not exist
            warn!(
                operation = "preflight_cleanup",
                status = "partial_failure",
                error = %e,
                "Pre-flight cleanup encountered issues (this may be normal if no resources existed)"
            );

            // Don't return an error for pre-flight cleanup failures unless they indicate
            // actual resource conflicts that would prevent new test runs
            let error_message = e.to_string().to_lowercase();
            if error_message.contains("already exists") || error_message.contains("in use") {
                return Err(PreflightCleanupError::ResourceConflicts {
                    details: e.to_string(),
                });
            }
            return Err(PreflightCleanupError::EmergencyDestroyFailed { source: e });
        }
    }
    Ok(())
}
