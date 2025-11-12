//! Test writer implementation for capturing output in tests
//!
//! Provides `TestWriter` that writes to a shared buffer, satisfying
//! the `Send + Sync` requirements of `UserOutput::with_writers`.

use std::io::Write;
use std::sync::Arc;

use parking_lot::Mutex;

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
