//! Progress message type for ongoing operations

use super::super::{Channel, OutputMessage, Theme, VerbosityLevel};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
