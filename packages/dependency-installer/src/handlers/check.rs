//! Check command handler
//!
//! This module handles checking whether dependencies are installed.

use thiserror::Error;
use tracing::{error, info};

use crate::detector::DetectionError;
use crate::{Dependency, DependencyManager};

/// Errors that can occur when handling the check command
#[derive(Debug, Error)]
pub enum CheckError {
    /// Failed to check all dependencies
    ///
    /// This occurs when checking all dependencies at once.
    #[error("Failed to check all dependencies: {source}")]
    CheckAllFailed {
        #[source]
        source: CheckAllToolsError,
    },

    /// Failed to check a specific dependency
    ///
    /// This occurs when checking a single specified dependency.
    #[error("Failed to check specific dependency: {source}")]
    CheckSpecificFailed {
        #[source]
        source: CheckSpecificToolError,
    },
}

impl From<CheckAllToolsError> for CheckError {
    fn from(source: CheckAllToolsError) -> Self {
        Self::CheckAllFailed { source }
    }
}

impl From<CheckSpecificToolError> for CheckError {
    fn from(source: CheckSpecificToolError) -> Self {
        Self::CheckSpecificFailed { source }
    }
}

/// Errors that can occur when checking all tools
#[derive(Debug, Error)]
pub enum CheckAllToolsError {
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

impl From<DetectionError> for CheckAllToolsError {
    fn from(source: DetectionError) -> Self {
        Self::DependencyCheckFailed { source }
    }
}

/// Errors that can occur when checking a specific dependency
#[derive(Debug, Error)]
pub enum CheckSpecificToolError {
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
        /// Name of the dependency that is not installed
        dependency: String,
    },
}

impl From<DetectionError> for CheckSpecificToolError {
    fn from(source: DetectionError) -> Self {
        Self::DetectionFailed { source }
    }
}

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

fn check_all_dependencies(manager: &DependencyManager) -> Result<(), CheckAllToolsError> {
    info!("Checking all dependencies");
    println!("Checking dependencies...\n");

    let results = manager.check_all()?;

    let mut missing_count = 0;

    for result in &results {
        let detector = manager.get_detector(result.dependency);
        let name = detector.name();
        if result.installed {
            println!("✓ {name}: installed");
        } else {
            println!("✗ {name}: not installed");
            missing_count += 1;
        }
    }

    println!();

    if missing_count > 0 {
        error!(
            "Missing {} out of {} required dependencies",
            missing_count,
            results.len()
        );
        eprintln!(
            "Missing {missing_count} out of {} required dependencies",
            results.len()
        );
        Err(CheckAllToolsError::MissingDependencies {
            missing_count,
            total_count: results.len(),
        })
    } else {
        info!("All dependencies are installed");
        println!("All dependencies are installed");
        Ok(())
    }
}

fn check_specific_dependency(
    manager: &DependencyManager,
    dependency: Dependency,
) -> Result<(), CheckSpecificToolError> {
    info!(dependency = %dependency, "Checking specific dependency");

    let detector = manager.get_detector(dependency);

    let installed = detector.is_installed()?;

    if installed {
        info!(dependency = detector.name(), "Dependency is installed");
        println!("✓ {}: installed", detector.name());
        Ok(())
    } else {
        error!(dependency = detector.name(), "Dependency is not installed");
        eprintln!("✗ {}: not installed", detector.name());
        Err(CheckSpecificToolError::DependencyNotInstalled {
            dependency: detector.name().to_string(),
        })
    }
}
