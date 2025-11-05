//! Success message type for completed operations

use super::super::{Channel, OutputMessage, Theme, VerbosityLevel};

/// Success message for completed operations
///
/// Success messages indicate that an operation completed successfully.
/// They provide positive feedback to users.
pub struct SuccessMessage {
    /// The success message text
    pub text: String,
}

impl OutputMessage for SuccessMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}\n", theme.success_symbol(), self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "SuccessMessage"
    }
}
