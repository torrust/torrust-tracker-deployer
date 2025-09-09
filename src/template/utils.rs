//! Template utility functions
//!
//! Provides utility functions for file operations and static file copying.

use anyhow::{Context, Result};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during file writing operations
#[derive(Error, Debug)]
pub enum FileWriteError {
    /// Failed to create the output directory
    #[error("Failed to create output directory: {path}")]
    DirectoryCreation { path: String },
    
    /// Failed to write the file to the output path
    #[error("Failed to write file to: {path}")]
    FileWrite { path: String },
}

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

/// Write content to a file, creating parent directories if necessary
///
/// # Errors
/// Returns `FileWriteError::DirectoryCreation` if the parent directory cannot be created,
/// or `FileWriteError::FileWrite` if the file cannot be written
pub fn write_file_with_dir_creation(
    output_path: &Path,
    content: &str,
) -> Result<(), FileWriteError> {
    // Create output directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| FileWriteError::DirectoryCreation {
            path: parent.display().to_string(),
        })?;
    }

    // Write the content to the file
    std::fs::write(output_path, content).map_err(|_| FileWriteError::FileWrite {
        path: output_path.display().to_string(),
    })?;

    Ok(())
}
