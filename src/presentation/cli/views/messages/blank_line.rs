//! Blank line message for visual spacing in output

use crate::presentation::cli::views::{Channel, OutputMessage, Theme, VerbosityLevel};

/// Message that outputs a blank line to stderr for visual spacing
///
/// Used to add spacing between sections of output to improve readability.
/// Only shown at Normal verbosity level and above.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::{UserOutput, VerbosityLevel};
///
/// let mut output = UserOutput::new(VerbosityLevel::Normal);
/// output.success("Operation completed");
/// output.blank_line();  // Adds visual spacing
/// output.progress("Starting next operation...");
/// ```
#[derive(Debug, Clone)]
pub struct BlankLineMessage;

impl OutputMessage for BlankLineMessage {
    fn format(&self, _theme: &Theme) -> String {
        "\n".to_string()
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }

    fn type_name(&self) -> &'static str {
        "BlankLineMessage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::cli::views::{Channel, Theme, VerbosityLevel};

    #[test]
    fn it_should_require_normal_verbosity_when_displaying_blank_line() {
        let message = BlankLineMessage;

        assert_eq!(message.required_verbosity(), VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_use_stderr_channel_when_displaying_blank_line() {
        let message = BlankLineMessage;

        assert_eq!(message.channel(), Channel::Stderr);
    }

    #[test]
    fn it_should_format_as_newline_when_displaying_blank_line() {
        let message = BlankLineMessage;
        let theme = Theme::emoji();

        assert_eq!(message.format(&theme), "\n");
    }

    #[test]
    fn it_should_have_correct_type_name_when_displaying_blank_line() {
        let message = BlankLineMessage;

        assert_eq!(message.type_name(), "BlankLineMessage");
    }

    #[test]
    fn it_should_work_with_all_themes_when_displaying_blank_line() {
        let message = BlankLineMessage;

        // Blank line should be the same regardless of theme
        assert_eq!(message.format(&Theme::emoji()), "\n");
        assert_eq!(message.format(&Theme::plain()), "\n");
        assert_eq!(message.format(&Theme::ascii()), "\n");
    }

    #[test]
    fn it_should_be_cloneable_when_creating_blank_line_message() {
        let message = BlankLineMessage;
        let cloned = message.clone();

        assert_eq!(message.type_name(), cloned.type_name());
        assert_eq!(message.channel(), cloned.channel());
        assert_eq!(message.required_verbosity(), cloned.required_verbosity());
    }
}
