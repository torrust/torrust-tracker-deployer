//! Core traits for output message handling
//!
//! This module defines the traits that enable extensibility and abstraction
//! in the user output system.

use super::{Channel, Theme, VerbosityLevel};

/// Trait for output messages that can be written to user-facing channels
///
/// This trait enables extensibility following the Open/Closed Principle.
/// Each message type encapsulates its own:
/// - Formatting logic (how it appears to users)
/// - Verbosity requirements (when it should be shown)
/// - Channel routing (stdout vs stderr)
///
/// # Design Philosophy
///
/// By implementing this trait, message types become self-contained and can be
/// added without modifying the `UserOutput` struct. This makes the system
/// extensible - new message types can be defined in external modules.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::cli::views::{OutputMessage, Theme, VerbosityLevel, Channel};
///
/// struct CustomMessage {
///     text: String,
/// }
///
/// impl OutputMessage for CustomMessage {
///     fn format(&self, theme: &Theme) -> String {
///         format!("🎉 {}", self.text)
///     }
///
///     fn required_verbosity(&self) -> VerbosityLevel {
///         VerbosityLevel::Normal
///     }
///
///     fn channel(&self) -> Channel {
///         Channel::Stderr
///     }
///
///     fn type_name(&self) -> &'static str {
///         "CustomMessage"
///     }
/// }
/// ```
pub trait OutputMessage {
    /// Format this message using the given theme
    ///
    /// This method defines how the message appears to users. It should
    /// incorporate theme symbols and any necessary formatting.
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme providing symbols for formatting
    ///
    /// # Returns
    ///
    /// A formatted string ready for display to users
    fn format(&self, theme: &Theme) -> String;

    /// Get the minimum verbosity level required to show this message
    ///
    /// Messages are only displayed if the current verbosity level is
    /// greater than or equal to the required level.
    ///
    /// # Returns
    ///
    /// The minimum verbosity level needed to display this message
    fn required_verbosity(&self) -> VerbosityLevel;

    /// Get the output channel for this message
    ///
    /// Determines whether the message goes to stdout or stderr following
    /// Unix conventions.
    ///
    /// # Returns
    ///
    /// The channel (Stdout or Stderr) where this message should be written
    fn channel(&self) -> Channel;

    /// Get the type name of this message
    ///
    /// Returns a human-readable type identifier for this message type.
    /// This is primarily used by formatter overrides (e.g., JSON formatter)
    /// to include type information in the output.
    ///
    /// # Returns
    ///
    /// A static string representing the message type name
    fn type_name(&self) -> &'static str;
}

/// Optional trait for post-processing message output
///
/// This allows transforming the standard message format without
/// modifying individual message types. Use sparingly - prefer
/// extending the message trait or using themes for most cases.
///
/// # When to Use
///
/// - **Machine-readable formats**: JSON, XML, structured logs
/// - **Additional decoration**: ANSI colors, markup codes
/// - **Output wrapping**: Adding metadata, timestamps, process info
///
/// # When NOT to Use
///
/// - **Symbol changes**: Use `Theme` instead
/// - **New message types**: Implement `OutputMessage` trait instead
/// - **Channel routing changes**: Define in message type's `channel()` method
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::cli::views::{FormatterOverride, OutputMessage};
///
/// struct JsonFormatter;
///
/// impl FormatterOverride for JsonFormatter {
///     fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String {
///         // Transform to JSON representation
///         format!(r#"{{"content": "{}"}}"#, formatted.trim())
///     }
/// }
/// ```
pub trait FormatterOverride: Send + Sync {
    /// Transform formatted message output
    ///
    /// This method receives the already-formatted message (with theme applied)
    /// and the original message object for context. It should return the
    /// transformed output.
    ///
    /// # Arguments
    ///
    /// * `formatted` - The message already formatted with theme
    /// * `message` - The original message object (for metadata/context)
    ///
    /// # Returns
    ///
    /// The transformed message string
    fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String;
}

/// Trait for output destinations
///
/// An output sink receives formatted messages and writes them to a destination.
/// Sinks handle the mechanics of where output goes, not how it's formatted.
///
/// # Design Philosophy
///
/// Sinks receive already-formatted messages (with theme applied) and route them
/// to appropriate destinations. They don't handle formatting or verbosity filtering -
/// those concerns are handled by message types and filters respectively.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::cli::views::{OutputSink, OutputMessage};
/// use std::fs::File;
///
/// struct FileSink {
///     file: File,
/// }
///
/// impl OutputSink for FileSink {
///     fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
///         use std::io::Write;
///         writeln!(self.file, "{}", formatted).ok();
///     }
/// }
/// ```
pub trait OutputSink: Send + Sync {
    /// Write a formatted message to this sink
    ///
    /// # Arguments
    ///
    /// * `message` - The message object (for metadata like channel)
    /// * `formatted` - The already-formatted message text
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str);
}
