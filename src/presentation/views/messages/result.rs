//! Result message type for final output data

use super::super::{Channel, OutputMessage, Theme, VerbosityLevel};

/// Result message for final output data
///
/// Result messages contain final output data that can be piped or redirected.
/// They go to stdout without any symbols or formatting.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_use_stdout_channel_when_displaying_result() {
        let message = ResultMessage {
            text: "Output data".to_string(),
        };

        assert_eq!(message.channel(), Channel::Stdout);
    }

    #[test]
    fn it_should_require_quiet_verbosity_when_displaying_result() {
        let message = ResultMessage {
            text: "Output data".to_string(),
        };

        assert_eq!(message.required_verbosity(), VerbosityLevel::Quiet);
    }

    #[test]
    fn it_should_not_include_symbols_when_formatting_result() {
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
}
