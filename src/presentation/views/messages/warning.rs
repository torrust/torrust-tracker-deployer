//! Warning message type for non-critical issues

use super::super::{Channel, OutputMessage, Theme, VerbosityLevel};

/// Warning message for non-critical issues
///
/// Warning messages alert users to potential issues that don't prevent
/// operation completion but may need attention.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_include_extra_space_when_formatting_warning() {
        let theme = Theme::emoji();
        let message = WarningMessage {
            text: "Warning text".to_string(),
        };

        let formatted = message.format(&theme);

        // Warning messages include two spaces after the symbol
        assert_eq!(formatted, "⚠️  Warning text\n");
    }

    #[test]
    fn it_should_use_stderr_channel_when_displaying_warning() {
        let message = WarningMessage {
            text: "Warning text".to_string(),
        };

        assert_eq!(message.channel(), Channel::Stderr);
    }
}
