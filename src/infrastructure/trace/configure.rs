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
        let _ = writeln!(trace, "Trace ID: {}", ctx.base.trace_id);
        let _ = writeln!(trace, "Failed At: {}", ctx.base.failed_at);
        let _ = writeln!(
            trace,
            "Execution Started: {}",
            ctx.base.execution_started_at
        );
        let _ = writeln!(
            trace,
            "Execution Duration: {:?}",
            ctx.base.execution_duration
        );
        let _ = writeln!(trace, "Failed Step: {:?}", ctx.failed_step);
        let _ = writeln!(trace, "Error Kind: {:?}", ctx.error_kind);
        let _ = writeln!(trace, "Error Summary: {}\n", ctx.base.error_summary);

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
        BaseFailureContext, ConfigureErrorKind, ConfigureFailureContext, ConfigureStep,
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

    // Test helpers - Arrange phase utilities

    /// Create a test error with the given message
    fn create_test_error(message: &str) -> TestError {
        TestError {
            message: message.to_string(),
        }
    }

    /// Create a test writer with a temporary directory
    ///
    /// Returns (writer, `temp_dir`, `traces_dir`)
    /// The `temp_dir` must be kept alive for the duration of the test
    fn create_test_writer() -> (ConfigureTraceWriter, TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let traces_dir = temp_dir.path().join("traces");
        let writer = ConfigureTraceWriter::new(traces_dir.clone());
        (writer, temp_dir, traces_dir)
    }

    /// Create a configure failure context with default test values
    ///
    /// # Arguments
    ///
    /// * `error_summary` - The error summary message
    ///
    /// # Returns
    ///
    /// A configure failure context with sensible defaults for testing
    fn create_test_context(error_summary: &str) -> ConfigureFailureContext {
        let now = Utc::now();
        ConfigureFailureContext {
            failed_step: ConfigureStep::InstallDocker,
            error_kind: ConfigureErrorKind::InstallationFailed,
            base: BaseFailureContext {
                error_summary: error_summary.to_string(),
                failed_at: now,
                execution_started_at: now,
                execution_duration: Duration::from_secs(3),
                trace_id: TraceId::new(),
                trace_file_path: None,
            },
        }
    }

    /// Create a configure failure context with a specific trace ID
    ///
    /// Useful when you need to verify trace ID in assertions
    fn create_test_context_with_trace_id(
        error_summary: &str,
        trace_id: TraceId,
    ) -> ConfigureFailureContext {
        let now = Utc::now();
        ConfigureFailureContext {
            failed_step: ConfigureStep::InstallDocker,
            error_kind: ConfigureErrorKind::InstallationFailed,
            base: BaseFailureContext {
                error_summary: error_summary.to_string(),
                failed_at: now,
                execution_started_at: now,
                execution_duration: Duration::from_secs(3),
                trace_id,
                trace_file_path: None,
            },
        }
    }

    #[test]
    fn it_should_create_configure_trace_writer_with_directory() {
        // Arrange
        let (writer, _temp_dir, traces_dir) = create_test_writer();

        // Assert
        assert_eq!(writer.traces_dir(), traces_dir);
    }

    #[test]
    fn it_should_create_traces_directory_on_first_write() {
        // Arrange
        let (writer, _temp_dir, traces_dir) = create_test_writer();
        let error = create_test_error("test error");
        let context = create_test_context(&error.to_string());

        // Directory should not exist yet
        assert!(!traces_dir.exists());

        // Act
        writer.write_trace(&context, &error).unwrap();

        // Assert
        assert!(traces_dir.exists());
    }

    #[test]
    fn it_should_write_configure_trace_with_correct_naming() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let error = create_test_error("configure test error");
        let context = create_test_context(&error.to_string());

        // Act
        let trace_file = writer.write_trace(&context, &error).unwrap();

        // Assert
        assert!(trace_file.exists());

        let filename = trace_file.file_name().unwrap().to_str().unwrap();
        assert!(filename.ends_with("-configure.log"));
    }

    #[test]
    fn it_should_use_timestamp_and_command_as_filename() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let error = create_test_error("test error");
        let context = create_test_context(&error.to_string());

        // Act
        let trace_file = writer.write_trace(&context, &error).unwrap();

        // Assert
        let filename = trace_file.file_name().unwrap().to_str().unwrap();

        // Verify filename format: {timestamp}-configure.log
        // Example: 20251007-103045-configure.log
        assert!(filename.ends_with("-configure.log"));

        // Verify timestamp prefix exists (YYYYmmdd-HHMMSS format)
        let parts: Vec<&str> = filename.split('-').collect();
        assert!(parts.len() >= 3); // At least YYYYmmdd, HHMMSS, configure.log

        // Verify first part is date (8 digits)
        assert_eq!(parts[0].len(), 8);
        assert!(parts[0].chars().all(|c| c.is_ascii_digit()));

        // Verify second part is time (6 digits)
        let time_part = parts[1];
        assert_eq!(time_part.len(), 6);
        assert!(time_part.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn it_should_include_trace_metadata_in_configure_trace() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let error = create_test_error("test error");
        let trace_id = TraceId::new();
        let context = create_test_context_with_trace_id("Test configure error", trace_id.clone());

        // Act
        let trace_file = writer.write_trace(&context, &error).unwrap();
        let trace_content = std::fs::read_to_string(trace_file).unwrap();

        // Assert - Verify metadata is included
        assert!(trace_content.contains("CONFIGURE FAILURE TRACE"));
        assert!(trace_content.contains(&format!("Trace ID: {trace_id}")));
        assert!(trace_content.contains("Failed Step: InstallDocker"));
        assert!(trace_content.contains("Error Kind: InstallationFailed"));
        assert!(trace_content.contains("Error Summary: Test configure error"));
    }

    #[test]
    fn it_should_generate_trace_files_with_correct_naming() {
        // This test verifies that trace files are created with correct naming convention
        // and follow the format: {timestamp}-{command}.log

        // Arrange
        let (writer, _temp_dir, traces_dir) = create_test_writer();
        let error = create_test_error("Simulated configure failure");
        let context = create_test_context(&error.to_string());

        // Act
        let _trace_path = writer
            .write_trace(&context, &error)
            .expect("Failed to write trace file");

        // Assert
        // Verify trace directory was created
        assert!(
            traces_dir.exists(),
            "Traces directory should be created at: {traces_dir:?}"
        );

        // Verify trace file was created with correct naming: {timestamp}-configure.log
        let trace_files: Vec<PathBuf> = std::fs::read_dir(&traces_dir)
            .expect("Failed to read traces directory")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect();

        assert_eq!(
            trace_files.len(),
            1,
            "Expected exactly 1 trace file, found: {trace_files:?}"
        );

        let trace_file = &trace_files[0];
        let filename = trace_file
            .file_name()
            .unwrap()
            .to_str()
            .expect("Filename should be valid UTF-8");

        // Verify filename format
        assert!(
            filename.ends_with("-configure.log"),
            "Filename should end with '-configure.log', got: {filename}"
        );

        // Verify file contains expected sections
        let trace_data = std::fs::read_to_string(trace_file).expect("Failed to read trace file");
        assert!(trace_data.contains("CONFIGURE FAILURE TRACE"));
        assert!(trace_data.contains("ERROR CHAIN"));
        assert!(trace_data.contains("END OF TRACE"));
    }
}
