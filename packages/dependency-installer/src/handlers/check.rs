//! Check command handler
//!
//! This module handles checking whether dependencies are installed.

// External crates
use thiserror::Error;
use tracing::info;

// Internal crate
use crate::detector::DetectionError;
use crate::{Dependency, DependencyManager};

// ============================================================================
// PUBLIC API - Functions
// ============================================================================

/// Handle the check command
///
/// # Errors
///
/// Returns an error if:
/// - Dependencies are missing
/// - Internal error occurs during dependency checking
pub fn handle_check(
    manager: &DependencyManager,
    dependency: Option<Dependency>,
) -> Result<(), CheckError> {
    match dependency {
        Some(dep) => check_specific_dependency(manager, dep)?,
        None => check_all_dependencies(manager)?,
    }

    Ok(())
}

// ============================================================================
// PRIVATE - Helper Functions
// ============================================================================

fn check_all_dependencies(manager: &DependencyManager) -> Result<(), CheckAllDependenciesError> {
    info!("Checking all dependencies");

    let results = manager.check_all()?;

    let mut missing_count = 0;

    for result in &results {
        let detector = manager.get_detector(result.dependency);
        let name = detector.name();
        if result.installed {
            info!(
                dependency = name,
                status = "installed",
                "Dependency check result"
            );
        } else {
            info!(
                dependency = name,
                status = "not installed",
                "Dependency check result"
            );
            missing_count += 1;
        }
    }

    if missing_count > 0 {
        info!(
            missing_count,
            total_count = results.len(),
            "Missing dependencies"
        );
        Err(CheckAllDependenciesError::MissingDependencies {
            missing_count,
            total_count: results.len(),
        })
    } else {
        info!("All dependencies are installed");
        Ok(())
    }
}

fn check_specific_dependency(
    manager: &DependencyManager,
    dependency: Dependency,
) -> Result<(), CheckSpecificDependencyError> {
    info!(dependency = %dependency, "Checking specific dependency");

    let detector = manager.get_detector(dependency);

    let installed = detector.is_installed()?;

    if installed {
        info!(
            dependency = detector.name(),
            status = "installed",
            "Dependency is installed"
        );
        Ok(())
    } else {
        info!(
            dependency = detector.name(),
            status = "not installed",
            "Dependency is not installed"
        );
        Err(CheckSpecificDependencyError::DependencyNotInstalled { dependency })
    }
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

/// Errors that can occur when handling the check command
#[derive(Debug, Error)]
pub enum CheckError {
    /// Failed to check all dependencies
    ///
    /// This occurs when checking all dependencies at once.
    #[error("Failed to check all dependencies: {source}")]
    CheckAllFailed {
        #[source]
        source: CheckAllDependenciesError,
    },

    /// Failed to check a specific dependency
    ///
    /// This occurs when checking a single specified dependency.
    #[error("Failed to check specific dependency: {source}")]
    CheckSpecificFailed {
        #[source]
        source: CheckSpecificDependencyError,
    },
}

impl From<CheckAllDependenciesError> for CheckError {
    fn from(source: CheckAllDependenciesError) -> Self {
        Self::CheckAllFailed { source }
    }
}

impl From<CheckSpecificDependencyError> for CheckError {
    fn from(source: CheckSpecificDependencyError) -> Self {
        Self::CheckSpecificFailed { source }
    }
}

/// Errors that can occur when checking all dependencies
#[derive(Debug, Error)]
pub enum CheckAllDependenciesError {
    /// Failed to check dependencies
    ///
    /// This occurs when the dependency detection system fails to check
    /// the status of installed tools.
    #[error("Failed to check dependencies: {source}")]
    DependencyCheckFailed {
        #[source]
        source: DetectionError,
    },

    /// One or more dependencies are missing
    ///
    /// This occurs when required tools are not installed on the system.
    #[error("Missing {missing_count} out of {total_count} required dependencies")]
    MissingDependencies {
        /// Number of missing dependencies
        missing_count: usize,
        /// Total number of dependencies checked
        total_count: usize,
    },
}

impl From<DetectionError> for CheckAllDependenciesError {
    fn from(source: DetectionError) -> Self {
        Self::DependencyCheckFailed { source }
    }
}

/// Errors that can occur when checking a specific dependency
#[derive(Debug, Error)]
pub enum CheckSpecificDependencyError {
    /// Failed to detect if the dependency is installed
    ///
    /// This occurs when the dependency detection system fails to check
    /// whether a specific dependency is installed.
    #[error("Failed to detect dependency installation: {source}")]
    DetectionFailed {
        #[source]
        source: DetectionError,
    },

    /// Dependency is not installed
    ///
    /// This occurs when the specified dependency is not found on the system.
    #[error("{dependency}: not installed")]
    DependencyNotInstalled {
        /// The dependency that is not installed
        dependency: Dependency,
    },
}

impl From<DetectionError> for CheckSpecificDependencyError {
    fn from(source: DetectionError) -> Self {
        Self::DetectionFailed { source }
    }
}
