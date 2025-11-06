//! File output sink implementation

use std::io::Write;

use super::super::{OutputMessage, OutputSink};

// ============================================================================
// Example Sink Implementations
// ============================================================================

/// Example: File sink that writes all output to a file
///
/// This is an example implementation showing how to create a custom sink
/// that writes to a file. In production, you might want to add buffering,
/// rotation, or other features.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::{FileSink, UserOutput, VerbosityLevel, CompositeSink, StandardSink};
///
/// // Write to both console and file
/// let composite = CompositeSink::new(vec![
///     Box::new(StandardSink::default_console()),
///     Box::new(FileSink::new("output.log").unwrap()),
/// ]);
/// let mut output = UserOutput::with_sink(VerbosityLevel::Normal, Box::new(composite));
/// ```
pub struct FileSink {
    file: std::fs::File,
}

impl FileSink {
    /// Create a new file sink
    ///
    /// Opens or creates the file at the given path in append mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or created.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::FileSink;
    ///
    /// let sink = FileSink::new("output.log")?;
    /// ```
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }
}

impl OutputSink for FileSink {
    fn write_message(&mut self, _message: &dyn OutputMessage, formatted: &str) {
        writeln!(self.file, "{formatted}").ok();
    }
}
