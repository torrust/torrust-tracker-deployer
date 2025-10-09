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

use clap::Parser;
use torrust_tracker_deploy::logging::{LogFormat, LogOutput};
use tracing::{debug, error, info, trace, warn};

#[derive(Parser)]
#[command(name = "test_logging")]
#[command(about = "Test binary for logging configuration validation")]
struct Cli {
    /// Logging format to use
    #[arg(long, value_enum)]
    format: LogFormat,

    /// Logging output target
    #[arg(long, value_enum)]
    output: LogOutput,
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging with the specified configuration
    torrust_tracker_deploy::logging::init_with_format(cli.output, &cli.format);

    // Emit one log message at each level for testing
    trace!("This is a TRACE level message");
    debug!("This is a DEBUG level message");
    info!("This is an INFO level message");
    warn!("This is a WARN level message");
    error!("This is an ERROR level message");

    // Print a simple marker to stdout to indicate successful completion
    println!("LOGGING_TEST_COMPLETE");
}
