//! Common trace file writer infrastructure
//!
//! Provides shared file I/O operations for all command-specific trace writers:
//! - File creation and writing
//! - Directory management
//! - Timestamp-based filename generation

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::error::TraceWriterError;
use crate::shared::Clock;

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
    clock: Arc<dyn Clock>,
}

impl CommonTraceWriter {
    /// Create a new common trace writer
    ///
    /// # Arguments
    ///
    /// * `traces_dir` - Directory where trace files will be written
    /// * `clock` - Clock for timestamp generation
    pub(super) fn new(traces_dir: impl Into<PathBuf>, clock: Arc<dyn Clock>) -> Self {
        Self {
            traces_dir: traces_dir.into(),
            clock,
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

        let trace_file = self.generate_trace_filename(command_name);

        self.write_trace_file(&trace_file, content)?;

        Ok(trace_file)
    }

    /// Generate a timestamp-based trace filename
    ///
    /// Creates a filename in the format: `{timestamp}-{command_name}.log`
    /// where timestamp is `YYYYmmdd-HHMMSS`.
    ///
    /// # Arguments
    ///
    /// * `command_name` - Name of the command to include in the filename
    ///
    /// # Returns
    ///
    /// Full path to the trace file
    fn generate_trace_filename(&self, command_name: &str) -> PathBuf {
        let timestamp = self.clock.now().format("%Y%m%d-%H%M%S");
        self.traces_dir
            .join(format!("{timestamp}-{command_name}.log"))
    }

    /// Write content to a trace file
    ///
    /// Creates the file and writes all content to it.
    ///
    /// # Arguments
    ///
    /// * `trace_file` - Path where the trace file should be created
    /// * `content` - Content to write to the file
    ///
    /// # Errors
    ///
    /// Returns an error if file creation or writing fails
    fn write_trace_file(&self, trace_file: &Path, content: &str) -> Result<(), TraceWriterError> {
        let mut file = self.create_trace_file(trace_file)?;

        file.write_all(content.as_bytes())
            .map_err(|source| TraceWriterError::FileWrite {
                path: trace_file.display().to_string(),
                source,
            })?;

        Ok(())
    }

    /// Create a new trace file
    ///
    /// # Arguments
    ///
    /// * `trace_file` - Path where the file should be created
    ///
    /// # Returns
    ///
    /// File handle for writing
    ///
    /// # Errors
    ///
    /// Returns an error if file creation fails
    #[allow(clippy::unused_self)] // Kept as instance method for consistency with other trace writer methods
    fn create_trace_file(&self, trace_file: &Path) -> Result<fs::File, TraceWriterError> {
        fs::File::create(trace_file).map_err(|source| TraceWriterError::FileWrite {
            path: trace_file.display().to_string(),
            source,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Test helpers - Arrange phase utilities

    use crate::domain::environment::TRACES_DIR_NAME;
    use crate::testing::MockClock;
    use chrono::TimeZone;
    use std::sync::Arc;

    /// Create a test writer with a temporary directory
    ///
    /// Returns (writer, `temp_dir`, `traces_dir`)
    /// The `temp_dir` must be kept alive for the duration of the test
    fn create_test_writer() -> (CommonTraceWriter, TempDir, PathBuf) {
        use crate::domain::environment::TRACES_DIR_NAME;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let traces_dir = temp_dir.path().join(TRACES_DIR_NAME);
        let fixed_time = chrono::Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = Arc::new(MockClock::new(fixed_time));
        let writer = CommonTraceWriter::new(traces_dir.clone(), clock);
        (writer, temp_dir, traces_dir)
    }

    #[test]
    fn it_should_create_common_trace_writer_with_directory() {
        // Arrange
        let (writer, _temp_dir, traces_dir) = create_test_writer();

        // Assert
        assert_eq!(writer.traces_dir(), traces_dir);
    }

    #[test]
    fn it_should_create_traces_directory_on_first_write() {
        // Arrange
        let (writer, _temp_dir, traces_dir) = create_test_writer();

        // Directory should not exist yet
        assert!(!traces_dir.exists());

        // Act
        writer
            .write_trace("test-command", "test content")
            .expect("Failed to write trace");

        // Assert
        assert!(traces_dir.exists());
    }

    #[test]
    fn it_should_write_trace_with_timestamp_and_command_name() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();

        // Act
        let trace_file = writer
            .write_trace("test-command", "test content")
            .expect("Failed to write trace");

        // Assert
        assert!(trace_file.exists());

        let filename = trace_file.file_name().unwrap().to_str().unwrap();
        assert!(
            filename.ends_with("-test-command.log"),
            "Filename should end with '-test-command.log', got: {filename}"
        );
    }

    #[test]
    fn it_should_write_correct_content_to_trace_file() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let test_content = "This is test trace content\nwith multiple lines\nand details";

        // Act
        let trace_file = writer
            .write_trace("test-command", test_content)
            .expect("Failed to write trace");

        // Assert
        let written_content =
            std::fs::read_to_string(trace_file).expect("Failed to read trace file");
        assert_eq!(written_content, test_content);
    }

    #[test]
    fn it_should_generate_timestamp_in_correct_format() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();

        // Act
        let trace_file = writer
            .write_trace("test-command", "content")
            .expect("Failed to write trace");

        // Assert
        let filename = trace_file.file_name().unwrap().to_str().unwrap();

        // Verify filename format: {timestamp}-test-command.log
        // Example: 20251007-143045-test-command.log
        let parts: Vec<&str> = filename.split('-').collect();
        assert!(
            parts.len() >= 3,
            "Filename should have at least 3 parts (date, time, command.log), got: {filename}"
        );

        // Verify first part is date (8 digits: YYYYmmdd)
        assert_eq!(
            parts[0].len(),
            8,
            "Date part should be 8 digits, got: {}",
            parts[0]
        );
        assert!(
            parts[0].chars().all(|c| c.is_ascii_digit()),
            "Date part should be all digits, got: {}",
            parts[0]
        );

        // Verify second part is time (6 digits: HHMMSS)
        assert_eq!(
            parts[1].len(),
            6,
            "Time part should be 6 digits, got: {}",
            parts[1]
        );
        assert!(
            parts[1].chars().all(|c| c.is_ascii_digit()),
            "Time part should be all digits, got: {}",
            parts[1]
        );
    }

    #[test]
    fn it_should_write_multiple_traces_to_same_directory() {
        // Arrange
        let (writer, _temp_dir, traces_dir) = create_test_writer();

        // Act
        let trace1 = writer
            .write_trace("command1", "content 1")
            .expect("Failed to write first trace");
        let trace2 = writer
            .write_trace("command2", "content 2")
            .expect("Failed to write second trace");

        // Assert
        assert!(trace1.exists());
        assert!(trace2.exists());

        // Both files should be in the same directory
        assert_eq!(trace1.parent().unwrap(), traces_dir);
        assert_eq!(trace2.parent().unwrap(), traces_dir);

        // Files should have different names (different commands)
        assert_ne!(trace1, trace2);
    }

    #[test]
    fn it_should_handle_empty_content() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();

        // Act
        let trace_file = writer
            .write_trace("test-command", "")
            .expect("Failed to write empty trace");

        // Assert
        assert!(trace_file.exists());
        let content = std::fs::read_to_string(trace_file).expect("Failed to read trace file");
        assert_eq!(content, "");
    }

    #[test]
    fn it_should_handle_large_content() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let large_content = "x".repeat(10_000); // 10KB of content

        // Act
        let trace_file = writer
            .write_trace("test-command", &large_content)
            .expect("Failed to write large trace");

        // Assert
        assert!(trace_file.exists());
        let content = std::fs::read_to_string(trace_file).expect("Failed to read trace file");
        assert_eq!(content.len(), 10_000);
        assert_eq!(content, large_content);
    }

    #[test]
    fn it_should_handle_special_characters_in_content() {
        // Arrange
        let (writer, _temp_dir, _traces_dir) = create_test_writer();
        let special_content = "Special chars: \n\t\r 你好 🚀 ⚡ €£¥";

        // Act
        let trace_file = writer
            .write_trace("test-command", special_content)
            .expect("Failed to write trace with special chars");

        // Assert
        assert!(trace_file.exists());
        let content = std::fs::read_to_string(trace_file).expect("Failed to read trace file");
        assert_eq!(content, special_content);
    }

    #[test]
    fn it_should_return_error_for_invalid_directory_permissions() {
        // This test verifies error handling when directory creation fails
        // Note: This test is platform-dependent and may not work on all systems

        // Skip on Windows as permission handling is different
        #[cfg(not(target_os = "windows"))]
        {
            use std::fs;
            use std::os::unix::fs::PermissionsExt;

            // Arrange: Create a read-only parent directory
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let readonly_dir = temp_dir.path().join("readonly");
            fs::create_dir(&readonly_dir).expect("Failed to create readonly dir");

            // Make directory read-only
            let mut perms = fs::metadata(&readonly_dir)
                .expect("Failed to get metadata")
                .permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&readonly_dir, perms).expect("Failed to set permissions");

            let traces_dir = readonly_dir.join(TRACES_DIR_NAME);
            let fixed_time = chrono::Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
            let clock = Arc::new(MockClock::new(fixed_time));
            let writer = CommonTraceWriter::new(traces_dir, clock);

            // Act
            let result = writer.write_trace("test-command", "content");

            // Assert
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                TraceWriterError::DirectoryCreation { .. }
            ));

            // Cleanup: Restore permissions so temp_dir can be deleted
            let mut perms = fs::metadata(&readonly_dir)
                .expect("Failed to get metadata")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&readonly_dir, perms).expect("Failed to restore permissions");
        }
    }
}
