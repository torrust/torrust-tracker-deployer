//! Output format for command results
//!
//! This module defines the output format enum that controls how command results
//! are presented to users. It provides options for both human-readable text output
//! and machine-readable JSON output for automation.

/// Output format for command results
///
/// Controls the format of user-facing output that goes to stdout.
/// This is independent of logging format (which goes to stderr/file).
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::input::cli::OutputFormat;
///
/// // Default is text format
/// let format = OutputFormat::default();
/// assert!(matches!(format, OutputFormat::Text));
///
/// // JSON format for automation
/// let json_format = OutputFormat::Json;
/// ```
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output (default)
    ///
    /// Produces formatted text with tables, sections, and visual elements
    /// optimized for terminal display and human consumption.
    ///
    /// Example output:
    /// ```text
    /// ✅ Environment 'my-env' created successfully
    ///
    /// Environment Details:
    /// 1. Environment name: my-env
    /// 2. Instance name: torrust-tracker-vm-my-env
    /// 3. Data directory: ./data/my-env
    /// 4. Build directory: ./build/my-env
    /// ```
    #[default]
    Text,

    /// JSON output for automation and programmatic parsing
    ///
    /// Produces machine-readable JSON objects that can be parsed by tools
    /// like jq, scripts, and AI agents for programmatic extraction of data.
    ///
    /// Example output:
    /// ```json
    /// {
    ///   "environment_name": "my-env",
    ///   "state": "Created",
    ///   "data_dir": "data/my-env",
    ///   "build_dir": "build/my-env",
    ///   "created_at": "2026-02-16T14:30:00Z"
    /// }
    /// ```
    Json,
}
