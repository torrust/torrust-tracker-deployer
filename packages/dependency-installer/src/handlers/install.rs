//! Install command handler
//!
//! This module handles installing dependencies.

// External crates
use thiserror::Error;
use tracing::info;

// Internal crate
use crate::installer::InstallationError;
use crate::{Dependency, DependencyManager};

// ============================================================================
// PUBLIC API - Functions
// ============================================================================

/// Handle the install command
///
/// # Errors
///
/// Returns an error if:
/// - Installation fails
/// - One or more dependencies fail to install
pub async fn handle_install(
    manager: &DependencyManager,
    dependency: Option<Dependency>,
) -> Result<(), InstallError> {
    match dependency {
        Some(dep) => install_specific_dependency(manager, dep).await?,
        None => install_all_dependencies(manager).await?,
    }

    Ok(())
}

// ============================================================================
// PRIVATE - Helper Functions
// ============================================================================

async fn install_all_dependencies(
    manager: &DependencyManager,
) -> Result<(), InstallAllDependenciesError> {
    info!("Installing all dependencies");

    let results = manager.install_all().await;

    let mut failed_count = 0;

    for result in &results {
        let installer = manager.get_installer(result.dependency);
        let name = installer.name();

        if result.success {
            info!(
                dependency = name,
                status = "installed",
                "Dependency installation result"
            );
        } else {
            info!(
                dependency = name,
                status = "failed",
                error = result.error.as_deref().unwrap_or("unknown error"),
                "Dependency installation result"
            );
            failed_count += 1;
        }
    }

    if failed_count > 0 {
        info!(
            failed_count,
            total_count = results.len(),
            "Some dependencies failed to install"
        );
        Err(InstallAllDependenciesError::InstallationsFailed {
            failed_count,
            total_count: results.len(),
        })
    } else {
        info!("All dependencies installed successfully");
        Ok(())
    }
}

async fn install_specific_dependency(
    manager: &DependencyManager,
    dependency: Dependency,
) -> Result<(), InstallSpecificDependencyError> {
    info!(dependency = %dependency, "Installing specific dependency");

    let installer = manager.get_installer(dependency);

    installer.install().await?;

    info!(
        dependency = installer.name(),
        status = "installed",
        "Dependency installation completed"
    );

    Ok(())
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

/// Errors that can occur when handling the install command
#[derive(Debug, Error)]
pub enum InstallError {
    /// Failed to install all dependencies
    ///
    /// This occurs when installing all dependencies at once.
    #[error("Failed to install all dependencies: {source}")]
    InstallAllFailed {
        #[source]
        source: InstallAllDependenciesError,
    },

    /// Failed to install a specific dependency
    ///
    /// This occurs when installing a single specified dependency.
    #[error("Failed to install specific dependency: {source}")]
    InstallSpecificFailed {
        #[source]
        source: InstallSpecificDependencyError,
    },
}

impl From<InstallAllDependenciesError> for InstallError {
    fn from(source: InstallAllDependenciesError) -> Self {
        Self::InstallAllFailed { source }
    }
}

impl From<InstallSpecificDependencyError> for InstallError {
    fn from(source: InstallSpecificDependencyError) -> Self {
        Self::InstallSpecificFailed { source }
    }
}

/// Errors that can occur when installing all dependencies
#[derive(Debug, Error)]
pub enum InstallAllDependenciesError {
    /// One or more dependencies failed to install
    ///
    /// This occurs when some installations fail.
    #[error("Failed to install {failed_count} out of {total_count} dependencies")]
    InstallationsFailed {
        /// Number of failed installations
        failed_count: usize,
        /// Total number of dependencies
        total_count: usize,
    },
}

/// Errors that can occur when installing a specific dependency
#[derive(Debug, Error)]
pub enum InstallSpecificDependencyError {
    /// Failed to install the dependency
    ///
    /// This occurs when the dependency installation fails.
    #[error("Installation failed: {source}")]
    InstallationFailed {
        #[source]
        source: InstallationError,
    },
}

impl From<InstallationError> for InstallSpecificDependencyError {
    fn from(source: InstallationError) -> Self {
        Self::InstallationFailed { source }
    }
}
