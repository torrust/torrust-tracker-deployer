//! Trace writer error types
//!
//! Defines errors that can occur during trace file writing operations.

use thiserror::Error;

/// Errors that can occur during trace file writing
#[derive(Debug, Error)]
pub enum TraceWriterError {
    /// Failed to create the traces directory
    #[error("Failed to create traces directory at {path}: {source}")]
    DirectoryCreation {
        /// Path where directory creation was attempted
        path: String,

        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Failed to write a trace file
    #[error("Failed to write trace file at {path}: {source}")]
    FileWrite {
        /// Path where file write was attempted
        path: String,

        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },
}
