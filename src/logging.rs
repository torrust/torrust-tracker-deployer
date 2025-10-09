//! Simplified Structured Logging Configuration
//!
//! Provides basic logging configuration with tracing spans for the three-level architecture:
//! - Commands (Level 1): Top-level orchestration
//! - Steps (Level 2): Mid-level execution units  
//! - Remote Actions (Level 3): Leaf-level operations
//!
//! ## Persistent Logging
//!
//! All logs are always written to `./data/logs/log.txt` for persistent storage.
//! This enables post-mortem analysis and troubleshooting of production deployments.
//!
//! ## Optional Stderr Output
//!
//! Logs can optionally be written to stderr for real-time visibility during development
//! and testing. This is controlled by the `LogOutput` parameter:
//!
//! - `LogOutput::FileOnly` - Production mode: logs to file only
//! - `LogOutput::FileAndStderr` - Development/testing: logs to both file and stderr
//!
//! ## Examples
//!
//! ```rust,no_run
//! use torrust_tracker_deploy::logging::{LogOutput, init_compact};
//!
//! // E2E tests - enable stderr visibility
//! init_compact(LogOutput::FileAndStderr);
//!
//! // Production - file only
//! init_compact(LogOutput::FileOnly);
//! ```

use std::io;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Output target for logging
#[derive(Clone, Copy, Debug)]
pub enum LogOutput {
    /// Write logs to file only (production mode)
    FileOnly,
    /// Write logs to both file and stderr (development/testing mode)
    FileAndStderr,
}

/// Logging format options for different environments
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum LogFormat {
    /// Pretty-printed console output for development (default)
    Pretty,
    /// JSON output for production environments
    Json,
    /// Compact console output for minimal verbosity
    Compact,
}

/// Create the log file appender that writes to `./data/logs/log.txt`
///
/// This function creates the log directory if it doesn't exist and returns
/// a file appender that will append to the log file.
///
/// # Panics
///
/// Panics if the log directory cannot be created. This is intentional as
/// logging is critical for observability.
fn create_log_file_appender() -> tracing_appender::non_blocking::NonBlocking {
    // Create directory if it doesn't exist
    std::fs::create_dir_all("./data/logs")
        .expect("Failed to create log directory: ./data/logs - check filesystem permissions");

    // Create file appender (appends to existing file)
    let file_appender = tracing_appender::rolling::never("./data/logs", "log.txt");

    // Use non-blocking writer for better performance
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Note: We intentionally leak the guard to keep the file open for the application lifetime
    std::mem::forget(guard);

    non_blocking
}

/// Initialize the tracing subscriber with default pretty formatting
///
/// Sets up structured logging with:
/// - File output to `./data/logs/log.txt` (always enabled)
/// - Optional stderr output based on `output` parameter
/// - Pretty-printed format for development
/// - Environment-based filtering via `RUST_LOG`
/// - Support for hierarchical spans across three levels
///
/// # Arguments
///
/// * `output` - Where to write logs (file only or file + stderr)
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Example
/// ```rust,no_run
/// use torrust_tracker_deploy::logging::{LogOutput, init};
///
/// // E2E tests - enable stderr visibility
/// init(LogOutput::FileAndStderr);
///
/// // Production - file only
/// init(LogOutput::FileOnly);
/// ```
pub fn init(output: LogOutput) {
    let file_appender = create_log_file_appender();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match output {
        LogOutput::FileOnly => {
            // File output only
            tracing_subscriber::registry()
                .with(fmt::layer().pretty().with_writer(file_appender))
                .with(env_filter)
                .init();
        }
        LogOutput::FileAndStderr => {
            // File + stderr output
            let file_layer = fmt::layer().pretty().with_writer(file_appender);
            let stderr_layer = fmt::layer().pretty().with_writer(io::stderr);

            tracing_subscriber::registry()
                .with(file_layer)
                .with(stderr_layer)
                .with(env_filter)
                .init();
        }
    }
}

/// Initialize the tracing subscriber with JSON formatting
///
/// Sets up structured logging with:
/// - File output to `./data/logs/log.txt` (always enabled)
/// - Optional stderr output based on `output` parameter
/// - JSON output format for production environments
/// - Environment-based filtering via `RUST_LOG`
/// - Machine-readable log format for monitoring systems
///
/// # Arguments
///
/// * `output` - Where to write logs (file only or file + stderr)
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Example
/// ```rust,no_run
/// use torrust_tracker_deploy::logging::{LogOutput, init_json};
///
/// // E2E tests - enable stderr visibility
/// init_json(LogOutput::FileAndStderr);
///
/// // Production - file only
/// init_json(LogOutput::FileOnly);
/// ```
pub fn init_json(output: LogOutput) {
    let file_appender = create_log_file_appender();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match output {
        LogOutput::FileOnly => {
            // File output only
            tracing_subscriber::registry()
                .with(fmt::layer().json().with_writer(file_appender))
                .with(env_filter)
                .init();
        }
        LogOutput::FileAndStderr => {
            // File + stderr output
            let file_layer = fmt::layer().json().with_writer(file_appender);
            let stderr_layer = fmt::layer().json().with_writer(io::stderr);

            tracing_subscriber::registry()
                .with(file_layer)
                .with(stderr_layer)
                .with(env_filter)
                .init();
        }
    }
}

/// Initialize the tracing subscriber with compact formatting
///
/// Sets up structured logging with:
/// - File output to `./data/logs/log.txt` (always enabled)
/// - Optional stderr output based on `output` parameter
/// - Compact console output for minimal verbosity
/// - Environment-based filtering via `RUST_LOG`
/// - Space-efficient format for development
///
/// # Arguments
///
/// * `output` - Where to write logs (file only or file + stderr)
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Example
/// ```rust,no_run
/// use torrust_tracker_deploy::logging::{LogOutput, init_compact};
///
/// // E2E tests - enable stderr visibility
/// init_compact(LogOutput::FileAndStderr);
///
/// // Production - file only
/// init_compact(LogOutput::FileOnly);
/// ```
pub fn init_compact(output: LogOutput) {
    let file_appender = create_log_file_appender();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match output {
        LogOutput::FileOnly => {
            // File output only
            tracing_subscriber::registry()
                .with(fmt::layer().compact().with_writer(file_appender))
                .with(env_filter)
                .init();
        }
        LogOutput::FileAndStderr => {
            // File + stderr output
            let file_layer = fmt::layer().compact().with_writer(file_appender);
            let stderr_layer = fmt::layer().compact().with_writer(io::stderr);

            tracing_subscriber::registry()
                .with(file_layer)
                .with(stderr_layer)
                .with(env_filter)
                .init();
        }
    }
}

/// Initialize logging based on the chosen format and output target
///
/// This is a convenience function that calls the appropriate initialization
/// function based on the provided `LogFormat` and `LogOutput`.
///
/// # Arguments
///
/// * `output` - Where to write logs (file only or file + stderr)
/// * `format` - The logging format to use
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Example
/// ```rust,no_run
/// use torrust_tracker_deploy::logging::{LogFormat, LogOutput, init_with_format};
///
/// // Initialize with JSON format for E2E tests
/// init_with_format(LogOutput::FileAndStderr, &LogFormat::Json);
///
/// // Initialize with compact format for production
/// init_with_format(LogOutput::FileOnly, &LogFormat::Compact);
/// ```
pub fn init_with_format(output: LogOutput, format: &LogFormat) {
    match format {
        LogFormat::Pretty => init(output),
        LogFormat::Json => init_json(output),
        LogFormat::Compact => init_compact(output),
    }
}
