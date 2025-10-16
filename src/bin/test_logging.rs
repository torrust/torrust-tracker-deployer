//! Test Binary for Logging Configuration Validation
//!
//! This binary is designed exclusively for testing the logging configuration.
//! It accepts different logging options via CLI and emits log messages at various levels.
//!
//! ## Purpose
//!
//! - Test different log formats (pretty, json, compact)
//! - Test different output modes (file-only, file-and-stderr)
//! - Verify log levels work correctly
//! - Enable integration testing of logging behavior
//!
//! ## Usage
//!
//! ```bash
//! # Test pretty format with stderr output
//! cargo run --bin test_logging -- --format pretty --output file-and-stderr
//!
//! # Test JSON format with file-only output
//! cargo run --bin test_logging -- --format json --output file-only
//!
//! # Test compact format
//! cargo run --bin test_logging -- --format compact --output file-and-stderr
//! ```
//!
//! ## Integration Tests
//!
//! This binary is primarily used by integration tests in `tests/logging_integration.rs`
//! to verify logging behavior across different configurations.

use std::path::PathBuf;

use clap::Parser;
use torrust_tracker_deployer_lib::logging::{LogFormat, LogOutput, LoggingBuilder};
use tracing::{debug, error, info, trace, warn};

#[derive(Parser)]
#[command(name = "test_logging")]
#[command(about = "Test binary for logging configuration validation")]
struct Cli {
    /// Logging format to use (backward compatibility - applies to both file and stderr)
    #[arg(long, value_enum)]
    format: Option<LogFormat>,

    /// Format for file logging (overrides --format for file output)
    #[arg(long, value_enum)]
    file_format: Option<LogFormat>,

    /// Format for stderr logging (overrides --format for stderr output)
    #[arg(long, value_enum)]
    stderr_format: Option<LogFormat>,

    /// Logging output target
    #[arg(long, value_enum)]
    output: LogOutput,

    /// Log directory path (e.g., ./data/logs for production, /tmp/test-xyz/data/logs for testing)
    #[arg(long, default_value = "./data/logs")]
    log_dir: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging with the specified configuration using the builder pattern
    let mut builder = LoggingBuilder::new(&cli.log_dir).with_output(cli.output);

    // Handle format arguments (backward compatible)
    match (cli.format, cli.file_format, cli.stderr_format) {
        // If only --format is provided, use it for both file and stderr (backward compatibility)
        (Some(format), None, None) => {
            builder = builder.with_format(format);
        }
        // If file and/or stderr formats are provided, use them specifically
        (_, file_fmt, stderr_fmt) => {
            if let Some(fmt) = file_fmt {
                builder = builder.with_file_format(fmt);
            }
            if let Some(fmt) = stderr_fmt {
                builder = builder.with_stderr_format(fmt);
            }
            // If --format is also provided along with specific formats, --format is ignored
            // (specific formats take precedence)
        }
    }

    builder.init();

    // Emit one log message at each level for testing
    trace!("This is a TRACE level message");
    debug!("This is a DEBUG level message");
    info!("This is an INFO level message");
    warn!("This is a WARN level message");
    error!("This is an ERROR level message");

    // IMPORTANT: Brief wait to allow non-blocking writer to flush
    // This test binary uses tracing_appender::non_blocking which writes via a background thread.
    // Since this is a short-lived test binary that exits immediately after logging, we need to
    // give the background thread time to flush logs to disk. The test's polling mechanism handles
    // additional waiting if needed, but this ensures the binary doesn't exit prematurely.
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Print a simple marker to stdout to indicate successful completion
    println!("LOGGING_TEST_COMPLETE");
}
