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
