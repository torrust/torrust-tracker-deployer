//! User-facing output handling
//!
//! This module provides user-facing output functionality separate from internal logging.
//! It implements a dual-channel strategy following Unix conventions and modern CLI best practices
//! (similar to cargo, docker, npm):
//!
//! - **stdout (Results Channel)**: Final results, structured data, output for piping/redirection
//! - **stderr (Progress/Operational Channel)**: Progress updates, status messages, warnings, errors
//!
//! This separation enables:
//! - Clean piping: `torrust-tracker-deployer destroy env | jq .status` works correctly
//! - Automation friendly: Scripts can redirect progress to /dev/null while capturing results
//! - Unix convention compliance: Follows established patterns from modern CLI tools
//! - Better UX: Progress feedback doesn't interfere with result data
//!
//! ## Example Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
//!
//! let mut output = UserOutput::new(VerbosityLevel::Normal);
//!
//! // Progress messages go to stderr
//! output.progress("Destroying environment...");
//!
//! // Success status goes to stderr
//! output.success("Environment destroyed successfully");
//!
//! // Results go to stdout for piping
//! output.result(r#"{"status": "destroyed"}"#);
//! ```
//!
//! ## Channel Strategy
//!
//! Based on research from [`docs/research/UX/console-app-output-patterns.md`](../../docs/research/UX/console-app-output-patterns.md):
//!
//! - **stdout**: Deployment results, configuration summaries, structured data (JSON)
//! - **stderr**: Step progress, status updates, warnings, error messages with actionable guidance
//!
//! See also: [`docs/research/UX/user-output-vs-logging-separation.md`](../../docs/research/UX/user-output-vs-logging-separation.md)

use std::io::Write;

/// Verbosity levels for user output
///
/// Controls the amount of detail shown to users. Higher verbosity levels include
/// all output from lower levels.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::VerbosityLevel;
///
/// let level = VerbosityLevel::Normal;
/// assert!(level >= VerbosityLevel::Quiet);
/// assert!(level < VerbosityLevel::Verbose);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum VerbosityLevel {
    /// Minimal output - only errors and final results
    Quiet,
    /// Default level - essential progress and results
    #[default]
    Normal,
    /// Detailed progress including intermediate steps
    Verbose,
    /// Very detailed including decisions and retries
    VeryVerbose,
    /// Maximum detail for troubleshooting
    Debug,
}

/// Determines what messages should be displayed based on verbosity level
///
/// This struct encapsulates verbosity filtering logic, making it testable
/// independently from output formatting.
struct VerbosityFilter {
    level: VerbosityLevel,
}

impl VerbosityFilter {
    /// Create a new verbosity filter with the specified level
    fn new(level: VerbosityLevel) -> Self {
        Self { level }
    }

    /// Check if messages at the given level should be shown
    fn should_show(&self, required_level: VerbosityLevel) -> bool {
        self.level >= required_level
    }

    /// Progress messages require Normal level
    fn should_show_progress(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Success messages require Normal level
    fn should_show_success(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Warning messages require Normal level
    fn should_show_warnings(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Errors are always shown regardless of verbosity level
    #[allow(clippy::unused_self)]
    fn should_show_errors(&self) -> bool {
        true
    }

    /// Blank lines require Normal level
    fn should_show_blank_lines(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Steps require Normal level
    fn should_show_steps(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Info blocks require Normal level
    fn should_show_info_blocks(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }
}

/// Handles user-facing output separate from internal logging
///
/// Uses dual channels following Unix conventions and modern CLI best practices:
/// - **stdout**: Final results and data for piping/redirection
/// - **stderr**: Progress updates, status messages, operational info, errors
///
/// This separation allows scripts to cleanly capture results while seeing progress:
///
/// ```bash
/// # Suppress progress, capture results only
/// torrust-tracker-deployer destroy env 2>/dev/null > result.json
///
/// # Suppress results, see progress only
/// torrust-tracker-deployer destroy env > /dev/null
/// ```
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
///
/// let mut output = UserOutput::new(VerbosityLevel::Normal);
///
/// // Progress to stderr (visible during execution, doesn't interfere with piping)
/// output.progress("Processing data...");
///
/// // Results to stdout (can be piped to other commands)
/// output.result("Processing complete");
/// ```
pub struct UserOutput {
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    /// Create new `UserOutput` with default stdout/stderr channels
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
        Self {
            verbosity_filter: VerbosityFilter::new(verbosity),
            stdout_writer: Box::new(std::io::stdout()),
            stderr_writer: Box::new(std::io::stderr()),
        }
    }

    /// Create `UserOutput` for testing with custom writers
    ///
    /// This constructor allows injecting custom writers for testing,
    /// enabling output capture and assertion.
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
        Self {
            verbosity_filter: VerbosityFilter::new(verbosity),
            stdout_writer,
            stderr_writer,
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.progress("Destroying environment...");
    /// // Output to stderr: ⏳ Destroying environment...
    /// ```
    pub fn progress(&mut self, message: &str) {
        if self.verbosity_filter.should_show_progress() {
            writeln!(self.stderr_writer, "⏳ {message}").ok();
        }
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
        if self.verbosity_filter.should_show_success() {
            writeln!(self.stderr_writer, "✅ {message}").ok();
        }
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
        if self.verbosity_filter.should_show_warnings() {
            writeln!(self.stderr_writer, "⚠️  {message}").ok();
        }
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
        if self.verbosity_filter.should_show_errors() {
            writeln!(self.stderr_writer, "❌ {message}").ok();
        }
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
        writeln!(self.stdout_writer, "{message}").ok();
    }

    /// Output structured data to stdout (JSON, etc.)
    ///
    /// For machine-readable output that should be piped or processed.
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
        writeln!(self.stdout_writer, "{data}").ok();
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
            writeln!(self.stderr_writer).ok();
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
        if self.verbosity_filter.should_show_steps() {
            writeln!(self.stderr_writer, "{title}").ok();
            for (idx, step) in steps.iter().enumerate() {
                writeln!(self.stderr_writer, "{}. {}", idx + 1, step).ok();
            }
        }
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
        if self.verbosity_filter.should_show_info_blocks() {
            writeln!(self.stderr_writer, "{title}").ok();
            for line in lines {
                writeln!(self.stderr_writer, "{line}").ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test support module for `UserOutput` testing
    ///
    /// Provides simplified test infrastructure for capturing and asserting on output.
    mod test_support {
        use super::*;
        use std::sync::{Arc, Mutex};

        /// Writer implementation for tests that writes to a shared buffer
        ///
        /// Uses `Arc<Mutex<Vec<u8>>>` to satisfy the `Send + Sync` requirements
        /// of the `UserOutput::with_writers` method.
        pub(super) struct TestWriter {
            buffer: Arc<Mutex<Vec<u8>>>,
        }

        impl TestWriter {
            /// Create a new `TestWriter` with a shared buffer
            pub(super) fn new(buffer: Arc<Mutex<Vec<u8>>>) -> Self {
                Self { buffer }
            }
        }

        impl Write for TestWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.buffer.lock().unwrap().write(buf)
            }

            fn flush(&mut self) -> std::io::Result<()> {
                self.buffer.lock().unwrap().flush()
            }
        }

        /// Test wrapper for `UserOutput` that simplifies test code
        ///
        /// Provides easy access to captured stdout and stderr content,
        /// eliminating the need for manual buffer management in tests.
        ///
        /// # Examples
        ///
        /// ```rust,ignore
        /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
        ///
        /// test_output.output.progress("Processing...");
        ///
        /// assert_eq!(test_output.stderr(), "⏳ Processing...\n");
        /// assert_eq!(test_output.stdout(), "");
        /// ```
        pub(super) struct TestUserOutput {
            /// The `UserOutput` instance being tested
            pub(super) output: UserOutput,
            stdout_buffer: Arc<Mutex<Vec<u8>>>,
            stderr_buffer: Arc<Mutex<Vec<u8>>>,
        }

        impl TestUserOutput {
            /// Create a new test output with the specified verbosity level
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let test_output = TestUserOutput::new(VerbosityLevel::Normal);
            /// ```
            pub(super) fn new(verbosity: VerbosityLevel) -> Self {
                let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
                let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

                let stdout_writer = Box::new(TestWriter::new(Arc::clone(&stdout_buffer)));
                let stderr_writer = Box::new(TestWriter::new(Arc::clone(&stderr_buffer)));

                let output = UserOutput::with_writers(verbosity, stdout_writer, stderr_writer);

                Self {
                    output,
                    stdout_buffer,
                    stderr_buffer,
                }
            }

            /// Get the content written to stdout as a String
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            /// test_output.output.result("Done");
            /// assert_eq!(test_output.stdout(), "Done\n");
            /// ```
            pub(super) fn stdout(&self) -> String {
                String::from_utf8(self.stdout_buffer.lock().unwrap().clone())
                    .expect("stdout should be valid UTF-8")
            }

            /// Get the content written to stderr as a String
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            /// test_output.output.progress("Working...");
            /// assert_eq!(test_output.stderr(), "⏳ Working...\n");
            /// ```
            pub(super) fn stderr(&self) -> String {
                String::from_utf8(self.stderr_buffer.lock().unwrap().clone())
                    .expect("stderr should be valid UTF-8")
            }

            /// Get both stdout and stderr content as a tuple
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            /// test_output.output.progress("Working...");
            /// test_output.output.result("Done");
            /// let (stdout, stderr) = test_output.output_pair();
            /// assert_eq!(stdout, "Done\n");
            /// assert_eq!(stderr, "⏳ Working...\n");
            /// ```
            #[allow(dead_code)]
            pub(super) fn output_pair(&self) -> (String, String) {
                (self.stdout(), self.stderr())
            }

            /// Clear all captured output
            ///
            /// Useful when testing multiple operations in the same test.
            ///
            /// # Examples
            ///
            /// ```rust,ignore
            /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            /// test_output.output.progress("Step 1");
            /// test_output.clear();
            /// test_output.output.progress("Step 2");
            /// assert_eq!(test_output.stderr(), "⏳ Step 2\n");
            /// ```
            #[allow(dead_code)]
            pub(super) fn clear(&mut self) {
                self.stdout_buffer.lock().unwrap().clear();
                self.stderr_buffer.lock().unwrap().clear();
            }
        }
    }

    #[test]
    fn it_should_write_progress_messages_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.progress("Testing progress message");

        // Verify message went to stderr
        assert_eq!(test_output.stderr(), "⏳ Testing progress message\n");

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_write_success_messages_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.success("Testing success message");

        // Verify message went to stderr
        assert_eq!(test_output.stderr(), "✅ Testing success message\n");

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_write_warning_messages_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.warn("Testing warning message");

        // Verify message went to stderr
        assert_eq!(test_output.stderr(), "⚠️  Testing warning message\n");

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_write_error_messages_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.error("Testing error message");

        // Verify message went to stderr
        assert_eq!(test_output.stderr(), "❌ Testing error message\n");

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_write_results_to_stdout() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.result("Test result data");

        // Verify message went to stdout
        assert_eq!(test_output.stdout(), "Test result data\n");

        // Verify stderr is empty
        assert_eq!(test_output.stderr(), "");
    }

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
    fn it_should_respect_verbosity_levels_for_progress() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output.output.progress("This should not appear");

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_respect_verbosity_levels_for_success() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output.output.success("This should not appear");

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_respect_verbosity_levels_for_warn() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output.output.warn("This should not appear");

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_always_show_errors_regardless_of_verbosity() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output.output.error("Critical error message");

        // Verify error appears even at Quiet level
        assert_eq!(test_output.stderr(), "❌ Critical error message\n");
    }

    #[test]
    fn it_should_use_normal_as_default_verbosity() {
        let default = VerbosityLevel::default();
        assert_eq!(default, VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_order_verbosity_levels_correctly() {
        assert!(VerbosityLevel::Quiet < VerbosityLevel::Normal);
        assert!(VerbosityLevel::Normal < VerbosityLevel::Verbose);
        assert!(VerbosityLevel::Verbose < VerbosityLevel::VeryVerbose);
        assert!(VerbosityLevel::VeryVerbose < VerbosityLevel::Debug);
    }

    #[test]
    fn it_should_support_equality_comparison() {
        assert_eq!(VerbosityLevel::Normal, VerbosityLevel::Normal);
        assert_ne!(VerbosityLevel::Normal, VerbosityLevel::Verbose);
    }

    #[test]
    fn it_should_support_ordering_comparison() {
        let normal = VerbosityLevel::Normal;
        assert!(normal >= VerbosityLevel::Quiet);
        assert!(normal >= VerbosityLevel::Normal);
        assert!(normal < VerbosityLevel::Verbose);
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
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output.output.steps("Next steps:", &["Step 1", "Step 2"]);

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
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output.output.info_block("Info:", &["Line 1", "Line 2"]);

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    // VerbosityFilter tests
    mod verbosity_filter {
        use super::super::*;

        #[test]
        fn it_should_show_progress_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_progress());
        }

        #[test]
        fn it_should_not_show_progress_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_progress());
        }

        #[test]
        fn it_should_show_progress_at_verbose_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Verbose);
            assert!(filter.should_show_progress());
        }

        #[test]
        fn it_should_always_show_errors_regardless_of_level() {
            assert!(VerbosityFilter::new(VerbosityLevel::Quiet).should_show_errors());
            assert!(VerbosityFilter::new(VerbosityLevel::Normal).should_show_errors());
            assert!(VerbosityFilter::new(VerbosityLevel::Verbose).should_show_errors());
            assert!(VerbosityFilter::new(VerbosityLevel::VeryVerbose).should_show_errors());
            assert!(VerbosityFilter::new(VerbosityLevel::Debug).should_show_errors());
        }

        #[test]
        fn it_should_show_success_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_success());
        }

        #[test]
        fn it_should_not_show_success_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_success());
        }

        #[test]
        fn it_should_show_warnings_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_warnings());
        }

        #[test]
        fn it_should_not_show_warnings_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_warnings());
        }

        #[test]
        fn it_should_show_blank_lines_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_blank_lines());
        }

        #[test]
        fn it_should_not_show_blank_lines_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_blank_lines());
        }

        #[test]
        fn it_should_show_steps_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_steps());
        }

        #[test]
        fn it_should_not_show_steps_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_steps());
        }

        #[test]
        fn it_should_show_info_blocks_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_info_blocks());
        }

        #[test]
        fn it_should_not_show_info_blocks_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_info_blocks());
        }

        #[test]
        fn it_should_show_when_level_meets_requirement() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show(VerbosityLevel::Quiet));
            assert!(filter.should_show(VerbosityLevel::Normal));
            assert!(!filter.should_show(VerbosityLevel::Verbose));
        }

        #[test]
        fn it_should_handle_all_verbosity_levels_in_should_show() {
            let quiet_filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(quiet_filter.should_show(VerbosityLevel::Quiet));
            assert!(!quiet_filter.should_show(VerbosityLevel::Normal));

            let debug_filter = VerbosityFilter::new(VerbosityLevel::Debug);
            assert!(debug_filter.should_show(VerbosityLevel::Quiet));
            assert!(debug_filter.should_show(VerbosityLevel::Normal));
            assert!(debug_filter.should_show(VerbosityLevel::Verbose));
            assert!(debug_filter.should_show(VerbosityLevel::VeryVerbose));
            assert!(debug_filter.should_show(VerbosityLevel::Debug));
        }
    }
}
