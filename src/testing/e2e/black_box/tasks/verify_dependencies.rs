//! Dependency verification task for E2E tests.
//!
//! This module provides a shared function to verify that all required
//! system dependencies are installed before running E2E tests.
//!
//! # Example
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::testing::e2e::black_box::tasks::verify_required_dependencies;
//! use torrust_dependency_installer::Dependency;
//!
//! // Verify dependencies for provision tests (only Ansible needed)
//! verify_required_dependencies(&[Dependency::Ansible])?;
//!
//! // Verify dependencies for full E2E tests
//! verify_required_dependencies(&[
//!     Dependency::OpenTofu,
//!     Dependency::Ansible,
//!     Dependency::Lxd,
//! ])?;
//! ```

use anyhow::Result;
use torrust_dependency_installer::{verify_dependencies, Dependency};
use tracing::error;

/// Verify that all required dependencies are installed for E2E tests.
///
/// This function checks that all specified dependencies are available on the system.
/// If any dependency is missing, it displays an actionable error message to help
/// the user install the missing tools.
///
/// # Arguments
///
/// * `required_deps` - Slice of dependencies required for the specific E2E test suite
///
/// # Errors
///
/// Returns an error if any required dependencies are missing or cannot be detected.
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::black_box::tasks::verify_required_dependencies;
/// use torrust_dependency_installer::Dependency;
///
/// // For provision-only tests
/// verify_required_dependencies(&[Dependency::Ansible])?;
///
/// // For full E2E tests
/// verify_required_dependencies(&[
///     Dependency::OpenTofu,
///     Dependency::Ansible,
///     Dependency::Lxd,
/// ])?;
/// ```
pub fn verify_required_dependencies(required_deps: &[Dependency]) -> Result<()> {
    if let Err(e) = verify_dependencies(required_deps) {
        error!(
            error = %e,
            "Dependency verification failed"
        );
        eprintln!("\n{}\n", e.actionable_message());
        return Err(anyhow::anyhow!("Missing required dependencies"));
    }

    Ok(())
}
