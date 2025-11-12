//! Test support infrastructure for `UserOutput` testing
//!
//! Provides simplified test infrastructure for capturing and asserting on output
//! in tests across the codebase.

use std::cell::RefCell;
use std::io::Write;
use std::sync::Arc;

use parking_lot::{Mutex, ReentrantMutex};

use super::{OutputMessage, Theme, UserOutput, VerbosityLevel};

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
        self.buffer.lock().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.lock().flush()
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
/// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
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
    /// let test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
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
    /// let output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
    /// // Use with APIs that expect Arc<Mutex<UserOutput>>
    /// ```
    #[must_use]
    pub fn wrapped(verbosity: VerbosityLevel) -> Arc<Mutex<UserOutput>> {
        let test_output = Self::new(verbosity);
        Arc::new(Mutex::new(test_output.output))
    }

    /// Create wrapped test output with silent verbosity for clean test output
    ///
    /// This is a convenience method specifically for tests to avoid user-facing
    /// messages appearing in test output.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let output = TestUserOutput::wrapped_silent();
    /// // Use with APIs that expect Arc<Mutex<UserOutput>> - no user output will appear
    /// ```
    #[must_use]
    pub fn wrapped_silent() -> Arc<Mutex<UserOutput>> {
        Self::wrapped(VerbosityLevel::Silent)
    }

    /// Wrap an existing `UserOutput` in an `Arc<Mutex<>>` for use with APIs that require it
    ///
    /// Returns a tuple of (`Arc<Mutex<UserOutput>>`, stdout buffer, stderr buffer) for tests
    /// that need access to both the wrapped output and the buffers.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
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

    /// Create wrapped `UserOutput` with `ReentrantMutex` for the new architecture
    ///
    /// Returns a tuple containing the wrapped `UserOutput` and its output buffers.
    /// This method supports the new `ReentrantMutex<RefCell<UserOutput>>` pattern.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
    /// let (wrapped_output, stdout_buf, stderr_buf) = test_output.into_reentrant_wrapped();
    /// // Use wrapped_output with functions that expect Arc<ReentrantMutex<RefCell<UserOutput>>>
    /// // Use buffers to assert on output content
    /// ```
    #[must_use]
    #[allow(clippy::type_complexity)]
    pub fn into_reentrant_wrapped(
        self,
    ) -> (
        Arc<ReentrantMutex<RefCell<UserOutput>>>,
        Arc<Mutex<Vec<u8>>>,
        Arc<Mutex<Vec<u8>>>,
    ) {
        let stdout_buf = Arc::clone(&self.stdout_buffer);
        let stderr_buf = Arc::clone(&self.stderr_buffer);
        (
            Arc::new(ReentrantMutex::new(RefCell::new(self.output))),
            stdout_buf,
            stderr_buf,
        )
    }

    /// Create wrapped `UserOutput` with `ReentrantMutex` for convenient testing
    ///
    /// Returns a convenient test wrapper that provides direct access to the wrapped output
    /// and methods for checking output content. This is the recommended method for tests
    /// that need to both use the wrapped output and check what was written.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_test_wrapper();
    /// test_output.output.progress("Working...");
    /// assert_eq!(test_output.stderr(), "⏳ Working...\n");
    /// ```
    #[must_use]
    pub fn into_reentrant_test_wrapper(
        self,
    ) -> TestOutputWrapper<ReentrantMutex<RefCell<UserOutput>>> {
        let stdout_buf = Arc::clone(&self.stdout_buffer);
        let stderr_buf = Arc::clone(&self.stderr_buffer);
        TestOutputWrapper {
            output: Arc::new(ReentrantMutex::new(RefCell::new(self.output))),
            stdout_buffer: stdout_buf,
            stderr_buffer: stderr_buf,
        }
    }

    /// Get the content written to stdout as a String
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
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
        String::from_utf8(self.stdout_buffer.lock().clone()).expect("stdout should be valid UTF-8")
    }

    /// Get the content written to stderr as a String
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
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
        String::from_utf8(self.stderr_buffer.lock().clone()).expect("stderr should be valid UTF-8")
    }

    /// Get both stdout and stderr content as a tuple
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
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
    /// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
    /// test_output.output.progress("Step 1");
    /// test_output.clear();
    /// test_output.output.progress("Step 2");
    /// assert_eq!(test_output.stderr(), "⏳ Step 2\n");
    /// ```
    ///
    /// # Panics
    ///
    /// Clear both stdout and stderr buffers
    ///
    /// Resets the captured content for both output streams. Useful for
    /// running multiple test scenarios with the same wrapper.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.stdout_buffer.lock().clear();
        self.stderr_buffer.lock().clear();
    }
}

/// Test wrapper that provides convenient access to wrapped `UserOutput` and output buffers
///
/// This struct makes testing easier by providing direct access to the wrapped output
/// along with convenient methods for checking what was written to stdout and stderr.
/// It supports both the legacy `Mutex` and new `ReentrantMutex` patterns.
///
/// # Examples
///
/// ```rust,ignore
/// let mut test_output = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_test_wrapper();
/// test_output.output.progress("Working...");
/// assert_eq!(test_output.stderr(), "⏳ Working...\n");
/// ```
pub struct TestOutputWrapper<T> {
    /// The wrapped `UserOutput` instance
    pub output: Arc<T>,
    stdout_buffer: Arc<Mutex<Vec<u8>>>,
    stderr_buffer: Arc<Mutex<Vec<u8>>>,
}

impl<T> TestOutputWrapper<T> {
    /// Get the content written to stdout as a String
    ///
    /// # Panics
    ///
    /// Panics if the stdout buffer contains invalid UTF-8.
    #[must_use]
    pub fn stdout(&self) -> String {
        String::from_utf8(self.stdout_buffer.lock().clone()).expect("stdout should be valid UTF-8")
    }

    /// Get the content written to stderr as a String
    ///
    /// # Panics
    ///
    /// Panics if the stderr buffer contains invalid UTF-8.
    #[must_use]
    pub fn stderr(&self) -> String {
        String::from_utf8(self.stderr_buffer.lock().clone()).expect("stderr should be valid UTF-8")
    }

    /// Get both stdout and stderr content as a tuple
    #[must_use]
    pub fn output_pair(&self) -> (String, String) {
        (self.stdout(), self.stderr())
    }

    /// Clear all captured output
    pub fn clear(&self) {
        self.stdout_buffer.lock().clear();
        self.stderr_buffer.lock().clear();
    }
}

impl TestOutputWrapper<ReentrantMutex<RefCell<UserOutput>>> {
    /// Execute a function with the locked `UserOutput`
    pub fn with_output<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&UserOutput) -> R,
    {
        let guard = self.output.lock();
        let user_output = guard.borrow();
        f(&user_output)
    }

    /// Execute a function with the locked `UserOutput` (mutable access)
    pub fn with_output_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut UserOutput) -> R,
    {
        let guard = self.output.lock();
        let mut user_output = guard.borrow_mut();
        f(&mut user_output)
    }

    // Convenience methods for direct calls

    /// Call progress method on the wrapped `UserOutput`
    pub fn progress(&self, message: &str) {
        self.with_output_mut(|output| output.progress(message));
    }

    /// Call success method on the wrapped `UserOutput`
    pub fn success(&self, message: &str) {
        self.with_output_mut(|output| output.success(message));
    }

    /// Call warn method on the wrapped `UserOutput`
    pub fn warn(&self, message: &str) {
        self.with_output_mut(|output| output.warn(message));
    }

    /// Call error method on the wrapped `UserOutput`
    pub fn error(&self, message: &str) {
        self.with_output_mut(|output| output.error(message));
    }

    /// Call result method on the wrapped `UserOutput`
    pub fn result(&self, data: &str) {
        self.with_output_mut(|output| output.result(data));
    }

    /// Call data method on the wrapped `UserOutput`
    pub fn data(&self, data: &str) {
        self.with_output_mut(|output| output.data(data));
    }

    /// Call `blank_line` method on the wrapped `UserOutput`
    pub fn blank_line(&self) {
        self.with_output_mut(UserOutput::blank_line);
    }

    /// Call steps method on the wrapped `UserOutput`
    pub fn steps(&self, title: &str, steps: &[&str]) {
        self.with_output_mut(|output| output.steps(title, steps));
    }

    /// Call `info_block` method on the wrapped `UserOutput`
    pub fn info_block(&self, title: &str, lines: &[&str]) {
        self.with_output_mut(|output| output.info_block(title, lines));
    }

    /// Call write method on the wrapped `UserOutput`
    pub fn write(&self, message: &dyn OutputMessage) {
        self.with_output_mut(|output| output.write(message));
    }

    /// Call flush method on the wrapped `UserOutput`
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying flush operation fails.
    pub fn flush(&self) -> std::io::Result<()> {
        self.with_output_mut(UserOutput::flush)
    }
}
