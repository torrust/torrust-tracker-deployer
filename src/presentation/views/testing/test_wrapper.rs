//! Test wrapper providing convenient access to wrapped `UserOutput` and output buffers
//!
//! Provides `TestOutputWrapper` that makes testing easier by providing direct access
//! to wrapped output along with convenient methods for checking output content.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::{Mutex, ReentrantMutex};

use crate::presentation::views::{OutputMessage, UserOutput};

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
/// test_output.progress("Working...");
/// assert_eq!(test_output.stderr(), "⏳ Working...\n");
/// ```
pub struct TestOutputWrapper<T> {
    /// The wrapped `UserOutput` instance
    pub output: Arc<T>,
    stdout_buffer: Arc<Mutex<Vec<u8>>>,
    stderr_buffer: Arc<Mutex<Vec<u8>>>,
}

impl<T> TestOutputWrapper<T> {
    /// Create a new `TestOutputWrapper`
    pub fn new(
        output: Arc<T>,
        stdout_buffer: Arc<Mutex<Vec<u8>>>,
        stderr_buffer: Arc<Mutex<Vec<u8>>>,
    ) -> Self {
        Self {
            output,
            stdout_buffer,
            stderr_buffer,
        }
    }

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
