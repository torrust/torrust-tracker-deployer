//! Step progress message for verbose output
//!
//! This message type is used for step-level progress headers that appear
//! at the Verbose (`-v`) verbosity level and above. It uses the detail symbol
//! (📋 in emoji theme) to visually indicate step boundaries.

use super::super::channel::Channel;
use super::super::theme::Theme;
use super::super::traits::OutputMessage;
use super::super::verbosity::VerbosityLevel;

/// A step progress message shown at Verbose verbosity level and above
///
/// Step progress messages mark the boundaries between workflow steps during
/// command execution. They are emitted by the `CommandProgressListener`
/// implementation via `on_step_started()`.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::cli::views::messages::StepProgressMessage;
///
/// let message = StepProgressMessage {
///     text: "  [Step 1/9] Rendering OpenTofu templates...".to_string(),
/// };
/// ```
pub struct StepProgressMessage {
    /// The step progress text to display
    pub text: String,
}

impl OutputMessage for StepProgressMessage {
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
        "StepProgressMessage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_step_progress_message() {
        let message = StepProgressMessage {
            text: "  [Step 1/9] Rendering templates...".to_string(),
        };

        assert_eq!(message.text, "  [Step 1/9] Rendering templates...");
    }

    #[test]
    fn it_should_have_verbose_level() {
        let message = StepProgressMessage {
            text: "test".to_string(),
        };

        assert_eq!(message.required_verbosity(), VerbosityLevel::Verbose);
    }

    #[test]
    fn it_should_use_stderr_channel() {
        let message = StepProgressMessage {
            text: "test".to_string(),
        };

        assert_eq!(message.channel(), Channel::Stderr);
    }
}
