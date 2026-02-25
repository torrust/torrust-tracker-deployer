//! CLI Argument Definitions
//!
//! This module contains the global CLI arguments that are shared across all commands,
//! primarily logging configuration options. These arguments follow clap conventions
//! and provide comprehensive documentation for users.

use std::path::PathBuf;

use crate::bootstrap::logging::{LogFormat, LogOutput, LoggingConfig};
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::VerbosityLevel;

/// Global CLI arguments for logging and output configuration
///
/// These arguments are available for all commands and control how logging
/// is handled throughout the application. They provide fine-grained control
/// over log output, formatting, and destinations.
#[derive(clap::Args, Debug, Clone)]
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

    /// Output format for command results (default: text)
    ///
    /// Controls the format of user-facing output (stdout channel).
    /// - text: Human-readable formatted output with tables and sections (default)
    /// - json: Machine-readable JSON for automation, scripts, and AI agents
    ///
    /// This is independent of logging format (--log-file-format, --log-stderr-format)
    /// which controls stderr/file output.
    ///
    /// Examples:
    /// - Default: Text format for human consumption
    /// - Automation: JSON format for programmatic parsing
    /// - CI/CD: JSON piped to jq for field extraction
    #[arg(long, value_enum, default_value = "text", global = true)]
    pub output_format: OutputFormat,

    /// Increase verbosity of user-facing output
    ///
    /// Controls the amount of detail shown during operations:
    /// - Default: Essential progress and results
    /// - -v: Detailed progress including intermediate steps
    /// - -vv: Very detailed including decisions and retries
    /// - -vvv: Maximum detail for troubleshooting
    ///
    /// Note: This controls user-facing messages only. For internal
    /// logging verbosity, use the `RUST_LOG` environment variable.
    ///
    /// Examples:
    ///   provision my-env        # Normal verbosity
    ///   provision my-env -v     # Verbose
    ///   provision my-env -vv    # Very verbose
    ///   provision my-env -vvv   # Debug
    #[arg(
        short = 'v',
        long = "verbose",
        action = clap::ArgAction::Count,
        global = true
    )]
    pub verbosity: u8,
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
    /// # use torrust_tracker_deployer_lib::presentation::cli::input::cli::args::GlobalArgs;
    /// # use torrust_tracker_deployer_lib::presentation::cli::input::cli::OutputFormat;
    /// # use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput, LoggingConfig};
    /// # use std::path::PathBuf;
    /// // Create args with log configuration
    /// let args = GlobalArgs {
    ///     log_file_format: LogFormat::Compact,
    ///     log_stderr_format: LogFormat::Pretty,
    ///     log_output: LogOutput::FileAndStderr,
    ///     log_dir: PathBuf::from("/tmp/logs"),
    ///     working_dir: PathBuf::from("."),
    ///     output_format: OutputFormat::Text,
    ///     verbosity: 0,
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

    /// Convert CLI verbosity count to `VerbosityLevel`
    ///
    /// Maps the number of `-v` flags provided by the user to the appropriate
    /// `VerbosityLevel` enum variant:
    /// - 0 flags (default) → Normal
    /// - 1 flag (-v) → Verbose
    /// - 2 flags (-vv) → `VeryVerbose`
    /// - 3+ flags (-vvv or more) → Debug
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::presentation::cli::input::cli::args::GlobalArgs;
    /// # use torrust_tracker_deployer_lib::presentation::cli::input::cli::OutputFormat;
    /// # use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput};
    /// # use torrust_tracker_deployer_lib::presentation::cli::views::VerbosityLevel;
    /// # use std::path::PathBuf;
    /// let args = GlobalArgs {
    ///     log_file_format: LogFormat::Compact,
    ///     log_stderr_format: LogFormat::Pretty,
    ///     log_output: LogOutput::FileOnly,
    ///     log_dir: PathBuf::from("./data/logs"),
    ///     working_dir: PathBuf::from("."),
    ///     output_format: OutputFormat::Text,
    ///     verbosity: 2,  // -vv
    /// };
    /// assert_eq!(args.verbosity_level(), VerbosityLevel::VeryVerbose);
    /// ```
    #[must_use]
    pub fn verbosity_level(&self) -> VerbosityLevel {
        match self.verbosity {
            0 => VerbosityLevel::Normal,      // Default
            1 => VerbosityLevel::Verbose,     // -v
            2 => VerbosityLevel::VeryVerbose, // -vv
            _ => VerbosityLevel::Debug,       // -vvv or more
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args(verbosity: u8) -> GlobalArgs {
        GlobalArgs {
            log_file_format: LogFormat::Compact,
            log_stderr_format: LogFormat::Pretty,
            log_output: LogOutput::FileOnly,
            log_dir: PathBuf::from("./data/logs"),
            working_dir: PathBuf::from("."),
            output_format: OutputFormat::Text,
            verbosity,
        }
    }

    #[test]
    fn it_should_return_normal_verbosity_when_no_flags_provided() {
        let args = create_test_args(0);
        assert_eq!(args.verbosity_level(), VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_return_verbose_level_when_single_v_flag_provided() {
        let args = create_test_args(1);
        assert_eq!(args.verbosity_level(), VerbosityLevel::Verbose);
    }

    #[test]
    fn it_should_return_very_verbose_level_when_double_v_flag_provided() {
        let args = create_test_args(2);
        assert_eq!(args.verbosity_level(), VerbosityLevel::VeryVerbose);
    }

    #[test]
    fn it_should_return_debug_level_when_triple_v_flag_provided() {
        let args = create_test_args(3);
        assert_eq!(args.verbosity_level(), VerbosityLevel::Debug);
    }

    #[test]
    fn it_should_cap_at_debug_level_when_more_than_three_v_flags_provided() {
        let args = create_test_args(4);
        assert_eq!(args.verbosity_level(), VerbosityLevel::Debug);

        let args = create_test_args(10);
        assert_eq!(args.verbosity_level(), VerbosityLevel::Debug);
    }
}
