//! Run command trace writer
//!
//! Generates trace files for run command failures with run-specific
//! context and metadata.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::environment::state::RunFailureContext;
use crate::shared::{Clock, Traceable};

use super::super::common::CommonTraceWriter;
use super::super::error::TraceWriterError;
use super::super::sections::TraceSections;

/// Run-specific trace writer
///
/// Generates trace files for run command failures with run-specific
/// context and metadata.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::infrastructure::trace::RunTraceWriter;
/// use torrust_tracker_deployer_lib::shared::SystemClock;
///
/// let traces_dir = PathBuf::from("data/my-env/traces");
/// let clock = Arc::new(SystemClock);
/// let writer = RunTraceWriter::new(traces_dir, clock);
/// ```
pub struct RunTraceWriter {
    common: CommonTraceWriter,
}

impl RunTraceWriter {
    /// Create a new run trace writer
    #[must_use]
    pub fn new(traces_dir: impl Into<PathBuf>, clock: Arc<dyn Clock>) -> Self {
        Self {
            common: CommonTraceWriter::new(traces_dir, clock),
        }
    }

    /// Write a run failure trace file
    ///
    /// Generates a trace file with run-specific context and logs the outcome.
    /// Success is logged at INFO level, failures at WARN level.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The run failure context with metadata
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
        ctx: &RunFailureContext,
        error: &E,
    ) -> Result<PathBuf, TraceWriterError> {
        use tracing::{info, warn};

        let trace_content = Self::format_trace(ctx, error);

        match self.common.write_trace("run", &trace_content) {
            Ok(trace_file_path) => {
                info!(
                    trace_id = %ctx.base.trace_id,
                    trace_file = ?trace_file_path,
                    "Generated trace file for run failure"
                );
                Ok(trace_file_path)
            }
            Err(e) => {
                warn!(
                    trace_id = %ctx.base.trace_id,
                    error = %e,
                    "Failed to generate trace file for run failure"
                );
                Err(e)
            }
        }
    }

    /// Format a complete run trace
    fn format_trace<E: Traceable>(ctx: &RunFailureContext, error: &E) -> String {
        use std::fmt::Write;

        let mut trace = String::new();

        // Header
        trace.push_str(&TraceSections::header("RUN FAILURE TRACE"));

        // Base metadata (common to all failures)
        trace.push_str(&TraceSections::format_base_metadata(&ctx.base));

        // Command-specific metadata
        let _ = writeln!(trace, "Failed Step: {}", ctx.failed_step);
        let _ = writeln!(trace, "Error Kind: {:?}\n", ctx.error_kind);

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

    use crate::domain::environment::state::{BaseFailureContext, RunFailureContext, RunStep};
    use crate::domain::environment::TraceId;
    use crate::shared::ErrorKind;

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

        fn error_kind(&self) -> crate::shared::ErrorKind {
            crate::shared::ErrorKind::CommandExecution
        }
    }

    // Test helpers - Arrange phase utilities

    /// Create a test error with the given message
    fn create_test_error(message: &str) -> TestError {
        TestError {
            message: message.to_string(),
            source: None,
        }
    }

    /// Create a test writer with a temporary directory
    ///
    /// Returns (writer, `temp_dir`, `traces_dir`)
    /// The `temp_dir` must be kept alive for the duration of the test
    fn create_test_writer() -> (RunTraceWriter, TempDir, PathBuf) {
        use crate::domain::environment::TRACES_DIR_NAME;
        use crate::testing::MockClock;
        use chrono::TimeZone;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let traces_dir = temp_dir.path().join(TRACES_DIR_NAME);
        let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = Arc::new(MockClock::new(fixed_time));
        let writer = RunTraceWriter::new(traces_dir.clone(), clock);
        (writer, temp_dir, traces_dir)
    }

    /// Create a run failure context with default test values
    ///
    /// # Arguments
    ///
    /// * `error_summary` - The error summary message
    ///
    /// # Returns
    ///
    /// A run failure context with sensible defaults for testing
    fn create_test_context(error_summary: &str) -> RunFailureContext {
        let now = Utc::now();
        RunFailureContext {
            failed_step: RunStep::StartServices,
            error_kind: ErrorKind::InfrastructureOperation,
            base: BaseFailureContext {
                error_summary: error_summary.to_string(),
                failed_at: now,
                execution_started_at: now,
                execution_duration: Duration::from_secs(5),
                trace_id: TraceId::new(),
                trace_file_path: None,
            },
        }
    }

    /// Create a run failure context with a specific trace ID
    ///
    /// Useful when you need to verify trace ID in assertions
    fn create_test_context_with_trace_id(
        error_summary: &str,
        trace_id: TraceId,
    ) -> RunFailureContext {
        let now = Utc::now();
        RunFailureContext {
            failed_step: RunStep::StartServices,
            error_kind: ErrorKind::InfrastructureOperation,
            base: BaseFailureContext {
                error_summary: error_summary.to_string(),
                failed_at: now,
                execution_started_at: now,
                execution_duration: Duration::from_secs(5),
                trace_id,
                trace_file_path: None,
            },
        }
    }

    #[test]
    fn it_should_create_run_trace_writer_with_directory() {
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
    fn it_should_write_run_trace_with_correct_naming() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let error = create_test_error("run test error");
        let context = create_test_context(&error.to_string());

        // Act
        let trace_file = writer.write_trace(&context, &error).unwrap();

        // Assert
        assert!(trace_file.exists());

        let filename = trace_file.file_name().unwrap().to_str().unwrap();
        assert!(filename.ends_with("-run.log"));
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

        // Verify filename format: {timestamp}-run.log
        // Example: 20251003-103045-run.log
        assert!(filename.ends_with("-run.log"));

        // Verify timestamp prefix exists (YYYYmmdd-HHMMSS format)
        let parts: Vec<&str> = filename.split('-').collect();
        assert!(parts.len() >= 3); // At least YYYYmmdd, HHMMSS, run.log

        // Verify first part is date (8 digits)
        assert_eq!(parts[0].len(), 8);
        assert!(parts[0].chars().all(|c| c.is_ascii_digit()));

        // Verify second part is time (6 digits)
        let time_part = parts[1];
        assert_eq!(time_part.len(), 6);
        assert!(time_part.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn it_should_include_trace_metadata_in_run_trace() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let error = create_test_error("test error");
        let trace_id = TraceId::new();
        let context = create_test_context_with_trace_id("Test error summary", trace_id.clone());

        // Act
        let trace_file = writer.write_trace(&context, &error).unwrap();
        let trace_data = std::fs::read_to_string(trace_file).unwrap();

        // Assert - Verify metadata is included
        assert!(trace_data.contains("RUN FAILURE TRACE"));
        assert!(trace_data.contains(&format!("Trace ID: {trace_id}")));
        assert!(trace_data.contains("Failed Step: Start Services"));
        assert!(trace_data.contains("Error Kind: InfrastructureOperation"));
        assert!(trace_data.contains("Error Summary: Test error summary"));
    }

    #[test]
    fn it_should_generate_trace_files_with_correct_naming() {
        // This test verifies that trace files are created with correct naming convention
        // and follow the format: {timestamp}-{command}.log

        // Arrange
        let (writer, _temp_dir, traces_dir) = create_test_writer();
        let error = create_test_error("Simulated run failure");
        let context = create_test_context(&error.to_string());

        // Act
        let _trace_path = writer
            .write_trace(&context, &error)
            .expect("Failed to write trace file");

        // Verify trace directory was created
        assert!(
            traces_dir.exists(),
            "Traces directory should be created at: {traces_dir:?}"
        );

        // Verify trace file was created with correct naming: {timestamp}-run.log
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
            filename.ends_with("-run.log"),
            "Filename should end with '-run.log', got: {filename}"
        );

        // Verify file contains expected sections
        let trace_data = std::fs::read_to_string(trace_file).expect("Failed to read trace file");
        assert!(trace_data.contains("RUN FAILURE TRACE"));
        assert!(trace_data.contains("ERROR CHAIN"));
        assert!(trace_data.contains("END OF TRACE"));
    }

    #[test]
    fn it_should_format_run_step_in_trace() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let error = create_test_error("Docker compose up failed");
        let now = Utc::now();
        let context = RunFailureContext {
            failed_step: RunStep::StartServices,
            error_kind: ErrorKind::InfrastructureOperation,
            base: BaseFailureContext {
                error_summary: "Container failed to start".to_string(),
                failed_at: now,
                execution_started_at: now,
                execution_duration: Duration::from_secs(30),
                trace_id: TraceId::new(),
                trace_file_path: None,
            },
        };

        // Act
        let trace_file = writer.write_trace(&context, &error).unwrap();
        let trace_data = std::fs::read_to_string(trace_file).unwrap();

        // Assert - Verify step name is formatted with Display trait
        assert!(trace_data.contains("Failed Step: Start Services"));
        assert!(trace_data.contains("Error Kind: InfrastructureOperation"));
    }
}
