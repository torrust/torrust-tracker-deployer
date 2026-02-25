//! Standard output sink implementation

use std::io::Write;

use super::super::{Channel, OutputMessage, OutputSink};
use super::writers::{StderrWriter, StdoutWriter};

// ============================================================================
// Output Sink Implementations
// ============================================================================

/// Standard sink writing to stdout/stderr
///
/// This is the default sink that maintains backward compatibility with the
/// existing console output behavior. It routes messages to stdout or stderr
/// based on the message's channel.
///
/// # Type Safety
///
/// Uses `StdoutWriter` and `StderrWriter` wrappers for compile-time channel safety.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::cli::views::StandardSink;
///
/// let sink = StandardSink::new(
///     Box::new(std::io::stdout()),
///     Box::new(std::io::stderr())
/// );
/// ```
pub struct StandardSink {
    stdout: StdoutWriter,
    stderr: StderrWriter,
}

impl StandardSink {
    /// Create a new standard sink with the given writers
    ///
    /// This is useful for testing or when you need custom writers.
    #[must_use]
    pub fn new(stdout: Box<dyn Write + Send + Sync>, stderr: Box<dyn Write + Send + Sync>) -> Self {
        Self {
            stdout: StdoutWriter::new(stdout),
            stderr: StderrWriter::new(stderr),
        }
    }

    /// Create a standard sink using default stdout/stderr
    ///
    /// This is the default console sink that writes to the standard
    /// output and error streams.
    #[must_use]
    pub fn default_console() -> Self {
        Self::new(Box::new(std::io::stdout()), Box::new(std::io::stderr()))
    }
}

impl OutputSink for StandardSink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        match message.channel() {
            Channel::Stdout => {
                self.stdout.write_line(formatted);
            }
            Channel::Stderr => {
                self.stderr.write_line(formatted);
            }
        }
    }
}
