//! Steps message type for sequential instructions

use super::super::{Channel, OutputMessage, Theme, VerbosityLevel};

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
/// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessage;
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
/// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessage;
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
    /// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessage;
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
    /// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessage;
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
/// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessage;
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
/// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessage;
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
    /// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessageBuilder;
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
    /// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessage;
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
    /// use torrust_tracker_deployer_lib::presentation::cli::views::StepsMessage;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_format_numbered_list_when_displaying_steps() {
        let theme = Theme::emoji();
        let message = StepsMessage {
            title: "Next steps:".to_string(),
            items: vec!["First step".to_string(), "Second step".to_string()],
        };

        let formatted = message.format(&theme);

        assert_eq!(formatted, "Next steps:\n1. First step\n2. Second step\n");
    }

    #[test]
    fn it_should_require_normal_verbosity_when_displaying_steps() {
        let message = StepsMessage {
            title: "Next steps:".to_string(),
            items: vec!["First step".to_string()],
        };

        assert_eq!(message.required_verbosity(), VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_use_stderr_channel_when_displaying_steps() {
        let message = StepsMessage {
            title: "Next steps:".to_string(),
            items: vec!["First step".to_string()],
        };

        assert_eq!(message.channel(), Channel::Stderr);
    }

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
        let message = StepsMessage::new("Title", vec!["Step 1".to_string(), "Step 2".to_string()]);

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
}
