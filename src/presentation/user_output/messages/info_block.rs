//! Info block message type for structured information

use super::super::{Channel, OutputMessage, Theme, VerbosityLevel};

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
