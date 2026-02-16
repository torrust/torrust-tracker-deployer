//! Debug detail message for debug-level progress output
//!
//! This message type is used for technical/debug details that appear
//! at the Debug (`-vvv`) verbosity level and above. It uses the debug symbol
//! (🔍 in emoji theme) to visually distinguish debug information from
//! normal and verbose progress messages.

use super::super::channel::Channel;
use super::super::theme::Theme;
use super::super::traits::OutputMessage;
use super::super::verbosity::VerbosityLevel;

/// A debug detail message shown at Debug verbosity level and above
///
/// Debug detail messages provide technical implementation details during
/// command execution, such as commands being executed, exit codes, and
/// raw output from external tools.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::views::messages::DebugDetailMessage;
///
/// let message = DebugDetailMessage {
///     text: "     → Command: tofu init".to_string(),
/// };
/// ```
pub struct DebugDetailMessage {
    /// The debug detail text to display
    pub text: String,
}

impl OutputMessage for DebugDetailMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}\n", theme.debug_symbol(), self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Debug
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "DebugDetailMessage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_format_with_debug_symbol() {
        let message = DebugDetailMessage {
            text: "     → Command: tofu init".to_string(),
        };
        let theme = Theme::emoji();
        let formatted = message.format(&theme);
        assert_eq!(formatted, "🔍      → Command: tofu init\n");
    }

    #[test]
    fn it_should_require_debug_verbosity_level() {
        let message = DebugDetailMessage {
            text: "test".to_string(),
        };
        assert_eq!(message.required_verbosity(), VerbosityLevel::Debug);
    }

    #[test]
    fn it_should_output_to_stderr() {
        let message = DebugDetailMessage {
            text: "test".to_string(),
        };
        assert_eq!(message.channel(), Channel::Stderr);
    }

    #[test]
    fn it_should_have_correct_type_name() {
        let message = DebugDetailMessage {
            text: "test".to_string(),
        };
        assert_eq!(message.type_name(), "DebugDetailMessage");
    }

    #[test]
    fn it_should_format_with_plain_theme() {
        let message = DebugDetailMessage {
            text: "     → Command: tofu init".to_string(),
        };
        let theme = Theme::plain();
        let formatted = message.format(&theme);
        assert_eq!(formatted, "[DEBUG]      → Command: tofu init\n");
    }
}
