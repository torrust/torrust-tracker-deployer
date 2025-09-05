//! Template utility functions
//!
//! Provides utility functions for file operations and static file copying.

use anyhow::{Context, Result};
use std::path::Path;

/// Copy a file without template processing (for static files)
///
/// # Errors
/// Returns an error if the source file cannot be read, destination directory
/// cannot be created, or the file cannot be copied
pub fn copy_static_file(source: &Path, destination: &Path) -> Result<()> {
    // Ensure destination directory exists
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).context(format!(
            "Failed to create destination directory: {}",
            parent.display()
        ))?;
    }

    std::fs::copy(source, destination).context(format!(
        "Failed to copy {} to {}",
        source.display(),
        destination.display()
    ))?;

    Ok(())
}
