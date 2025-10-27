//! Temporary Workspace Management
//!
//! Provides utilities for creating and managing temporary workspaces
//! for black-box testing with automatic cleanup.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Manages a temporary workspace for black-box testing
///
/// This struct provides a temporary directory with helper methods
/// for creating test configurations and accessing workspace paths.
/// The temporary directory is automatically cleaned up when dropped.
pub struct TempWorkspace {
    temp_dir: TempDir,
}

impl TempWorkspace {
    /// Create a new temporary workspace
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created.
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self { temp_dir })
    }

    /// Get the path to the temporary workspace
    #[must_use]
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Write a configuration file to the workspace
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_config_file(&self, filename: &str, config: &str) -> Result<()> {
        let file_path = self.temp_dir.path().join(filename);
        fs::write(file_path, config)?;
        Ok(())
    }

    /// Write a file to the workspace
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    #[allow(dead_code)]
    pub fn write_file(&self, filename: &str, content: &str) -> Result<()> {
        let file_path = self.temp_dir.path().join(filename);
        fs::write(file_path, content)?;
        Ok(())
    }

    /// Get the data directory path
    #[must_use]
    #[allow(dead_code)]
    pub fn data_dir(&self) -> PathBuf {
        self.temp_dir.path().join("data")
    }

    /// Get the build directory path
    #[must_use]
    #[allow(dead_code)]
    pub fn build_dir(&self) -> PathBuf {
        self.temp_dir.path().join("build")
    }
}
