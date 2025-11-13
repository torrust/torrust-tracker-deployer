//! Type-safe writer wrappers for output channels

use std::io::Write;

/// Stdout writer wrapper for type safety
///
/// This newtype wrapper ensures that stdout-specific operations
/// can only be performed on stdout writers, preventing accidental
/// channel confusion at compile time.
///
/// The wrapper provides a zero-cost abstraction - it has the same
/// memory layout and performance characteristics as the wrapped type,
/// but provides compile-time type safety.
pub(in crate::presentation::views) struct StdoutWriter(Box<dyn Write + Send + Sync>);

impl StdoutWriter {
    /// Create a new stdout writer wrapper
    pub(in crate::presentation::views) fn new(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self(writer)
    }

    /// Write a line to stdout
    ///
    /// Writes the given message to the stdout channel.
    /// The message should include any necessary newline characters.
    /// Errors are silently ignored as output operations are best-effort.
    pub(in crate::presentation::views) fn write_line(&mut self, message: &str) {
        write!(self.0, "{message}").ok();
    }

    /// Write with a newline to stdout
    ///
    /// Writes the given message followed by a newline to the stdout channel.
    /// Errors are silently ignored as output operations are best-effort.
    #[allow(dead_code)]
    pub(in crate::presentation::views) fn writeln(&mut self, message: &str) {
        writeln!(self.0, "{message}").ok();
    }
}

/// Stderr writer wrapper for type safety
///
/// This newtype wrapper ensures that stderr-specific operations
/// can only be performed on stderr writers, preventing accidental
/// channel confusion at compile time.
///
/// The wrapper provides a zero-cost abstraction - it has the same
/// memory layout and performance characteristics as the wrapped type,
/// but provides compile-time type safety.
pub(in crate::presentation::views) struct StderrWriter(Box<dyn Write + Send + Sync>);

impl StderrWriter {
    /// Create a new stderr writer wrapper
    pub(in crate::presentation::views) fn new(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self(writer)
    }

    /// Write a line to stderr
    ///
    /// Writes the given message to the stderr channel.
    /// The message should include any necessary newline characters.
    /// Errors are silently ignored as output operations are best-effort.
    pub(in crate::presentation::views) fn write_line(&mut self, message: &str) {
        write!(self.0, "{message}").ok();
    }

    /// Write with a newline to stderr
    ///
    /// Writes the given message followed by a newline to the stderr channel.
    /// Errors are silently ignored as output operations are best-effort.
    #[allow(dead_code)]
    pub(in crate::presentation::views) fn writeln(&mut self, message: &str) {
        writeln!(self.0, "{message}").ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::views::testing;
    use parking_lot::Mutex;
    use std::sync::Arc;

    #[test]
    fn stdout_writer_should_wrap_writer() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(testing::TestWriter::new(Arc::clone(&buffer)));

        let mut stdout = StdoutWriter::new(writer);
        stdout.write_line("Test output");

        let output = String::from_utf8(buffer.lock().clone()).unwrap();
        assert_eq!(output, "Test output");
    }

    #[test]
    fn stderr_writer_should_wrap_writer() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(testing::TestWriter::new(Arc::clone(&buffer)));

        let mut stderr = StderrWriter::new(writer);
        stderr.write_line("Test error");

        let output = String::from_utf8(buffer.lock().clone()).unwrap();
        assert_eq!(output, "Test error");
    }

    #[test]
    fn stdout_writer_should_write_multiple_lines() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(testing::TestWriter::new(Arc::clone(&buffer)));

        let mut stdout = StdoutWriter::new(writer);
        stdout.write_line("Line 1\n");
        stdout.write_line("Line 2\n");

        let output = String::from_utf8(buffer.lock().clone()).unwrap();
        assert_eq!(output, "Line 1\nLine 2\n");
    }

    #[test]
    fn stderr_writer_should_write_multiple_lines() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(testing::TestWriter::new(Arc::clone(&buffer)));

        let mut stderr = StderrWriter::new(writer);
        stderr.write_line("Error 1\n");
        stderr.write_line("Error 2\n");

        let output = String::from_utf8(buffer.lock().clone()).unwrap();
        assert_eq!(output, "Error 1\nError 2\n");
    }

    #[test]
    fn stdout_writer_writeln_adds_newline() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(testing::TestWriter::new(Arc::clone(&buffer)));

        let mut stdout = StdoutWriter::new(writer);
        stdout.writeln("Test");

        let output = String::from_utf8(buffer.lock().clone()).unwrap();
        assert_eq!(output, "Test\n");
    }

    #[test]
    fn stderr_writer_writeln_adds_newline() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(testing::TestWriter::new(Arc::clone(&buffer)));

        let mut stderr = StderrWriter::new(writer);
        stderr.writeln("Error");

        let output = String::from_utf8(buffer.lock().clone()).unwrap();
        assert_eq!(output, "Error\n");
    }
}
