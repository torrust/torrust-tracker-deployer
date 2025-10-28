//! Common failure context building utilities
//!
//! This module provides helper functions for building failure contexts that are
//! shared across multiple command handlers, reducing code duplication.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::domain::environment::state::BaseFailureContext;
use crate::domain::environment::TraceId;
use crate::shared::Clock;

/// Builds the base failure context that is common to all command failures
///
/// This helper extracts the common logic of:
/// - Calculating execution duration from start time to now
/// - Generating a unique trace ID
/// - Building the `BaseFailureContext` structure
///
/// # Arguments
///
/// * `clock` - Clock service for getting current time
/// * `command_start_time` - When the command started executing
/// * `error_summary` - Human-readable error summary
///
/// # Returns
///
/// A `BaseFailureContext` with `trace_file_path` set to None (handler-specific trace writers will set this)
pub fn build_base_failure_context(
    clock: &Arc<dyn Clock>,
    command_start_time: DateTime<Utc>,
    error_summary: String,
) -> BaseFailureContext {
    let now = clock.now();
    let trace_id = TraceId::new();

    // Calculate actual execution duration
    let execution_duration = now
        .signed_duration_since(command_start_time)
        .to_std()
        .unwrap_or_default();

    BaseFailureContext {
        error_summary,
        failed_at: now,
        execution_started_at: command_start_time,
        execution_duration,
        trace_id,
        trace_file_path: None, // Will be set by handler-specific trace writer
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration as StdDuration;

    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::testing::MockClock;

    #[test]
    fn it_should_calculate_correct_execution_duration() {
        // Arrange
        let start_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 5, 30).unwrap(); // 5 minutes 30 seconds later
        let clock: Arc<dyn Clock> = Arc::new(MockClock::new(end_time));

        // Act
        let base_context = build_base_failure_context(&clock, start_time, "Test error".to_string());

        // Assert
        let expected_duration = StdDuration::from_secs(5 * 60 + 30); // 330 seconds
        assert_eq!(base_context.execution_duration, expected_duration);
    }

    #[test]
    fn it_should_generate_unique_trace_ids() {
        // Arrange
        let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Utc::now()));
        let start_time = Utc::now();

        // Act
        let context1 = build_base_failure_context(&clock, start_time, "Error 1".to_string());
        let context2 = build_base_failure_context(&clock, start_time, "Error 2".to_string());

        // Assert
        assert_ne!(
            context1.trace_id, context2.trace_id,
            "Each call should generate a unique trace ID"
        );
    }

    #[test]
    fn it_should_handle_zero_duration_when_start_equals_end() {
        // Arrange
        let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock: Arc<dyn Clock> = Arc::new(MockClock::new(fixed_time));

        // Act
        let base_context = build_base_failure_context(&clock, fixed_time, "Test error".to_string());

        // Assert
        assert_eq!(base_context.execution_duration, StdDuration::from_secs(0));
    }

    #[test]
    fn it_should_preserve_error_summary_in_base_context() {
        // Arrange
        let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Utc::now()));
        let start_time = Utc::now();
        let expected_summary = "Custom error message";

        // Act
        let base_context =
            build_base_failure_context(&clock, start_time, expected_summary.to_string());

        // Assert
        assert_eq!(base_context.error_summary, expected_summary);
    }
}
