//! Trace file formatting sections
//!
//! Provides formatting utilities for trace file sections:
//! - Headers and footers
//! - Error chain formatting
//! - Base metadata formatting

use crate::domain::environment::state::BaseFailureContext;
use crate::shared::Traceable;

/// Common sections for all trace files
///
/// Provides static methods for formatting various sections of trace files
/// with consistent styling and structure.
pub(super) struct TraceSections;

impl TraceSections {
    /// Format a trace header
    ///
    /// Creates a centered header with decorative borders for trace files.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to display in the header
    ///
    /// # Returns
    ///
    /// Formatted header string with borders
    pub(super) fn header(title: &str) -> String {
        format!(
            "═══════════════════════════════════════════════════════════════\n\
             {title:^63}\n\
             ═══════════════════════════════════════════════════════════════\n\n"
        )
    }

    /// Format a trace footer
    ///
    /// Creates a footer section to mark the end of a trace file.
    pub(super) fn footer() -> &'static str {
        "\n═══════════════════════════════════════════════════════════════\n\
                             END OF TRACE\n\
         ═══════════════════════════════════════════════════════════════\n"
    }

    /// Format an error chain section header
    ///
    /// Creates a section header for the error chain portion of trace files.
    pub(super) fn error_chain_header() -> &'static str {
        "───────────────────────────────────────────────────────────────\n\
                             ERROR CHAIN\n\
         ───────────────────────────────────────────────────────────────\n\n"
    }

    /// Format a complete error chain by walking the `Traceable` hierarchy
    ///
    /// Recursively formats all errors in the error chain, numbering each
    /// level for easy navigation.
    ///
    /// # Arguments
    ///
    /// * `error` - Root error implementing `Traceable`
    ///
    /// # Returns
    ///
    /// Formatted string containing the complete error chain
    pub(super) fn format_error_chain<E: Traceable>(error: &E) -> String {
        let mut chain = String::new();
        Self::format_error_chain_recursive(error, &mut chain, 0);
        chain
    }

    /// Recursively format error chain levels
    fn format_error_chain_recursive<E: Traceable + ?Sized>(
        error: &E,
        output: &mut String,
        level: usize,
    ) {
        use std::fmt::Write;
        let _ = writeln!(output, "[Level {level}] {}", error.trace_format());

        if let Some(source) = error.trace_source() {
            Self::format_error_chain_recursive(source, output, level + 1);
        }
    }

    /// Format base metadata section common to all failure contexts
    ///
    /// Formats the metadata fields from `BaseFailureContext`:
    /// - Trace ID
    /// - Failed At timestamp
    /// - Execution Started timestamp
    /// - Execution Duration
    /// - Error Summary
    ///
    /// # Arguments
    ///
    /// * `base` - Base failure context containing common metadata
    ///
    /// # Returns
    ///
    /// Formatted string with base metadata, ready to be included in a trace file
    pub(super) fn format_base_metadata(base: &BaseFailureContext) -> String {
        use std::fmt::Write;

        let mut metadata = String::new();
        let _ = writeln!(metadata, "Trace ID: {}", base.trace_id);
        let _ = writeln!(metadata, "Failed At: {}", base.failed_at);
        let _ = writeln!(metadata, "Execution Started: {}", base.execution_started_at);
        let _ = writeln!(
            metadata,
            "Execution Duration: {:?}",
            base.execution_duration
        );
        let _ = writeln!(metadata, "Error Summary: {}", base.error_summary);
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test error implementing Traceable
    #[derive(Debug)]
    struct TestError {
        message: String,
        source: Option<Box<dyn Traceable>>,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError: {}", self.message)
        }
    }

    impl std::error::Error for TestError {}

    impl Traceable for TestError {
        fn trace_format(&self) -> String {
            format!("TestError: {}", self.message)
        }

        fn trace_source(&self) -> Option<&dyn Traceable> {
            self.source.as_deref()
        }

        fn error_kind(&self) -> crate::shared::ErrorKind {
            crate::shared::ErrorKind::CommandExecution
        }
    }

    #[test]
    fn it_should_format_error_chain_with_multiple_levels() {
        // Create nested error chain: level 0 -> level 1 -> level 2
        let level_2_error = TestError {
            message: "root cause".to_string(),
            source: None,
        };

        let level_1_error = TestError {
            message: "intermediate error".to_string(),
            source: Some(Box::new(level_2_error)),
        };

        let level_0_error = TestError {
            message: "top level error".to_string(),
            source: Some(Box::new(level_1_error)),
        };

        let chain = TraceSections::format_error_chain(&level_0_error);

        // Verify all levels are present
        assert!(chain.contains("[Level 0]"));
        assert!(chain.contains("[Level 1]"));
        assert!(chain.contains("[Level 2]"));
        assert!(chain.contains("top level error"));
        assert!(chain.contains("intermediate error"));
        assert!(chain.contains("root cause"));
    }
}
