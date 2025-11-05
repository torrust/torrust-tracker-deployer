//! Test support infrastructure for `UserOutput` testing
//!
//! Provides simplified test infrastructure for capturing and asserting on output
//! in tests across the codebase.

use std::io::Write;
use std::sync::{Arc, Mutex};

use super::*;

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

        let output =
            UserOutput::with_theme_and_writers(verbosity, theme, stdout_writer, stderr_writer);

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
