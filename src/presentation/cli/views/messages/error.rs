//! Error message type for critical failures

use super::super::{Channel, OutputMessage, Theme, VerbosityLevel};

/// Error message for critical failures
///
/// Error messages indicate critical failures that prevent operation completion.
/// They are always shown regardless of verbosity level.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_always_be_shown_when_error_occurs() {
        let message = ErrorMessage {
            text: "Critical error".to_string(),
        };

        // Errors should require Quiet level (always shown)
        assert_eq!(message.required_verbosity(), VerbosityLevel::Quiet);
    }

    #[test]
    fn it_should_use_stderr_channel_when_displaying_error() {
        let message = ErrorMessage {
            text: "Critical error".to_string(),
        };

        assert_eq!(message.channel(), Channel::Stderr);
    }
}
