//! List command handler
//!
//! This module handles listing all available dependencies and their status.

// External crates
use thiserror::Error;
use tracing::info;

// Internal crate
use crate::detector::DetectionError;
use crate::DependencyManager;

// ============================================================================
// PUBLIC API - Functions
// ============================================================================

/// Handle the list command
///
/// # Errors
///
/// Returns an error if dependency checking fails
pub fn handle_list(manager: &DependencyManager) -> Result<(), ListError> {
    info!("Listing all available dependencies");

    let results = manager.check_all()?;

    for result in results {
        let detector = manager.get_detector(result.dependency);
        let name = detector.name();
        let status = if result.installed {
            "installed"
        } else {
            "not installed"
        };
        info!(dependency = name, status, "Available dependency");
    }

    Ok(())
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

/// Errors that can occur when listing dependencies
#[derive(Debug, Error)]
pub enum ListError {
    /// Failed to check dependencies
    ///
    /// This occurs when the dependency detection system fails to check
    /// the status of installed tools.
    #[error("Failed to check dependencies: {source}")]
    DependencyCheckFailed {
        #[source]
        source: DetectionError,
    },
}

impl From<DetectionError> for ListError {
    fn from(source: DetectionError) -> Self {
        Self::DependencyCheckFailed { source }
    }
}
