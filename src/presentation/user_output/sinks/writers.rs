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
pub(in crate::presentation::user_output) struct StdoutWriter(Box<dyn Write + Send + Sync>);

impl StdoutWriter {
    /// Create a new stdout writer wrapper
    pub(in crate::presentation::user_output) fn new(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self(writer)
    }

    /// Write a line to stdout
    ///
    /// Writes the given message to the stdout channel.
    /// The message should include any necessary newline characters.
    /// Errors are silently ignored as output operations are best-effort.
    pub(in crate::presentation::user_output) fn write_line(&mut self, message: &str) {
        write!(self.0, "{message}").ok();
    }

    /// Write with a newline to stdout
    ///
    /// Writes the given message followed by a newline to the stdout channel.
    /// Errors are silently ignored as output operations are best-effort.
    #[allow(dead_code)]
    pub(in crate::presentation::user_output) fn writeln(&mut self, message: &str) {
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
pub(in crate::presentation::user_output) struct StderrWriter(Box<dyn Write + Send + Sync>);

impl StderrWriter {
    /// Create a new stderr writer wrapper
    pub(in crate::presentation::user_output) fn new(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self(writer)
    }

    /// Write a line to stderr
    ///
    /// Writes the given message to the stderr channel.
    /// The message should include any necessary newline characters.
    /// Errors are silently ignored as output operations are best-effort.
    pub(in crate::presentation::user_output) fn write_line(&mut self, message: &str) {
        write!(self.0, "{message}").ok();
    }

    /// Write with a newline to stderr
    ///
    /// Writes the given message followed by a newline to the stderr channel.
    /// Errors are silently ignored as output operations are best-effort.
    #[allow(dead_code)]
    pub(in crate::presentation::user_output) fn writeln(&mut self, message: &str) {
        writeln!(self.0, "{message}").ok();
    }
}
