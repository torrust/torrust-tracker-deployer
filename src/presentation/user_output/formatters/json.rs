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
/// use torrust_tracker_deployer_lib::presentation::user_output::{JsonFormatter, UserOutput, VerbosityLevel};
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
