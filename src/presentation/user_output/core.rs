//! Core `UserOutput` struct and implementation

use std::io::Write;

use super::messages::{
    ErrorMessage, InfoBlockMessage, ProgressMessage, ResultMessage, StepsMessage, SuccessMessage,
    WarningMessage,
};
use super::sinks::StandardSink;
use super::verbosity::VerbosityFilter;
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel, Theme};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel, Theme};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel, ProgressMessage};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.success("Configuration template generated");
    /// output.blank_line();
    /// output.progress("Starting next steps...");
    /// ```
    pub fn blank_line(&mut self) {
        if self.verbosity_filter.should_show_blank_lines() {
            // Create a simple message that just outputs a newline
            struct BlankLineMessage;
            impl OutputMessage for BlankLineMessage {
                fn format(&self, _theme: &Theme) -> String {
                    "\n".to_string()
                }
                fn required_verbosity(&self) -> VerbosityLevel {
                    VerbosityLevel::Normal
                }
                fn channel(&self) -> Channel {
                    Channel::Stderr
                }
                fn type_name(&self) -> &'static str {
                    "BlankLineMessage"
                }
            }
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
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

#[cfg(test)]
mod tests {
    use super::*;

    // These imports are used by nested test modules
    #[allow(unused_imports)]
    use crate::presentation::user_output::formatters::JsonFormatter;
    #[allow(unused_imports)]
    use crate::presentation::user_output::sinks::writers::{StderrWriter, StdoutWriter};
    #[allow(unused_imports)]
    use crate::presentation::user_output::sinks::{CompositeSink, FileSink, TelemetrySink};
    #[allow(unused_imports)]
    use crate::presentation::user_output::test_support::{self, TestUserOutput, TestWriter};

    // ============================================================================
    // Type-Safe Writer Wrapper Tests
    // ============================================================================

    mod type_safe_wrappers {
        use super::*;
        use parking_lot::Mutex;
        use std::sync::Arc;

        #[test]
        fn type_safe_dispatch_prevents_channel_confusion() {
            // This test demonstrates that the type system prevents channel confusion
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let stdout_writer = Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer)));
            let stderr_writer = Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer)));

            let mut stdout = StdoutWriter::new(stdout_writer);
            let mut stderr = StderrWriter::new(stderr_writer);

            // Type-safe: These methods can only be called on the correct writer type
            stdout.write_line("stdout data");
            stderr.write_line("stderr message");

            let stdout_output = String::from_utf8(stdout_buffer.lock().clone()).unwrap();
            let stderr_output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();

            assert_eq!(stdout_output, "stdout data");
            assert_eq!(stderr_output, "stderr message");

            // The following would not compile (demonstrating compile-time safety):
            // stderr.write_line("this should go to stdout");  // Type mismatch!
            // stdout.write_line("this should go to stderr");  // Type mismatch!
        }

        #[test]
        fn user_output_uses_typed_wrappers_internally() {
            // This test verifies that UserOutput uses typed wrappers internally
            // and that channel routing is type-safe
            let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

            // These calls go through type-safe dispatch
            test_output.output.progress("Progress message");
            test_output.output.result("Result data");

            // Verify correct channel routing via type system
            assert!(test_output.stderr().contains("Progress message"));
            assert!(test_output.stdout().contains("Result data"));
        }
    }

    // ============================================================================
    // UserOutput Tests - Parameterized Tests
    // ============================================================================
    //
    // These tests use rstest for parameterized testing to reduce duplication
    // and make the test matrix clear and maintainable.
    //
    // Test Matrix:
    // Message Type | Symbol | Min Verbosity | Channel | Always Shown
    // -------------|--------|---------------|---------|-------------
    // progress     | ⏳     | Normal        | stderr  | No
    // success      | ✅     | Normal        | stderr  | No
    // warning      | ⚠️     | Normal        | stderr  | No
    // error        | ❌     | Quiet         | stderr  | Yes
    // result       | (none) | Quiet         | stdout  | Yes
    // data         | (none) | Quiet         | stdout  | Yes

    mod parameterized_tests {
        use super::*;
        use rstest::rstest;

        /// Test that each message type routes to the correct output channel
        ///
        /// Verifies stdout vs stderr routing for all message types.
        /// This replaces 5 individual channel routing tests with one parameterized test.
        #[rstest]
        #[case("progress", "⏳ Test message\n", VerbosityLevel::Normal, "stderr")]
        #[case("success", "✅ Test message\n", VerbosityLevel::Normal, "stderr")]
        #[case("warning", "⚠️  Test message\n", VerbosityLevel::Normal, "stderr")]
        #[case("error", "❌ Test message\n", VerbosityLevel::Normal, "stderr")]
        #[case("result", "Test message\n", VerbosityLevel::Normal, "stdout")]
        fn it_should_route_message_to_correct_channel(
            #[case] method: &str,
            #[case] expected_output: &str,
            #[case] verbosity: VerbosityLevel,
            #[case] expected_channel: &str,
        ) {
            let mut test_output = test_support::TestUserOutput::new(verbosity);

            // Call the appropriate method
            match method {
                "progress" => test_output.output.progress("Test message"),
                "success" => test_output.output.success("Test message"),
                "warning" => test_output.output.warn("Test message"),
                "error" => test_output.output.error("Test message"),
                "result" => test_output.output.result("Test message"),
                _ => panic!("Unknown method: {method}"),
            }

            // Verify output went to the correct channel
            match expected_channel {
                "stdout" => {
                    assert_eq!(test_output.stdout(), expected_output);
                    assert_eq!(test_output.stderr(), "");
                }
                "stderr" => {
                    assert_eq!(test_output.stderr(), expected_output);
                    assert_eq!(test_output.stdout(), "");
                }
                _ => panic!("Unknown channel: {expected_channel}"),
            }
        }

        /// Test that normal-level messages respect verbosity settings
        ///
        /// Progress, success, and warning messages should only appear at Normal or higher.
        /// This replaces 3 individual verbosity tests with one parameterized test.
        #[rstest]
        #[case("progress", VerbosityLevel::Quiet, false)]
        #[case("progress", VerbosityLevel::Normal, true)]
        #[case("progress", VerbosityLevel::Verbose, true)]
        #[case("success", VerbosityLevel::Quiet, false)]
        #[case("success", VerbosityLevel::Normal, true)]
        #[case("success", VerbosityLevel::Verbose, true)]
        #[case("warning", VerbosityLevel::Quiet, false)]
        #[case("warning", VerbosityLevel::Normal, true)]
        #[case("warning", VerbosityLevel::Verbose, true)]
        fn it_should_respect_verbosity_for_normal_level_messages(
            #[case] method: &str,
            #[case] verbosity: VerbosityLevel,
            #[case] should_show: bool,
        ) {
            let mut test_output = test_support::TestUserOutput::new(verbosity);

            match method {
                "progress" => test_output.output.progress("Test"),
                "success" => test_output.output.success("Test"),
                "warning" => test_output.output.warn("Test"),
                _ => panic!("Unknown method: {method}"),
            }

            if should_show {
                assert!(!test_output.stderr().is_empty());
            } else {
                assert_eq!(test_output.stderr(), "");
            }
        }

        /// Test that error messages are always shown regardless of verbosity
        ///
        /// Errors are critical and must be shown at all verbosity levels.
        #[rstest]
        #[case(VerbosityLevel::Quiet)]
        #[case(VerbosityLevel::Normal)]
        #[case(VerbosityLevel::Verbose)]
        #[case(VerbosityLevel::VeryVerbose)]
        #[case(VerbosityLevel::Debug)]
        fn it_should_always_show_errors_at_all_verbosity_levels(#[case] verbosity: VerbosityLevel) {
            let mut test_output = test_support::TestUserOutput::new(verbosity);

            test_output.output.error("Critical error");

            assert!(!test_output.stderr().is_empty());
            assert!(test_output.stderr().contains("Critical error"));
        }

        /// Test that result messages are always shown at all verbosity levels
        ///
        /// Results are final outputs and must be shown at all verbosity levels.
        #[rstest]
        #[case(VerbosityLevel::Quiet)]
        #[case(VerbosityLevel::Normal)]
        #[case(VerbosityLevel::Verbose)]
        #[case(VerbosityLevel::VeryVerbose)]
        #[case(VerbosityLevel::Debug)]
        fn it_should_always_show_results_at_all_verbosity_levels(
            #[case] verbosity: VerbosityLevel,
        ) {
            let mut test_output = test_support::TestUserOutput::new(verbosity);

            test_output.output.result("Result data");

            assert_eq!(test_output.stdout(), "Result data\n");
            assert_eq!(test_output.stderr(), "");
        }
    }

    // ============================================================================
    // UserOutput Tests - Basic Output (Non-parameterized)
    // ============================================================================
    //
    // These tests cover specific functionality not included in parameterized tests:

    #[test]
    fn it_should_write_data_to_stdout() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.data(r#"{"status": "destroyed"}"#);

        // Verify message went to stdout
        assert_eq!(test_output.stdout(), "{\"status\": \"destroyed\"}\n");

        // Verify stderr is empty
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_write_blank_line_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.blank_line();

        // Verify blank line went to stderr
        assert_eq!(test_output.stderr(), "\n");

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_not_write_blank_line_at_quiet_level() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output.output.blank_line();

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_write_steps_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.steps(
            "Next steps:",
            &[
                "Edit the configuration file",
                "Review the settings",
                "Run the deploy command",
            ],
        );

        // Verify steps went to stderr with correct formatting
        assert_eq!(
            test_output.stderr(),
            "Next steps:\n1. Edit the configuration file\n2. Review the settings\n3. Run the deploy command\n"
        );

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_not_write_steps_at_quiet_level() {
        let test_output =
            test_support::TestUserOutput::new(VerbosityLevel::Quiet).into_reentrant_test_wrapper();

        test_output.steps("Next steps:", &["Step 1", "Step 2"]);

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_write_info_block_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.info_block(
            "Configuration options:",
            &[
                "  - username: 'torrust' (default)",
                "  - port: 22 (default SSH port)",
            ],
        );

        // Verify info block went to stderr
        assert_eq!(
            test_output.stderr(),
            "Configuration options:\n  - username: 'torrust' (default)\n  - port: 22 (default SSH port)\n"
        );

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_not_write_info_block_at_quiet_level() {
        let test_output =
            test_support::TestUserOutput::new(VerbosityLevel::Quiet).into_reentrant_test_wrapper();

        test_output.info_block("Info:", &["Line 1", "Line 2"]);

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    // ============================================================================
    // OutputMessage Trait Tests
    // ============================================================================

    mod output_message_trait {
        use super::super::*;
        use crate::presentation::user_output::test_support::TestUserOutput;

        #[test]
        fn progress_message_should_format_with_theme() {
            let theme = Theme::emoji();
            let message = ProgressMessage {
                text: "Test message".to_string(),
            };

            let formatted = message.format(&theme);

            assert_eq!(formatted, "⏳ Test message\n");
        }

        #[test]
        fn progress_message_should_require_normal_verbosity() {
            let message = ProgressMessage {
                text: "Test".to_string(),
            };

            assert_eq!(message.required_verbosity(), VerbosityLevel::Normal);
        }

        #[test]
        fn progress_message_should_use_stderr_channel() {
            let message = ProgressMessage {
                text: "Test".to_string(),
            };

            assert_eq!(message.channel(), Channel::Stderr);
        }

        #[test]
        fn success_message_should_format_with_theme() {
            let theme = Theme::plain();
            let message = SuccessMessage {
                text: "Operation complete".to_string(),
            };

            let formatted = message.format(&theme);

            assert_eq!(formatted, "[OK] Operation complete\n");
        }

        #[test]
        fn error_message_should_always_be_shown() {
            let message = ErrorMessage {
                text: "Critical error".to_string(),
            };

            // Errors should require Quiet level (always shown)
            assert_eq!(message.required_verbosity(), VerbosityLevel::Quiet);
        }

        #[test]
        fn result_message_should_use_stdout_channel() {
            let message = ResultMessage {
                text: "Output data".to_string(),
            };

            assert_eq!(message.channel(), Channel::Stdout);
        }

        #[test]
        fn result_message_should_not_include_symbols() {
            let theme = Theme::emoji();
            let message = ResultMessage {
                text: "Plain output".to_string(),
            };

            let formatted = message.format(&theme);

            // Result messages should not include theme symbols
            assert!(!formatted.contains("⏳"));
            assert!(!formatted.contains("✅"));
            assert_eq!(formatted, "Plain output\n");
        }

        #[test]
        fn steps_message_should_format_numbered_list() {
            let theme = Theme::emoji();
            let message = StepsMessage {
                title: "Next steps:".to_string(),
                items: vec!["First step".to_string(), "Second step".to_string()],
            };

            let formatted = message.format(&theme);

            assert_eq!(formatted, "Next steps:\n1. First step\n2. Second step\n");
        }

        #[test]
        fn warning_message_should_include_extra_space() {
            let theme = Theme::emoji();
            let message = WarningMessage {
                text: "Warning text".to_string(),
            };

            let formatted = message.format(&theme);

            // Warning messages include two spaces after the symbol
            assert_eq!(formatted, "⚠️  Warning text\n");
        }

        #[test]
        fn user_output_write_should_respect_verbosity_filter() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Quiet);

            // Normal-level message should be filtered
            test_output.output.write(&ProgressMessage {
                text: "Should not appear".to_string(),
            });

            assert_eq!(test_output.stderr(), "");

            // Quiet-level message should be shown
            test_output.output.write(&ErrorMessage {
                text: "Should appear".to_string(),
            });

            assert_eq!(test_output.stderr(), "❌ Should appear\n");
        }

        #[test]
        fn user_output_write_should_route_to_correct_channel() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Stderr message
            test_output.output.write(&ProgressMessage {
                text: "Progress".to_string(),
            });

            // Stdout message
            test_output.output.write(&ResultMessage {
                text: "Result".to_string(),
            });

            assert_eq!(test_output.stderr(), "⏳ Progress\n");
            assert_eq!(test_output.stdout(), "Result\n");
        }

        #[test]
        fn channel_enum_should_support_equality() {
            assert_eq!(Channel::Stdout, Channel::Stdout);
            assert_eq!(Channel::Stderr, Channel::Stderr);
            assert_ne!(Channel::Stdout, Channel::Stderr);
        }

        // Custom message type to demonstrate extensibility
        struct CustomDebugMessage {
            text: String,
        }

        impl OutputMessage for CustomDebugMessage {
            fn format(&self, _theme: &Theme) -> String {
                format!("[DEBUG] {}\n", self.text)
            }

            fn required_verbosity(&self) -> VerbosityLevel {
                VerbosityLevel::Debug
            }

            fn channel(&self) -> Channel {
                Channel::Stderr
            }

            fn type_name(&self) -> &'static str {
                "CustomDebugMessage"
            }
        }

        #[test]
        fn custom_message_type_should_work_with_write() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Debug);

            test_output.output.write(&CustomDebugMessage {
                text: "Custom debug message".to_string(),
            });

            assert_eq!(test_output.stderr(), "[DEBUG] Custom debug message\n");
        }

        #[test]
        fn custom_message_type_should_respect_verbosity() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Debug-level custom message should not appear at Normal level
            test_output.output.write(&CustomDebugMessage {
                text: "Should not appear".to_string(),
            });

            assert_eq!(test_output.stderr(), "");
        }

        #[test]
        fn open_closed_principle_demonstration() {
            // This test demonstrates that new message types can be added
            // without modifying the UserOutput struct

            struct CustomInfoMessage {
                category: String,
                message: String,
            }

            impl OutputMessage for CustomInfoMessage {
                fn format(&self, _theme: &Theme) -> String {
                    format!("ℹ️  [{}] {}\n", self.category, self.message)
                }

                fn required_verbosity(&self) -> VerbosityLevel {
                    VerbosityLevel::Verbose
                }

                fn channel(&self) -> Channel {
                    Channel::Stderr
                }

                fn type_name(&self) -> &'static str {
                    "CustomInfoMessage"
                }
            }

            let mut test_output = TestUserOutput::new(VerbosityLevel::Verbose);

            test_output.output.write(&CustomInfoMessage {
                category: "CONFIG".to_string(),
                message: "Loading configuration".to_string(),
            });

            assert_eq!(test_output.stderr(), "ℹ️  [CONFIG] Loading configuration\n");
        }
    }

    // ============================================================================
    // UserOutput with Theme Tests
    // ============================================================================

    mod user_output_with_themes {
        use super::super::*;
        use crate::presentation::user_output::test_support::TestUserOutput;

        #[test]
        fn it_should_use_emoji_theme_by_default() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            test_output.output.progress("Test");

            assert_eq!(test_output.stderr(), "⏳ Test\n");
        }

        #[test]
        fn it_should_use_plain_theme_when_specified() {
            let mut test_output =
                TestUserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());

            test_output.output.progress("Test");
            test_output.output.success("Success");
            test_output.output.warn("Warning");
            test_output.output.error("Error");

            let stderr = test_output.stderr();
            assert!(stderr.contains("[INFO] Test"));
            assert!(stderr.contains("[OK] Success"));
            assert!(stderr.contains("[WARN]  Warning"));
            assert!(stderr.contains("[ERROR] Error"));
        }

        #[test]
        fn it_should_use_ascii_theme_when_specified() {
            let mut test_output =
                TestUserOutput::with_theme(VerbosityLevel::Normal, Theme::ascii());

            test_output.output.progress("Test");
            test_output.output.success("Success");
            test_output.output.warn("Warning");
            test_output.output.error("Error");

            let stderr = test_output.stderr();
            assert!(stderr.contains("=> Test"));
            assert!(stderr.contains("[+] Success"));
            assert!(stderr.contains("[!]  Warning"));
            assert!(stderr.contains("[x] Error"));
        }

        #[test]
        fn it_should_support_with_theme_constructor() {
            let output = UserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());

            // Verify it compiles and creates output with the theme
            // (actual output testing done through TestUserOutput)
            drop(output);
        }
    }

    // ============================================================================
    // FormatterOverride Tests
    // ============================================================================

    mod formatter_override {
        use super::super::*;
        use crate::presentation::user_output::formatters::JsonFormatter;
        use crate::presentation::user_output::test_support::{self, TestUserOutput};
        use parking_lot::Mutex;
        use std::sync::Arc;

        // Custom test formatter to verify override is applied
        struct TestFormatter {
            prefix: String,
        }

        impl FormatterOverride for TestFormatter {
            fn transform(&self, formatted: &str, _message: &dyn OutputMessage) -> String {
                format!("{}{}", self.prefix, formatted)
            }
        }

        #[test]
        fn it_should_apply_formatter_override_to_messages() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(TestFormatter {
                prefix: "[TEST] ".to_string(),
            });

            let mut output = UserOutput {
                theme: Theme::plain(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test message");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            assert_eq!(stderr, "[TEST] [INFO] Test message\n");
        }

        #[test]
        fn it_should_not_apply_override_when_none() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            test_output.output.progress("Test message");

            // Without override, should see normal formatted output
            assert_eq!(test_output.stderr(), "⏳ Test message\n");
        }

        #[test]
        fn it_should_work_with_json_formatter() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test message");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();

            // Parse JSON to verify structure (trim to remove trailing newline)
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).expect("Valid JSON");

            assert_eq!(json["type"], "ProgressMessage");
            assert_eq!(json["channel"], "Stderr");
            assert_eq!(json["content"], "⏳ Test message");
            assert!(json["timestamp"].is_string());
        }

        #[test]
        fn it_should_include_correct_type_name_in_json() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            // Test different message types
            output.progress("Progress");
            output.success("Success");
            output.warn("Warning");
            output.error("Error");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let lines: Vec<&str> = stderr.lines().collect();

            let progress_json: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
            assert_eq!(progress_json["type"], "ProgressMessage");

            let success_json: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
            assert_eq!(success_json["type"], "SuccessMessage");

            let warning_json: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
            assert_eq!(warning_json["type"], "WarningMessage");

            let error_json: serde_json::Value = serde_json::from_str(lines[3]).unwrap();
            assert_eq!(error_json["type"], "ErrorMessage");
        }

        #[test]
        fn it_should_include_correct_channel_in_json() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Stderr message");
            output.result("Stdout message");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let stdout = String::from_utf8(stdout_buffer.lock().clone()).unwrap();

            let stderr_json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
            assert_eq!(stderr_json["channel"], "Stderr");

            let stdout_json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
            assert_eq!(stdout_json["channel"], "Stdout");
        }

        #[test]
        fn it_should_trim_trailing_newlines_in_json() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            // Content should not have trailing newline
            let content = json["content"].as_str().unwrap();
            assert!(!content.ends_with('\n'));
            assert_eq!(content, "⏳ Test");
        }

        #[test]
        fn it_should_respect_theme_with_json_formatter() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::plain(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            // Content should reflect plain theme
            assert_eq!(json["content"], "[INFO] Test");
        }

        #[test]
        fn it_should_respect_verbosity_with_formatter_override() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Quiet),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            // Normal-level message should be filtered at Quiet level
            output.progress("Should not appear");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            assert_eq!(stderr, "");

            // Quiet-level message should appear
            output.error("Should appear");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
            assert_eq!(json["type"], "ErrorMessage");
        }

        #[test]
        fn it_should_create_output_with_formatter_override_constructor() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::default(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test");

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            assert_eq!(json["type"], "ProgressMessage");
        }

        #[test]
        fn it_should_work_with_steps_message() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.steps("Next steps:", &["Step 1", "Step 2"]);

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            assert_eq!(json["type"], "StepsMessage");
            assert_eq!(json["channel"], "Stderr");
            // Content should include formatted steps
            let content = json["content"].as_str().unwrap();
            assert!(content.contains("Next steps:"));
            assert!(content.contains("1. Step 1"));
            assert!(content.contains("2. Step 2"));
        }

        #[test]
        fn it_should_produce_valid_json_for_all_message_types() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            // Test all message types
            output.progress("Progress");
            output.success("Success");
            output.warn("Warning");
            output.error("Error");
            output.result("Result");
            output.steps("Steps:", &["Step 1"]);

            // Verify all stderr messages are valid JSON
            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            for line in stderr.lines() {
                let json: Result<serde_json::Value, _> = serde_json::from_str(line);
                assert!(json.is_ok(), "Invalid JSON: {line}");
            }

            // Verify stdout message is valid JSON
            let stdout = String::from_utf8(stdout_buffer.lock().clone()).unwrap();
            let json: Result<serde_json::Value, _> = serde_json::from_str(stdout.trim());
            assert!(json.is_ok(), "Invalid JSON in stdout");
        }
    }

    // ============================================================================
    // Buffering Tests
    // ============================================================================

    mod buffering {
        use super::super::*;
        use crate::presentation::user_output::test_support::TestUserOutput;

        #[test]
        fn it_should_flush_all_writers() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            test_output.output.progress("Test message");

            // Flush should succeed
            test_output.output.flush().expect("Flush should succeed");

            // Verify output is present (flushed)
            assert!(!test_output.stderr().is_empty());
            assert!(test_output.stderr().contains("Test message"));
        }

        #[test]
        fn it_should_be_safe_to_flush_multiple_times() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            test_output.output.progress("Test message");

            // Multiple flushes should be safe
            test_output
                .output
                .flush()
                .expect("First flush should succeed");
            test_output
                .output
                .flush()
                .expect("Second flush should succeed");
            test_output
                .output
                .flush()
                .expect("Third flush should succeed");

            // Output should still be present
            assert!(!test_output.stderr().is_empty());
        }

        #[test]
        fn it_should_flush_empty_buffers_safely() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Flushing with no output should be safe
            test_output
                .output
                .flush()
                .expect("Flushing empty buffers should succeed");

            // No output should be present
            assert_eq!(test_output.stderr(), "");
            assert_eq!(test_output.stdout(), "");
        }

        #[test]
        fn it_should_flush_both_stdout_and_stderr() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Write to both channels
            test_output.output.progress("Progress message");
            test_output.output.result("Result data");

            // Flush should handle both channels
            test_output
                .output
                .flush()
                .expect("Flush should succeed for both channels");

            // Verify both outputs are present
            assert!(test_output.stderr().contains("Progress message"));
            assert!(test_output.stdout().contains("Result data"));
        }

        #[test]
        fn it_should_work_with_sequential_flush_calls() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Write, flush, write, flush pattern
            test_output.output.progress("Message 1");
            test_output
                .output
                .flush()
                .expect("First flush should succeed");

            test_output.output.progress("Message 2");
            test_output
                .output
                .flush()
                .expect("Second flush should succeed");

            // Both messages should be present
            let stderr = test_output.stderr();
            assert!(stderr.contains("Message 1"));
            assert!(stderr.contains("Message 2"));
        }
    }

    // ============================================================================
    // Builder Pattern Tests
    // ============================================================================

    mod builder_pattern {
        use super::super::*;
        use crate::presentation::user_output::formatters::JsonFormatter;
        use crate::presentation::user_output::test_support::{self, TestUserOutput};

        // ========================================================================
        // StepsMessageBuilder Tests
        // ========================================================================

        #[test]
        fn it_should_build_steps_with_fluent_api() {
            let message = StepsMessage::builder("Title")
                .add("Step 1")
                .add("Step 2")
                .add("Step 3")
                .build();

            assert_eq!(message.title, "Title");
            assert_eq!(message.items, vec!["Step 1", "Step 2", "Step 3"]);
        }

        #[test]
        fn it_should_create_simple_steps_directly() {
            let message =
                StepsMessage::new("Title", vec!["Step 1".to_string(), "Step 2".to_string()]);

            assert_eq!(message.title, "Title");
            assert_eq!(message.items, vec!["Step 1", "Step 2"]);
        }

        #[test]
        fn it_should_build_empty_steps() {
            let message = StepsMessage::builder("Title").build();

            assert_eq!(message.title, "Title");
            assert!(message.items.is_empty());
        }

        #[test]
        fn it_should_build_single_step() {
            let message = StepsMessage::builder("Title").add("Single step").build();

            assert_eq!(message.title, "Title");
            assert_eq!(message.items, vec!["Single step"]);
        }

        #[test]
        fn it_should_accept_string_types_in_builder() {
            let message = StepsMessage::builder("Title")
                .add("String literal")
                .add(String::from("Owned string"))
                .add("Another literal".to_string())
                .build();

            assert_eq!(message.items.len(), 3);
        }

        #[test]
        fn it_should_accept_string_types_in_constructor() {
            let message =
                StepsMessage::new("Title", vec!["Step 1".to_string(), String::from("Step 2")]);

            assert_eq!(message.items.len(), 2);
        }

        #[test]
        fn it_should_format_builder_messages_correctly() {
            let theme = Theme::emoji();
            let message = StepsMessage::builder("Next steps:")
                .add("Configure")
                .add("Deploy")
                .build();

            let formatted = message.format(&theme);
            assert!(formatted.contains("Next steps:"));
            assert!(formatted.contains("1. Configure"));
            assert!(formatted.contains("2. Deploy"));
        }

        #[test]
        fn it_should_integrate_builder_with_user_output() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            let message = StepsMessage::builder("Next steps:")
                .add("Edit config")
                .add("Run tests")
                .build();

            test_output.output.write(&message);

            let stderr = test_output.stderr();
            assert!(stderr.contains("Next steps:"));
            assert!(stderr.contains("1. Edit config"));
            assert!(stderr.contains("2. Run tests"));
        }

        // ========================================================================
        // InfoBlockMessageBuilder Tests
        // ========================================================================

        #[test]
        fn it_should_build_info_block_with_fluent_api() {
            let message = InfoBlockMessage::builder("Environment")
                .add_line("Name: production")
                .add_line("Status: active")
                .build();

            assert_eq!(message.title, "Environment");
            assert_eq!(message.lines, vec!["Name: production", "Status: active"]);
        }

        #[test]
        fn it_should_create_simple_info_block_directly() {
            let message = InfoBlockMessage::new(
                "Environment",
                vec!["Name: production".to_string(), "Status: active".to_string()],
            );

            assert_eq!(message.title, "Environment");
            assert_eq!(message.lines, vec!["Name: production", "Status: active"]);
        }

        #[test]
        fn it_should_build_empty_info_block() {
            let message = InfoBlockMessage::builder("Title").build();

            assert_eq!(message.title, "Title");
            assert!(message.lines.is_empty());
        }

        #[test]
        fn it_should_build_single_line_info_block() {
            let message = InfoBlockMessage::builder("Title")
                .add_line("Single line")
                .build();

            assert_eq!(message.title, "Title");
            assert_eq!(message.lines, vec!["Single line"]);
        }

        #[test]
        fn it_should_accept_string_types_in_info_block_builder() {
            let message = InfoBlockMessage::builder("Title")
                .add_line("String literal")
                .add_line(String::from("Owned string"))
                .add_line("Another literal".to_string())
                .build();

            assert_eq!(message.lines.len(), 3);
        }

        #[test]
        fn it_should_accept_string_types_in_info_block_constructor() {
            let message =
                InfoBlockMessage::new("Title", vec!["Line 1".to_string(), String::from("Line 2")]);

            assert_eq!(message.lines.len(), 2);
        }

        #[test]
        fn it_should_format_info_block_messages_correctly() {
            let theme = Theme::emoji();
            let message = InfoBlockMessage::builder("Environment")
                .add_line("Name: production")
                .add_line("Status: active")
                .build();

            let formatted = message.format(&theme);
            assert!(formatted.contains("Environment"));
            assert!(formatted.contains("Name: production"));
            assert!(formatted.contains("Status: active"));
        }

        #[test]
        fn it_should_integrate_info_block_builder_with_user_output() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            let message = InfoBlockMessage::builder("Configuration")
                .add_line("  - username: torrust")
                .add_line("  - port: 22")
                .build();

            test_output.output.write(&message);

            let stderr = test_output.stderr();
            assert!(stderr.contains("Configuration"));
            assert!(stderr.contains("  - username: torrust"));
            assert!(stderr.contains("  - port: 22"));
        }

        #[test]
        fn it_should_show_info_block_message_has_correct_properties() {
            let message = InfoBlockMessage::new("Title", vec!["Line 1".to_string()]);

            assert_eq!(message.required_verbosity(), VerbosityLevel::Normal);
            assert_eq!(message.channel(), Channel::Stderr);
            assert_eq!(message.type_name(), "InfoBlockMessage");
        }

        #[test]
        fn it_should_respect_verbosity_for_info_block_messages() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Quiet);

            let message = InfoBlockMessage::builder("Info").add_line("Line 1").build();

            test_output.output.write(&message);

            // Should not appear at Quiet level
            assert_eq!(test_output.stderr(), "");
        }

        #[test]
        fn it_should_show_info_block_at_normal_level() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            let message = InfoBlockMessage::builder("Info").add_line("Line 1").build();

            test_output.output.write(&message);

            // Should appear at Normal level
            assert!(!test_output.stderr().is_empty());
            assert!(test_output.stderr().contains("Info"));
        }

        // ========================================================================
        // Backward Compatibility Tests
        // ========================================================================

        #[test]
        fn it_should_maintain_backward_compatibility_for_steps() {
            // Old way: direct construction
            let old_message = StepsMessage {
                title: "Steps".to_string(),
                items: vec!["Step 1".to_string()],
            };

            // New way: constructor
            let new_message = StepsMessage::new("Steps", vec!["Step 1".to_string()]);

            // Should produce identical results
            assert_eq!(old_message.title, new_message.title);
            assert_eq!(old_message.items, new_message.items);
        }

        #[test]
        fn it_should_maintain_backward_compatibility_for_info_blocks() {
            // Old way: UserOutput::info_block helper
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            test_output
                .output
                .info_block("Title", &["Line 1", "Line 2"]);
            let old_output = test_output.stderr();

            // New way: Direct message construction
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            let message =
                InfoBlockMessage::new("Title", vec!["Line 1".to_string(), "Line 2".to_string()]);
            test_output.output.write(&message);
            let new_output = test_output.stderr();

            // Should produce identical output
            assert_eq!(old_output, new_output);
        }

        // ========================================================================
        // Integration Tests
        // ========================================================================

        #[test]
        fn it_should_work_with_json_formatter() {
            use parking_lot::Mutex;
            use std::sync::Arc;

            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            let message = StepsMessage::builder("Steps").add("Step 1").build();
            output.write(&message);

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            assert_eq!(json["type"], "StepsMessage");
        }

        #[test]
        fn it_should_work_with_info_block_json_formatter() {
            use parking_lot::Mutex;
            use std::sync::Arc;

            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            let message = InfoBlockMessage::builder("Info").add_line("Line 1").build();
            output.write(&message);

            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            assert_eq!(json["type"], "InfoBlockMessage");
        }

        #[test]
        fn it_should_handle_many_items_in_builder() {
            let mut builder = StepsMessage::builder("Many steps");
            for i in 1..=100 {
                builder = builder.add(format!("Step {i}"));
            }
            let message = builder.build();

            assert_eq!(message.items.len(), 100);
            assert_eq!(message.items[0], "Step 1");
            assert_eq!(message.items[99], "Step 100");
        }

        #[test]
        fn it_should_handle_many_lines_in_info_block_builder() {
            let mut builder = InfoBlockMessage::builder("Many lines");
            for i in 1..=100 {
                builder = builder.add_line(format!("Line {i}"));
            }
            let message = builder.build();

            assert_eq!(message.lines.len(), 100);
            assert_eq!(message.lines[0], "Line 1");
            assert_eq!(message.lines[99], "Line 100");
        }
    }

    // ============================================================================
    // OutputSink Tests
    // ============================================================================

    mod output_sink {
        use super::super::*;
        use crate::presentation::user_output::test_support;
        use crate::presentation::user_output::{CompositeSink, FileSink, TelemetrySink};
        use parking_lot::Mutex;
        use std::sync::Arc;

        /// Mock sink for testing that captures messages
        struct MockSink {
            messages: Arc<Mutex<Vec<String>>>,
        }

        impl MockSink {
            fn new(messages: Arc<Mutex<Vec<String>>>) -> Self {
                Self { messages }
            }
        }

        impl OutputSink for MockSink {
            fn write_message(&mut self, _message: &dyn OutputMessage, formatted: &str) {
                self.messages.lock().push(formatted.to_string());
            }
        }

        // ========================================================================
        // StandardSink Tests
        // ========================================================================

        #[test]
        fn standard_sink_should_route_stdout_messages() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let mut sink = StandardSink::new(
                Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
            );

            let message = ResultMessage {
                text: "Test result".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            sink.write_message(&message, &formatted);

            let stdout = String::from_utf8(stdout_buffer.lock().clone()).unwrap();
            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();

            assert_eq!(stdout, "Test result\n");
            assert_eq!(stderr, "");
        }

        #[test]
        fn standard_sink_should_route_stderr_messages() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let mut sink = StandardSink::new(
                Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
            );

            let message = ProgressMessage {
                text: "Test progress".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            sink.write_message(&message, &formatted);

            let stdout = String::from_utf8(stdout_buffer.lock().clone()).unwrap();
            let stderr = String::from_utf8(stderr_buffer.lock().clone()).unwrap();

            assert_eq!(stdout, "");
            assert_eq!(stderr, "⏳ Test progress\n");
        }

        #[test]
        fn standard_sink_default_console_should_create_default_sink() {
            let _sink = StandardSink::default_console();
            // If we got here without panicking, the sink was created successfully
        }

        // ========================================================================
        // CompositeSink Tests
        // ========================================================================

        #[test]
        fn composite_sink_should_write_to_all_sinks() {
            let messages1 = Arc::new(Mutex::new(Vec::new()));
            let messages2 = Arc::new(Mutex::new(Vec::new()));
            let messages3 = Arc::new(Mutex::new(Vec::new()));

            let mut composite = CompositeSink::new(vec![
                Box::new(MockSink::new(Arc::clone(&messages1))),
                Box::new(MockSink::new(Arc::clone(&messages2))),
                Box::new(MockSink::new(Arc::clone(&messages3))),
            ]);

            let message = ProgressMessage {
                text: "Test".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            composite.write_message(&message, &formatted);

            // Verify all sinks received the message
            assert_eq!(messages1.lock().len(), 1);
            assert_eq!(messages2.lock().len(), 1);
            assert_eq!(messages3.lock().len(), 1);

            assert_eq!(messages1.lock()[0], "⏳ Test\n");
            assert_eq!(messages2.lock()[0], "⏳ Test\n");
            assert_eq!(messages3.lock()[0], "⏳ Test\n");
        }

        #[test]
        fn composite_sink_should_support_empty_sink_list() {
            let mut composite = CompositeSink::new(vec![]);

            let message = ProgressMessage {
                text: "Test".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            // Should not panic with empty sink list
            composite.write_message(&message, &formatted);
        }

        #[test]
        fn composite_sink_should_support_add_sink() {
            let messages1 = Arc::new(Mutex::new(Vec::new()));
            let messages2 = Arc::new(Mutex::new(Vec::new()));

            let mut composite =
                CompositeSink::new(vec![Box::new(MockSink::new(Arc::clone(&messages1)))]);

            // Add another sink
            composite.add_sink(Box::new(MockSink::new(Arc::clone(&messages2))));

            let message = ProgressMessage {
                text: "Test".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            composite.write_message(&message, &formatted);

            // Verify both sinks received the message
            assert_eq!(messages1.lock().len(), 1);
            assert_eq!(messages2.lock().len(), 1);
        }

        #[test]
        fn composite_sink_should_write_multiple_messages() {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let mut composite =
                CompositeSink::new(vec![Box::new(MockSink::new(Arc::clone(&messages)))]);

            let theme = Theme::emoji();

            // Write multiple messages
            let msg1 = ProgressMessage {
                text: "First".to_string(),
            };
            composite.write_message(&msg1, &msg1.format(&theme));

            let msg2 = SuccessMessage {
                text: "Second".to_string(),
            };
            composite.write_message(&msg2, &msg2.format(&theme));

            let msg3 = ErrorMessage {
                text: "Third".to_string(),
            };
            composite.write_message(&msg3, &msg3.format(&theme));

            // Verify all messages were received
            let captured = messages.lock();
            assert_eq!(captured.len(), 3);
            assert_eq!(captured[0], "⏳ First\n");
            assert_eq!(captured[1], "✅ Second\n");
            assert_eq!(captured[2], "❌ Third\n");
        }

        // ========================================================================
        // UserOutput with Custom Sinks Tests
        // ========================================================================

        #[test]
        fn user_output_should_work_with_custom_sink() {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let sink = Box::new(MockSink::new(Arc::clone(&messages)));

            let mut output = UserOutput::with_sink(VerbosityLevel::Normal, sink);

            output.progress("Progress message");
            output.success("Success message");
            output.error("Error message");

            let captured = messages.lock();
            assert_eq!(captured.len(), 3);
            assert!(captured[0].contains("Progress message"));
            assert!(captured[1].contains("Success message"));
            assert!(captured[2].contains("Error message"));
        }

        #[test]
        fn user_output_should_work_with_composite_sink() {
            let messages1 = Arc::new(Mutex::new(Vec::new()));
            let messages2 = Arc::new(Mutex::new(Vec::new()));

            let composite = CompositeSink::new(vec![
                Box::new(MockSink::new(Arc::clone(&messages1))),
                Box::new(MockSink::new(Arc::clone(&messages2))),
            ]);

            let mut output = UserOutput::with_sink(VerbosityLevel::Normal, Box::new(composite));

            output.progress("Test message");

            // Verify both sinks received the message
            assert_eq!(messages1.lock().len(), 1);
            assert_eq!(messages2.lock().len(), 1);
            assert!(messages1.lock()[0].contains("Test message"));
            assert!(messages2.lock()[0].contains("Test message"));
        }

        #[test]
        fn user_output_with_sink_should_respect_verbosity() {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let sink = Box::new(MockSink::new(Arc::clone(&messages)));

            let mut output = UserOutput::with_sink(VerbosityLevel::Quiet, sink);

            // Normal-level message should not appear
            output.progress("Should not appear");

            // Quiet-level message should appear
            output.error("Should appear");

            let captured = messages.lock();
            assert_eq!(captured.len(), 1);
            assert!(captured[0].contains("Should appear"));
        }

        #[test]
        fn user_output_with_sink_should_use_default_theme() {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let sink = Box::new(MockSink::new(Arc::clone(&messages)));

            let mut output = UserOutput::with_sink(VerbosityLevel::Normal, sink);

            output.progress("Test");

            let captured = messages.lock();
            // Should use emoji theme by default
            assert!(captured[0].contains("⏳"));
        }

        // ========================================================================
        // FileSink Tests
        // ========================================================================

        #[test]
        fn file_sink_should_create_and_write_to_file() {
            use std::io::Read;
            use tempfile::NamedTempFile;

            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();

            let mut sink = FileSink::new(path).unwrap();

            let message = ProgressMessage {
                text: "Test message".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            sink.write_message(&message, &formatted);

            // Read back the file content
            let mut file = std::fs::File::open(path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            assert_eq!(content, "⏳ Test message\n\n");
        }

        #[test]
        fn file_sink_should_append_to_existing_file() {
            use std::io::Read;
            use tempfile::NamedTempFile;

            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();

            // Write first message
            let mut sink1 = FileSink::new(path).unwrap();
            let message1 = ProgressMessage {
                text: "First".to_string(),
            };
            let theme = Theme::emoji();
            sink1.write_message(&message1, &message1.format(&theme));
            drop(sink1);

            // Write second message
            let mut sink2 = FileSink::new(path).unwrap();
            let message2 = SuccessMessage {
                text: "Second".to_string(),
            };
            sink2.write_message(&message2, &message2.format(&theme));
            drop(sink2);

            // Read back the file content
            let mut file = std::fs::File::open(path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            assert!(content.contains("First"));
            assert!(content.contains("Second"));
        }

        // ========================================================================
        // TelemetrySink Tests
        // ========================================================================

        #[test]
        fn telemetry_sink_should_create_with_endpoint() {
            let sink = TelemetrySink::new("https://example.com".to_string());
            assert_eq!(sink.endpoint(), "https://example.com");
        }

        #[test]
        fn telemetry_sink_should_log_messages() {
            let mut sink = TelemetrySink::new("https://example.com".to_string());

            let message = ProgressMessage {
                text: "Test".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            // This just logs via tracing, so we verify it doesn't panic
            sink.write_message(&message, &formatted);
        }
    }
}
