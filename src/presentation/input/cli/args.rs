//! CLI Argument Definitions
//!
//! This module contains the global CLI arguments that are shared across all commands,
//! primarily logging configuration options. These arguments follow clap conventions
//! and provide comprehensive documentation for users.

use std::path::PathBuf;

use crate::bootstrap::logging::{LogFormat, LogOutput, LoggingConfig};

/// Global CLI arguments for logging configuration
///
/// These arguments are available for all commands and control how logging
/// is handled throughout the application. They provide fine-grained control
/// over log output, formatting, and destinations.
#[derive(clap::Args, Debug)]
pub struct GlobalArgs {
    /// Format for file logging (default: compact, without ANSI codes)
    ///
    /// - pretty: Pretty-printed output for development (no ANSI in files)
    /// - json: JSON output for production environments (no ANSI)
    /// - compact: Compact output for minimal verbosity (no ANSI in files)
    ///
    /// Note: ANSI color codes are automatically disabled for file output
    /// to ensure logs are easily parsed with standard text tools (grep, awk, sed).
    #[arg(long, value_enum, default_value = "compact", global = true)]
    pub log_file_format: LogFormat,

    /// Format for stderr logging (default: pretty, with ANSI codes)
    ///
    /// - pretty: Pretty-printed output with colors for development
    /// - json: JSON output for machine processing
    /// - compact: Compact output with colors for minimal verbosity
    ///
    /// Note: ANSI color codes are automatically enabled for stderr output
    /// to provide colored terminal output for better readability.
    #[arg(long, value_enum, default_value = "pretty", global = true)]
    pub log_stderr_format: LogFormat,

    /// Log output mode (default: file-only for production)
    ///
    /// - file-only: Write logs to file only (production mode)
    /// - file-and-stderr: Write logs to both file and stderr (development/testing mode)
    #[arg(long, value_enum, default_value = "file-only", global = true)]
    pub log_output: LogOutput,

    /// Log directory (default: ./data/logs)
    ///
    /// Directory where log files will be written. The log file will be
    /// named 'log.txt' inside this directory. Parent directories will be
    /// created automatically if they don't exist.
    ///
    /// Note: If the directory cannot be created due to filesystem permissions,
    /// the application will exit with an error. Logging is critical for
    /// observability and the application cannot function without it.
    #[arg(long, default_value = "./data/logs", global = true)]
    pub log_dir: PathBuf,

    /// Working directory for environment data (default: .)
    ///
    /// Root directory where environment data will be stored. Each environment
    /// creates subdirectories within this location for build files and state.
    /// This is useful for testing or when you want to manage environments in
    /// a different location than the current directory.
    ///
    /// Examples:
    /// - Default: './data' (relative to current directory)
    /// - Testing: '/tmp/test-workspace' (absolute path)
    /// - Production: '/var/lib/torrust-deployer' (system location)
    #[arg(long, default_value = ".", global = true)]
    pub working_dir: PathBuf,
}

impl GlobalArgs {
    /// Create a logging configuration from these global arguments
    ///
    /// This method extracts the logging-specific configuration from CLI arguments
    /// and creates a domain-appropriate `LoggingConfig` struct. This encapsulates
    /// the conversion logic and avoids spreading logging configuration details
    /// throughout the application bootstrap code.
    ///
    /// # Returns
    ///
    /// A `LoggingConfig` that can be used to initialize the logging system
    ///
    /// # Example
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::presentation::input::cli::args::GlobalArgs;
    /// # use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput, LoggingConfig};
    /// # use std::path::PathBuf;
    /// // Create args with log configuration
    /// let args = GlobalArgs {
    ///     log_file_format: LogFormat::Compact,
    ///     log_stderr_format: LogFormat::Pretty,
    ///     log_output: LogOutput::FileAndStderr,
    ///     log_dir: PathBuf::from("/tmp/logs"),
    ///     working_dir: PathBuf::from("."),
    /// };
    /// let config = args.logging_config();
    /// // config will have specified log formats and directory
    /// ```
    #[must_use]
    pub fn logging_config(&self) -> LoggingConfig {
        LoggingConfig::new(
            self.log_dir.clone(),
            self.log_file_format.clone(),
            self.log_stderr_format.clone(),
            self.log_output,
        )
    }
}
