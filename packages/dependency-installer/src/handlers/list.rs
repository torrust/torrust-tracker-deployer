//! List command handler
//!
//! This module handles listing all available dependencies and their status.

use tracing::info;

use crate::DependencyManager;

/// Handle the list command
///
/// # Errors
///
/// Returns an error if dependency checking fails
pub fn handle_list(manager: &DependencyManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("Listing all available tools");
    println!("Available tools:\n");

    let results = manager.check_all()?;
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
