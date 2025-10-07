//! Provision command trace writer
//!
//! Generates trace files for provision command failures with provision-specific
//! context and metadata.

use std::path::{Path, PathBuf};

use crate::domain::environment::state::ProvisionFailureContext;
use crate::shared::Traceable;

use super::common::{CommonTraceWriter, TraceSections, TraceWriterError};

/// Provision-specific trace writer
///
/// Generates trace files for provision command failures with provision-specific
/// context and metadata.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use torrust_tracker_deploy::infrastructure::trace::ProvisionTraceWriter;
///
/// let traces_dir = PathBuf::from("data/my-env/traces");
/// let writer = ProvisionTraceWriter::new(traces_dir);
/// ```
pub struct ProvisionTraceWriter {
    common: CommonTraceWriter,
}

impl ProvisionTraceWriter {
    /// Create a new provision trace writer
    #[must_use]
    pub fn new(traces_dir: impl Into<PathBuf>) -> Self {
        Self {
            common: CommonTraceWriter::new(traces_dir),
        }
    }

    /// Write a provision failure trace file
    ///
    /// # Arguments
    ///
    /// * `ctx` - The provision failure context with metadata
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
        ctx: &ProvisionFailureContext,
        error: &E,
    ) -> Result<PathBuf, TraceWriterError> {
        let trace_content = Self::format_trace(ctx, error);
        self.common.write_trace("provision", &trace_content)
    }

    /// Format a complete provision trace
    fn format_trace<E: Traceable>(ctx: &ProvisionFailureContext, error: &E) -> String {
        use std::fmt::Write;

        let mut trace = String::new();

        // Header
        trace.push_str(&TraceSections::header("PROVISION FAILURE TRACE"));

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
        ProvisionErrorKind, ProvisionFailureContext, ProvisionStep, TraceId,
    };

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
    fn it_should_create_provision_trace_writer_with_directory() {
        let temp_dir = TempDir::new().unwrap();
        let traces_dir = temp_dir.path().join("traces");

        let writer = ProvisionTraceWriter::new(traces_dir.clone());

        assert_eq!(writer.traces_dir(), traces_dir);
    }

    #[test]
    fn it_should_create_traces_directory_on_first_write() {
        let temp_dir = TempDir::new().unwrap();
        let traces_dir = temp_dir.path().join("traces");

        let writer = ProvisionTraceWriter::new(traces_dir.clone());

        // Directory should not exist yet
        assert!(!traces_dir.exists());

        let error = TestError {
            message: "test error".to_string(),
            source: None,
        };

        let now = Utc::now();
        let context = ProvisionFailureContext {
            failed_step: ProvisionStep::RenderOpenTofuTemplates,
            error_kind: ProvisionErrorKind::TemplateRendering,
            error_summary: error.to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(5),
            trace_id: TraceId::new(),
            trace_file_path: None,
        };

        writer.write_trace(&context, &error).unwrap();

        // Directory should now exist
        assert!(traces_dir.exists());
    }

    #[test]
    fn it_should_write_provision_trace_with_correct_naming() {
        let temp_dir = TempDir::new().unwrap();
        let traces_dir = temp_dir.path().join("traces");
        let writer = ProvisionTraceWriter::new(traces_dir.clone());

        let error = TestError {
            message: "provision test error".to_string(),
            source: None,
        };

        let now = Utc::now();
        let context = ProvisionFailureContext {
            failed_step: ProvisionStep::RenderOpenTofuTemplates,
            error_kind: ProvisionErrorKind::TemplateRendering,
            error_summary: error.to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(5),
            trace_id: TraceId::new(),
            trace_file_path: None,
        };

        let trace_file = writer.write_trace(&context, &error).unwrap();

        // Verify file exists
        assert!(trace_file.exists());

        // Verify filename ends with -provision.log
        let filename = trace_file.file_name().unwrap().to_str().unwrap();
        assert!(filename.ends_with("-provision.log"));
    }

    #[test]
    fn it_should_use_timestamp_and_command_as_filename() {
        let temp_dir = TempDir::new().unwrap();
        let traces_dir = temp_dir.path().join("traces");
        let writer = ProvisionTraceWriter::new(traces_dir.clone());

        let error = TestError {
            message: "test error".to_string(),
            source: None,
        };

        let now = Utc::now();
        let context = ProvisionFailureContext {
            failed_step: ProvisionStep::RenderOpenTofuTemplates,
            error_kind: ProvisionErrorKind::TemplateRendering,
            error_summary: error.to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(5),
            trace_id: TraceId::new(),
            trace_file_path: None,
        };

        let trace_file = writer.write_trace(&context, &error).unwrap();

        let filename = trace_file.file_name().unwrap().to_str().unwrap();

        // Verify filename format: {timestamp}-provision.log
        // Example: 20251003-103045-provision.log
        assert!(filename.ends_with("-provision.log"));

        // Verify timestamp prefix exists (YYYYmmdd-HHMMSS format)
        let parts: Vec<&str> = filename.split('-').collect();
        assert!(parts.len() >= 3); // At least YYYYmmdd, HHMMSS, provision.log

        // Verify first part is date (8 digits)
        assert_eq!(parts[0].len(), 8);
        assert!(parts[0].chars().all(|c| c.is_ascii_digit()));

        // Verify second part is time (6 digits)
        let time_part = parts[1];
        assert_eq!(time_part.len(), 6);
        assert!(time_part.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn it_should_include_trace_metadata_in_provision_trace() {
        let temp_dir = TempDir::new().unwrap();
        let traces_dir = temp_dir.path().join("traces");
        let writer = ProvisionTraceWriter::new(traces_dir);

        let error = TestError {
            message: "test error".to_string(),
            source: None,
        };

        let now = Utc::now();
        let trace_id = TraceId::new();
        let context = ProvisionFailureContext {
            failed_step: ProvisionStep::RenderOpenTofuTemplates,
            error_kind: ProvisionErrorKind::TemplateRendering,
            error_summary: "Test error summary".to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(5),
            trace_id: trace_id.clone(),
            trace_file_path: None,
        };

        let trace_file = writer.write_trace(&context, &error).unwrap();
        let trace_data = std::fs::read_to_string(trace_file).unwrap();

        // Verify metadata is included
        assert!(trace_data.contains("PROVISION FAILURE TRACE"));
        assert!(trace_data.contains(&format!("Trace ID: {trace_id}")));
        assert!(trace_data.contains("Failed Step: RenderOpenTofuTemplates"));
        assert!(trace_data.contains("Error Kind: TemplateRendering"));
        assert!(trace_data.contains("Error Summary: Test error summary"));
    }

    #[test]
    fn it_should_generate_trace_files_with_correct_naming() {
        // This test was moved from tests/trace_file_generation.rs
        // It verifies that trace files are created with correct naming convention
        // and follow the format: {timestamp}-{command}.log

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let traces_dir = temp_dir.path().join("traces");

        let writer = ProvisionTraceWriter::new(traces_dir.clone());

        let error = TestError {
            message: "Simulated provision failure".to_string(),
            source: None,
        };

        let now = Utc::now();
        let context = ProvisionFailureContext {
            failed_step: ProvisionStep::RenderOpenTofuTemplates,
            error_kind: ProvisionErrorKind::TemplateRendering,
            error_summary: error.to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(5),
            trace_id: TraceId::new(),
            trace_file_path: None,
        };

        let _trace_path = writer
            .write_trace(&context, &error)
            .expect("Failed to write trace file");

        // Verify trace directory was created
        assert!(
            traces_dir.exists(),
            "Traces directory should be created at: {traces_dir:?}"
        );

        // Verify trace file was created with correct naming: {timestamp}-provision.log
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
            filename.ends_with("-provision.log"),
            "Filename should end with '-provision.log', got: {filename}"
        );

        // Verify file contains expected sections
        let trace_data = std::fs::read_to_string(trace_file).expect("Failed to read trace file");
        assert!(trace_data.contains("PROVISION FAILURE TRACE"));
        assert!(trace_data.contains("ERROR CHAIN"));
        assert!(trace_data.contains("END OF TRACE"));
    }
}
