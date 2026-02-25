//! JSON formatter implementation

use super::super::{FormatterOverride, OutputMessage};

// ============================================================================
// Formatter Override Implementations
// ============================================================================

/// JSON formatter for machine-readable output
///
/// Transforms messages into JSON objects with metadata including:
/// - Message type (for programmatic filtering)
/// - Channel (stdout/stderr)
/// - Content (the formatted message)
/// - Timestamp (ISO 8601 format)
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::cli::views::{JsonFormatter, UserOutput, VerbosityLevel};
///
/// let formatter = JsonFormatter;
/// let mut output = UserOutput::with_formatter_override(
///     VerbosityLevel::Normal,
///     Box::new(formatter)
/// );
///
/// output.progress("Starting process");
/// // Output: {"type":"ProgressMessage","channel":"Stderr","content":"⏳ Starting process","timestamp":"2025-11-04T12:34:56Z"}
/// ```
pub struct JsonFormatter;

impl FormatterOverride for JsonFormatter {
    fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String {
        let json = serde_json::json!({
            "type": message.type_name(),
            "channel": format!("{:?}", message.channel()),
            "content": formatted.trim(), // Remove trailing newlines for cleaner JSON
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
        .to_string();
        format!("{json}\n")
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::cli::views::{
        Channel, ErrorMessage, OutputMessage, ProgressMessage, ResultMessage, StepsMessage,
        SuccessMessage, Theme, VerbosityLevel, WarningMessage,
    };
    use chrono::{DateTime, Utc};
    use rstest::rstest;
    use serde_json::Value;

    // ========================================================================
    // Test Fixtures
    // ========================================================================

    /// Mock message for testing transformer behavior in isolation
    struct TestMessage {
        content: String,
        channel: Channel,
        type_name: &'static str,
    }

    impl TestMessage {
        fn new(content: impl Into<String>, channel: Channel, type_name: &'static str) -> Self {
            Self {
                content: content.into(),
                channel,
                type_name,
            }
        }

        fn stderr(content: impl Into<String>) -> Self {
            Self::new(content, Channel::Stderr, "TestMessage")
        }
    }

    impl OutputMessage for TestMessage {
        fn format(&self, _theme: &Theme) -> String {
            self.content.clone()
        }

        fn required_verbosity(&self) -> VerbosityLevel {
            VerbosityLevel::Normal
        }

        fn channel(&self) -> Channel {
            self.channel
        }

        fn type_name(&self) -> &'static str {
            self.type_name
        }
    }

    // ========================================================================
    // Test Helpers
    // ========================================================================

    /// Parse JSON output and panic with helpful error message on failure
    fn parse_json(output: &str) -> Value {
        let json_line = output.trim_end_matches('\n');
        serde_json::from_str(json_line)
            .unwrap_or_else(|e| panic!("Failed to parse JSON: {e}\nOutput: {output}"))
    }

    /// Validate RFC3339 timestamp format
    fn is_valid_rfc3339_timestamp(timestamp: &str) -> bool {
        DateTime::parse_from_rfc3339(timestamp).is_ok()
    }

    /// Extract timestamp from JSON output and parse it
    fn parse_timestamp(json: &Value) -> DateTime<Utc> {
        let timestamp_str = json["timestamp"]
            .as_str()
            .expect("timestamp should be a string");

        DateTime::parse_from_rfc3339(timestamp_str)
            .expect("timestamp should be valid RFC3339")
            .with_timezone(&Utc)
    }

    /// Create real message instances for integration-style tests
    fn create_message(message_type: &str, text: &str) -> Box<dyn OutputMessage> {
        match message_type {
            "ProgressMessage" => Box::new(ProgressMessage {
                text: text.to_string(),
            }),
            "SuccessMessage" => Box::new(SuccessMessage {
                text: text.to_string(),
            }),
            "ErrorMessage" => Box::new(ErrorMessage {
                text: text.to_string(),
            }),
            "WarningMessage" => Box::new(WarningMessage {
                text: text.to_string(),
            }),
            "ResultMessage" => Box::new(ResultMessage {
                text: text.to_string(),
            }),
            "StepsMessage" => Box::new(StepsMessage {
                title: text.to_string(),
                items: vec!["Step 1".to_string(), "Step 2".to_string()],
            }),
            _ => panic!("Unknown message type: {message_type}"),
        }
    }

    // ========================================================================
    // JSON Structure Tests
    // ========================================================================

    #[test]
    fn it_should_produce_valid_json_with_all_required_fields() {
        let formatter = JsonFormatter;
        let message = TestMessage::stderr("Test message");

        let output = formatter.transform("Test message", &message);

        let json = parse_json(output.as_str());
        assert!(json.is_object(), "output should be a JSON object");

        // Verify structure and types
        assert_eq!(json["type"].as_str(), Some("TestMessage"));
        assert_eq!(json["channel"].as_str(), Some("Stderr"));
        assert_eq!(json["content"].as_str(), Some("Test message"));

        let timestamp = json["timestamp"].as_str().expect("timestamp should exist");
        assert!(
            is_valid_rfc3339_timestamp(timestamp),
            "timestamp should be valid RFC3339: {timestamp}"
        );
    }

    #[test]
    fn it_should_produce_exactly_four_fields_without_extras() {
        let formatter = JsonFormatter;
        let message = TestMessage::stderr("Test");

        let output = formatter.transform("Test", &message);
        let json = parse_json(output.as_str());

        assert_eq!(
            json.as_object().unwrap().len(),
            4,
            "JSON should have exactly 4 fields: type, channel, content, timestamp"
        );
    }

    #[test]
    fn it_should_append_newline_for_line_buffered_output() {
        let formatter = JsonFormatter;
        let message = TestMessage::stderr("Test");

        let output = formatter.transform("Test", &message);

        assert!(
            output.ends_with('\n'),
            "output should end with newline for line-buffered streams"
        );
    }

    // ========================================================================
    // Content Processing Tests
    // ========================================================================

    #[rstest]
    #[case("Test message\n", "Test message")]
    #[case("Test message\n\n\n", "Test message")]
    #[case("Line 1\nLine 2\nLine 3\n", "Line 1\nLine 2\nLine 3")]
    #[case("", "")]
    #[case("   \t  ", "")]
    fn it_should_trim_trailing_whitespace_from_content(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let formatter = JsonFormatter;
        let message = TestMessage::stderr(input);

        let output = formatter.transform(input, &message);
        let json = parse_json(output.as_str());

        assert_eq!(
            json["content"].as_str(),
            Some(expected),
            "content should have trailing whitespace trimmed"
        );
    }

    // ========================================================================
    // Channel Routing Tests
    // ========================================================================

    #[rstest]
    #[case(Channel::Stderr, "Stderr")]
    #[case(Channel::Stdout, "Stdout")]
    fn it_should_serialize_channel_as_debug_string(
        #[case] channel: Channel,
        #[case] expected: &str,
    ) {
        let formatter = JsonFormatter;
        let message = TestMessage::new("Test", channel, "TestMessage");

        let output = formatter.transform("Test", &message);
        let json = parse_json(output.as_str());

        assert_eq!(
            json["channel"].as_str(),
            Some(expected),
            "channel should be serialized using Debug format"
        );
    }

    // ========================================================================
    // Integration Tests with Real Message Types
    // ========================================================================

    #[rstest]
    #[case("ProgressMessage", "Processing...", "Stderr")]
    #[case("SuccessMessage", "Operation completed", "Stderr")]
    #[case("ErrorMessage", "Something went wrong", "Stderr")]
    #[case("WarningMessage", "This is a warning", "Stderr")]
    #[case("ResultMessage", "Final result", "Stdout")]
    #[case("StepsMessage", "Next steps:", "Stderr")]
    fn it_should_correctly_transform_real_message_types(
        #[case] message_type: &str,
        #[case] text: &str,
        #[case] expected_channel: &str,
    ) {
        let formatter = JsonFormatter;
        let theme = Theme::emoji();
        let message = create_message(message_type, text);

        let formatted = message.format(&theme);
        let output = formatter.transform(&formatted, message.as_ref());
        let json = parse_json(output.as_str());

        assert_eq!(
            json["type"].as_str(),
            Some(message_type),
            "type should match message type name"
        );
        assert_eq!(
            json["channel"].as_str(),
            Some(expected_channel),
            "channel should match message's designated channel"
        );
        assert!(
            json["content"].as_str().unwrap_or("").contains(text),
            "content should contain the original message text"
        );
    }

    // ========================================================================
    // JSON Escaping and Special Characters Tests
    // ========================================================================

    #[rstest]
    #[case(r#"Message with "quotes""#, r#"Message with "quotes""#)]
    #[case(r"Message with \backslashes\", r"Message with \backslashes\")]
    #[case("Message with emoji 🎉🚀", "Message with emoji 🎉🚀")]
    #[case("Message\twith\ttabs\r\n", "Message\twith\ttabs")]
    fn it_should_properly_escape_special_characters_in_json(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let formatter = JsonFormatter;
        let message = TestMessage::stderr(input);

        let output = formatter.transform(input, &message);

        // Verify the entire output is valid JSON (implicitly tests escaping)
        let json = parse_json(output.as_str());
        assert_eq!(
            json["content"].as_str(),
            Some(expected),
            "special characters should be properly escaped in JSON"
        );
    }

    // ========================================================================
    // Timestamp Tests
    // ========================================================================

    #[test]
    fn it_should_generate_timestamp_within_reasonable_time_window() {
        let formatter = JsonFormatter;
        let message = TestMessage::stderr("Test");

        let before = Utc::now();
        let output = formatter.transform("Test", &message);
        let after = Utc::now();

        let json = parse_json(output.as_str());
        let timestamp = parse_timestamp(&json);

        assert!(
            timestamp >= before && timestamp <= after,
            "timestamp should be between {before:?} and {after:?}, got {timestamp:?}"
        );
    }

    #[test]
    fn it_should_generate_increasing_timestamps_for_sequential_calls() {
        let formatter = JsonFormatter;
        let message = TestMessage::stderr("Test");

        let output1 = formatter.transform("First", &message);
        std::thread::sleep(std::time::Duration::from_millis(2));
        let output2 = formatter.transform("Second", &message);

        let json1 = parse_json(output1.as_str());
        let json2 = parse_json(output2.as_str());

        let ts1 = parse_timestamp(&json1);
        let ts2 = parse_timestamp(&json2);

        assert!(
            ts2 >= ts1,
            "second timestamp should be >= first timestamp: {ts1:?} vs {ts2:?}"
        );
    }
}
