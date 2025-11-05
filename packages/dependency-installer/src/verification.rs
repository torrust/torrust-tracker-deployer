//! Dependency Verification
//!
//! This module provides dependency verification functionality for applications
//! that need to ensure required system dependencies are installed before execution.
//! It checks dependencies and provides clear error messages with installation guidance.

use thiserror::Error;
use tracing::{error, info};

use crate::{Dependency, DependencyManager, DetectionError};

// ============================================================================
// PUBLIC API - Main Functions
// ============================================================================

/// Verify that all required dependencies are installed
///
/// This function checks each dependency in the provided list and reports
/// clear errors if any are missing. It does NOT attempt automatic installation,
/// allowing the user to control when and how dependencies are installed.
///
/// # Errors
///
/// Returns an error if:
/// - One or more dependencies are not installed
/// - Detection system fails to check a dependency
///
/// # Example
///
/// ```no_run
/// use torrust_dependency_installer::{Dependency, verify_dependencies};
///
/// // Verify all dependencies for a full workflow
/// let deps = &[Dependency::OpenTofu, Dependency::Ansible, Dependency::Lxd];
/// verify_dependencies(deps)?;
///
/// // Verify only specific dependencies
/// let deps = &[Dependency::Ansible];
/// verify_dependencies(deps)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn verify_dependencies(dependencies: &[Dependency]) -> Result<(), DependencyVerificationError> {
    let manager = DependencyManager::new();
    let mut missing = Vec::new();

    info!("Verifying dependencies");

    for &dep in dependencies {
        let detector = manager.get_detector(dep);

        match detector.is_installed() {
            Ok(true) => {
                info!(
                    dependency = detector.name(),
                    status = "installed",
                    "Dependency check passed"
                );
            }
            Ok(false) => {
                error!(
                    dependency = detector.name(),
                    status = "not installed",
                    "Dependency check failed"
                );
                missing.push(dep);
            }
            Err(e) => {
                error!(
                    dependency = detector.name(),
                    error = %e,
                    "Failed to detect dependency"
                );
                return Err(DependencyVerificationError::DetectionFailed {
                    dependency: dep,
                    source: e,
                });
            }
        }
    }

    if missing.is_empty() {
        info!("All required dependencies are available");
        Ok(())
    } else {
        Err(DependencyVerificationError::MissingDependencies {
            dependencies: missing,
        })
    }
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

/// Errors that can occur during dependency verification
#[derive(Debug, Error)]
pub enum DependencyVerificationError {
    /// One or more required dependencies are not installed
    #[error("Missing required dependencies: {}", format_dependency_list(.dependencies))]
    MissingDependencies {
        /// List of missing dependencies
        dependencies: Vec<Dependency>,
    },

    /// Failed to detect if a dependency is installed
    #[error("Failed to detect dependency '{dependency}': {source}")]
    DetectionFailed {
        /// The dependency that could not be detected
        dependency: Dependency,
        /// The underlying detection error
        #[source]
        source: DetectionError,
    },
}

impl DependencyVerificationError {
    /// Get actionable error message with installation instructions
    #[must_use]
    pub fn actionable_message(&self) -> String {
        match self {
            Self::MissingDependencies { dependencies } => {
                let dep_list = format_dependency_list(dependencies);
                format!(
                    "Missing required dependencies: {dep_list}\n\n\
                    To install all dependencies automatically, run:\n  \
                    cargo run --bin dependency-installer install\n\n\
                    Or install specific dependencies:\n  \
                    cargo run --bin dependency-installer install <dependency>\n\n\
                    For manual installation instructions, see:\n  \
                    https://github.com/torrust/torrust-tracker-deployer/blob/main/packages/dependency-installer/README.md"
                )
            }
            Self::DetectionFailed { dependency, source } => {
                format!(
                    "Failed to detect dependency '{dependency}': {source}\n\n\
                    This may indicate a system configuration issue.\n\
                    Please ensure the dependency detection tool is working correctly."
                )
            }
        }
    }
}

// ============================================================================
// PRIVATE - Helper Functions
// ============================================================================

fn format_dependency_list(dependencies: &[Dependency]) -> String {
    dependencies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_dependencies_with_installed_dependency() {
        // Test with cargo-machete which should be installed in CI
        let deps = &[Dependency::CargoMachete];
        let result = verify_dependencies(deps);

        // This should succeed since cargo-machete is installed in CI
        assert!(result.is_ok(), "cargo-machete dependency check should pass");
    }

    #[test]
    fn test_dependency_verification_error_actionable_message() {
        let error = DependencyVerificationError::MissingDependencies {
            dependencies: vec![Dependency::OpenTofu, Dependency::Lxd],
        };

        let message = error.actionable_message();

        // Verify the message contains key information
        assert!(message.contains("opentofu"));
        assert!(message.contains("lxd"));
        assert!(message.contains("cargo run --bin dependency-installer install"));
        assert!(message.contains("README.md"));
    }

    #[test]
    fn test_dependency_verification_error_display() {
        let error = DependencyVerificationError::MissingDependencies {
            dependencies: vec![Dependency::Ansible],
        };

        let error_string = error.to_string();

        // Verify the error display format
        assert!(error_string.contains("Missing required dependencies"));
        assert!(error_string.contains("ansible"));
    }

    #[test]
    fn test_format_dependency_list_single() {
        let deps = vec![Dependency::Ansible];
        let formatted = format_dependency_list(&deps);
        assert_eq!(formatted, "ansible");
    }

    #[test]
    fn test_format_dependency_list_multiple() {
        let deps = vec![Dependency::OpenTofu, Dependency::Ansible, Dependency::Lxd];
        let formatted = format_dependency_list(&deps);
        assert_eq!(formatted, "opentofu, ansible, lxd");
    }
}
