//! Infrastructure cleanup task for E2E testing
//!
//! This module provides functionality to clean up test infrastructure after E2E
//! testing is complete. It handles the destruction of provisioned resources while
//! respecting configuration flags for environment preservation.
//!
//! ## Key Features
//!
//! - Conditional cleanup based on `keep_env` configuration flag
//! - Infrastructure destruction via `OpenTofu` destroy operations
//! - Resource cleanup logging and error handling
//! - Support for preserving environments for debugging and inspection
//!
//! ## Cleanup Behavior
//!
//! - If `keep_env` is true: Environment is preserved with connection instructions
//! - If `keep_env` is false: All infrastructure resources are destroyed
//! - Failures are logged as warnings but don't fail the overall process
//!
//! This allows for both automated cleanup and manual inspection workflows.

use tracing::{info, warn};

use crate::e2e::environment::TestEnvironment;

/// Clean up test infrastructure
///
/// This function destroys the test infrastructure using `OpenTofu`.
/// If `keep_env` is set in the environment configuration, the cleanup
/// is skipped and the environment is preserved.
///
/// # Arguments
///
/// * `env` - The test environment containing configuration and services
///
/// # Behavior
///
/// - If `env.config.keep_env` is `true`, logs a message and returns without cleanup
/// - Otherwise, attempts to destroy infrastructure using `OpenTofu`
/// - Logs success or failure appropriately
/// - Does not return errors - failures are logged as warnings
pub fn cleanup_infrastructure(env: &TestEnvironment) {
    if env.config.keep_env {
        info!(
            operation = "cleanup",
            action = "keep_environment",
            instance = "torrust-vm",
            connect_command = "lxc exec torrust-vm -- /bin/bash",
            "Keeping test environment as requested"
        );
        return;
    }

    info!(operation = "cleanup", "Cleaning up test environment");

    // Destroy infrastructure using OpenTofuClient
    let result = env
        .services
        .opentofu_client
        .destroy(true) // auto_approve = true
        .map_err(anyhow::Error::from);

    match result {
        Ok(_) => info!(
            operation = "cleanup",
            status = "success",
            "Test environment cleaned up successfully"
        ),
        Err(e) => warn!(
            operation = "cleanup",
            status = "failed",
            error = %e,
            "Cleanup failed"
        ),
    }
}
