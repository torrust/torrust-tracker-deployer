//! Telemetry output sink implementation

use super::super::{OutputMessage, OutputSink};

pub struct TelemetrySink {
    endpoint: String,
}

impl TelemetrySink {
    /// Create a new telemetry sink
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::views::TelemetrySink;
    ///
    /// let sink = TelemetrySink::new("https://telemetry.example.com".to_string());
    /// ```
    #[must_use]
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    /// Get the endpoint URL
    #[cfg(test)]
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl OutputSink for TelemetrySink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        // In real implementation, send to telemetry service
        tracing::debug!(
            endpoint = %self.endpoint,
            message_type = message.type_name(),
            channel = ?message.channel(),
            content = formatted,
            "Telemetry event"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_sink_should_create_with_endpoint() {
        let sink = TelemetrySink::new("https://example.com".to_string());
        assert_eq!(sink.endpoint(), "https://example.com");
    }
}
