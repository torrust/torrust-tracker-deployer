//! Test wrapper providing convenient access to wrapped `UserOutput` and output buffers
//!
//! Provides `TestOutputWrapper` that makes testing easier by providing direct access
//! to wrapped output along with convenient methods for checking output content.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::{Mutex, ReentrantMutex};

use crate::presentation::views::UserOutput;

/// Test wrapper that provides convenient access to wrapped `UserOutput` and output buffers
///
/// This struct makes testing easier by providing direct access to the wrapped output
/// along with convenient methods for checking what was written to stdout and stderr.
/// It supports both the legacy `Mutex` and new `ReentrantMutex` patterns.
///
/// # Examples
///
/// ```rust,ignore
/// let test_user_output = TestUserOutput::new(VerbosityLevel::Normal);
/// let stdout_buf = Arc::clone(&test_user_output.stdout_buffer);
/// let stderr_buf = Arc::clone(&test_user_output.stderr_buffer);
/// let test_output = TestOutputWrapper::new(
///     Arc::new(ReentrantMutex::new(RefCell::new(test_user_output.output))),
///     stdout_buf,
///     stderr_buf,
/// );
/// test_output.steps("Working...", &["Step 1"]);
/// assert_eq!(test_output.stderr(), "Working...\n  1. Step 1\n");
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
}

impl TestOutputWrapper<ReentrantMutex<RefCell<UserOutput>>> {
    /// Execute a function with the locked `UserOutput` (mutable access)
    pub fn with_output_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut UserOutput) -> R,
    {
        let guard = self.output.lock();
        let mut user_output = guard.borrow_mut();
        f(&mut user_output)
    }

    // Essential convenience methods that are actively used

    /// Call steps method on the wrapped `UserOutput`
    pub fn steps(&self, title: &str, steps: &[&str]) {
        self.with_output_mut(|output| output.steps(title, steps));
    }

    /// Call `info_block` method on the wrapped `UserOutput`
    pub fn info_block(&self, title: &str, lines: &[&str]) {
        self.with_output_mut(|output| output.info_block(title, lines));
    }
}
