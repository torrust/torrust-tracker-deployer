//! Integration tests for logging configuration
//!
//! These tests verify that the logging system works correctly with different
//! configurations by running the `test_logging` binary and examining its output.
//!
//! ## Test Coverage
//!
//! - File-only output mode (no stderr)
//! - File-and-stderr output mode (both outputs)
//! - Pretty format logging
//! - JSON format logging
//! - Compact format logging
//! - Log file append mode
//! - All log levels (trace, debug, info, warn, error)

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use torrust_tracker_deployer_lib::logging::LOG_FILE_NAME;

/// Helper struct to manage test execution and cleanup
struct LoggingTest {
    /// Temporary directory that's automatically cleaned up when dropped.
    /// Must be kept to prevent premature cleanup.
    temp_dir: TempDir,
    test_dir: PathBuf,
    log_file_path: PathBuf,
}

impl LoggingTest {
    /// Create a new test environment with isolated data directory
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory for test");
        let test_dir = temp_dir.path().to_path_buf();

        let log_file_path = test_dir.join("data/logs").join(LOG_FILE_NAME);

        let instance = Self {
            temp_dir,
            test_dir,
            log_file_path,
        };

        tracing::info!(
            temp_dir = %instance.temp_dir.path().display(),
            "Created isolated test environment"
        );

        instance
    }

    /// Run the `test_logging` binary with specified options
    fn run_test_logging(&self, format: &str, output: &str) -> TestOutput {
        // Use the cargo binary path to find the compiled test_logging binary
        let binary_path = std::env::current_exe()
            .expect("Failed to get current test executable path")
            .parent()
            .expect("Failed to get parent directory")
            .parent()
            .expect("Failed to get deps parent directory")
            .join("test_logging");

        // Use isolated temp directory for logs (follows testing conventions)
        let log_dir = self.test_dir.join("data/logs");

        let output = Command::new(&binary_path)
            .args([
                "--format",
                format,
                "--output",
                output,
                "--log-dir",
                &log_dir.to_string_lossy(),
            ])
            .current_dir(&self.test_dir)
            .output()
            .expect("Failed to execute test_logging binary");

        TestOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        }
    }

    /// Read the log file contents, waiting for it to be written by the non-blocking logger
    ///
    /// The logging system uses `tracing_appender::non_blocking` which writes logs
    /// via a background thread. This method polls the file until it has content,
    /// avoiding unnecessary delays when logs are already written while handling
    /// the race condition when the test reads before the background thread writes.
    fn read_log_file(&self) -> String {
        use std::time::Duration;

        const MAX_RETRIES: u32 = 20;
        const RETRY_DELAY_MS: u64 = 50;
        const INITIAL_WAIT_MS: u64 = 10;

        // Small initial wait to allow the non-blocking writer's background thread to start
        std::thread::sleep(Duration::from_millis(INITIAL_WAIT_MS));

        for attempt in 0..MAX_RETRIES {
            if let Ok(content) = fs::read_to_string(&self.log_file_path) {
                if !content.is_empty() {
                    return content;
                }
            }

            // If file is empty or doesn't exist yet, wait and retry
            if attempt < MAX_RETRIES - 1 {
                std::thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            }
        }

        panic!(
            "Log file at {} never received content after {} retries ({}ms total wait)",
            self.log_file_path.display(),
            MAX_RETRIES,
            INITIAL_WAIT_MS + (u64::from(MAX_RETRIES) * RETRY_DELAY_MS)
        );
    }

    /// Check if log file exists
    fn log_file_exists(&self) -> bool {
        self.log_file_path.exists()
    }

    /// Get line count in log file
    fn log_file_line_count(&self) -> usize {
        if !self.log_file_exists() {
            return 0;
        }
        self.read_log_file().lines().count()
    }
}

struct TestOutput {
    stdout: String,
    stderr: String,
    success: bool,
}

#[test]
fn it_should_write_logs_to_file_only_in_file_only_mode() {
    let test = LoggingTest::new();

    let output = test.run_test_logging("pretty", "file-only");

    // Should complete successfully
    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    // Should print completion marker to stdout
    assert!(
        output.stdout.contains("LOGGING_TEST_COMPLETE"),
        "stdout should contain completion marker"
    );

    // Stderr should be empty (no logging output)
    // Note: May contain compilation messages, but no log messages
    assert!(
        !output.stderr.contains("INFO"),
        "stderr should not contain INFO log messages in file-only mode"
    );
    assert!(
        !output.stderr.contains("WARN"),
        "stderr should not contain WARN log messages in file-only mode"
    );
    assert!(
        !output.stderr.contains("ERROR"),
        "stderr should not contain ERROR log messages in file-only mode"
    );

    // Log file should exist and contain logs
    assert!(
        test.log_file_exists(),
        "log file should be created automatically"
    );

    let log_content = test.read_log_file();
    assert!(
        log_content.contains("INFO"),
        "log file should contain INFO level logs"
    );
    assert!(
        log_content.contains("WARN"),
        "log file should contain WARN level logs"
    );
    assert!(
        log_content.contains("ERROR"),
        "log file should contain ERROR level logs"
    );
}

#[test]
fn it_should_write_logs_to_both_file_and_stderr_in_file_and_stderr_mode() {
    let test = LoggingTest::new();

    let output = test.run_test_logging("pretty", "file-and-stderr");

    // Should complete successfully
    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    // Should print completion marker to stdout
    assert!(
        output.stdout.contains("LOGGING_TEST_COMPLETE"),
        "stdout should contain completion marker"
    );

    // Stderr should contain log messages
    assert!(
        output.stderr.contains("INFO"),
        "stderr should contain INFO log messages in file-and-stderr mode"
    );
    assert!(
        output.stderr.contains("WARN"),
        "stderr should contain WARN log messages in file-and-stderr mode"
    );
    assert!(
        output.stderr.contains("ERROR"),
        "stderr should contain ERROR log messages in file-and-stderr mode"
    );

    // Log file should also exist and contain logs
    assert!(
        test.log_file_exists(),
        "log file should be created automatically"
    );

    let log_content = test.read_log_file();
    assert!(
        log_content.contains("INFO"),
        "log file should contain INFO level logs"
    );
    assert!(
        log_content.contains("WARN"),
        "log file should contain WARN level logs"
    );
    assert!(
        log_content.contains("ERROR"),
        "log file should contain ERROR level logs"
    );
}

#[test]
fn it_should_format_logs_in_json_format() {
    let test = LoggingTest::new();

    let output = test.run_test_logging("json", "file-and-stderr");

    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    // JSON format should have specific structure
    let log_content = test.read_log_file();

    // JSON logs should contain timestamp and level fields
    assert!(
        log_content.contains(r#""timestamp":"#),
        "JSON logs should contain timestamp field"
    );
    assert!(
        log_content.contains(r#""level":"INFO"#) || log_content.contains(r#""level":"info"#),
        "JSON logs should contain INFO level"
    );
    assert!(
        log_content.contains(r#""level":"WARN"#) || log_content.contains(r#""level":"warn"#),
        "JSON logs should contain WARN level"
    );
    assert!(
        log_content.contains(r#""level":"ERROR"#) || log_content.contains(r#""level":"error"#),
        "JSON logs should contain ERROR level"
    );
}

#[test]
fn it_should_format_logs_in_compact_format() {
    let test = LoggingTest::new();

    let output = test.run_test_logging("compact", "file-and-stderr");

    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    let log_content = test.read_log_file();

    // Compact format should still contain log levels and messages
    assert!(
        log_content.contains("INFO"),
        "compact logs should contain INFO level"
    );
    assert!(
        log_content.contains("WARN"),
        "compact logs should contain WARN level"
    );
    assert!(
        log_content.contains("ERROR"),
        "compact logs should contain ERROR level"
    );

    // Compact format should be more concise than pretty format
    // (This is a heuristic - compact format typically has fewer lines)
    let line_count = log_content.lines().count();
    assert!(
        line_count <= 20,
        "compact format should produce relatively few lines, got: {line_count}"
    );
}

#[test]
fn it_should_append_to_existing_log_file() {
    let test = LoggingTest::new();

    // Run first time
    let output1 = test.run_test_logging("compact", "file-only");
    assert!(output1.success, "first test_logging run should succeed");

    let line_count_after_first = test.log_file_line_count();
    assert!(
        line_count_after_first > 0,
        "log file should have content after first run"
    );

    // Run second time
    let output2 = test.run_test_logging("compact", "file-only");
    assert!(output2.success, "second test_logging run should succeed");

    let line_count_after_second = test.log_file_line_count();

    // Line count should increase (append mode)
    assert!(
        line_count_after_second > line_count_after_first,
        "log file should grow after second run (append mode). Before: {line_count_after_first}, After: {line_count_after_second}"
    );

    // Verify that logs were appended (line count increased)
    // Note: We're being lenient here because other processes might also be writing
    // to the log file during test execution
    assert!(
        line_count_after_second >= line_count_after_first + 3,
        "log file should have at least 3 more lines after second run (INFO, WARN, ERROR). Before: {line_count_after_first}, After: {line_count_after_second}"
    );
}

#[test]
fn it_should_emit_all_log_levels_when_trace_enabled() {
    let test = LoggingTest::new();

    // Run with RUST_LOG=trace to enable all levels
    let binary_path = std::env::current_exe()
        .expect("Failed to get current test executable path")
        .parent()
        .expect("Failed to get parent directory")
        .parent()
        .expect("Failed to get deps parent directory")
        .join("test_logging");

    let output = Command::new(&binary_path)
        .args(["--format", "pretty", "--output", "file-and-stderr"])
        .env("RUST_LOG", "trace")
        .current_dir(&test.test_dir)
        .output()
        .expect("Failed to execute test_logging binary");

    let success = output.status.success();
    assert!(success, "test_logging binary should execute successfully");

    let log_content = test.read_log_file();

    // With trace enabled, all levels should appear
    assert!(
        log_content.contains("TRACE"),
        "log file should contain TRACE level logs when RUST_LOG=trace"
    );
    assert!(
        log_content.contains("DEBUG"),
        "log file should contain DEBUG level logs when RUST_LOG=trace"
    );
    assert!(
        log_content.contains("INFO"),
        "log file should contain INFO level logs"
    );
    assert!(
        log_content.contains("WARN"),
        "log file should contain WARN level logs"
    );
    assert!(
        log_content.contains("ERROR"),
        "log file should contain ERROR level logs"
    );
}

#[test]
fn it_should_create_log_directory_automatically() {
    let test = LoggingTest::new();

    // Verify data/logs directory doesn't exist initially
    let logs_dir = test.test_dir.join("data/logs");
    assert!(
        !logs_dir.exists(),
        "logs directory should not exist before running test"
    );

    // Run test_logging
    let output = test.run_test_logging("pretty", "file-only");
    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    // Verify directory was created
    assert!(
        logs_dir.exists(),
        "logs directory should be created automatically"
    );
    assert!(logs_dir.is_dir(), "data/logs should be a directory");

    // Verify log file was created inside
    assert!(
        test.log_file_exists(),
        "log file should be created inside the logs directory"
    );
}

#[test]
fn it_should_not_include_ansi_codes_in_file_output_compact_format() {
    let test = LoggingTest::new();

    let output = test.run_test_logging("compact", "file-only");
    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    // Read raw bytes from log file
    let log_bytes = fs::read(&test.log_file_path).expect("Failed to read log file");

    // Check for ANSI escape sequence marker (0x1B)
    assert!(
        !log_bytes.contains(&0x1B),
        "log file should not contain ANSI escape sequences (0x1B) in compact format"
    );

    // Verify log content is still present
    let log_content = String::from_utf8_lossy(&log_bytes);
    assert!(
        log_content.contains("INFO"),
        "log file should still contain INFO level logs"
    );
    assert!(
        log_content.contains("WARN"),
        "log file should still contain WARN level logs"
    );
    assert!(
        log_content.contains("ERROR"),
        "log file should still contain ERROR level logs"
    );
}

#[test]
fn it_should_not_include_ansi_codes_in_file_output_pretty_format() {
    let test = LoggingTest::new();

    let output = test.run_test_logging("pretty", "file-only");
    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    // Read raw bytes from log file
    let log_bytes = fs::read(&test.log_file_path).expect("Failed to read log file");

    // Check for ANSI escape sequence marker (0x1B)
    assert!(
        !log_bytes.contains(&0x1B),
        "log file should not contain ANSI escape sequences (0x1B) in pretty format"
    );

    // Verify log content is still present
    let log_content = String::from_utf8_lossy(&log_bytes);
    assert!(
        log_content.contains("INFO"),
        "log file should still contain INFO level logs"
    );
    assert!(
        log_content.contains("WARN"),
        "log file should still contain WARN level logs"
    );
    assert!(
        log_content.contains("ERROR"),
        "log file should still contain ERROR level logs"
    );
}

#[test]
fn it_should_not_include_ansi_codes_in_file_output_json_format() {
    let test = LoggingTest::new();

    let output = test.run_test_logging("json", "file-only");
    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    // Read raw bytes from log file
    let log_bytes = fs::read(&test.log_file_path).expect("Failed to read log file");

    // Check for ANSI escape sequence marker (0x1B)
    assert!(
        !log_bytes.contains(&0x1B),
        "log file should not contain ANSI escape sequences (0x1B) in JSON format"
    );

    // Verify JSON structure is valid
    let log_content = String::from_utf8_lossy(&log_bytes);
    assert!(
        log_content.contains(r#""level":"INFO"#) || log_content.contains(r#""level":"info"#),
        "JSON logs should contain INFO level"
    );
}

#[test]
fn it_should_not_include_ansi_codes_in_file_output_dual_mode() {
    let test = LoggingTest::new();

    // Run with file-and-stderr mode to test dual output
    let output = test.run_test_logging("compact", "file-and-stderr");
    assert!(
        output.success,
        "test_logging binary should execute successfully"
    );

    // Read raw bytes from log file
    let log_bytes = fs::read(&test.log_file_path).expect("Failed to read log file");

    // File output should NOT contain ANSI codes
    assert!(
        !log_bytes.contains(&0x1B),
        "log file should not contain ANSI escape sequences (0x1B) in file-and-stderr mode"
    );

    // Verify log content is still present in file
    let log_content = String::from_utf8_lossy(&log_bytes);
    assert!(
        log_content.contains("INFO"),
        "log file should still contain INFO level logs"
    );
}
