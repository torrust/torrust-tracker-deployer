//! `UserOutput` struct and implementation
//!
//! This module provides the main `UserOutput` struct which handles user-facing output
//! formatting and routing. It implements a sink-based architecture with support for
//! multiple output destinations, themes, verbosity levels, and custom formatters.
//!
//! The `UserOutput` struct is the primary interface for displaying messages to users,
//! following Unix conventions with dual-channel output (stdout for results, stderr
//! for progress and status messages).

// Standard library imports
use std::io::Write;

// Internal crate imports
use super::messages::{
    BlankLineMessage, DebugDetailMessage, DetailMessage, ErrorMessage, InfoBlockMessage,
    ProgressMessage, ResultMessage, StepProgressMessage, StepsMessage, SuccessMessage,
    WarningMessage,
};
use super::sinks::StandardSink;
use super::verbosity::VerbosityFilter;
use super::{FormatterOverride, OutputMessage, OutputSink, Theme, VerbosityLevel};

/// User-facing output handler with sink-based architecture
///
/// `UserOutput` provides a clean interface for displaying messages to users with support for:
/// - Multiple output sinks (console, file, telemetry, etc.)
/// - Verbosity levels (quiet, normal, verbose, debug)  
/// - Customizable themes (emoji, plain text, ASCII)
/// - Optional formatter overrides (JSON, colored output)
/// - Dual-channel routing (stdout for results, stderr for progress)
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
///
/// let mut output = UserOutput::new(VerbosityLevel::Normal);
/// output.progress("Starting operation...");
/// output.success("Operation completed successfully");
/// output.result(r#"{"status": "completed"}"#);
/// ```
///
/// With custom theme:
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel, Theme};
///
/// let mut output = UserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());
/// output.progress("Processing...");
/// ```
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

    /// Display a step progress message to stderr (Verbose level and above)
    ///
    /// Step progress messages mark workflow step boundaries during command
    /// execution. They are shown when the user requests verbose output (`-v`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Verbose);
    /// output.step_progress("  [Step 1/9] Rendering OpenTofu templates...");
    /// // Output to stderr: 📋   [Step 1/9] Rendering OpenTofu templates...
    /// ```
    pub fn step_progress(&mut self, message: &str) {
        self.write(&StepProgressMessage {
            text: message.to_string(),
        });
    }

    /// Display a detail message to stderr (`VeryVerbose` level and above)
    ///
    /// Detail messages provide contextual information within steps during command
    /// execution. They are shown when the user requests very verbose output (`-vv`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::VeryVerbose);
    /// output.detail("     → Instance IP: 10.140.190.235");
    /// // Output to stderr: 📋      → Instance IP: 10.140.190.235
    /// ```
    pub fn detail(&mut self, message: &str) {
        self.write(&DetailMessage {
            text: message.to_string(),
        });
    }

    /// Display a debug detail message to stderr (Debug level and above)
    ///
    /// Debug detail messages provide technical implementation details during command
    /// execution. They are shown when the user requests maximum verbosity (`-vvv`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Debug);
    /// output.debug_detail("     → Command: tofu init");
    /// // Output to stderr: 🔍      → Command: tofu init
    /// ```
    pub fn debug_detail(&mut self, message: &str) {
        self.write(&DebugDetailMessage {
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
        self.write(&BlankLineMessage);
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
    fn with_sink(verbosity: VerbosityLevel, sink: Box<dyn OutputSink>) -> Self {
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
    fn write(&mut self, message: &dyn OutputMessage) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    mod verbosity {
        use super::*;
        use crate::presentation::views::testing::TestUserOutput;
        use crate::presentation::views::Channel;

        /// Test message that requires Verbose level
        struct TestVerboseMessage {
            text: String,
        }

        impl OutputMessage for TestVerboseMessage {
            fn format(&self, _theme: &Theme) -> String {
                format!("TEST: {}\n", self.text)
            }

            fn required_verbosity(&self) -> VerbosityLevel {
                VerbosityLevel::Verbose
            }

            fn channel(&self) -> Channel {
                Channel::Stderr
            }

            fn type_name(&self) -> &'static str {
                "TestVerboseMessage"
            }
        }

        #[test]
        fn it_should_ignore_message_when_verbosity_level_is_below_required() {
            // Create UserOutput with Normal verbosity
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Create a message that requires Verbose level (higher than Normal)
            let message = TestVerboseMessage {
                text: "This should not appear".to_string(),
            };

            // Try to write the message - should be ignored due to insufficient verbosity
            test_output.output.write(&message);

            // Both stdout and stderr should be empty since message was filtered out
            assert_eq!(test_output.stdout(), "");
            assert_eq!(test_output.stderr(), "");
        }
    }

    mod formatter {
        use super::*;
        use crate::presentation::views::formatters::JsonFormatter;
        use crate::presentation::views::testing::TestWriter;
        use crate::presentation::views::Channel;
        use parking_lot::Mutex;
        use std::sync::Arc;

        /// Test message with Normal verbosity for formatter testing
        struct TestNormalMessage {
            text: String,
        }

        impl OutputMessage for TestNormalMessage {
            fn format(&self, _theme: &Theme) -> String {
                format!("MSG: {}\n", self.text)
            }

            fn required_verbosity(&self) -> VerbosityLevel {
                VerbosityLevel::Normal
            }

            fn channel(&self) -> Channel {
                Channel::Stderr
            }

            fn type_name(&self) -> &'static str {
                "TestNormalMessage"
            }
        }

        #[test]
        fn it_should_apply_formatter_override_to_transform_message() {
            // Create buffers for capturing output
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));

            // Create UserOutput with JsonFormatter
            let mut output = UserOutput {
                theme: Theme::default(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(Box::new(JsonFormatter)),
            };

            // Create and write a test message
            let message = TestNormalMessage {
                text: "test message".to_string(),
            };
            output.write(&message);

            // Verify the formatter transformed the output to JSON format
            let stderr_output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();

            // Parse JSON to verify structure (timestamp is dynamic, so we check fields exist)
            let json: serde_json::Value = serde_json::from_str(&stderr_output).unwrap();
            assert_eq!(json["type"], "TestNormalMessage");
            assert_eq!(json["channel"], "Stderr");
            assert_eq!(json["content"], "MSG: test message");
            assert!(json["timestamp"].is_string());

            // Stdout should be empty (message goes to stderr)
            let stdout_output = String::from_utf8(stdout_buffer.lock().clone()).unwrap();
            assert_eq!(stdout_output, "");
        }
    }

    mod theme {
        use super::*;
        use crate::presentation::views::testing::TestUserOutput;
        use crate::presentation::views::Channel;
        use rstest::rstest;

        /// Test message that uses theme symbols in formatting
        struct TestThemedMessage {
            text: String,
        }

        impl OutputMessage for TestThemedMessage {
            fn format(&self, theme: &Theme) -> String {
                format!("{} {}\n", theme.success_symbol(), self.text)
            }

            fn required_verbosity(&self) -> VerbosityLevel {
                VerbosityLevel::Normal
            }

            fn channel(&self) -> Channel {
                Channel::Stderr
            }

            fn type_name(&self) -> &'static str {
                "TestThemedMessage"
            }
        }

        #[rstest]
        #[case(Theme::emoji(), "✅ Operation completed\n")]
        #[case(Theme::plain(), "[OK] Operation completed\n")]
        #[case(Theme::ascii(), "[+] Operation completed\n")]
        fn it_should_format_message_differently_with_different_themes(
            #[case] theme: Theme,
            #[case] expected_output: &str,
        ) {
            let mut test_output = TestUserOutput::with_theme(VerbosityLevel::Normal, theme);

            let message = TestThemedMessage {
                text: "Operation completed".to_string(),
            };

            test_output.output.write(&message);

            assert_eq!(test_output.stderr(), expected_output);
        }
    }

    mod sink {
        use super::*;
        use crate::presentation::views::testing::TestWriter;
        use crate::presentation::views::Channel;
        use parking_lot::Mutex;
        use std::sync::Arc;

        /// Test message for sink redirection testing
        struct TestSinkMessage {
            text: String,
        }

        impl OutputMessage for TestSinkMessage {
            fn format(&self, _theme: &Theme) -> String {
                format!("SINK_TEST: {}\n", self.text)
            }

            fn required_verbosity(&self) -> VerbosityLevel {
                VerbosityLevel::Normal
            }

            fn channel(&self) -> Channel {
                Channel::Stderr
            }

            fn type_name(&self) -> &'static str {
                "TestSinkMessage"
            }
        }

        #[test]
        fn it_should_write_output_to_custom_sink() {
            // Create custom buffers to capture output
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));

            // Create UserOutput with custom sink using TestWriter
            let mut output = UserOutput {
                theme: Theme::default(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: None,
            };

            // Write a message
            let message = TestSinkMessage {
                text: "custom sink output".to_string(),
            };
            output.write(&message);

            // Verify output was captured in custom sink (stderr buffer)
            let stderr_output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            assert_eq!(stderr_output, "SINK_TEST: custom sink output\n");

            // Stdout should be empty (message goes to stderr)
            let stdout_output = String::from_utf8(stdout_buffer.lock().clone()).unwrap();
            assert_eq!(stdout_output, "");
        }
    }

    mod channel_routing {
        use super::*;
        use crate::presentation::views::testing::TestUserOutput;
        use crate::presentation::views::Channel;
        use rstest::rstest;

        /// Test message that can be configured to go to either channel
        struct TestChannelMessage {
            text: String,
            target_channel: Channel,
        }

        impl OutputMessage for TestChannelMessage {
            fn format(&self, _theme: &Theme) -> String {
                format!("CHANNEL: {}\n", self.text)
            }

            fn required_verbosity(&self) -> VerbosityLevel {
                VerbosityLevel::Normal
            }

            fn channel(&self) -> Channel {
                self.target_channel
            }

            fn type_name(&self) -> &'static str {
                "TestChannelMessage"
            }
        }

        #[rstest]
        #[case(Channel::Stdout, "CHANNEL: stdout message\n", "")]
        #[case(Channel::Stderr, "", "CHANNEL: stderr message\n")]
        fn it_should_route_message_to_correct_channel(
            #[case] channel: Channel,
            #[case] expected_stdout: &str,
            #[case] expected_stderr: &str,
        ) {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            let message_text = match channel {
                Channel::Stdout => "stdout message",
                Channel::Stderr => "stderr message",
            };

            let message = TestChannelMessage {
                text: message_text.to_string(),
                target_channel: channel,
            };

            test_output.output.write(&message);

            assert_eq!(test_output.stdout(), expected_stdout);
            assert_eq!(test_output.stderr(), expected_stderr);
        }
    }
}
