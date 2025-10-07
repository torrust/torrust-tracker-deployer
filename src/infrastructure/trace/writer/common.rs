//! Common trace file writer infrastructure
//!
//! Provides shared file I/O operations for all command-specific trace writers:
//! - File creation and writing
//! - Directory management
//! - Timestamp-based filename generation

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;

use super::error::TraceWriterError;

/// Common trace file writer infrastructure
///
/// Provides shared functionality for all command-specific trace writers:
/// - File I/O operations
/// - Directory management
/// - Timestamp-based filename generation
///
/// This is used as a collaborator by command-specific writers.
pub(super) struct CommonTraceWriter {
    traces_dir: PathBuf,
}

impl CommonTraceWriter {
    /// Create a new common trace writer
    ///
    /// # Arguments
    ///
    /// * `traces_dir` - Directory where trace files will be written
    pub(super) fn new(traces_dir: impl Into<PathBuf>) -> Self {
        Self {
            traces_dir: traces_dir.into(),
        }
    }

    /// Write trace content to a file
    ///
    /// Creates the traces directory if needed, generates a timestamp-based
    /// filename, and writes the content.
    ///
    /// # Arguments
    ///
    /// * `command_name` - Name of the command (used in filename: `{timestamp}-{command_name}.log`)
    /// * `content` - Content to write to the trace file
    ///
    /// # Returns
    ///
    /// Path to the created trace file
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation or file writing fails
    pub(super) fn write_trace(
        &self,
        command_name: &str,
        content: &str,
    ) -> Result<PathBuf, TraceWriterError> {
        self.ensure_traces_dir()?;

        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        let trace_file = self
            .traces_dir
            .join(format!("{timestamp}-{command_name}.log"));

        let mut file =
            fs::File::create(&trace_file).map_err(|source| TraceWriterError::FileWrite {
                path: trace_file.display().to_string(),
                source,
            })?;

        file.write_all(content.as_bytes())
            .map_err(|source| TraceWriterError::FileWrite {
                path: trace_file.display().to_string(),
                source,
            })?;

        Ok(trace_file)
    }

    /// Ensure the traces directory exists
    ///
    /// Creates the directory if it doesn't exist.
    fn ensure_traces_dir(&self) -> Result<(), TraceWriterError> {
        if !self.traces_dir.exists() {
            fs::create_dir_all(&self.traces_dir).map_err(|source| {
                TraceWriterError::DirectoryCreation {
                    path: self.traces_dir.display().to_string(),
                    source,
                }
            })?;
        }
        Ok(())
    }

    /// Get the traces directory path
    pub(super) fn traces_dir(&self) -> &Path {
        &self.traces_dir
    }
}
