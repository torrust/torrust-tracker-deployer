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
