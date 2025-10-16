//! Simplified Structured Logging Configuration
//!
//! Provides basic logging configuration with tracing spans for the three-level architecture:
//! - Commands (Level 1): Top-level orchestration
//! - Steps (Level 2): Mid-level execution units  
//! - Remote Actions (Level 3): Leaf-level operations
//!
//! ## Persistent Logging
//!
//! All logs are always written to a log file for persistent storage.
//! This enables post-mortem analysis and troubleshooting of production deployments.
//!
//! By default, logs are written to `./data/logs/log.txt` in production environments.
//! For testing, a different log directory can be specified to avoid polluting production data.
//!
//! ## Optional Stderr Output
//!
//! Logs can optionally be written to stderr for real-time visibility during development
//! and testing. This is controlled by the `LogOutput` parameter:
//!
//! - `LogOutput::FileOnly` - Production mode: logs to file only
//! - `LogOutput::FileAndStderr` - Development/testing: logs to both file and stderr
//!
//! ## Usage
//!
//! ### Builder Pattern (Recommended)
//!
//! ```rust,no_run
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::logging::{LogOutput, LogFormat, LoggingBuilder};
//!
//! // Flexible builder API
//! LoggingBuilder::new(Path::new("./data/logs"))
//!     .with_format(LogFormat::Compact)
//!     .with_output(LogOutput::FileAndStderr)
//!     .init();
//! ```
//!
//! ### Convenience Functions
//!
//! ```rust,no_run
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::logging::{LogOutput, init_compact};
//!
//! // E2E tests - enable stderr visibility with production log location
//! init_compact(Path::new("./data/logs"), LogOutput::FileAndStderr);
//!
//! // Production - file only
//! init_compact(Path::new("./data/logs"), LogOutput::FileOnly);
//!
//! // Integration tests - isolated temp directory
//! init_compact(Path::new("/tmp/test-xyz/data/logs"), LogOutput::FileAndStderr);
//! ```

use std::io;
use std::path::Path;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log file name used by the logging system
pub const LOG_FILE_NAME: &str = "log.txt";

/// Output target for logging
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
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

// ============================================================================
// BUILDER PATTERN - Core Implementation
// ============================================================================

/// Builder for constructing a tracing subscriber with flexible configuration
///
/// This builder provides a fluent API for configuring logging with different
/// formats and output targets. It eliminates code duplication by centralizing
/// layer creation and subscriber initialization.
///
/// Supports independent format control for file and stderr outputs, with
/// automatic ANSI code handling (disabled for files, enabled for stderr).
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
/// use torrust_tracker_deployer_lib::logging::{LogOutput, LogFormat, LoggingBuilder};
///
/// // Basic usage with defaults (Compact file format, Pretty stderr format, FileAndStderr output)
/// LoggingBuilder::new(Path::new("./data/logs")).init();
///
/// // Custom configuration with independent formats
/// LoggingBuilder::new(Path::new("./data/logs"))
///     .with_file_format(LogFormat::Json)
///     .with_stderr_format(LogFormat::Pretty)
///     .with_output(LogOutput::FileAndStderr)
///     .init();
///
/// // Backward compatible with single format for both outputs
/// LoggingBuilder::new(Path::new("./data/logs"))
///     .with_format(LogFormat::Compact)
///     .with_output(LogOutput::FileOnly)
///     .init();
/// ```
pub struct LoggingBuilder {
    log_dir: std::path::PathBuf,
    file_format: LogFormat,
    stderr_format: LogFormat,
    output: LogOutput,
}

impl LoggingBuilder {
    /// Create a new logging builder with default settings
    ///
    /// Default configuration:
    /// - File Format: `LogFormat::Compact` (no ANSI codes)
    /// - Stderr Format: `LogFormat::Pretty` (with ANSI codes)
    /// - Output: `LogOutput::FileAndStderr`
    ///
    /// # Arguments
    ///
    /// * `log_dir` - Directory where log files should be written (e.g., `./data/logs`)
    #[must_use]
    pub fn new(log_dir: &Path) -> Self {
        Self {
            log_dir: log_dir.to_path_buf(),
            file_format: LogFormat::Compact,
            stderr_format: LogFormat::Pretty,
            output: LogOutput::FileAndStderr,
        }
    }

    /// Set the logging format for both file and stderr outputs
    ///
    /// This is a convenience method for backward compatibility.
    /// For independent format control, use `with_file_format()` and `with_stderr_format()`.
    ///
    /// # Arguments
    ///
    /// * `format` - The desired logging format (Pretty, Json, or Compact)
    #[must_use]
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.file_format = format.clone();
        self.stderr_format = format;
        self
    }

    /// Set the logging format for file output
    ///
    /// ANSI codes are automatically disabled for file output to ensure
    /// logs are easily parsed with standard text tools (grep, awk, sed).
    ///
    /// # Arguments
    ///
    /// * `format` - The desired logging format for files (Pretty, Json, or Compact)
    #[must_use]
    pub fn with_file_format(mut self, format: LogFormat) -> Self {
        self.file_format = format;
        self
    }

    /// Set the logging format for stderr output
    ///
    /// ANSI codes are automatically enabled for stderr output to provide
    /// colored terminal output for better readability.
    ///
    /// # Arguments
    ///
    /// * `format` - The desired logging format for stderr (Pretty, Json, or Compact)
    #[must_use]
    pub fn with_stderr_format(mut self, format: LogFormat) -> Self {
        self.stderr_format = format;
        self
    }

    /// Set the output target
    ///
    /// # Arguments
    ///
    /// * `output` - Where to write logs (`FileOnly` or `FileAndStderr`)
    #[must_use]
    pub fn with_output(mut self, output: LogOutput) -> Self {
        self.output = output;
        self
    }

    /// Initialize the global tracing subscriber with the configured settings
    ///
    /// This consumes the builder and sets up the global logging infrastructure.
    /// After calling this, all logging macros (`tracing::info!`, etc.) will use
    /// this configuration.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Log directory cannot be created (filesystem permissions issue)
    /// - Subscriber initialization fails (usually means it was already initialized)
    ///
    /// Both panics are intentional as logging is critical for observability.
    pub fn init(self) {
        init_subscriber(
            &self.log_dir,
            self.output,
            &self.file_format,
            &self.stderr_format,
        );
    }
}

// ============================================================================
// INTERNAL INITIALIZATION - Single Source of Truth
// ============================================================================

/// Internal initialization function that handles all subscriber setup
///
/// This is the single source of truth for subscriber initialization.
/// All public init functions delegate to this to eliminate duplication.
///
/// Automatically configures ANSI codes:
/// - File output: ANSI codes disabled (clean text for parsing)
/// - Stderr output: ANSI codes enabled (colored terminal output)
///
/// Note: We cannot extract the format-specific layer creation into a separate
/// function because each format (Pretty, Json, Compact) creates a different
/// concrete type, and Rust's type system requires all match arms to return
/// the same type. Type erasure with boxed layers would work but adds runtime
/// overhead for a one-time initialization cost.
#[allow(clippy::too_many_lines)]
fn init_subscriber(
    log_dir: &Path,
    output: LogOutput,
    file_format: &LogFormat,
    stderr_format: &LogFormat,
) {
    let file_appender = create_log_file_appender(log_dir);
    let env_filter = create_env_filter();

    match output {
        LogOutput::FileOnly => {
            // File-only mode: single layer with ANSI disabled
            match file_format {
                LogFormat::Pretty => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .pretty()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(env_filter)
                        .init();
                }
                LogFormat::Json => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .json()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(env_filter)
                        .init();
                }
                LogFormat::Compact => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .compact()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(env_filter)
                        .init();
                }
            }
        }
        LogOutput::FileAndStderr => {
            // Dual output mode: file layer (no ANSI) + stderr layer (with ANSI)
            match (file_format, stderr_format) {
                // Pretty file format combinations
                (LogFormat::Pretty, LogFormat::Pretty) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .pretty()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(
                            fmt::layer()
                                .pretty()
                                .with_ansi(true)
                                .with_writer(io::stderr),
                        )
                        .with(env_filter)
                        .init();
                }
                (LogFormat::Pretty, LogFormat::Json) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .pretty()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(fmt::layer().json().with_ansi(true).with_writer(io::stderr))
                        .with(env_filter)
                        .init();
                }
                (LogFormat::Pretty, LogFormat::Compact) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .pretty()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(
                            fmt::layer()
                                .compact()
                                .with_ansi(true)
                                .with_writer(io::stderr),
                        )
                        .with(env_filter)
                        .init();
                }
                // JSON file format combinations
                (LogFormat::Json, LogFormat::Pretty) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .json()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(
                            fmt::layer()
                                .pretty()
                                .with_ansi(true)
                                .with_writer(io::stderr),
                        )
                        .with(env_filter)
                        .init();
                }
                (LogFormat::Json, LogFormat::Json) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .json()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(fmt::layer().json().with_ansi(true).with_writer(io::stderr))
                        .with(env_filter)
                        .init();
                }
                (LogFormat::Json, LogFormat::Compact) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .json()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(
                            fmt::layer()
                                .compact()
                                .with_ansi(true)
                                .with_writer(io::stderr),
                        )
                        .with(env_filter)
                        .init();
                }
                // Compact file format combinations
                (LogFormat::Compact, LogFormat::Pretty) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .compact()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(
                            fmt::layer()
                                .pretty()
                                .with_ansi(true)
                                .with_writer(io::stderr),
                        )
                        .with(env_filter)
                        .init();
                }
                (LogFormat::Compact, LogFormat::Json) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .compact()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(fmt::layer().json().with_ansi(true).with_writer(io::stderr))
                        .with(env_filter)
                        .init();
                }
                (LogFormat::Compact, LogFormat::Compact) => {
                    tracing_subscriber::registry()
                        .with(
                            fmt::layer()
                                .compact()
                                .with_ansi(false)
                                .with_writer(file_appender),
                        )
                        .with(
                            fmt::layer()
                                .compact()
                                .with_ansi(true)
                                .with_writer(io::stderr),
                        )
                        .with(env_filter)
                        .init();
                }
            }
        }
    }
}

/// Create the environment filter from `RUST_LOG` or default to "info"
///
/// This reads the `RUST_LOG` environment variable to determine the log level.
/// If not set, defaults to "info" level logging.
fn create_env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
}

/// Create the log file appender that writes to `{log_dir}/log.txt`
///
/// This function creates the log directory if it doesn't exist and returns
/// a non-blocking file appender that will append to the log file.
///
/// # Arguments
///
/// * `log_dir` - Directory where log files should be written (e.g., `./data/logs` for production)
///
/// # Panics
///
/// Panics if the log directory cannot be created. This is intentional as
/// logging is critical for observability.
fn create_log_file_appender(log_dir: &Path) -> tracing_appender::non_blocking::NonBlocking {
    // Create directory if it doesn't exist
    std::fs::create_dir_all(log_dir).unwrap_or_else(|_| {
        panic!(
            "Failed to create log directory: {} - check filesystem permissions",
            log_dir.display()
        )
    });

    // Create file appender (appends to existing file)
    let file_appender = tracing_appender::rolling::never(log_dir, LOG_FILE_NAME);

    // Use non-blocking writer for better performance
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Note: We intentionally leak the guard to keep the file open for the application lifetime
    std::mem::forget(guard);

    non_blocking
}

// ============================================================================
// CONVENIENCE FUNCTIONS - Thin Wrappers for Backward Compatibility
// ============================================================================

/// Initialize the tracing subscriber with default pretty formatting
///
/// This is a convenience wrapper around `LoggingBuilder` for backward compatibility.
/// Consider using `LoggingBuilder` directly for more flexibility.
///
/// Sets up structured logging with:
/// - File output to `{log_dir}/log.txt` (always enabled)
/// - Optional stderr output based on `output` parameter
/// - Pretty-printed format for development
/// - Environment-based filtering via `RUST_LOG`
/// - Support for hierarchical spans across three levels
///
/// # Arguments
///
/// * `log_dir` - Directory where log files should be written (e.g., `./data/logs` for production)
/// * `output` - Where to write logs (file only or file + stderr)
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Example
/// ```rust,no_run
/// use std::path::Path;
/// use torrust_tracker_deployer_lib::logging::{LogOutput, init};
///
/// // E2E tests - enable stderr visibility with production location
/// init(Path::new("./data/logs"), LogOutput::FileAndStderr);
///
/// // Production - file only
/// init(Path::new("./data/logs"), LogOutput::FileOnly);
///
/// // Testing - isolated temp directory
/// init(Path::new("/tmp/test-xyz/data/logs"), LogOutput::FileAndStderr);
/// ```
pub fn init(log_dir: &Path, output: LogOutput) {
    LoggingBuilder::new(log_dir)
        .with_format(LogFormat::Pretty)
        .with_output(output)
        .init();
}

/// Initialize the tracing subscriber with JSON formatting
///
/// This is a convenience wrapper around `LoggingBuilder` for backward compatibility.
/// Consider using `LoggingBuilder` directly for more flexibility.
///
/// Sets up structured logging with:
/// - File output to `{log_dir}/log.txt` (always enabled)
/// - Optional stderr output based on `output` parameter
/// - JSON output format for production environments
/// - Environment-based filtering via `RUST_LOG`
/// - Machine-readable log format for monitoring systems
///
/// # Arguments
///
/// * `log_dir` - Directory where log files should be written (e.g., `./data/logs` for production)
/// * `output` - Where to write logs (file only or file + stderr)
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Example
/// ```rust,no_run
/// use std::path::Path;
/// use torrust_tracker_deployer_lib::logging::{LogOutput, init_json};
///
/// // E2E tests - enable stderr visibility with production location
/// init_json(Path::new("./data/logs"), LogOutput::FileAndStderr);
///
/// // Production - file only
/// init_json(Path::new("./data/logs"), LogOutput::FileOnly);
///
/// // Testing - isolated temp directory
/// init_json(Path::new("/tmp/test-xyz/data/logs"), LogOutput::FileAndStderr);
/// ```
pub fn init_json(log_dir: &Path, output: LogOutput) {
    LoggingBuilder::new(log_dir)
        .with_format(LogFormat::Json)
        .with_output(output)
        .init();
}

/// Initialize the tracing subscriber with compact formatting
///
/// This is a convenience wrapper around `LoggingBuilder` for backward compatibility.
/// Consider using `LoggingBuilder` directly for more flexibility.
///
/// Sets up structured logging with:
/// - File output to `{log_dir}/log.txt` (always enabled)
/// - Optional stderr output based on `output` parameter
/// - Compact console output for minimal verbosity
/// - Environment-based filtering via `RUST_LOG`
/// - Space-efficient format for development
///
/// # Arguments
///
/// * `log_dir` - Directory where log files should be written (e.g., `./data/logs` for production)
/// * `output` - Where to write logs (file only or file + stderr)
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Example
/// ```rust,no_run
/// use std::path::Path;
/// use torrust_tracker_deployer_lib::logging::{LogOutput, init_compact};
///
/// // E2E tests - enable stderr visibility with production location
/// init_compact(Path::new("./data/logs"), LogOutput::FileAndStderr);
///
/// // Production - file only
/// init_compact(Path::new("./data/logs"), LogOutput::FileOnly);
///
/// // Testing - isolated temp directory
/// init_compact(Path::new("/tmp/test-xyz/data/logs"), LogOutput::FileAndStderr);
/// ```
pub fn init_compact(log_dir: &Path, output: LogOutput) {
    LoggingBuilder::new(log_dir)
        .with_format(LogFormat::Compact)
        .with_output(output)
        .init();
}

/// Initialize logging based on the chosen format and output target
///
/// This is a convenience wrapper around `LoggingBuilder` for backward compatibility.
/// Consider using `LoggingBuilder` directly for more flexibility.
///
/// This function applies the same format to both file and stderr outputs.
/// For independent format control, use `LoggingBuilder` with `with_file_format()`
/// and `with_stderr_format()`.
///
/// # Arguments
///
/// * `log_dir` - Directory where log files should be written (e.g., `./data/logs` for production)
/// * `output` - Where to write logs (file only or file + stderr)
/// * `format` - The logging format to use for both outputs
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Example
/// ```rust,no_run
/// use std::path::Path;
/// use torrust_tracker_deployer_lib::logging::{LogFormat, LogOutput, init_with_format};
///
/// // Initialize with JSON format for E2E tests with production location
/// init_with_format(Path::new("./data/logs"), LogOutput::FileAndStderr, &LogFormat::Json);
///
/// // Initialize with compact format for production
/// init_with_format(Path::new("./data/logs"), LogOutput::FileOnly, &LogFormat::Compact);
///
/// // Initialize for testing with isolated directory
/// init_with_format(Path::new("/tmp/test-xyz/data/logs"), LogOutput::FileAndStderr, &LogFormat::Pretty);
/// ```
pub fn init_with_format(log_dir: &Path, output: LogOutput, format: &LogFormat) {
    LoggingBuilder::new(log_dir)
        .with_format(format.clone())
        .with_output(output)
        .init();
}
