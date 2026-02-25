//! Detail message for very verbose progress output
//!
//! This message type is used for detailed contextual information that appears
//! at the `VeryVerbose` (`-vv`) verbosity level and above. It uses the detail symbol
//! (📋 in emoji theme) to visually distinguish detailed context from step headers.

use super::super::channel::Channel;
use super::super::theme::Theme;
use super::super::traits::OutputMessage;
use super::super::verbosity::VerbosityLevel;

/// A detail message shown at `VeryVerbose` verbosity level and above
///
/// Detail messages provide contextual information within steps during command
/// execution. They are emitted by the `CommandProgressListener` implementation
/// via `on_detail()` to show results, file paths, retry attempts, etc.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::cli::views::messages::DetailMessage;
///
/// let message = DetailMessage {
///     text: "     → Instance IP: 10.140.190.235".to_string(),
/// };
/// ```
pub struct DetailMessage {
    /// The detail text to display
    pub text: String,
}

impl OutputMessage for DetailMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}\n", theme.detail_symbol(), self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::VeryVerbose
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "DetailMessage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_format_with_detail_symbol() {
        let message = DetailMessage {
            text: "     → Instance IP: 10.140.190.235".to_string(),
        };
        let theme = Theme::emoji();
        let formatted = message.format(&theme);
        assert_eq!(formatted, "📋      → Instance IP: 10.140.190.235\n");
    }

    #[test]
    fn it_should_require_very_verbose_verbosity_level() {
        let message = DetailMessage {
            text: "test".to_string(),
        };
        assert_eq!(message.required_verbosity(), VerbosityLevel::VeryVerbose);
    }

    #[test]
    fn it_should_output_to_stderr() {
        let message = DetailMessage {
            text: "test".to_string(),
        };
        assert_eq!(message.channel(), Channel::Stderr);
    }

    #[test]
    fn it_should_have_correct_type_name() {
        let message = DetailMessage {
            text: "test".to_string(),
        };
        assert_eq!(message.type_name(), "DetailMessage");
    }

    #[test]
    fn it_should_format_with_plain_theme() {
        let message = DetailMessage {
            text: "     → Instance IP: 10.140.190.235".to_string(),
        };
        let theme = Theme::plain();
        let formatted = message.format(&theme);
        assert_eq!(formatted, "[DETAIL]      → Instance IP: 10.140.190.235\n");
    }
}
