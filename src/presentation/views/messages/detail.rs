//! Detail message for verbose progress output
//!
//! This message type is used for detailed progress information that appears
//! at the Verbose (`-v`) verbosity level and above. It uses the detail symbol
//! (📋 in emoji theme) to visually distinguish detailed progress from normal
//! progress messages.

use super::super::channel::Channel;
use super::super::theme::Theme;
use super::super::traits::OutputMessage;
use super::super::verbosity::VerbosityLevel;

/// A detail message shown at Verbose verbosity level and above
///
/// Detail messages provide step-level progress information during command
/// execution. They are emitted by the `CommandProgressListener` implementation
/// to show which internal steps are being executed.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::views::messages::DetailMessage;
///
/// let message = DetailMessage {
///     text: "  [Step 1/9] Rendering OpenTofu templates...".to_string(),
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
        VerbosityLevel::Verbose
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
            text: "  [Step 1/9] Rendering templates...".to_string(),
        };
        let theme = Theme::emoji();
        let formatted = message.format(&theme);
        assert_eq!(formatted, "📋   [Step 1/9] Rendering templates...\n");
    }

    #[test]
    fn it_should_require_verbose_verbosity_level() {
        let message = DetailMessage {
            text: "test".to_string(),
        };
        assert_eq!(message.required_verbosity(), VerbosityLevel::Verbose);
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
            text: "  [Step 1/9] Rendering templates...".to_string(),
        };
        let theme = Theme::plain();
        let formatted = message.format(&theme);
        assert_eq!(formatted, "[DETAIL]   [Step 1/9] Rendering templates...\n");
    }
}
