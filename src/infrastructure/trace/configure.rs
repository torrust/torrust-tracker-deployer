//! Configure command trace writer
//!
//! Generates trace files for configure command failures with configure-specific
//! context and metadata.

use std::path::{Path, PathBuf};

use crate::domain::environment::state::ConfigureFailureContext;
use crate::shared::Traceable;

use super::common::{CommonTraceWriter, TraceSections, TraceWriterError};

/// Configure-specific trace writer
///
/// Generates trace files for configure command failures with configure-specific
/// context and metadata.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use torrust_tracker_deploy::infrastructure::trace::ConfigureTraceWriter;
///
/// let traces_dir = PathBuf::from("data/my-env/traces");
/// let writer = ConfigureTraceWriter::new(traces_dir);
/// ```
pub struct ConfigureTraceWriter {
    common: CommonTraceWriter,
}

impl ConfigureTraceWriter {
    /// Create a new configure trace writer
    #[must_use]
    pub fn new(traces_dir: impl Into<PathBuf>) -> Self {
        Self {
            common: CommonTraceWriter::new(traces_dir),
        }
    }

    /// Write a configure failure trace file
    ///
    /// # Arguments
    ///
    /// * `ctx` - The configure failure context with metadata
    /// * `error` - The error that implements `Traceable` for chain extraction
    ///
    /// # Returns
    ///
    /// Path to the generated trace file
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation or file writing fails
    pub fn write_trace<E: Traceable>(
        &self,
        ctx: &ConfigureFailureContext,
        error: &E,
    ) -> Result<PathBuf, TraceWriterError> {
        let trace_content = Self::format_trace(ctx, error);
        self.common.write_trace("configure", &trace_content)
    }

    /// Format a complete configure trace
    fn format_trace<E: Traceable>(ctx: &ConfigureFailureContext, error: &E) -> String {
        use std::fmt::Write;

        let mut trace = String::new();

        // Header
        trace.push_str(&TraceSections::header("CONFIGURE FAILURE TRACE"));

        // Metadata
        let _ = writeln!(trace, "Trace ID: {}", ctx.trace_id);
        let _ = writeln!(trace, "Failed At: {}", ctx.failed_at);
        let _ = writeln!(trace, "Execution Started: {}", ctx.execution_started_at);
        let _ = writeln!(trace, "Execution Duration: {:?}", ctx.execution_duration);
        let _ = writeln!(trace, "Failed Step: {:?}", ctx.failed_step);
        let _ = writeln!(trace, "Error Kind: {:?}", ctx.error_kind);
        let _ = writeln!(trace, "Error Summary: {}\n", ctx.error_summary);

        // Error chain
        trace.push_str(TraceSections::error_chain_header());
        trace.push_str(&TraceSections::format_error_chain(error));

        // Footer
        trace.push_str(TraceSections::footer());

        trace
    }

    /// Get the traces directory path
    #[must_use]
    pub fn traces_dir(&self) -> &Path {
        self.common.traces_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;
    use tempfile::TempDir;

    use crate::domain::environment::state::{
        ConfigureErrorKind, ConfigureFailureContext, ConfigureStep,
    };
    use crate::domain::environment::TraceId;

    // Test error implementing Traceable
    #[derive(Debug)]
    struct TestError {
        message: String,
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
            None
        }
    }

    #[test]
    fn it_should_write_configure_trace_with_correct_naming() {
        let temp_dir = TempDir::new().unwrap();
        let traces_dir = temp_dir.path().join("traces");
        let writer = ConfigureTraceWriter::new(traces_dir.clone());

        let error = TestError {
            message: "configure test error".to_string(),
        };

        let now = Utc::now();
        let context = ConfigureFailureContext {
            failed_step: ConfigureStep::InstallDocker,
            error_kind: ConfigureErrorKind::InstallationFailed,
            error_summary: error.to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(3),
            trace_id: TraceId::new(),
            trace_file_path: None,
        };

        let trace_file = writer.write_trace(&context, &error).unwrap();

        // Verify file exists
        assert!(trace_file.exists());

        // Verify filename ends with -configure.log
        let filename = trace_file.file_name().unwrap().to_str().unwrap();
        assert!(filename.ends_with("-configure.log"));
    }

    #[test]
    fn it_should_include_trace_metadata_in_configure_trace() {
        let temp_dir = TempDir::new().unwrap();
        let traces_dir = temp_dir.path().join("traces");
        let writer = ConfigureTraceWriter::new(traces_dir);

        let error = TestError {
            message: "test error".to_string(),
        };

        let now = Utc::now();
        let trace_id = TraceId::new();
        let context = ConfigureFailureContext {
            failed_step: ConfigureStep::InstallDocker,
            error_kind: ConfigureErrorKind::InstallationFailed,
            error_summary: "Test configure error".to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(3),
            trace_id: trace_id.clone(),
            trace_file_path: None,
        };

        let trace_file = writer.write_trace(&context, &error).unwrap();
        let trace_content = std::fs::read_to_string(trace_file).unwrap();

        // Verify metadata is included
        assert!(trace_content.contains("CONFIGURE FAILURE TRACE"));
        assert!(trace_content.contains(&format!("Trace ID: {trace_id}")));
        assert!(trace_content.contains("Failed Step: InstallDocker"));
        assert!(trace_content.contains("Error Kind: InstallationFailed"));
        assert!(trace_content.contains("Error Summary: Test configure error"));
    }
}
