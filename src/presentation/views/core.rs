//! Core `UserOutput` struct and implementation

use std::io::Write;

use super::messages::{
    BlankLineMessage, ErrorMessage, InfoBlockMessage, ProgressMessage, ResultMessage, StepsMessage,
    SuccessMessage, WarningMessage,
};
use super::sinks::StandardSink;
use super::verbosity::VerbosityFilter;
#[allow(unused_imports)] // Channel is used in tests
use super::{Channel, FormatterOverride, OutputMessage, OutputSink, Theme, VerbosityLevel};

pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    sink: Box<dyn OutputSink>,
    formatter_override: Option<Box<dyn FormatterOverride>>,
}

impl UserOutput {
    /// Create new `UserOutput` with default stdout/stderr channels and emoji theme
    ///
    /// Uses `StandardSink` for backward compatibility with existing console output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let output = UserOutput::new(VerbosityLevel::Normal);
    /// ```
    #[must_use]
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self::with_theme(verbosity, Theme::default())
    }

    /// Create `UserOutput` with a specific theme
    ///
    /// Allows customization of output symbols while using default stdout/stderr channels.
    /// Uses `StandardSink` internally for backward compatibility.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel, Theme};
    ///
    /// // Use plain text theme for CI/CD
    /// let output = UserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());
    ///
    /// // Use ASCII theme for limited terminals
    /// let output = UserOutput::with_theme(VerbosityLevel::Normal, Theme::ascii());
    /// ```
    #[must_use]
    pub fn with_theme(verbosity: VerbosityLevel, theme: Theme) -> Self {
        Self::with_sink(verbosity, Box::new(StandardSink::default_console()))
            .with_theme_applied(theme)
    }

    /// Create `UserOutput` with a custom sink
    ///
    /// This constructor enables the use of alternative output destinations,
    /// including composite sinks for multi-destination output.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::views::{
    ///     UserOutput, VerbosityLevel, CompositeSink, StandardSink, FileSink
    /// };
    ///
    /// // Console + File output
    /// let composite = CompositeSink::new(vec![
    ///     Box::new(StandardSink::default_console()),
    ///     Box::new(FileSink::new("output.log").unwrap()),
    /// ]);
    /// let output = UserOutput::with_sink(VerbosityLevel::Normal, Box::new(composite));
    /// ```
    #[must_use]
    pub fn with_sink(verbosity: VerbosityLevel, sink: Box<dyn OutputSink>) -> Self {
        Self {
            theme: Theme::default(),
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink,
            formatter_override: None,
        }
    }

    /// Internal helper to apply theme to an existing `UserOutput`
    fn with_theme_applied(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Create `UserOutput` with theme and custom writers (for testing)
    ///
    /// This constructor allows full customization including theme and writers,
    /// primarily used for testing where output needs to be captured.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel, Theme};
    /// use std::io::Cursor;
    ///
    /// let stdout_buf = Vec::new();
    /// let stderr_buf = Vec::new();
    ///
    /// let output = UserOutput::with_theme_and_writers(
    ///     VerbosityLevel::Normal,
    ///     Theme::plain(),
    ///     Box::new(Cursor::new(stdout_buf)),
    ///     Box::new(Cursor::new(stderr_buf)),
    /// );
    /// ```
    #[must_use]
    pub fn with_theme_and_writers(
        verbosity: VerbosityLevel,
        theme: Theme,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self {
            theme,
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink: Box::new(StandardSink::new(stdout_writer, stderr_writer)),
            formatter_override: None,
        }
    }

    /// Create `UserOutput` with an optional formatter override
    ///
    /// This allows applying custom formatting (e.g., JSON, colored output)
    /// on top of the theme-based formatting.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::views::{
    ///     UserOutput, VerbosityLevel, JsonFormatter
    /// };
    ///
    /// let mut output = UserOutput::with_formatter_override(
    ///     VerbosityLevel::Normal,
    ///     Box::new(JsonFormatter),
    /// );
    ///
    /// output.progress("Processing");
    /// // Output: {"type":"ProgressMessage","channel":"Stderr","content":"⏳ Processing","timestamp":"..."}
    /// ```
    #[must_use]
    pub fn with_formatter_override(
        verbosity: VerbosityLevel,
        formatter_override: Box<dyn FormatterOverride>,
    ) -> Self {
        Self {
            theme: Theme::default(),
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink: Box::new(StandardSink::default_console()),
            formatter_override: Some(formatter_override),
        }
    }

    /// Create `UserOutput` with theme and optional formatter override
    ///
    /// Combines theme selection with optional formatter override.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::views::{
    ///     UserOutput, VerbosityLevel, Theme, JsonFormatter
    /// };
    ///
    /// let mut output = UserOutput::with_theme_and_formatter(
    ///     VerbosityLevel::Normal,
    ///     Theme::plain(),
    ///     Some(Box::new(JsonFormatter)),
    /// );
    /// ```
    #[must_use]
    pub fn with_theme_and_formatter(
        verbosity: VerbosityLevel,
        theme: Theme,
        formatter_override: Option<Box<dyn FormatterOverride>>,
    ) -> Self {
        Self {
            theme,
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink: Box::new(StandardSink::default_console()),
            formatter_override,
        }
    }

    /// Create `UserOutput` for testing with custom writers (uses default emoji theme)
    ///
    /// This constructor allows injecting custom writers for testing,
    /// enabling output capture and assertion. Uses the default emoji theme.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    /// use std::io::Cursor;
    ///
    /// let stdout_buf = Vec::new();
    /// let stderr_buf = Vec::new();
    ///
    /// let output = UserOutput::with_writers(
    ///     VerbosityLevel::Normal,
    ///     Box::new(Cursor::new(stdout_buf)),
    ///     Box::new(Cursor::new(stderr_buf)),
    /// );
    /// ```
    #[must_use]
    pub fn with_writers(
        verbosity: VerbosityLevel,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self::with_theme_and_writers(verbosity, Theme::default(), stdout_writer, stderr_writer)
    }

    /// Write a message to the appropriate channel using trait dispatch
    ///
    /// This is the core method for extensible message handling. It uses the
    /// `OutputMessage` trait to determine formatting, verbosity requirements,
    /// and channel routing. Messages are routed through the configured sink,
    /// enabling multi-destination output.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel, ProgressMessage};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.write(&ProgressMessage {
    ///     text: "Processing...".to_string(),
    /// });
    /// ```
    pub fn write(&mut self, message: &dyn OutputMessage) {
        if !self
            .verbosity_filter
            .should_show(message.required_verbosity())
        {
            return;
        }

        let mut formatted = message.format(&self.theme);

        // Apply optional format override
        if let Some(override_formatter) = &self.formatter_override {
            formatted = override_formatter.transform(&formatted, message);
        }

        // Write through sink
        self.sink.write_message(message, &formatted);
    }

    /// Flush all pending output to stdout and stderr
    ///
    /// **Note**: With the `OutputSink` abstraction, flush behavior depends on the
    /// sink implementation. `StandardSink` does not support explicit flushing.
    /// This method is kept for API compatibility but is currently a no-op.
    ///
    /// For `StandardSink` (default), writes are typically line-buffered by the OS.
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok(())` as flush is not supported through sinks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.progress("Starting long operation...");
    /// output.flush().expect("Failed to flush output");
    /// // Now perform long operation...
    /// ```
    pub fn flush(&mut self) -> std::io::Result<()> {
        // Note: Flush is not supported through the OutputSink abstraction.
        // This is a known limitation. StandardSink relies on OS line-buffering.
        Ok(())
    }

    /// Display progress message to stderr (Normal level and above)
    ///
    /// Progress messages go to stderr following cargo/docker patterns.
    /// This keeps stdout clean for result data that may be piped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.progress("Destroying environment...");
    /// // Output to stderr: ⏳ Destroying environment...
    /// ```
    pub fn progress(&mut self, message: &str) {
        self.write(&ProgressMessage {
            text: message.to_string(),
        });
    }

    /// Display success message to stderr (Normal level and above)
    ///
    /// Success status goes to stderr to allow clean result piping.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.success("Environment destroyed successfully");
    /// // Output to stderr: ✅ Environment destroyed successfully
    /// ```
    pub fn success(&mut self, message: &str) {
        self.write(&SuccessMessage {
            text: message.to_string(),
        });
    }

    /// Display warning message to stderr (Normal level and above)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.warn("Infrastructure may already be destroyed");
    /// // Output to stderr: ⚠️  Infrastructure may already be destroyed
    /// ```
    pub fn warn(&mut self, message: &str) {
        self.write(&WarningMessage {
            text: message.to_string(),
        });
    }

    /// Display error message to stderr (all levels)
    ///
    /// Errors are always shown regardless of verbosity level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Quiet);
    /// output.error("Failed to destroy environment");
    /// // Output to stderr: ❌ Failed to destroy environment
    /// ```
    pub fn error(&mut self, message: &str) {
        self.write(&ErrorMessage {
            text: message.to_string(),
        });
    }

    /// Output final results to stdout for piping/redirection
    ///
    /// This is where deployment results, configuration summaries, etc. go.
    /// Since this goes to stdout, it can be cleanly piped to other commands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.result("Deployment complete");
    /// // Output to stdout: Deployment complete
    /// ```
    pub fn result(&mut self, message: &str) {
        self.write(&ResultMessage {
            text: message.to_string(),
        });
    }

    /// Output structured data to stdout (JSON, etc.)
    ///
    /// For machine-readable output that should be piped or processed.
    /// This is equivalent to `result()` but exists for semantic clarity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.data(r#"{"status": "destroyed", "environment": "test"}"#);
    /// // Output to stdout: {"status": "destroyed", "environment": "test"}
    /// ```
    pub fn data(&mut self, data: &str) {
        self.result(data);
    }

    /// Display a blank line to stderr (Normal level and above)
    ///
    /// Used for spacing between sections of output to improve readability.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.success("Configuration template generated");
    /// output.blank_line();
    /// output.progress("Starting next steps...");
    /// ```
    pub fn blank_line(&mut self) {
        if self.verbosity_filter.should_show_blank_lines() {
            self.write(&BlankLineMessage);
        }
    }

    /// Display a numbered list of steps to stderr (Normal level and above)
    ///
    /// Useful for displaying sequential instructions or action items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.steps("Next steps:", &[
    ///     "Edit the configuration file",
    ///     "Review the settings",
    ///     "Run the deploy command",
    /// ]);
    /// // Output to stderr:
    /// // Next steps:
    /// // 1. Edit the configuration file
    /// // 2. Review the settings
    /// // 3. Run the deploy command
    /// ```
    pub fn steps(&mut self, title: &str, steps: &[&str]) {
        self.write(&StepsMessage {
            title: title.to_string(),
            items: steps.iter().map(|s| (*s).to_string()).collect(),
        });
    }

    /// Display a multi-line information block to stderr (Normal level and above)
    ///
    /// Useful for displaying grouped information or detailed messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.info_block("Configuration options:", &[
    ///     "  - username: 'torrust' (default)",
    ///     "  - port: 22 (default SSH port)",
    ///     "  - key_path: path/to/key",
    /// ]);
    /// // Output to stderr:
    /// // Configuration options:
    /// //   - username: 'torrust' (default)
    /// //   - port: 22 (default SSH port)
    /// //   - key_path: path/to/key
    /// ```
    pub fn info_block(&mut self, title: &str, lines: &[&str]) {
        self.write(&InfoBlockMessage {
            title: title.to_string(),
            lines: lines.iter().map(|s| (*s).to_string()).collect(),
        });
    }
}
