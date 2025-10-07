//! Common trace file infrastructure
//!
//! This module provides shared functionality for all command-specific trace writers:
//! - Formatting sections (header, footer, error chain)
//! - File I/O operations
//! - Directory management
//! - Timestamp-based filename generation

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use thiserror::Error;

use crate::shared::Traceable;

/// Errors that can occur during trace file writing
#[derive(Debug, Error)]
pub enum TraceWriterError {
    #[error("Failed to create traces directory at {path}: {source}")]
    DirectoryCreation {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to write trace file at {path}: {source}")]
    FileWrite {
        path: String,
        source: std::io::Error,
    },
}

/// Common sections for all trace files
pub(crate) struct TraceSections;

impl TraceSections {
    /// Format a trace header
    pub(crate) fn header(title: &str) -> String {
        format!(
            "═══════════════════════════════════════════════════════════════\n\
             {title:^63}\n\
             ═══════════════════════════════════════════════════════════════\n\n"
        )
    }

    /// Format a trace footer
    pub(crate) fn footer() -> &'static str {
        "\n═══════════════════════════════════════════════════════════════\n\
                            END OF TRACE\n\
         ═══════════════════════════════════════════════════════════════\n"
    }

    /// Format an error chain section header
    pub(crate) fn error_chain_header() -> &'static str {
        "───────────────────────────────────────────────────────────────\n\
                             ERROR CHAIN\n\
         ───────────────────────────────────────────────────────────────\n\n"
    }

    /// Format a complete error chain by walking the `Traceable` hierarchy
    pub(crate) fn format_error_chain<E: Traceable>(error: &E) -> String {
        let mut chain = String::new();
        Self::format_error_chain_recursive(error, &mut chain, 0);
        chain
    }

    /// Recursively format error chain levels
    fn format_error_chain_recursive<E: Traceable + ?Sized>(
        error: &E,
        output: &mut String,
        level: usize,
    ) {
        use std::fmt::Write;
        let _ = writeln!(output, "[Level {level}] {}", error.trace_format());

        if let Some(source) = error.trace_source() {
            Self::format_error_chain_recursive(source, output, level + 1);
        }
    }
}

/// Common trace file writer infrastructure
///
/// Provides shared functionality for all command-specific trace writers:
/// - File I/O operations
/// - Directory management
/// - Timestamp-based filename generation
///
/// This is used as a collaborator by command-specific writers.
pub(crate) struct CommonTraceWriter {
    traces_dir: PathBuf,
}

impl CommonTraceWriter {
    /// Create a new common trace writer
    pub(crate) fn new(traces_dir: impl Into<PathBuf>) -> Self {
        Self {
            traces_dir: traces_dir.into(),
        }
    }

    /// Write trace content to a file
    ///
    /// Creates the traces directory if needed, generates a timestamp-based
    /// filename, and writes the content.
    pub(crate) fn write_trace(
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
    pub(crate) fn traces_dir(&self) -> &Path {
        &self.traces_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test error implementing Traceable
    #[derive(Debug)]
    struct TestError {
        message: String,
        source: Option<Box<dyn Traceable>>,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError: {}", self.message)
        }
    }

    impl std::error::Error for TestError {}

    impl Traceable for TestError {
        fn trace_format(&self) -> String {
            format!("TestError: {}", self.message)
        }

        fn trace_source(&self) -> Option<&dyn Traceable> {
            self.source.as_deref()
        }
    }

    #[test]
    fn it_should_format_error_chain_with_multiple_levels() {
        // Create nested error chain: level 0 -> level 1 -> level 2
        let level_2_error = TestError {
            message: "root cause".to_string(),
            source: None,
        };

        let level_1_error = TestError {
            message: "intermediate error".to_string(),
            source: Some(Box::new(level_2_error)),
        };

        let level_0_error = TestError {
            message: "top level error".to_string(),
            source: Some(Box::new(level_1_error)),
        };

        let chain = TraceSections::format_error_chain(&level_0_error);

        // Verify all levels are present
        assert!(chain.contains("[Level 0]"));
        assert!(chain.contains("[Level 1]"));
        assert!(chain.contains("[Level 2]"));
        assert!(chain.contains("top level error"));
        assert!(chain.contains("intermediate error"));
        assert!(chain.contains("root cause"));
    }
}
