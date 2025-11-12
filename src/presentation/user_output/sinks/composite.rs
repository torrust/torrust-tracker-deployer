//! Composite output sink for multiple destinations

use super::super::{OutputMessage, OutputSink};

pub struct CompositeSink {
    sinks: Vec<Box<dyn OutputSink>>,
}

impl CompositeSink {
    /// Create a new composite sink with the given child sinks
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::CompositeSink;
    ///
    /// let composite = CompositeSink::new(vec![
    ///     Box::new(StandardSink::default_console()),
    ///     Box::new(FileSink::new("output.log").unwrap()),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(sinks: Vec<Box<dyn OutputSink>>) -> Self {
        Self { sinks }
    }

    /// Add a sink to the composite
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::CompositeSink;
    ///
    /// let mut composite = CompositeSink::new(vec![]);
    /// composite.add_sink(Box::new(StandardSink::default_console()));
    /// composite.add_sink(Box::new(FileSink::new("output.log").unwrap()));
    /// ```
    pub fn add_sink(&mut self, sink: Box<dyn OutputSink>) {
        self.sinks.push(sink);
    }
}

impl OutputSink for CompositeSink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        for sink in &mut self.sinks {
            sink.write_message(message, formatted);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_sink_should_support_empty_sink_list() {
        let composite = CompositeSink::new(vec![]);

        // Should not panic with empty sink list
        assert_eq!(composite.sinks.len(), 0);
    }

    #[test]
    fn composite_sink_should_support_add_sink() {
        // Create a mock sink for testing
        struct MockSink;
        impl OutputSink for MockSink {
            fn write_message(&mut self, _message: &dyn OutputMessage, _formatted: &str) {}
        }

        let mut composite = CompositeSink::new(vec![]);

        // Add a sink
        composite.add_sink(Box::new(MockSink));

        // Verify sink was added
        assert_eq!(composite.sinks.len(), 1);
    }
}
