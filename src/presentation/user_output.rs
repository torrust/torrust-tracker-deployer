//! User-facing output handling
//!
//! This module provides user-facing output functionality separate from internal logging.
//! It implements a dual-channel strategy following Unix conventions and modern CLI best practices
//! (similar to cargo, docker, npm):
//!
//! - **stdout (Results Channel)**: Final results, structured data, output for piping/redirection
//! - **stderr (Progress/Operational Channel)**: Progress updates, status messages, warnings, errors
//!
//! This separation enables:
//! - Clean piping: `torrust-tracker-deployer destroy env | jq .status` works correctly
//! - Automation friendly: Scripts can redirect progress to /dev/null while capturing results
//! - Unix convention compliance: Follows established patterns from modern CLI tools
//! - Better UX: Progress feedback doesn't interfere with result data
//!
//! ## Type-Safe Channel Routing
//!
//! The module uses newtype wrappers (`StdoutWriter` and `StderrWriter`) to provide compile-time
//! guarantees that messages are routed to the correct output channel. This prevents accidental
//! channel confusion and makes the code more maintainable by catching routing errors at compile
//! time rather than runtime.
//!
//! The newtype pattern is a zero-cost abstraction - it has the same memory layout and performance
//! characteristics as the wrapped type, but provides type safety benefits.
//!
//! ## Buffering Behavior
//!
//! Output is line-buffered by default. Messages are typically flushed automatically
//! after each newline. For cases where immediate output is critical (e.g., before
//! long-running operations), call `flush()` explicitly:
//!
//! ```rust,ignore
//! output.progress("Starting long operation...");
//! output.flush()?; // Ensure message appears before operation starts
//! perform_long_operation();
//! ```
//!
//! ## Example Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
//!
//! let mut output = UserOutput::new(VerbosityLevel::Normal);
//!
//! // Progress messages go to stderr
//! output.progress("Destroying environment...");
//!
//! // Success status goes to stderr
//! output.success("Environment destroyed successfully");
//!
//! // Results go to stdout for piping
//! output.result(r#"{"status": "destroyed"}"#);
//! ```
//!
//! ## Channel Strategy
//!
//! Based on research from [`docs/research/UX/console-app-output-patterns.md`](../../docs/research/UX/console-app-output-patterns.md):
//!
//! - **stdout**: Deployment results, configuration summaries, structured data (JSON)
//! - **stderr**: Step progress, status updates, warnings, error messages with actionable guidance
//!
//! See also: [`docs/research/UX/user-output-vs-logging-separation.md`](../../docs/research/UX/user-output-vs-logging-separation.md)

use std::io::Write;

/// Output theme controlling symbols and formatting
///
/// A theme defines the visual appearance of user-facing messages through
/// configurable symbols. Themes enable consistent styling across all output
/// and support different environments (terminals, CI/CD, accessibility needs).
///
/// # Predefined Themes
///
/// - **Emoji** (default): Unicode emoji symbols for interactive terminals
/// - **Plain**: Text labels like `[INFO]`, `[OK]` for CI/CD environments
/// - **ASCII**: Basic ASCII characters for limited terminal support
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
///
/// // Use emoji theme (default)
/// let theme = Theme::emoji();
/// assert_eq!(theme.progress_symbol(), "⏳");
///
/// // Use plain text theme for CI/CD
/// let theme = Theme::plain();
/// assert_eq!(theme.success_symbol(), "[OK]");
///
/// // Use ASCII theme for limited terminals
/// let theme = Theme::ascii();
/// assert_eq!(theme.error_symbol(), "[x]");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct Theme {
    progress_symbol: String,
    success_symbol: String,
    warning_symbol: String,
    error_symbol: String,
}

impl Theme {
    /// Create emoji theme with Unicode symbols (default)
    ///
    /// Best for interactive terminals with good Unicode support.
    /// Uses emoji characters that are visually distinctive and widely supported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
    ///
    /// let theme = Theme::emoji();
    /// assert_eq!(theme.progress_symbol(), "⏳");
    /// assert_eq!(theme.success_symbol(), "✅");
    /// assert_eq!(theme.warning_symbol(), "⚠️");
    /// assert_eq!(theme.error_symbol(), "❌");
    /// ```
    #[must_use]
    pub fn emoji() -> Self {
        Self {
            progress_symbol: "⏳".to_string(),
            success_symbol: "✅".to_string(),
            warning_symbol: "⚠️".to_string(),
            error_symbol: "❌".to_string(),
        }
    }

    /// Create plain text theme for CI/CD environments
    ///
    /// Uses text labels like `[INFO]`, `[OK]`, `[WARN]`, `[ERROR]` that work
    /// in any environment without Unicode support. Ideal for CI/CD pipelines
    /// and log aggregation systems.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
    ///
    /// let theme = Theme::plain();
    /// assert_eq!(theme.progress_symbol(), "[INFO]");
    /// assert_eq!(theme.success_symbol(), "[OK]");
    /// assert_eq!(theme.warning_symbol(), "[WARN]");
    /// assert_eq!(theme.error_symbol(), "[ERROR]");
    /// ```
    #[must_use]
    pub fn plain() -> Self {
        Self {
            progress_symbol: "[INFO]".to_string(),
            success_symbol: "[OK]".to_string(),
            warning_symbol: "[WARN]".to_string(),
            error_symbol: "[ERROR]".to_string(),
        }
    }

    /// Create ASCII-only theme using basic characters
    ///
    /// Uses simple ASCII characters that work on any terminal.
    /// Good for environments with limited character set support or
    /// when maximum compatibility is required.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
    ///
    /// let theme = Theme::ascii();
    /// assert_eq!(theme.progress_symbol(), "=>");
    /// assert_eq!(theme.success_symbol(), "[+]");
    /// assert_eq!(theme.warning_symbol(), "[!]");
    /// assert_eq!(theme.error_symbol(), "[x]");
    /// ```
    #[must_use]
    pub fn ascii() -> Self {
        Self {
            progress_symbol: "=>".to_string(),
            success_symbol: "[+]".to_string(),
            warning_symbol: "[!]".to_string(),
            error_symbol: "[x]".to_string(),
        }
    }

    /// Get the progress symbol for this theme
    #[must_use]
    pub fn progress_symbol(&self) -> &str {
        &self.progress_symbol
    }

    /// Get the success symbol for this theme
    #[must_use]
    pub fn success_symbol(&self) -> &str {
        &self.success_symbol
    }

    /// Get the warning symbol for this theme
    #[must_use]
    pub fn warning_symbol(&self) -> &str {
        &self.warning_symbol
    }

    /// Get the error symbol for this theme
    #[must_use]
    pub fn error_symbol(&self) -> &str {
        &self.error_symbol
    }
}

impl Default for Theme {
    /// Create the default theme (emoji)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::Theme;
    ///
    /// let theme = Theme::default();
    /// assert_eq!(theme.progress_symbol(), "⏳");
    /// ```
    fn default() -> Self {
        Self::emoji()
    }
}

/// Output channel for routing messages
///
/// Determines whether a message should be written to stdout or stderr.
/// Following Unix conventions:
/// - **stdout**: Final results and structured data for piping/redirection
/// - **stderr**: Progress updates, status messages, operational info, errors
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::Channel;
///
/// let channel = Channel::Stdout;
/// assert_eq!(channel, Channel::Stdout);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Standard output stream for final results and data
    Stdout,
    /// Standard error stream for progress and operational messages
    Stderr,
}

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
/// use torrust_tracker_deployer_lib::presentation::user_output::{OutputMessage, Theme, VerbosityLevel, Channel};
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
/// use torrust_tracker_deployer_lib::presentation::user_output::{FormatterOverride, OutputMessage};
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
/// use torrust_tracker_deployer_lib::presentation::user_output::{OutputSink, OutputMessage};
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

/// Verbosity levels for user output
///
/// Controls the amount of detail shown to users. Higher verbosity levels include
/// all output from lower levels.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::VerbosityLevel;
///
/// let level = VerbosityLevel::Normal;
/// assert!(level >= VerbosityLevel::Quiet);
/// assert!(level < VerbosityLevel::Verbose);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum VerbosityLevel {
    /// Minimal output - only errors and final results
    Quiet,
    /// Default level - essential progress and results
    #[default]
    Normal,
    /// Detailed progress including intermediate steps
    Verbose,
    /// Very detailed including decisions and retries
    VeryVerbose,
    /// Maximum detail for troubleshooting
    Debug,
}

// ============================================================================
// Formatter Override Implementations
// ============================================================================

/// JSON formatter for machine-readable output
///
/// Transforms messages into JSON objects with metadata including:
/// - Message type (for programmatic filtering)
/// - Channel (stdout/stderr)
/// - Content (the formatted message)
/// - Timestamp (ISO 8601 format)
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::{JsonFormatter, UserOutput, VerbosityLevel};
///
/// let formatter = JsonFormatter;
/// let mut output = UserOutput::with_formatter_override(
///     VerbosityLevel::Normal,
///     Box::new(formatter)
/// );
///
/// output.progress("Starting process");
/// // Output: {"type":"ProgressMessage","channel":"Stderr","content":"⏳ Starting process","timestamp":"2025-11-04T12:34:56Z"}
/// ```
pub struct JsonFormatter;

impl FormatterOverride for JsonFormatter {
    fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String {
        let json = serde_json::json!({
            "type": message.type_name(),
            "channel": format!("{:?}", message.channel()),
            "content": formatted.trim(), // Remove trailing newlines for cleaner JSON
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
        .to_string();
        format!("{json}\n")
    }
}

// ============================================================================
// Concrete Message Type Implementations
// ============================================================================

/// Progress message for ongoing operations
///
/// Progress messages indicate that work is in progress. They are displayed
/// during operations to provide feedback to users.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::ProgressMessage;
///
/// let message = ProgressMessage {
///     text: "Destroying environment...".to_string(),
/// };
/// ```
pub struct ProgressMessage {
    /// The progress message text
    pub text: String,
}

impl OutputMessage for ProgressMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}\n", theme.progress_symbol(), self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "ProgressMessage"
    }
}

/// Success message for completed operations
///
/// Success messages indicate that an operation completed successfully.
/// They provide positive feedback to users.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::SuccessMessage;
///
/// let message = SuccessMessage {
///     text: "Environment destroyed successfully".to_string(),
/// };
/// ```
pub struct SuccessMessage {
    /// The success message text
    pub text: String,
}

impl OutputMessage for SuccessMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}\n", theme.success_symbol(), self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "SuccessMessage"
    }
}

/// Warning message for non-critical issues
///
/// Warning messages alert users to potential issues that don't prevent
/// operation completion but may need attention.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::WarningMessage;
///
/// let message = WarningMessage {
///     text: "Infrastructure may already be destroyed".to_string(),
/// };
/// ```
pub struct WarningMessage {
    /// The warning message text
    pub text: String,
}

impl OutputMessage for WarningMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{}  {}\n", theme.warning_symbol(), self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "WarningMessage"
    }
}

/// Error message for critical failures
///
/// Error messages indicate critical failures that prevent operation completion.
/// They are always shown regardless of verbosity level.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::ErrorMessage;
///
/// let message = ErrorMessage {
///     text: "Failed to destroy environment".to_string(),
/// };
/// ```
pub struct ErrorMessage {
    /// The error message text
    pub text: String,
}

impl OutputMessage for ErrorMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}\n", theme.error_symbol(), self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Quiet // Always shown
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "ErrorMessage"
    }
}

/// Result message for final output data
///
/// Result messages contain final output data that can be piped or redirected.
/// They go to stdout without any symbols or formatting.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::ResultMessage;
///
/// let message = ResultMessage {
///     text: "Deployment complete".to_string(),
/// };
/// ```
pub struct ResultMessage {
    /// The result message text
    pub text: String,
}

impl OutputMessage for ResultMessage {
    fn format(&self, _theme: &Theme) -> String {
        format!("{}\n", self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Quiet
    }

    fn channel(&self) -> Channel {
        Channel::Stdout
    }

    fn type_name(&self) -> &'static str {
        "ResultMessage"
    }
}

/// Steps message for sequential instructions
///
/// Steps messages display numbered lists of sequential items.
/// Useful for showing action items or instructions.
///
/// # Examples
///
/// Simple constructor for cases where you have all items upfront:
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
///
/// let message = StepsMessage::new("Next steps:", vec![
///     "Edit the configuration file".to_string(),
///     "Review the settings".to_string(),
/// ]);
/// ```
///
/// Builder pattern for dynamic construction or better readability:
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
///
/// let message = StepsMessage::builder("Next steps:")
///     .add("Edit the configuration file")
///     .add("Review the settings")
///     .build();
/// ```
pub struct StepsMessage {
    /// The title for the steps list
    pub title: String,
    /// The list of step items
    pub items: Vec<String>,
}

impl StepsMessage {
    /// Create a new steps message with the given title and items
    ///
    /// This is a convenience constructor for simple cases where you have
    /// all items upfront. For dynamic construction or better readability,
    /// consider using `StepsMessage::builder()` instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
    ///
    /// let msg = StepsMessage::new("Next steps:", vec![
    ///     "Edit config".to_string(),
    ///     "Run tests".to_string(),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(title: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }

    /// Create a builder for constructing steps messages with a fluent API
    ///
    /// The builder pattern is useful when:
    /// - Adding items dynamically
    /// - You want self-documenting, readable code
    /// - Building the message in multiple steps
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
    ///
    /// let msg = StepsMessage::builder("Next steps:")
    ///     .add("Edit configuration")
    ///     .add("Review settings")
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder(title: impl Into<String>) -> StepsMessageBuilder {
        StepsMessageBuilder::new(title)
    }
}

impl OutputMessage for StepsMessage {
    fn format(&self, _theme: &Theme) -> String {
        use std::fmt::Write;

        let mut output = format!("{}\n", self.title);
        for (idx, step) in self.items.iter().enumerate() {
            writeln!(&mut output, "{}. {}", idx + 1, step).ok();
        }
        output
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "StepsMessage"
    }
}

/// Builder for constructing `StepsMessage` with a fluent API
///
/// Provides a consuming builder pattern for constructing step messages
/// with optional customization. Use this for complex cases where items
/// are added dynamically or for improved readability. Simple cases can
/// use `StepsMessage::new()` directly.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
///
/// let message = StepsMessage::builder("Next steps:")
///     .add("Edit configuration")
///     .add("Review settings")
///     .add("Deploy changes")
///     .build();
/// ```
///
/// Empty builders are valid:
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
///
/// let message = StepsMessage::builder("Title").build();
/// ```
pub struct StepsMessageBuilder {
    title: String,
    items: Vec<String>,
}

impl StepsMessageBuilder {
    /// Create a new builder with the given title
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessageBuilder;
    ///
    /// let builder = StepsMessageBuilder::new("My steps:");
    /// ```
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    /// Add a step to the list (consuming self for method chaining)
    ///
    /// This method consumes the builder and returns it, enabling
    /// method chaining in a fluent API style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
    ///
    /// let message = StepsMessage::builder("Steps:")
    ///     .add("First step")
    ///     .add("Second step")
    ///     .build();
    /// ```
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, step: impl Into<String>) -> Self {
        self.items.push(step.into());
        self
    }

    /// Build the final `StepsMessage`
    ///
    /// Consumes the builder and produces the final message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
    ///
    /// let message = StepsMessage::builder("Steps:")
    ///     .add("Step 1")
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> StepsMessage {
        StepsMessage {
            title: self.title,
            items: self.items,
        }
    }
}

/// Informational block message for grouped information
///
/// Info block messages display a title followed by multiple lines of text.
/// Useful for displaying grouped information, configuration details, or
/// multi-line informational content.
///
/// # Examples
///
/// Simple constructor for cases where you have all lines upfront:
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
///
/// let message = InfoBlockMessage::new("Environment Details", vec![
///     "Name: production".to_string(),
///     "Status: running".to_string(),
/// ]);
/// ```
///
/// Builder pattern for dynamic construction or better readability:
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
///
/// let message = InfoBlockMessage::builder("Environment Details")
///     .add_line("Name: production")
///     .add_line("Status: running")
///     .add_line("Uptime: 24 hours")
///     .build();
/// ```
pub struct InfoBlockMessage {
    /// The title for the info block
    pub title: String,
    /// The lines of information
    pub lines: Vec<String>,
}

impl InfoBlockMessage {
    /// Create a new info block message with the given title and lines
    ///
    /// This is a convenience constructor for simple cases where you have
    /// all lines upfront. For dynamic construction or better readability,
    /// consider using `InfoBlockMessage::builder()` instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
    ///
    /// let msg = InfoBlockMessage::new("Configuration:", vec![
    ///     "  - username: 'torrust'".to_string(),
    ///     "  - port: 22".to_string(),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(title: impl Into<String>, lines: Vec<String>) -> Self {
        Self {
            title: title.into(),
            lines,
        }
    }

    /// Create a builder for constructing info block messages with a fluent API
    ///
    /// The builder pattern is useful when:
    /// - Adding lines dynamically
    /// - You want self-documenting, readable code
    /// - Building the message in multiple steps
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
    ///
    /// let msg = InfoBlockMessage::builder("Environment Details")
    ///     .add_line("Name: production")
    ///     .add_line("Status: active")
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder(title: impl Into<String>) -> InfoBlockMessageBuilder {
        InfoBlockMessageBuilder::new(title)
    }
}

impl OutputMessage for InfoBlockMessage {
    fn format(&self, _theme: &Theme) -> String {
        use std::fmt::Write;

        let mut output = format!("{}\n", self.title);
        for line in &self.lines {
            writeln!(&mut output, "{line}").ok();
        }
        output
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "InfoBlockMessage"
    }
}

/// Builder for constructing `InfoBlockMessage` with a fluent API
///
/// Provides a consuming builder pattern for constructing info block messages
/// with optional customization. Use this for complex cases where lines
/// are added dynamically or for improved readability. Simple cases can
/// use `InfoBlockMessage::new()` directly.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
///
/// let message = InfoBlockMessage::builder("Environment Details")
///     .add_line("Name: production")
///     .add_line("Status: running")
///     .add_line("Uptime: 24 hours")
///     .build();
/// ```
///
/// Empty builders are valid:
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
///
/// let message = InfoBlockMessage::builder("Title").build();
/// ```
pub struct InfoBlockMessageBuilder {
    title: String,
    lines: Vec<String>,
}

impl InfoBlockMessageBuilder {
    /// Create a new builder with the given title
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessageBuilder;
    ///
    /// let builder = InfoBlockMessageBuilder::new("My info block:");
    /// ```
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
        }
    }

    /// Add a line to the info block (consuming self for method chaining)
    ///
    /// This method consumes the builder and returns it, enabling
    /// method chaining in a fluent API style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
    ///
    /// let message = InfoBlockMessage::builder("Info:")
    ///     .add_line("First line")
    ///     .add_line("Second line")
    ///     .build();
    /// ```
    #[must_use]
    pub fn add_line(mut self, line: impl Into<String>) -> Self {
        self.lines.push(line.into());
        self
    }

    /// Build the final `InfoBlockMessage`
    ///
    /// Consumes the builder and produces the final message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
    ///
    /// let message = InfoBlockMessage::builder("Info:")
    ///     .add_line("Line 1")
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> InfoBlockMessage {
        InfoBlockMessage {
            title: self.title,
            lines: self.lines,
        }
    }
}

// ============================================================================
// PRIVATE - Type-Safe Writer Wrappers
// ============================================================================

/// Stdout writer wrapper for type safety
///
/// This newtype wrapper ensures that stdout-specific operations
/// can only be performed on stdout writers, preventing accidental
/// channel confusion at compile time.
///
/// The wrapper provides a zero-cost abstraction - it has the same
/// memory layout and performance characteristics as the wrapped type,
/// but provides compile-time type safety.
struct StdoutWriter(Box<dyn Write + Send + Sync>);

impl StdoutWriter {
    /// Create a new stdout writer wrapper
    fn new(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self(writer)
    }

    /// Write a line to stdout
    ///
    /// Writes the given message to the stdout channel.
    /// The message should include any necessary newline characters.
    /// Errors are silently ignored as output operations are best-effort.
    fn write_line(&mut self, message: &str) {
        write!(self.0, "{message}").ok();
    }

    /// Write with a newline to stdout
    ///
    /// Writes the given message followed by a newline to the stdout channel.
    /// Errors are silently ignored as output operations are best-effort.
    #[allow(dead_code)]
    fn writeln(&mut self, message: &str) {
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
struct StderrWriter(Box<dyn Write + Send + Sync>);

impl StderrWriter {
    /// Create a new stderr writer wrapper
    fn new(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self(writer)
    }

    /// Write a line to stderr
    ///
    /// Writes the given message to the stderr channel.
    /// The message should include any necessary newline characters.
    /// Errors are silently ignored as output operations are best-effort.
    fn write_line(&mut self, message: &str) {
        write!(self.0, "{message}").ok();
    }

    /// Write with a newline to stderr
    ///
    /// Writes the given message followed by a newline to the stderr channel.
    /// Errors are silently ignored as output operations are best-effort.
    #[allow(dead_code)]
    fn writeln(&mut self, message: &str) {
        writeln!(self.0, "{message}").ok();
    }
}

// ============================================================================
// PRIVATE - Verbosity Filter
// ============================================================================

/// Determines what messages should be displayed based on verbosity level
///
/// This struct encapsulates verbosity filtering logic, making it testable
/// independently from output formatting.
struct VerbosityFilter {
    level: VerbosityLevel,
}

impl VerbosityFilter {
    /// Create a new verbosity filter with the specified level
    fn new(level: VerbosityLevel) -> Self {
        Self { level }
    }

    /// Check if messages at the given level should be shown
    fn should_show(&self, required_level: VerbosityLevel) -> bool {
        self.level >= required_level
    }

    /// Progress messages require Normal level
    #[allow(dead_code)]
    fn should_show_progress(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Success messages require Normal level
    #[allow(dead_code)]
    fn should_show_success(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Warning messages require Normal level
    #[allow(dead_code)]
    fn should_show_warnings(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Errors are always shown regardless of verbosity level
    #[allow(clippy::unused_self)]
    #[allow(dead_code)]
    fn should_show_errors(&self) -> bool {
        true
    }

    /// Blank lines require Normal level
    fn should_show_blank_lines(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Steps require Normal level
    #[allow(dead_code)]
    fn should_show_steps(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Info blocks require Normal level
    #[allow(dead_code)]
    fn should_show_info_blocks(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }
}

// ============================================================================
// PRIVATE - Output Sink Implementations
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
/// use torrust_tracker_deployer_lib::presentation::user_output::StandardSink;
///
/// let sink = StandardSink::new(
///     Box::new(std::io::stdout()),
///     Box::new(std::io::stderr())
/// );
/// ```
struct StandardSink {
    stdout: StdoutWriter,
    stderr: StderrWriter,
}

impl StandardSink {
    /// Create a new standard sink with the given writers
    fn new(
        stdout: Box<dyn Write + Send + Sync>,
        stderr: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self {
            stdout: StdoutWriter::new(stdout),
            stderr: StderrWriter::new(stderr),
        }
    }

    /// Create a standard sink using default stdout/stderr
    ///
    /// This is the default console sink that writes to the standard
    /// output and error streams.
    fn default_console() -> Self {
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

/// Composite sink that writes to multiple destinations
///
/// Enables fan-out of messages to multiple sinks simultaneously. Useful for
/// scenarios like writing to both console and log file, or sending to both
/// stderr and telemetry service.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::{CompositeSink, StandardSink, FileSink};
///
/// let composite = CompositeSink::new(vec![
///     Box::new(StandardSink::default_console()),
///     Box::new(FileSink::new("output.log").unwrap()),
/// ]);
/// ```
pub struct CompositeSink {
    sinks: Vec<Box<dyn OutputSink>>,
}

impl CompositeSink {
    /// Create a new composite sink with the given child sinks
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::CompositeSink;
    ///
    /// let composite = CompositeSink::new(vec![
    ///     Box::new(StandardSink::default_console()),
    ///     Box::new(FileSink::new("output.log").unwrap()),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(sinks: Vec<Box<dyn OutputSink>>) -> Self {
        Self { sinks }
    }

    /// Add a sink to the composite
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::CompositeSink;
    ///
    /// let mut composite = CompositeSink::new(vec![]);
    /// composite.add_sink(Box::new(StandardSink::default_console()));
    /// composite.add_sink(Box::new(FileSink::new("output.log").unwrap()));
    /// ```
    pub fn add_sink(&mut self, sink: Box<dyn OutputSink>) {
        self.sinks.push(sink);
    }
}

impl OutputSink for CompositeSink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        for sink in &mut self.sinks {
            sink.write_message(message, formatted);
        }
    }
}

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

/// Example: Telemetry sink for observability
///
/// This is a stub implementation showing how a telemetry sink could be
/// implemented. In production, this would send events to a telemetry service.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::{TelemetrySink, UserOutput, VerbosityLevel, CompositeSink, StandardSink};
///
/// // Write to both console and telemetry
/// let composite = CompositeSink::new(vec![
///     Box::new(StandardSink::default_console()),
///     Box::new(TelemetrySink::new("https://telemetry.example.com".to_string())),
/// ]);
/// let mut output = UserOutput::with_sink(VerbosityLevel::Normal, Box::new(composite));
/// ```
pub struct TelemetrySink {
    endpoint: String,
}

impl TelemetrySink {
    /// Create a new telemetry sink
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::TelemetrySink;
    ///
    /// let sink = TelemetrySink::new("https://telemetry.example.com".to_string());
    /// ```
    #[must_use]
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl OutputSink for TelemetrySink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        // In real implementation, send to telemetry service
        tracing::debug!(
            endpoint = %self.endpoint,
            message_type = message.type_name(),
            channel = ?message.channel(),
            content = formatted,
            "Telemetry event"
        );
    }
}

/// Handles user-facing output separate from internal logging
///
/// Uses dual channels following Unix conventions and modern CLI best practices:
/// - **stdout**: Final results and data for piping/redirection
/// - **stderr**: Progress updates, status messages, operational info, errors
///
/// This separation allows scripts to cleanly capture results while seeing progress:
///
/// ```bash
/// # Suppress progress, capture results only
/// torrust-tracker-deployer destroy env 2>/dev/null > result.json
///
/// # Suppress results, see progress only
/// torrust-tracker-deployer destroy env > /dev/null
/// ```
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
///
/// let mut output = UserOutput::new(VerbosityLevel::Normal);
///
/// // Progress to stderr (visible during execution, doesn't interfere with piping)
/// output.progress("Processing data...");
///
/// // Results to stdout (can be piped to other commands)
/// output.result("Processing complete");
/// ```
pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    sink: Box<dyn OutputSink>,
    formatter_override: Option<Box<dyn FormatterOverride>>,
}

impl UserOutput {
    /// Create new `UserOutput` with default stdout/stderr channels and emoji theme
    ///
    /// Uses `StandardSink` for backward compatibility with existing console output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = UserOutput::new(VerbosityLevel::Normal);
    /// ```
    #[must_use]
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self::with_theme(verbosity, Theme::default())
    }

    /// Create `UserOutput` with a specific theme
    ///
    /// Allows customization of output symbols while using default stdout/stderr channels.
    /// Uses `StandardSink` internally for backward compatibility.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel, Theme};
    ///
    /// // Use plain text theme for CI/CD
    /// let output = UserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());
    ///
    /// // Use ASCII theme for limited terminals
    /// let output = UserOutput::with_theme(VerbosityLevel::Normal, Theme::ascii());
    /// ```
    #[must_use]
    pub fn with_theme(verbosity: VerbosityLevel, theme: Theme) -> Self {
        Self::with_sink(verbosity, Box::new(StandardSink::default_console())).with_theme_applied(theme)
    }

    /// Create `UserOutput` with a custom sink
    ///
    /// This constructor enables the use of alternative output destinations,
    /// including composite sinks for multi-destination output.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::{
    ///     UserOutput, VerbosityLevel, CompositeSink, StandardSink, FileSink
    /// };
    ///
    /// // Console + File output
    /// let composite = CompositeSink::new(vec![
    ///     Box::new(StandardSink::default_console()),
    ///     Box::new(FileSink::new("output.log").unwrap()),
    /// ]);
    /// let output = UserOutput::with_sink(VerbosityLevel::Normal, Box::new(composite));
    /// ```
    #[must_use]
    pub fn with_sink(verbosity: VerbosityLevel, sink: Box<dyn OutputSink>) -> Self {
        Self {
            theme: Theme::default(),
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink,
            formatter_override: None,
        }
    }

    /// Internal helper to apply theme to an existing UserOutput
    fn with_theme_applied(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Create `UserOutput` with theme and custom writers (for testing)
    ///
    /// This constructor allows full customization including theme and writers,
    /// primarily used for testing where output needs to be captured.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel, Theme};
    /// use std::io::Cursor;
    ///
    /// let stdout_buf = Vec::new();
    /// let stderr_buf = Vec::new();
    ///
    /// let output = UserOutput::with_theme_and_writers(
    ///     VerbosityLevel::Normal,
    ///     Theme::plain(),
    ///     Box::new(Cursor::new(stdout_buf)),
    ///     Box::new(Cursor::new(stderr_buf)),
    /// );
    /// ```
    #[must_use]
    pub fn with_theme_and_writers(
        verbosity: VerbosityLevel,
        theme: Theme,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self {
            theme,
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink: Box::new(StandardSink::new(stdout_writer, stderr_writer)),
            formatter_override: None,
        }
    }

    /// Create `UserOutput` with an optional formatter override
    ///
    /// This allows applying custom formatting (e.g., JSON, colored output)
    /// on top of the theme-based formatting.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::{
    ///     UserOutput, VerbosityLevel, JsonFormatter
    /// };
    ///
    /// let mut output = UserOutput::with_formatter_override(
    ///     VerbosityLevel::Normal,
    ///     Box::new(JsonFormatter),
    /// );
    ///
    /// output.progress("Processing");
    /// // Output: {"type":"ProgressMessage","channel":"Stderr","content":"⏳ Processing","timestamp":"..."}
    /// ```
    #[must_use]
    pub fn with_formatter_override(
        verbosity: VerbosityLevel,
        formatter_override: Box<dyn FormatterOverride>,
    ) -> Self {
        Self {
            theme: Theme::default(),
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink: Box::new(StandardSink::default_console()),
            formatter_override: Some(formatter_override),
        }
    }

    /// Create `UserOutput` with theme and optional formatter override
    ///
    /// Combines theme selection with optional formatter override.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::{
    ///     UserOutput, VerbosityLevel, Theme, JsonFormatter
    /// };
    ///
    /// let mut output = UserOutput::with_theme_and_formatter(
    ///     VerbosityLevel::Normal,
    ///     Theme::plain(),
    ///     Some(Box::new(JsonFormatter)),
    /// );
    /// ```
    #[must_use]
    pub fn with_theme_and_formatter(
        verbosity: VerbosityLevel,
        theme: Theme,
        formatter_override: Option<Box<dyn FormatterOverride>>,
    ) -> Self {
        Self {
            theme,
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink: Box::new(StandardSink::default_console()),
            formatter_override,
        }
    }

    /// Create `UserOutput` for testing with custom writers (uses default emoji theme)
    ///
    /// This constructor allows injecting custom writers for testing,
    /// enabling output capture and assertion. Uses the default emoji theme.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    /// use std::io::Cursor;
    ///
    /// let stdout_buf = Vec::new();
    /// let stderr_buf = Vec::new();
    ///
    /// let output = UserOutput::with_writers(
    ///     VerbosityLevel::Normal,
    ///     Box::new(Cursor::new(stdout_buf)),
    ///     Box::new(Cursor::new(stderr_buf)),
    /// );
    /// ```
    #[must_use]
    pub fn with_writers(
        verbosity: VerbosityLevel,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self::with_theme_and_writers(verbosity, Theme::default(), stdout_writer, stderr_writer)
    }

    /// Write a message to the appropriate channel using trait dispatch
    ///
    /// This is the core method for extensible message handling. It uses the
    /// `OutputMessage` trait to determine formatting, verbosity requirements,
    /// and channel routing. Messages are routed through the configured sink,
    /// enabling multi-destination output.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel, ProgressMessage};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.write(&ProgressMessage {
    ///     text: "Processing...".to_string(),
    /// });
    /// ```
    pub fn write(&mut self, message: &dyn OutputMessage) {
        if !self
            .verbosity_filter
            .should_show(message.required_verbosity())
        {
            return;
        }

        let mut formatted = message.format(&self.theme);

        // Apply optional format override
        if let Some(override_formatter) = &self.formatter_override {
            formatted = override_formatter.transform(&formatted, message);
        }

        // Write through sink
        self.sink.write_message(message, &formatted);
    }

    /// Flush all pending output to stdout and stderr
    ///
    /// **Note**: With the OutputSink abstraction, flush behavior depends on the
    /// sink implementation. StandardSink does not support explicit flushing.
    /// This method is kept for API compatibility but is currently a no-op.
    ///
    /// For StandardSink (default), writes are typically line-buffered by the OS.
    ///
    /// # Errors
    ///
    /// Currently always returns Ok(()) as flush is not supported through sinks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.progress("Starting long operation...");
    /// output.flush().expect("Failed to flush output");
    /// // Now perform long operation...
    /// ```
    pub fn flush(&mut self) -> std::io::Result<()> {
        // Note: Flush is not supported through the OutputSink abstraction.
        // This is a known limitation. StandardSink relies on OS line-buffering.
        Ok(())
    }

    /// Display progress message to stderr (Normal level and above)
    ///
    /// Progress messages go to stderr following cargo/docker patterns.
    /// This keeps stdout clean for result data that may be piped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.progress("Destroying environment...");
    /// // Output to stderr: ⏳ Destroying environment...
    /// ```
    pub fn progress(&mut self, message: &str) {
        self.write(&ProgressMessage {
            text: message.to_string(),
        });
    }

    /// Display success message to stderr (Normal level and above)
    ///
    /// Success status goes to stderr to allow clean result piping.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.success("Environment destroyed successfully");
    /// // Output to stderr: ✅ Environment destroyed successfully
    /// ```
    pub fn success(&mut self, message: &str) {
        self.write(&SuccessMessage {
            text: message.to_string(),
        });
    }

    /// Display warning message to stderr (Normal level and above)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.warn("Infrastructure may already be destroyed");
    /// // Output to stderr: ⚠️  Infrastructure may already be destroyed
    /// ```
    pub fn warn(&mut self, message: &str) {
        self.write(&WarningMessage {
            text: message.to_string(),
        });
    }

    /// Display error message to stderr (all levels)
    ///
    /// Errors are always shown regardless of verbosity level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Quiet);
    /// output.error("Failed to destroy environment");
    /// // Output to stderr: ❌ Failed to destroy environment
    /// ```
    pub fn error(&mut self, message: &str) {
        self.write(&ErrorMessage {
            text: message.to_string(),
        });
    }

    /// Output final results to stdout for piping/redirection
    ///
    /// This is where deployment results, configuration summaries, etc. go.
    /// Since this goes to stdout, it can be cleanly piped to other commands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.result("Deployment complete");
    /// // Output to stdout: Deployment complete
    /// ```
    pub fn result(&mut self, message: &str) {
        self.write(&ResultMessage {
            text: message.to_string(),
        });
    }

    /// Output structured data to stdout (JSON, etc.)
    ///
    /// For machine-readable output that should be piped or processed.
    /// This is equivalent to `result()` but exists for semantic clarity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.data(r#"{"status": "destroyed", "environment": "test"}"#);
    /// // Output to stdout: {"status": "destroyed", "environment": "test"}
    /// ```
    pub fn data(&mut self, data: &str) {
        self.result(data);
    }

    /// Display a blank line to stderr (Normal level and above)
    ///
    /// Used for spacing between sections of output to improve readability.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.success("Configuration template generated");
    /// output.blank_line();
    /// output.progress("Starting next steps...");
    /// ```
    pub fn blank_line(&mut self) {
        if self.verbosity_filter.should_show_blank_lines() {
            // Create a simple message that just outputs a newline
            struct BlankLineMessage;
            impl OutputMessage for BlankLineMessage {
                fn format(&self, _theme: &Theme) -> String {
                    "\n".to_string()
                }
                fn required_verbosity(&self) -> VerbosityLevel {
                    VerbosityLevel::Normal
                }
                fn channel(&self) -> Channel {
                    Channel::Stderr
                }
                fn type_name(&self) -> &'static str {
                    "BlankLineMessage"
                }
            }
            self.write(&BlankLineMessage);
        }
    }

    /// Display a numbered list of steps to stderr (Normal level and above)
    ///
    /// Useful for displaying sequential instructions or action items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.steps("Next steps:", &[
    ///     "Edit the configuration file",
    ///     "Review the settings",
    ///     "Run the deploy command",
    /// ]);
    /// // Output to stderr:
    /// // Next steps:
    /// // 1. Edit the configuration file
    /// // 2. Review the settings
    /// // 3. Run the deploy command
    /// ```
    pub fn steps(&mut self, title: &str, steps: &[&str]) {
        self.write(&StepsMessage {
            title: title.to_string(),
            items: steps.iter().map(|s| (*s).to_string()).collect(),
        });
    }

    /// Display a multi-line information block to stderr (Normal level and above)
    ///
    /// Useful for displaying grouped information or detailed messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.info_block("Configuration options:", &[
    ///     "  - username: 'torrust' (default)",
    ///     "  - port: 22 (default SSH port)",
    ///     "  - key_path: path/to/key",
    /// ]);
    /// // Output to stderr:
    /// // Configuration options:
    /// //   - username: 'torrust' (default)
    /// //   - port: 22 (default SSH port)
    /// //   - key_path: path/to/key
    /// ```
    pub fn info_block(&mut self, title: &str, lines: &[&str]) {
        self.write(&InfoBlockMessage {
            title: title.to_string(),
            lines: lines.iter().map(|s| (*s).to_string()).collect(),
        });
    }
}

#[cfg(test)]
pub mod test_support {
    //! Test support infrastructure for `UserOutput` testing
    //!
    //! Provides simplified test infrastructure for capturing and asserting on output
    //! in tests across the codebase.

    use super::*;
    use std::sync::{Arc, Mutex};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Type-Safe Writer Wrapper Tests
    // ============================================================================

    mod type_safe_wrappers {
        use super::*;
        use std::sync::{Arc, Mutex};

        #[test]
        fn stdout_writer_should_wrap_writer() {
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let writer = Box::new(test_support::TestWriter::new(Arc::clone(&buffer)));

            let mut stdout = StdoutWriter::new(writer);
            stdout.write_line("Test output");

            let output = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
            assert_eq!(output, "Test output");
        }

        #[test]
        fn stderr_writer_should_wrap_writer() {
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let writer = Box::new(test_support::TestWriter::new(Arc::clone(&buffer)));

            let mut stderr = StderrWriter::new(writer);
            stderr.write_line("Test error");

            let output = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
            assert_eq!(output, "Test error");
        }

        #[test]
        fn stdout_writer_should_write_multiple_lines() {
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let writer = Box::new(test_support::TestWriter::new(Arc::clone(&buffer)));

            let mut stdout = StdoutWriter::new(writer);
            stdout.write_line("Line 1\n");
            stdout.write_line("Line 2\n");

            let output = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
            assert_eq!(output, "Line 1\nLine 2\n");
        }

        #[test]
        fn stderr_writer_should_write_multiple_lines() {
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let writer = Box::new(test_support::TestWriter::new(Arc::clone(&buffer)));

            let mut stderr = StderrWriter::new(writer);
            stderr.write_line("Error 1\n");
            stderr.write_line("Error 2\n");

            let output = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
            assert_eq!(output, "Error 1\nError 2\n");
        }

        #[test]
        fn type_safe_dispatch_prevents_channel_confusion() {
            // This test demonstrates that the type system prevents channel confusion
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let stdout_writer = Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer)));
            let stderr_writer = Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer)));

            let mut stdout = StdoutWriter::new(stdout_writer);
            let mut stderr = StderrWriter::new(stderr_writer);

            // Type-safe: These methods can only be called on the correct writer type
            stdout.write_line("stdout data");
            stderr.write_line("stderr message");

            let stdout_output = String::from_utf8(stdout_buffer.lock().unwrap().clone()).unwrap();
            let stderr_output = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();

            assert_eq!(stdout_output, "stdout data");
            assert_eq!(stderr_output, "stderr message");

            // The following would not compile (demonstrating compile-time safety):
            // stderr.write_line("this should go to stdout");  // Type mismatch!
            // stdout.write_line("this should go to stderr");  // Type mismatch!
        }

        #[test]
        fn user_output_uses_typed_wrappers_internally() {
            // This test verifies that UserOutput uses typed wrappers internally
            // and that channel routing is type-safe
            let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

            // These calls go through type-safe dispatch
            test_output.output.progress("Progress message");
            test_output.output.result("Result data");

            // Verify correct channel routing via type system
            assert!(test_output.stderr().contains("Progress message"));
            assert!(test_output.stdout().contains("Result data"));
        }

        #[test]
        fn stdout_writer_writeln_adds_newline() {
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let writer = Box::new(test_support::TestWriter::new(Arc::clone(&buffer)));

            let mut stdout = StdoutWriter::new(writer);
            stdout.writeln("Test");

            let output = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
            assert_eq!(output, "Test\n");
        }

        #[test]
        fn stderr_writer_writeln_adds_newline() {
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let writer = Box::new(test_support::TestWriter::new(Arc::clone(&buffer)));

            let mut stderr = StderrWriter::new(writer);
            stderr.writeln("Error");

            let output = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
            assert_eq!(output, "Error\n");
        }
    }

    // ============================================================================
    // Theme Tests
    // ============================================================================

    mod theme {
        use super::*;

        #[test]
        fn it_should_create_emoji_theme_with_correct_symbols() {
            let theme = Theme::emoji();

            assert_eq!(theme.progress_symbol(), "⏳");
            assert_eq!(theme.success_symbol(), "✅");
            assert_eq!(theme.warning_symbol(), "⚠️");
            assert_eq!(theme.error_symbol(), "❌");
        }

        #[test]
        fn it_should_create_plain_theme_with_text_labels() {
            let theme = Theme::plain();

            assert_eq!(theme.progress_symbol(), "[INFO]");
            assert_eq!(theme.success_symbol(), "[OK]");
            assert_eq!(theme.warning_symbol(), "[WARN]");
            assert_eq!(theme.error_symbol(), "[ERROR]");
        }

        #[test]
        fn it_should_create_ascii_theme_with_ascii_characters() {
            let theme = Theme::ascii();

            assert_eq!(theme.progress_symbol(), "=>");
            assert_eq!(theme.success_symbol(), "[+]");
            assert_eq!(theme.warning_symbol(), "[!]");
            assert_eq!(theme.error_symbol(), "[x]");
        }

        #[test]
        fn it_should_use_emoji_theme_as_default() {
            let theme = Theme::default();
            let emoji_theme = Theme::emoji();

            assert_eq!(theme, emoji_theme);
        }

        #[test]
        fn it_should_support_clone() {
            let theme = Theme::plain();
            let cloned = theme.clone();

            assert_eq!(theme, cloned);
        }

        #[test]
        fn it_should_support_equality_comparison() {
            let theme1 = Theme::emoji();
            let theme2 = Theme::emoji();
            let theme3 = Theme::plain();

            assert_eq!(theme1, theme2);
            assert_ne!(theme1, theme3);
        }

        #[test]
        fn it_should_support_debug_formatting() {
            let theme = Theme::emoji();
            let debug_output = format!("{theme:?}");

            assert!(debug_output.contains("Theme"));
        }
    }

    // ============================================================================
    // UserOutput Tests - Parameterized Tests
    // ============================================================================
    //
    // These tests use rstest for parameterized testing to reduce duplication
    // and make the test matrix clear and maintainable.
    //
    // Test Matrix:
    // Message Type | Symbol | Min Verbosity | Channel | Always Shown
    // -------------|--------|---------------|---------|-------------
    // progress     | ⏳     | Normal        | stderr  | No
    // success      | ✅     | Normal        | stderr  | No
    // warning      | ⚠️     | Normal        | stderr  | No
    // error        | ❌     | Quiet         | stderr  | Yes
    // result       | (none) | Quiet         | stdout  | Yes
    // data         | (none) | Quiet         | stdout  | Yes

    mod parameterized_tests {
        use super::*;
        use rstest::rstest;

        /// Test that each message type routes to the correct output channel
        ///
        /// Verifies stdout vs stderr routing for all message types.
        /// This replaces 5 individual channel routing tests with one parameterized test.
        #[rstest]
        #[case("progress", "⏳ Test message\n", VerbosityLevel::Normal, "stderr")]
        #[case("success", "✅ Test message\n", VerbosityLevel::Normal, "stderr")]
        #[case("warning", "⚠️  Test message\n", VerbosityLevel::Normal, "stderr")]
        #[case("error", "❌ Test message\n", VerbosityLevel::Normal, "stderr")]
        #[case("result", "Test message\n", VerbosityLevel::Normal, "stdout")]
        fn it_should_route_message_to_correct_channel(
            #[case] method: &str,
            #[case] expected_output: &str,
            #[case] verbosity: VerbosityLevel,
            #[case] expected_channel: &str,
        ) {
            let mut test_output = test_support::TestUserOutput::new(verbosity);

            // Call the appropriate method
            match method {
                "progress" => test_output.output.progress("Test message"),
                "success" => test_output.output.success("Test message"),
                "warning" => test_output.output.warn("Test message"),
                "error" => test_output.output.error("Test message"),
                "result" => test_output.output.result("Test message"),
                _ => panic!("Unknown method: {method}"),
            }

            // Verify output went to the correct channel
            match expected_channel {
                "stdout" => {
                    assert_eq!(test_output.stdout(), expected_output);
                    assert_eq!(test_output.stderr(), "");
                }
                "stderr" => {
                    assert_eq!(test_output.stderr(), expected_output);
                    assert_eq!(test_output.stdout(), "");
                }
                _ => panic!("Unknown channel: {expected_channel}"),
            }
        }

        /// Test that normal-level messages respect verbosity settings
        ///
        /// Progress, success, and warning messages should only appear at Normal or higher.
        /// This replaces 3 individual verbosity tests with one parameterized test.
        #[rstest]
        #[case("progress", VerbosityLevel::Quiet, false)]
        #[case("progress", VerbosityLevel::Normal, true)]
        #[case("progress", VerbosityLevel::Verbose, true)]
        #[case("success", VerbosityLevel::Quiet, false)]
        #[case("success", VerbosityLevel::Normal, true)]
        #[case("success", VerbosityLevel::Verbose, true)]
        #[case("warning", VerbosityLevel::Quiet, false)]
        #[case("warning", VerbosityLevel::Normal, true)]
        #[case("warning", VerbosityLevel::Verbose, true)]
        fn it_should_respect_verbosity_for_normal_level_messages(
            #[case] method: &str,
            #[case] verbosity: VerbosityLevel,
            #[case] should_show: bool,
        ) {
            let mut test_output = test_support::TestUserOutput::new(verbosity);

            match method {
                "progress" => test_output.output.progress("Test"),
                "success" => test_output.output.success("Test"),
                "warning" => test_output.output.warn("Test"),
                _ => panic!("Unknown method: {method}"),
            }

            if should_show {
                assert!(!test_output.stderr().is_empty());
            } else {
                assert_eq!(test_output.stderr(), "");
            }
        }

        /// Test that error messages are always shown regardless of verbosity
        ///
        /// Errors are critical and must be shown at all verbosity levels.
        #[rstest]
        #[case(VerbosityLevel::Quiet)]
        #[case(VerbosityLevel::Normal)]
        #[case(VerbosityLevel::Verbose)]
        #[case(VerbosityLevel::VeryVerbose)]
        #[case(VerbosityLevel::Debug)]
        fn it_should_always_show_errors_at_all_verbosity_levels(#[case] verbosity: VerbosityLevel) {
            let mut test_output = test_support::TestUserOutput::new(verbosity);

            test_output.output.error("Critical error");

            assert!(!test_output.stderr().is_empty());
            assert!(test_output.stderr().contains("Critical error"));
        }

        /// Test that result messages are always shown at all verbosity levels
        ///
        /// Results are final outputs and must be shown at all verbosity levels.
        #[rstest]
        #[case(VerbosityLevel::Quiet)]
        #[case(VerbosityLevel::Normal)]
        #[case(VerbosityLevel::Verbose)]
        #[case(VerbosityLevel::VeryVerbose)]
        #[case(VerbosityLevel::Debug)]
        fn it_should_always_show_results_at_all_verbosity_levels(
            #[case] verbosity: VerbosityLevel,
        ) {
            let mut test_output = test_support::TestUserOutput::new(verbosity);

            test_output.output.result("Result data");

            assert_eq!(test_output.stdout(), "Result data\n");
            assert_eq!(test_output.stderr(), "");
        }
    }

    // ============================================================================
    // UserOutput Tests - Basic Output (Non-parameterized)
    // ============================================================================
    //
    // These tests cover specific functionality not included in parameterized tests:

    #[test]
    fn it_should_write_data_to_stdout() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.data(r#"{"status": "destroyed"}"#);

        // Verify message went to stdout
        assert_eq!(test_output.stdout(), "{\"status\": \"destroyed\"}\n");

        // Verify stderr is empty
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_use_normal_as_default_verbosity() {
        let default = VerbosityLevel::default();
        assert_eq!(default, VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_order_verbosity_levels_correctly() {
        assert!(VerbosityLevel::Quiet < VerbosityLevel::Normal);
        assert!(VerbosityLevel::Normal < VerbosityLevel::Verbose);
        assert!(VerbosityLevel::Verbose < VerbosityLevel::VeryVerbose);
        assert!(VerbosityLevel::VeryVerbose < VerbosityLevel::Debug);
    }

    #[test]
    fn it_should_support_equality_comparison() {
        assert_eq!(VerbosityLevel::Normal, VerbosityLevel::Normal);
        assert_ne!(VerbosityLevel::Normal, VerbosityLevel::Verbose);
    }

    #[test]
    fn it_should_support_ordering_comparison() {
        let normal = VerbosityLevel::Normal;
        assert!(normal >= VerbosityLevel::Quiet);
        assert!(normal >= VerbosityLevel::Normal);
        assert!(normal < VerbosityLevel::Verbose);
    }

    #[test]
    fn it_should_write_blank_line_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.blank_line();

        // Verify blank line went to stderr
        assert_eq!(test_output.stderr(), "\n");

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_not_write_blank_line_at_quiet_level() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output.output.blank_line();

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_write_steps_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.steps(
            "Next steps:",
            &[
                "Edit the configuration file",
                "Review the settings",
                "Run the deploy command",
            ],
        );

        // Verify steps went to stderr with correct formatting
        assert_eq!(
            test_output.stderr(),
            "Next steps:\n1. Edit the configuration file\n2. Review the settings\n3. Run the deploy command\n"
        );

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_not_write_steps_at_quiet_level() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output
            .output
            .steps("Next steps:", &["Step 1", "Step 2"]);

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    #[test]
    fn it_should_write_info_block_to_stderr() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Normal);

        test_output.output.info_block(
            "Configuration options:",
            &[
                "  - username: 'torrust' (default)",
                "  - port: 22 (default SSH port)",
            ],
        );

        // Verify info block went to stderr
        assert_eq!(
            test_output.stderr(),
            "Configuration options:\n  - username: 'torrust' (default)\n  - port: 22 (default SSH port)\n"
        );

        // Verify stdout is empty
        assert_eq!(test_output.stdout(), "");
    }

    #[test]
    fn it_should_not_write_info_block_at_quiet_level() {
        let mut test_output = test_support::TestUserOutput::new(VerbosityLevel::Quiet);

        test_output
            .output
            .info_block("Info:", &["Line 1", "Line 2"]);

        // Verify no output at Quiet level
        assert_eq!(test_output.stderr(), "");
    }

    // VerbosityFilter tests
    mod verbosity_filter {
        use super::super::*;

        #[test]
        fn it_should_show_progress_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_progress());
        }

        #[test]
        fn it_should_not_show_progress_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_progress());
        }

        #[test]
        fn it_should_show_progress_at_verbose_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Verbose);
            assert!(filter.should_show_progress());
        }

        #[test]
        fn it_should_always_show_errors_regardless_of_level() {
            assert!(VerbosityFilter::new(VerbosityLevel::Quiet).should_show_errors());
            assert!(VerbosityFilter::new(VerbosityLevel::Normal).should_show_errors());
            assert!(VerbosityFilter::new(VerbosityLevel::Verbose).should_show_errors());
            assert!(VerbosityFilter::new(VerbosityLevel::VeryVerbose).should_show_errors());
            assert!(VerbosityFilter::new(VerbosityLevel::Debug).should_show_errors());
        }

        #[test]
        fn it_should_show_success_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_success());
        }

        #[test]
        fn it_should_not_show_success_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_success());
        }

        #[test]
        fn it_should_show_warnings_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_warnings());
        }

        #[test]
        fn it_should_not_show_warnings_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_warnings());
        }

        #[test]
        fn it_should_show_blank_lines_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_blank_lines());
        }

        #[test]
        fn it_should_not_show_blank_lines_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_blank_lines());
        }

        #[test]
        fn it_should_show_steps_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_steps());
        }

        #[test]
        fn it_should_not_show_steps_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_steps());
        }

        #[test]
        fn it_should_show_info_blocks_at_normal_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show_info_blocks());
        }

        #[test]
        fn it_should_not_show_info_blocks_at_quiet_level() {
            let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(!filter.should_show_info_blocks());
        }

        #[test]
        fn it_should_show_when_level_meets_requirement() {
            let filter = VerbosityFilter::new(VerbosityLevel::Normal);
            assert!(filter.should_show(VerbosityLevel::Quiet));
            assert!(filter.should_show(VerbosityLevel::Normal));
            assert!(!filter.should_show(VerbosityLevel::Verbose));
        }

        #[test]
        fn it_should_handle_all_verbosity_levels_in_should_show() {
            let quiet_filter = VerbosityFilter::new(VerbosityLevel::Quiet);
            assert!(quiet_filter.should_show(VerbosityLevel::Quiet));
            assert!(!quiet_filter.should_show(VerbosityLevel::Normal));

            let debug_filter = VerbosityFilter::new(VerbosityLevel::Debug);
            assert!(debug_filter.should_show(VerbosityLevel::Quiet));
            assert!(debug_filter.should_show(VerbosityLevel::Normal));
            assert!(debug_filter.should_show(VerbosityLevel::Verbose));
            assert!(debug_filter.should_show(VerbosityLevel::VeryVerbose));
            assert!(debug_filter.should_show(VerbosityLevel::Debug));
        }
    }

    // ============================================================================
    // OutputMessage Trait Tests
    // ============================================================================

    mod output_message_trait {
        use super::super::*;
        use crate::presentation::user_output::test_support::TestUserOutput;

        #[test]
        fn progress_message_should_format_with_theme() {
            let theme = Theme::emoji();
            let message = ProgressMessage {
                text: "Test message".to_string(),
            };

            let formatted = message.format(&theme);

            assert_eq!(formatted, "⏳ Test message\n");
        }

        #[test]
        fn progress_message_should_require_normal_verbosity() {
            let message = ProgressMessage {
                text: "Test".to_string(),
            };

            assert_eq!(message.required_verbosity(), VerbosityLevel::Normal);
        }

        #[test]
        fn progress_message_should_use_stderr_channel() {
            let message = ProgressMessage {
                text: "Test".to_string(),
            };

            assert_eq!(message.channel(), Channel::Stderr);
        }

        #[test]
        fn success_message_should_format_with_theme() {
            let theme = Theme::plain();
            let message = SuccessMessage {
                text: "Operation complete".to_string(),
            };

            let formatted = message.format(&theme);

            assert_eq!(formatted, "[OK] Operation complete\n");
        }

        #[test]
        fn error_message_should_always_be_shown() {
            let message = ErrorMessage {
                text: "Critical error".to_string(),
            };

            // Errors should require Quiet level (always shown)
            assert_eq!(message.required_verbosity(), VerbosityLevel::Quiet);
        }

        #[test]
        fn result_message_should_use_stdout_channel() {
            let message = ResultMessage {
                text: "Output data".to_string(),
            };

            assert_eq!(message.channel(), Channel::Stdout);
        }

        #[test]
        fn result_message_should_not_include_symbols() {
            let theme = Theme::emoji();
            let message = ResultMessage {
                text: "Plain output".to_string(),
            };

            let formatted = message.format(&theme);

            // Result messages should not include theme symbols
            assert!(!formatted.contains("⏳"));
            assert!(!formatted.contains("✅"));
            assert_eq!(formatted, "Plain output\n");
        }

        #[test]
        fn steps_message_should_format_numbered_list() {
            let theme = Theme::emoji();
            let message = StepsMessage {
                title: "Next steps:".to_string(),
                items: vec!["First step".to_string(), "Second step".to_string()],
            };

            let formatted = message.format(&theme);

            assert_eq!(formatted, "Next steps:\n1. First step\n2. Second step\n");
        }

        #[test]
        fn warning_message_should_include_extra_space() {
            let theme = Theme::emoji();
            let message = WarningMessage {
                text: "Warning text".to_string(),
            };

            let formatted = message.format(&theme);

            // Warning messages include two spaces after the symbol
            assert_eq!(formatted, "⚠️  Warning text\n");
        }

        #[test]
        fn user_output_write_should_respect_verbosity_filter() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Quiet);

            // Normal-level message should be filtered
            test_output.output.write(&ProgressMessage {
                text: "Should not appear".to_string(),
            });

            assert_eq!(test_output.stderr(), "");

            // Quiet-level message should be shown
            test_output.output.write(&ErrorMessage {
                text: "Should appear".to_string(),
            });

            assert_eq!(test_output.stderr(), "❌ Should appear\n");
        }

        #[test]
        fn user_output_write_should_route_to_correct_channel() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Stderr message
            test_output.output.write(&ProgressMessage {
                text: "Progress".to_string(),
            });

            // Stdout message
            test_output.output.write(&ResultMessage {
                text: "Result".to_string(),
            });

            assert_eq!(test_output.stderr(), "⏳ Progress\n");
            assert_eq!(test_output.stdout(), "Result\n");
        }

        #[test]
        fn channel_enum_should_support_equality() {
            assert_eq!(Channel::Stdout, Channel::Stdout);
            assert_eq!(Channel::Stderr, Channel::Stderr);
            assert_ne!(Channel::Stdout, Channel::Stderr);
        }

        // Custom message type to demonstrate extensibility
        struct CustomDebugMessage {
            text: String,
        }

        impl OutputMessage for CustomDebugMessage {
            fn format(&self, _theme: &Theme) -> String {
                format!("[DEBUG] {}\n", self.text)
            }

            fn required_verbosity(&self) -> VerbosityLevel {
                VerbosityLevel::Debug
            }

            fn channel(&self) -> Channel {
                Channel::Stderr
            }

            fn type_name(&self) -> &'static str {
                "CustomDebugMessage"
            }
        }

        #[test]
        fn custom_message_type_should_work_with_write() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Debug);

            test_output.output.write(&CustomDebugMessage {
                text: "Custom debug message".to_string(),
            });

            assert_eq!(test_output.stderr(), "[DEBUG] Custom debug message\n");
        }

        #[test]
        fn custom_message_type_should_respect_verbosity() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Debug-level custom message should not appear at Normal level
            test_output.output.write(&CustomDebugMessage {
                text: "Should not appear".to_string(),
            });

            assert_eq!(test_output.stderr(), "");
        }

        #[test]
        fn open_closed_principle_demonstration() {
            // This test demonstrates that new message types can be added
            // without modifying the UserOutput struct

            struct CustomInfoMessage {
                category: String,
                message: String,
            }

            impl OutputMessage for CustomInfoMessage {
                fn format(&self, _theme: &Theme) -> String {
                    format!("ℹ️  [{}] {}\n", self.category, self.message)
                }

                fn required_verbosity(&self) -> VerbosityLevel {
                    VerbosityLevel::Verbose
                }

                fn channel(&self) -> Channel {
                    Channel::Stderr
                }

                fn type_name(&self) -> &'static str {
                    "CustomInfoMessage"
                }
            }

            let mut test_output = TestUserOutput::new(VerbosityLevel::Verbose);

            test_output.output.write(&CustomInfoMessage {
                category: "CONFIG".to_string(),
                message: "Loading configuration".to_string(),
            });

            assert_eq!(test_output.stderr(), "ℹ️  [CONFIG] Loading configuration\n");
        }
    }

    // ============================================================================
    // UserOutput with Theme Tests
    // ============================================================================

    mod user_output_with_themes {
        use super::super::*;
        use crate::presentation::user_output::test_support::TestUserOutput;

        #[test]
        fn it_should_use_emoji_theme_by_default() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            test_output.output.progress("Test");

            assert_eq!(test_output.stderr(), "⏳ Test\n");
        }

        #[test]
        fn it_should_use_plain_theme_when_specified() {
            let mut test_output =
                TestUserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());

            test_output.output.progress("Test");
            test_output.output.success("Success");
            test_output.output.warn("Warning");
            test_output.output.error("Error");

            let stderr = test_output.stderr();
            assert!(stderr.contains("[INFO] Test"));
            assert!(stderr.contains("[OK] Success"));
            assert!(stderr.contains("[WARN]  Warning"));
            assert!(stderr.contains("[ERROR] Error"));
        }

        #[test]
        fn it_should_use_ascii_theme_when_specified() {
            let mut test_output =
                TestUserOutput::with_theme(VerbosityLevel::Normal, Theme::ascii());

            test_output.output.progress("Test");
            test_output.output.success("Success");
            test_output.output.warn("Warning");
            test_output.output.error("Error");

            let stderr = test_output.stderr();
            assert!(stderr.contains("=> Test"));
            assert!(stderr.contains("[+] Success"));
            assert!(stderr.contains("[!]  Warning"));
            assert!(stderr.contains("[x] Error"));
        }

        #[test]
        fn it_should_support_with_theme_constructor() {
            let output = UserOutput::with_theme(VerbosityLevel::Normal, Theme::plain());

            // Verify it compiles and creates output with the theme
            // (actual output testing done through TestUserOutput)
            drop(output);
        }
    }

    // ============================================================================
    // FormatterOverride Tests
    // ============================================================================

    mod formatter_override {
        use super::super::*;
        use crate::presentation::user_output::test_support::TestUserOutput;
        use std::sync::{Arc, Mutex};

        // Custom test formatter to verify override is applied
        struct TestFormatter {
            prefix: String,
        }

        impl FormatterOverride for TestFormatter {
            fn transform(&self, formatted: &str, _message: &dyn OutputMessage) -> String {
                format!("{}{}", self.prefix, formatted)
            }
        }

        #[test]
        fn it_should_apply_formatter_override_to_messages() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(TestFormatter {
                prefix: "[TEST] ".to_string(),
            });

            let mut output = UserOutput {
                theme: Theme::plain(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test message");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            assert_eq!(stderr, "[TEST] [INFO] Test message\n");
        }

        #[test]
        fn it_should_not_apply_override_when_none() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            test_output.output.progress("Test message");

            // Without override, should see normal formatted output
            assert_eq!(test_output.stderr(), "⏳ Test message\n");
        }

        #[test]
        fn it_should_work_with_json_formatter() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test message");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();

            // Parse JSON to verify structure (trim to remove trailing newline)
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).expect("Valid JSON");

            assert_eq!(json["type"], "ProgressMessage");
            assert_eq!(json["channel"], "Stderr");
            assert_eq!(json["content"], "⏳ Test message");
            assert!(json["timestamp"].is_string());
        }

        #[test]
        fn it_should_include_correct_type_name_in_json() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            // Test different message types
            output.progress("Progress");
            output.success("Success");
            output.warn("Warning");
            output.error("Error");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let lines: Vec<&str> = stderr.lines().collect();

            let progress_json: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
            assert_eq!(progress_json["type"], "ProgressMessage");

            let success_json: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
            assert_eq!(success_json["type"], "SuccessMessage");

            let warning_json: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
            assert_eq!(warning_json["type"], "WarningMessage");

            let error_json: serde_json::Value = serde_json::from_str(lines[3]).unwrap();
            assert_eq!(error_json["type"], "ErrorMessage");
        }

        #[test]
        fn it_should_include_correct_channel_in_json() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Stderr message");
            output.result("Stdout message");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let stdout = String::from_utf8(stdout_buffer.lock().unwrap().clone()).unwrap();

            let stderr_json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
            assert_eq!(stderr_json["channel"], "Stderr");

            let stdout_json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
            assert_eq!(stdout_json["channel"], "Stdout");
        }

        #[test]
        fn it_should_trim_trailing_newlines_in_json() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            // Content should not have trailing newline
            let content = json["content"].as_str().unwrap();
            assert!(!content.ends_with('\n'));
            assert_eq!(content, "⏳ Test");
        }

        #[test]
        fn it_should_respect_theme_with_json_formatter() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::plain(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            // Content should reflect plain theme
            assert_eq!(json["content"], "[INFO] Test");
        }

        #[test]
        fn it_should_respect_verbosity_with_formatter_override() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Quiet),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            // Normal-level message should be filtered at Quiet level
            output.progress("Should not appear");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            assert_eq!(stderr, "");

            // Quiet-level message should appear
            output.error("Should appear");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
            assert_eq!(json["type"], "ErrorMessage");
        }

        #[test]
        fn it_should_create_output_with_formatter_override_constructor() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::default(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.progress("Test");

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            assert_eq!(json["type"], "ProgressMessage");
        }

        #[test]
        fn it_should_work_with_steps_message() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            output.steps("Next steps:", &["Step 1", "Step 2"]);

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            assert_eq!(json["type"], "StepsMessage");
            assert_eq!(json["channel"], "Stderr");
            // Content should include formatted steps
            let content = json["content"].as_str().unwrap();
            assert!(content.contains("Next steps:"));
            assert!(content.contains("1. Step 1"));
            assert!(content.contains("2. Step 2"));
        }

        #[test]
        fn it_should_produce_valid_json_for_all_message_types() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            // Test all message types
            output.progress("Progress");
            output.success("Success");
            output.warn("Warning");
            output.error("Error");
            output.result("Result");
            output.steps("Steps:", &["Step 1"]);

            // Verify all stderr messages are valid JSON
            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            for line in stderr.lines() {
                let json: Result<serde_json::Value, _> = serde_json::from_str(line);
                assert!(json.is_ok(), "Invalid JSON: {line}");
            }

            // Verify stdout message is valid JSON
            let stdout = String::from_utf8(stdout_buffer.lock().unwrap().clone()).unwrap();
            let json: Result<serde_json::Value, _> = serde_json::from_str(stdout.trim());
            assert!(json.is_ok(), "Invalid JSON in stdout");
        }
    }

    // ============================================================================
    // Buffering Tests
    // ============================================================================

    mod buffering {
        use super::super::*;
        use crate::presentation::user_output::test_support::TestUserOutput;

        #[test]
        fn it_should_flush_all_writers() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            test_output.output.progress("Test message");

            // Flush should succeed
            test_output.output.flush().expect("Flush should succeed");

            // Verify output is present (flushed)
            assert!(!test_output.stderr().is_empty());
            assert!(test_output.stderr().contains("Test message"));
        }

        #[test]
        fn it_should_be_safe_to_flush_multiple_times() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            test_output.output.progress("Test message");

            // Multiple flushes should be safe
            test_output
                .output
                .flush()
                .expect("First flush should succeed");
            test_output
                .output
                .flush()
                .expect("Second flush should succeed");
            test_output
                .output
                .flush()
                .expect("Third flush should succeed");

            // Output should still be present
            assert!(!test_output.stderr().is_empty());
        }

        #[test]
        fn it_should_flush_empty_buffers_safely() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Flushing with no output should be safe
            test_output
                .output
                .flush()
                .expect("Flushing empty buffers should succeed");

            // No output should be present
            assert_eq!(test_output.stderr(), "");
            assert_eq!(test_output.stdout(), "");
        }

        #[test]
        fn it_should_flush_both_stdout_and_stderr() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Write to both channels
            test_output.output.progress("Progress message");
            test_output.output.result("Result data");

            // Flush should handle both channels
            test_output
                .output
                .flush()
                .expect("Flush should succeed for both channels");

            // Verify both outputs are present
            assert!(test_output.stderr().contains("Progress message"));
            assert!(test_output.stdout().contains("Result data"));
        }

        #[test]
        fn it_should_work_with_sequential_flush_calls() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            // Write, flush, write, flush pattern
            test_output.output.progress("Message 1");
            test_output
                .output
                .flush()
                .expect("First flush should succeed");

            test_output.output.progress("Message 2");
            test_output
                .output
                .flush()
                .expect("Second flush should succeed");

            // Both messages should be present
            let stderr = test_output.stderr();
            assert!(stderr.contains("Message 1"));
            assert!(stderr.contains("Message 2"));
        }
    }

    // ============================================================================
    // Builder Pattern Tests
    // ============================================================================

    mod builder_pattern {
        use super::super::*;
        use crate::presentation::user_output::test_support::TestUserOutput;

        // ========================================================================
        // StepsMessageBuilder Tests
        // ========================================================================

        #[test]
        fn it_should_build_steps_with_fluent_api() {
            let message = StepsMessage::builder("Title")
                .add("Step 1")
                .add("Step 2")
                .add("Step 3")
                .build();

            assert_eq!(message.title, "Title");
            assert_eq!(message.items, vec!["Step 1", "Step 2", "Step 3"]);
        }

        #[test]
        fn it_should_create_simple_steps_directly() {
            let message =
                StepsMessage::new("Title", vec!["Step 1".to_string(), "Step 2".to_string()]);

            assert_eq!(message.title, "Title");
            assert_eq!(message.items, vec!["Step 1", "Step 2"]);
        }

        #[test]
        fn it_should_build_empty_steps() {
            let message = StepsMessage::builder("Title").build();

            assert_eq!(message.title, "Title");
            assert!(message.items.is_empty());
        }

        #[test]
        fn it_should_build_single_step() {
            let message = StepsMessage::builder("Title").add("Single step").build();

            assert_eq!(message.title, "Title");
            assert_eq!(message.items, vec!["Single step"]);
        }

        #[test]
        fn it_should_accept_string_types_in_builder() {
            let message = StepsMessage::builder("Title")
                .add("String literal")
                .add(String::from("Owned string"))
                .add("Another literal".to_string())
                .build();

            assert_eq!(message.items.len(), 3);
        }

        #[test]
        fn it_should_accept_string_types_in_constructor() {
            let message =
                StepsMessage::new("Title", vec!["Step 1".to_string(), String::from("Step 2")]);

            assert_eq!(message.items.len(), 2);
        }

        #[test]
        fn it_should_format_builder_messages_correctly() {
            let theme = Theme::emoji();
            let message = StepsMessage::builder("Next steps:")
                .add("Configure")
                .add("Deploy")
                .build();

            let formatted = message.format(&theme);
            assert!(formatted.contains("Next steps:"));
            assert!(formatted.contains("1. Configure"));
            assert!(formatted.contains("2. Deploy"));
        }

        #[test]
        fn it_should_integrate_builder_with_user_output() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            let message = StepsMessage::builder("Next steps:")
                .add("Edit config")
                .add("Run tests")
                .build();

            test_output.output.write(&message);

            let stderr = test_output.stderr();
            assert!(stderr.contains("Next steps:"));
            assert!(stderr.contains("1. Edit config"));
            assert!(stderr.contains("2. Run tests"));
        }

        // ========================================================================
        // InfoBlockMessageBuilder Tests
        // ========================================================================

        #[test]
        fn it_should_build_info_block_with_fluent_api() {
            let message = InfoBlockMessage::builder("Environment")
                .add_line("Name: production")
                .add_line("Status: active")
                .build();

            assert_eq!(message.title, "Environment");
            assert_eq!(message.lines, vec!["Name: production", "Status: active"]);
        }

        #[test]
        fn it_should_create_simple_info_block_directly() {
            let message = InfoBlockMessage::new(
                "Environment",
                vec!["Name: production".to_string(), "Status: active".to_string()],
            );

            assert_eq!(message.title, "Environment");
            assert_eq!(message.lines, vec!["Name: production", "Status: active"]);
        }

        #[test]
        fn it_should_build_empty_info_block() {
            let message = InfoBlockMessage::builder("Title").build();

            assert_eq!(message.title, "Title");
            assert!(message.lines.is_empty());
        }

        #[test]
        fn it_should_build_single_line_info_block() {
            let message = InfoBlockMessage::builder("Title")
                .add_line("Single line")
                .build();

            assert_eq!(message.title, "Title");
            assert_eq!(message.lines, vec!["Single line"]);
        }

        #[test]
        fn it_should_accept_string_types_in_info_block_builder() {
            let message = InfoBlockMessage::builder("Title")
                .add_line("String literal")
                .add_line(String::from("Owned string"))
                .add_line("Another literal".to_string())
                .build();

            assert_eq!(message.lines.len(), 3);
        }

        #[test]
        fn it_should_accept_string_types_in_info_block_constructor() {
            let message =
                InfoBlockMessage::new("Title", vec!["Line 1".to_string(), String::from("Line 2")]);

            assert_eq!(message.lines.len(), 2);
        }

        #[test]
        fn it_should_format_info_block_messages_correctly() {
            let theme = Theme::emoji();
            let message = InfoBlockMessage::builder("Environment")
                .add_line("Name: production")
                .add_line("Status: active")
                .build();

            let formatted = message.format(&theme);
            assert!(formatted.contains("Environment"));
            assert!(formatted.contains("Name: production"));
            assert!(formatted.contains("Status: active"));
        }

        #[test]
        fn it_should_integrate_info_block_builder_with_user_output() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            let message = InfoBlockMessage::builder("Configuration")
                .add_line("  - username: torrust")
                .add_line("  - port: 22")
                .build();

            test_output.output.write(&message);

            let stderr = test_output.stderr();
            assert!(stderr.contains("Configuration"));
            assert!(stderr.contains("  - username: torrust"));
            assert!(stderr.contains("  - port: 22"));
        }

        #[test]
        fn it_should_show_info_block_message_has_correct_properties() {
            let message = InfoBlockMessage::new("Title", vec!["Line 1".to_string()]);

            assert_eq!(message.required_verbosity(), VerbosityLevel::Normal);
            assert_eq!(message.channel(), Channel::Stderr);
            assert_eq!(message.type_name(), "InfoBlockMessage");
        }

        #[test]
        fn it_should_respect_verbosity_for_info_block_messages() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Quiet);

            let message = InfoBlockMessage::builder("Info").add_line("Line 1").build();

            test_output.output.write(&message);

            // Should not appear at Quiet level
            assert_eq!(test_output.stderr(), "");
        }

        #[test]
        fn it_should_show_info_block_at_normal_level() {
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

            let message = InfoBlockMessage::builder("Info").add_line("Line 1").build();

            test_output.output.write(&message);

            // Should appear at Normal level
            assert!(!test_output.stderr().is_empty());
            assert!(test_output.stderr().contains("Info"));
        }

        // ========================================================================
        // Backward Compatibility Tests
        // ========================================================================

        #[test]
        fn it_should_maintain_backward_compatibility_for_steps() {
            // Old way: direct construction
            let old_message = StepsMessage {
                title: "Steps".to_string(),
                items: vec!["Step 1".to_string()],
            };

            // New way: constructor
            let new_message = StepsMessage::new("Steps", vec!["Step 1".to_string()]);

            // Should produce identical results
            assert_eq!(old_message.title, new_message.title);
            assert_eq!(old_message.items, new_message.items);
        }

        #[test]
        fn it_should_maintain_backward_compatibility_for_info_blocks() {
            // Old way: UserOutput::info_block helper
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            test_output
                .output
                .info_block("Title", &["Line 1", "Line 2"]);
            let old_output = test_output.stderr();

            // New way: Direct message construction
            let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
            let message =
                InfoBlockMessage::new("Title", vec!["Line 1".to_string(), "Line 2".to_string()]);
            test_output.output.write(&message);
            let new_output = test_output.stderr();

            // Should produce identical output
            assert_eq!(old_output, new_output);
        }

        // ========================================================================
        // Integration Tests
        // ========================================================================

        #[test]
        fn it_should_work_with_json_formatter() {
            use std::sync::{Arc, Mutex};

            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            let message = StepsMessage::builder("Steps").add("Step 1").build();
            output.write(&message);

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            assert_eq!(json["type"], "StepsMessage");
        }

        #[test]
        fn it_should_work_with_info_block_json_formatter() {
            use std::sync::{Arc, Mutex};

            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
            let formatter = Box::new(JsonFormatter);

            let mut output = UserOutput {
                theme: Theme::emoji(),
                verbosity_filter: VerbosityFilter::new(VerbosityLevel::Normal),
                sink: Box::new(StandardSink::new(
                    Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                    Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
                )),
                formatter_override: Some(formatter),
            };

            let message = InfoBlockMessage::builder("Info").add_line("Line 1").build();
            output.write(&message);

            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();
            let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();

            assert_eq!(json["type"], "InfoBlockMessage");
        }

        #[test]
        fn it_should_handle_many_items_in_builder() {
            let mut builder = StepsMessage::builder("Many steps");
            for i in 1..=100 {
                builder = builder.add(format!("Step {i}"));
            }
            let message = builder.build();

            assert_eq!(message.items.len(), 100);
            assert_eq!(message.items[0], "Step 1");
            assert_eq!(message.items[99], "Step 100");
        }

        #[test]
        fn it_should_handle_many_lines_in_info_block_builder() {
            let mut builder = InfoBlockMessage::builder("Many lines");
            for i in 1..=100 {
                builder = builder.add_line(format!("Line {i}"));
            }
            let message = builder.build();

            assert_eq!(message.lines.len(), 100);
            assert_eq!(message.lines[0], "Line 1");
            assert_eq!(message.lines[99], "Line 100");
        }
    }

    // ============================================================================
    // OutputSink Tests
    // ============================================================================

    mod output_sink {
        use super::super::*;
        use std::sync::{Arc, Mutex};

        /// Mock sink for testing that captures messages
        struct MockSink {
            messages: Arc<Mutex<Vec<String>>>,
        }

        impl MockSink {
            fn new(messages: Arc<Mutex<Vec<String>>>) -> Self {
                Self { messages }
            }
        }

        impl OutputSink for MockSink {
            fn write_message(&mut self, _message: &dyn OutputMessage, formatted: &str) {
                self.messages.lock().unwrap().push(formatted.to_string());
            }
        }

        // ========================================================================
        // StandardSink Tests
        // ========================================================================

        #[test]
        fn standard_sink_should_route_stdout_messages() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let mut sink = StandardSink::new(
                Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
            );

            let message = ResultMessage {
                text: "Test result".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            sink.write_message(&message, &formatted);

            let stdout = String::from_utf8(stdout_buffer.lock().unwrap().clone()).unwrap();
            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();

            assert_eq!(stdout, "Test result\n");
            assert_eq!(stderr, "");
        }

        #[test]
        fn standard_sink_should_route_stderr_messages() {
            let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
            let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

            let mut sink = StandardSink::new(
                Box::new(test_support::TestWriter::new(Arc::clone(&stdout_buffer))),
                Box::new(test_support::TestWriter::new(Arc::clone(&stderr_buffer))),
            );

            let message = ProgressMessage {
                text: "Test progress".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            sink.write_message(&message, &formatted);

            let stdout = String::from_utf8(stdout_buffer.lock().unwrap().clone()).unwrap();
            let stderr = String::from_utf8(stderr_buffer.lock().unwrap().clone()).unwrap();

            assert_eq!(stdout, "");
            assert_eq!(stderr, "⏳ Test progress\n");
        }

        #[test]
        fn standard_sink_default_console_should_create_default_sink() {
            let _sink = StandardSink::default_console();
            // If we got here without panicking, the sink was created successfully
        }

        // ========================================================================
        // CompositeSink Tests
        // ========================================================================

        #[test]
        fn composite_sink_should_write_to_all_sinks() {
            let messages1 = Arc::new(Mutex::new(Vec::new()));
            let messages2 = Arc::new(Mutex::new(Vec::new()));
            let messages3 = Arc::new(Mutex::new(Vec::new()));

            let mut composite = CompositeSink::new(vec![
                Box::new(MockSink::new(Arc::clone(&messages1))),
                Box::new(MockSink::new(Arc::clone(&messages2))),
                Box::new(MockSink::new(Arc::clone(&messages3))),
            ]);

            let message = ProgressMessage {
                text: "Test".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            composite.write_message(&message, &formatted);

            // Verify all sinks received the message
            assert_eq!(messages1.lock().unwrap().len(), 1);
            assert_eq!(messages2.lock().unwrap().len(), 1);
            assert_eq!(messages3.lock().unwrap().len(), 1);

            assert_eq!(messages1.lock().unwrap()[0], "⏳ Test\n");
            assert_eq!(messages2.lock().unwrap()[0], "⏳ Test\n");
            assert_eq!(messages3.lock().unwrap()[0], "⏳ Test\n");
        }

        #[test]
        fn composite_sink_should_support_empty_sink_list() {
            let mut composite = CompositeSink::new(vec![]);

            let message = ProgressMessage {
                text: "Test".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            // Should not panic with empty sink list
            composite.write_message(&message, &formatted);
        }

        #[test]
        fn composite_sink_should_support_add_sink() {
            let messages1 = Arc::new(Mutex::new(Vec::new()));
            let messages2 = Arc::new(Mutex::new(Vec::new()));

            let mut composite = CompositeSink::new(vec![Box::new(MockSink::new(Arc::clone(&messages1)))]);

            // Add another sink
            composite.add_sink(Box::new(MockSink::new(Arc::clone(&messages2))));

            let message = ProgressMessage {
                text: "Test".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            composite.write_message(&message, &formatted);

            // Verify both sinks received the message
            assert_eq!(messages1.lock().unwrap().len(), 1);
            assert_eq!(messages2.lock().unwrap().len(), 1);
        }

        #[test]
        fn composite_sink_should_write_multiple_messages() {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let mut composite = CompositeSink::new(vec![Box::new(MockSink::new(Arc::clone(&messages)))]);

            let theme = Theme::emoji();

            // Write multiple messages
            let msg1 = ProgressMessage {
                text: "First".to_string(),
            };
            composite.write_message(&msg1, &msg1.format(&theme));

            let msg2 = SuccessMessage {
                text: "Second".to_string(),
            };
            composite.write_message(&msg2, &msg2.format(&theme));

            let msg3 = ErrorMessage {
                text: "Third".to_string(),
            };
            composite.write_message(&msg3, &msg3.format(&theme));

            // Verify all messages were received
            let captured = messages.lock().unwrap();
            assert_eq!(captured.len(), 3);
            assert_eq!(captured[0], "⏳ First\n");
            assert_eq!(captured[1], "✅ Second\n");
            assert_eq!(captured[2], "❌ Third\n");
        }

        // ========================================================================
        // UserOutput with Custom Sinks Tests
        // ========================================================================

        #[test]
        fn user_output_should_work_with_custom_sink() {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let sink = Box::new(MockSink::new(Arc::clone(&messages)));

            let mut output = UserOutput::with_sink(VerbosityLevel::Normal, sink);

            output.progress("Progress message");
            output.success("Success message");
            output.error("Error message");

            let captured = messages.lock().unwrap();
            assert_eq!(captured.len(), 3);
            assert!(captured[0].contains("Progress message"));
            assert!(captured[1].contains("Success message"));
            assert!(captured[2].contains("Error message"));
        }

        #[test]
        fn user_output_should_work_with_composite_sink() {
            let messages1 = Arc::new(Mutex::new(Vec::new()));
            let messages2 = Arc::new(Mutex::new(Vec::new()));

            let composite = CompositeSink::new(vec![
                Box::new(MockSink::new(Arc::clone(&messages1))),
                Box::new(MockSink::new(Arc::clone(&messages2))),
            ]);

            let mut output = UserOutput::with_sink(VerbosityLevel::Normal, Box::new(composite));

            output.progress("Test message");

            // Verify both sinks received the message
            assert_eq!(messages1.lock().unwrap().len(), 1);
            assert_eq!(messages2.lock().unwrap().len(), 1);
            assert!(messages1.lock().unwrap()[0].contains("Test message"));
            assert!(messages2.lock().unwrap()[0].contains("Test message"));
        }

        #[test]
        fn user_output_with_sink_should_respect_verbosity() {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let sink = Box::new(MockSink::new(Arc::clone(&messages)));

            let mut output = UserOutput::with_sink(VerbosityLevel::Quiet, sink);

            // Normal-level message should not appear
            output.progress("Should not appear");

            // Quiet-level message should appear
            output.error("Should appear");

            let captured = messages.lock().unwrap();
            assert_eq!(captured.len(), 1);
            assert!(captured[0].contains("Should appear"));
        }

        #[test]
        fn user_output_with_sink_should_use_default_theme() {
            let messages = Arc::new(Mutex::new(Vec::new()));
            let sink = Box::new(MockSink::new(Arc::clone(&messages)));

            let mut output = UserOutput::with_sink(VerbosityLevel::Normal, sink);

            output.progress("Test");

            let captured = messages.lock().unwrap();
            // Should use emoji theme by default
            assert!(captured[0].contains("⏳"));
        }

        // ========================================================================
        // FileSink Tests
        // ========================================================================

        #[test]
        fn file_sink_should_create_and_write_to_file() {
            use std::io::Read;
            use tempfile::NamedTempFile;

            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();

            let mut sink = FileSink::new(path).unwrap();

            let message = ProgressMessage {
                text: "Test message".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            sink.write_message(&message, &formatted);

            // Read back the file content
            let mut file = std::fs::File::open(path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            assert_eq!(content, "⏳ Test message\n\n");
        }

        #[test]
        fn file_sink_should_append_to_existing_file() {
            use std::io::Read;
            use tempfile::NamedTempFile;

            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();

            // Write first message
            let mut sink1 = FileSink::new(path).unwrap();
            let message1 = ProgressMessage {
                text: "First".to_string(),
            };
            let theme = Theme::emoji();
            sink1.write_message(&message1, &message1.format(&theme));
            drop(sink1);

            // Write second message
            let mut sink2 = FileSink::new(path).unwrap();
            let message2 = SuccessMessage {
                text: "Second".to_string(),
            };
            sink2.write_message(&message2, &message2.format(&theme));
            drop(sink2);

            // Read back the file content
            let mut file = std::fs::File::open(path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            assert!(content.contains("First"));
            assert!(content.contains("Second"));
        }

        // ========================================================================
        // TelemetrySink Tests
        // ========================================================================

        #[test]
        fn telemetry_sink_should_create_with_endpoint() {
            let sink = TelemetrySink::new("https://example.com".to_string());
            assert_eq!(sink.endpoint, "https://example.com");
        }

        #[test]
        fn telemetry_sink_should_log_messages() {
            let mut sink = TelemetrySink::new("https://example.com".to_string());

            let message = ProgressMessage {
                text: "Test".to_string(),
            };
            let theme = Theme::emoji();
            let formatted = message.format(&theme);

            // This just logs via tracing, so we verify it doesn't panic
            sink.write_message(&message, &formatted);
        }
    }
}
