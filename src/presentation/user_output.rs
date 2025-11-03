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

/// Output theme controlling symbols and formatting
///
/// A theme defines the visual appearance of user-facing messages through
/// configurable symbols. Themes enable consistent styling across all output
/// and support different environments (terminals, CI/CD, accessibility needs).
///
/// # Predefined Themes
///
/// - **Emoji** (default): Unicode emoji symbols for interactive terminals
/// - **Plain**: Text labels like `[INFO]`, `[OK]` for CI/CD environments
/// - **ASCII**: Basic ASCII characters for limited terminal support
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
///
/// // Use emoji theme (default)
/// let theme = Theme::emoji();
/// assert_eq!(theme.progress_symbol(), "⏳");
///
/// // Use plain text theme for CI/CD
/// let theme = Theme::plain();
/// assert_eq!(theme.success_symbol(), "[OK]");
///
/// // Use ASCII theme for limited terminals
/// let theme = Theme::ascii();
/// assert_eq!(theme.error_symbol(), "[x]");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    progress_symbol: String,
    success_symbol: String,
    warning_symbol: String,
    error_symbol: String,
}

impl Theme {
    /// Create emoji theme with Unicode symbols (default)
    ///
    /// Best for interactive terminals with good Unicode support.
    /// Uses emoji characters that are visually distinctive and widely supported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
    ///
    /// let theme = Theme::emoji();
    /// assert_eq!(theme.progress_symbol(), "⏳");
    /// assert_eq!(theme.success_symbol(), "✅");
    /// assert_eq!(theme.warning_symbol(), "⚠️");
    /// assert_eq!(theme.error_symbol(), "❌");
    /// ```
    #[must_use]
    pub fn emoji() -> Self {
        Self {
            progress_symbol: "⏳".to_string(),
            success_symbol: "✅".to_string(),
            warning_symbol: "⚠️".to_string(),
            error_symbol: "❌".to_string(),
        }
    }

    /// Create plain text theme for CI/CD environments
    ///
    /// Uses text labels like `[INFO]`, `[OK]`, `[WARN]`, `[ERROR]` that work
    /// in any environment without Unicode support. Ideal for CI/CD pipelines
    /// and log aggregation systems.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
    ///
    /// let theme = Theme::plain();
    /// assert_eq!(theme.progress_symbol(), "[INFO]");
    /// assert_eq!(theme.success_symbol(), "[OK]");
    /// assert_eq!(theme.warning_symbol(), "[WARN]");
    /// assert_eq!(theme.error_symbol(), "[ERROR]");
    /// ```
    #[must_use]
    pub fn plain() -> Self {
        Self {
            progress_symbol: "[INFO]".to_string(),
            success_symbol: "[OK]".to_string(),
            warning_symbol: "[WARN]".to_string(),
            error_symbol: "[ERROR]".to_string(),
        }
    }

    /// Create ASCII-only theme using basic characters
    ///
    /// Uses simple ASCII characters that work on any terminal.
    /// Good for environments with limited character set support or
    /// when maximum compatibility is required.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
    ///
    /// let theme = Theme::ascii();
    /// assert_eq!(theme.progress_symbol(), "=>");
    /// assert_eq!(theme.success_symbol(), "[+]");
    /// assert_eq!(theme.warning_symbol(), "[!]");
    /// assert_eq!(theme.error_symbol(), "[x]");
    /// ```
    #[must_use]
    pub fn ascii() -> Self {
        Self {
            progress_symbol: "=>".to_string(),
            success_symbol: "[+]".to_string(),
            warning_symbol: "[!]".to_string(),
            error_symbol: "[x]".to_string(),
        }
    }

    /// Get the progress symbol for this theme
    #[must_use]
    pub fn progress_symbol(&self) -> &str {
        &self.progress_symbol
    }

    /// Get the success symbol for this theme
    #[must_use]
    pub fn success_symbol(&self) -> &str {
        &self.success_symbol
    }

    /// Get the warning symbol for this theme
    #[must_use]
    pub fn warning_symbol(&self) -> &str {
        &self.warning_symbol
    }

    /// Get the error symbol for this theme
    #[must_use]
    pub fn error_symbol(&self) -> &str {
        &self.error_symbol
    }
}

impl Default for Theme {
    /// Create the default theme (emoji)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
    ///
    /// let theme = Theme::default();
    /// assert_eq!(theme.progress_symbol(), "⏳");
    /// ```
    fn default() -> Self {
        Self::emoji()
    }
}

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
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    /// Create new `UserOutput` with default stdout/stderr channels and emoji theme
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
        Self::with_theme_and_writers(
            verbosity,
            theme,
            Box::new(std::io::stdout()),
            Box::new(std::io::stderr()),
        )
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
            stdout_writer,
            stderr_writer,
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
            writeln!(self.stderr_writer, "{} {message}", self.theme.progress_symbol()).ok();
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
            writeln!(self.stderr_writer, "{} {message}", self.theme.success_symbol()).ok();
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
            writeln!(self.stderr_writer, "{}  {message}", self.theme.warning_symbol()).ok();
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
            writeln!(self.stderr_writer, "{} {message}", self.theme.error_symbol()).ok();
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
pub mod test_support {
    //! Test support infrastructure for `UserOutput` testing
    //!
    //! Provides simplified test infrastructure for capturing and asserting on output
    //! in tests across the codebase.

    use super::*;
    use std::sync::{Arc, Mutex};

    /// Writer implementation for tests that writes to a shared buffer
    ///
    /// Uses `Arc<Mutex<Vec<u8>>>` to satisfy the `Send + Sync` requirements
    /// of the `UserOutput::with_writers` method.
    pub struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        /// Create a new `TestWriter` with a shared buffer
        pub fn new(buffer: Arc<Mutex<Vec<u8>>>) -> Self {
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
    /// use torrust_tracker_deployer_lib::presentation::user_output::test_support::TestUserOutput;
    /// use torrust_tracker_deployer_lib::presentation::user_output::VerbosityLevel;
    ///
    /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
    ///
    /// test_output.output.progress("Processing...");
    ///
    /// assert_eq!(test_output.stderr(), "⏳ Processing...\n");
    /// assert_eq!(test_output.stdout(), "");
    /// ```
    pub struct TestUserOutput {
        /// The `UserOutput` instance being tested
        pub output: UserOutput,
        stdout_buffer: Arc<Mutex<Vec<u8>>>,
        stderr_buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestUserOutput {
        /// Create a new test output with the specified verbosity level and default theme
        ///
        /// # Examples
        ///
        /// ```rust,ignore
        /// let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        /// ```
        #[must_use]
        pub fn new(verbosity: VerbosityLevel) -> Self {
            Self::with_theme(verbosity, Theme::default())
        }

        /// Create a new test output with the specified verbosity level and theme
        ///
        /// # Examples
        ///
        /// ```rust,ignore
        /// let test_output = TestUserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());
        /// ```
        #[must_use]
        pub fn with_theme(verbosity: VerbosityLevel, theme: Theme) -> Self {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let stdout_writer = Box::new(TestWriter::new(Arc::clone(&stdout_buffer)));
            let stderr_writer = Box::new(TestWriter::new(Arc::clone(&stderr_buffer)));

            let output = UserOutput::with_theme_and_writers(verbosity, theme, stdout_writer, stderr_writer);

            Self {
                output,
                stdout_buffer,
                stderr_buffer,
            }
        }

        /// Create wrapped test output for use with APIs that require `Arc<Mutex<UserOutput>>`
        ///
        /// This is a convenience method for tests that just need a wrapped output
        /// without access to the buffers.
        ///
        /// # Examples
        ///
        /// ```rust,ignore
        /// let output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        /// // Use with APIs that expect Arc<Mutex<UserOutput>>
        /// ```
        #[must_use]
        pub fn wrapped(verbosity: VerbosityLevel) -> Arc<Mutex<UserOutput>> {
            let test_output = Self::new(verbosity);
            Arc::new(Mutex::new(test_output.output))
        }

        /// Wrap an existing `UserOutput` in an `Arc<Mutex<>>` for use with APIs that require it
        ///
        /// Returns a tuple of (`Arc<Mutex<UserOutput>>`, stdout buffer, stderr buffer) for tests
        /// that need access to both the wrapped output and the buffers.
        ///
        /// # Examples
        ///
        /// ```rust,ignore
        /// let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        /// let (wrapped, stdout_buf, stderr_buf) = test_output.into_wrapped();
        /// // Use `wrapped` with APIs that expect Arc<Mutex<UserOutput>>
        /// // Use buffers to assert on output content
        /// ```
        #[must_use]
        #[allow(clippy::type_complexity)]
        pub fn into_wrapped(
            self,
        ) -> (
            Arc<Mutex<UserOutput>>,
            Arc<Mutex<Vec<u8>>>,
            Arc<Mutex<Vec<u8>>>,
        ) {
            let stdout_buf = Arc::clone(&self.stdout_buffer);
            let stderr_buf = Arc::clone(&self.stderr_buffer);
            (Arc::new(Mutex::new(self.output)), stdout_buf, stderr_buf)
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
        ///
        /// # Panics
        ///
        /// Panics if the mutex is poisoned or if the buffer contains invalid UTF-8.
        /// These conditions indicate a test bug and should never occur in practice.
        #[must_use]
        pub fn stdout(&self) -> String {
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
        ///
        /// # Panics
        ///
        /// Panics if the mutex is poisoned or if the buffer contains invalid UTF-8.
        /// These conditions indicate a test bug and should never occur in practice.
        #[must_use]
        pub fn stderr(&self) -> String {
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
        #[must_use]
        #[allow(dead_code)]
        pub fn output_pair(&self) -> (String, String) {
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
        ///
        /// # Panics
        ///
        /// Panics if the mutex is poisoned. This indicates a test bug and should
        /// never occur in practice.
        #[allow(dead_code)]
        pub fn clear(&mut self) {
            self.stdout_buffer.lock().unwrap().clear();
            self.stderr_buffer.lock().unwrap().clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Theme Tests
    // ============================================================================

    mod theme {
        use super::*;

        #[test]
        fn it_should_create_emoji_theme_with_correct_symbols() {
            let theme = Theme::emoji();

            assert_eq!(theme.progress_symbol(), "⏳");
            assert_eq!(theme.success_symbol(), "✅");
            assert_eq!(theme.warning_symbol(), "⚠️");
            assert_eq!(theme.error_symbol(), "❌");
        }

        #[test]
        fn it_should_create_plain_theme_with_text_labels() {
            let theme = Theme::plain();

            assert_eq!(theme.progress_symbol(), "[INFO]");
            assert_eq!(theme.success_symbol(), "[OK]");
            assert_eq!(theme.warning_symbol(), "[WARN]");
            assert_eq!(theme.error_symbol(), "[ERROR]");
        }

        #[test]
        fn it_should_create_ascii_theme_with_ascii_characters() {
            let theme = Theme::ascii();

            assert_eq!(theme.progress_symbol(), "=>");
            assert_eq!(theme.success_symbol(), "[+]");
            assert_eq!(theme.warning_symbol(), "[!]");
            assert_eq!(theme.error_symbol(), "[x]");
        }

        #[test]
        fn it_should_use_emoji_theme_as_default() {
            let theme = Theme::default();
            let emoji_theme = Theme::emoji();

            assert_eq!(theme, emoji_theme);
        }

        #[test]
        fn it_should_support_clone() {
            let theme = Theme::plain();
            let cloned = theme.clone();

            assert_eq!(theme, cloned);
        }

        #[test]
        fn it_should_support_equality_comparison() {
            let theme1 = Theme::emoji();
            let theme2 = Theme::emoji();
            let theme3 = Theme::plain();

            assert_eq!(theme1, theme2);
            assert_ne!(theme1, theme3);
        }

        #[test]
        fn it_should_support_debug_formatting() {
            let theme = Theme::emoji();
            let debug_output = format!("{:?}", theme);

            assert!(debug_output.contains("Theme"));
        }
    }

    // ============================================================================
    // UserOutput Tests - Basic Output
    // ============================================================================

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

        test_output
            .output
            .steps("Next steps:", &["Step 1", "Step 2"]);

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

        test_output
            .output
            .info_block("Info:", &["Line 1", "Line 2"]);

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
            let mut test_output = TestUserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());

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
            let mut test_output = TestUserOutput::with_theme(VerbosityLevel::Normal, Theme::ascii());

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
}
