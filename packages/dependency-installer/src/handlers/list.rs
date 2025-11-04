//! List command handler
//!
//! This module handles listing all available dependencies and their status.

use thiserror::Error;
use tracing::info;

use crate::detector::DetectionError;
use crate::DependencyManager;

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

/// Handle the list command
///
/// # Errors
///
/// Returns an error if dependency checking fails
pub fn handle_list(manager: &DependencyManager) -> Result<(), ListError> {
    info!("Listing all available tools");
    println!("Available tools:\n");

    let results = manager
        .check_all()
        .map_err(|source| ListError::DependencyCheckFailed { source })?;

    for result in results {
        let status = if result.installed {
            "installed"
        } else {
            "not installed"
        };
        println!("- {} ({status})", result.tool);
    }

    Ok(())
}
