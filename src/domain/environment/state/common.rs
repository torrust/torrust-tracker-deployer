//! Common failure context structure
//!
//! Provides the `BaseFailureContext` type that contains fields shared across
//! all command failure contexts (provision, configure, etc.).
//!
//! This reduces duplication and provides a consistent structure for:
//! - Timing information (execution start, failure time, duration)
//! - Error summary
//! - Trace identification and file location

use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::environment::TraceId;

/// Base failure context shared across all command failures
///
/// Contains common fields that all failure contexts need:
/// - Error summary for display
/// - Timing information (start, fail, duration)
/// - Trace identification and file path
///
/// This is embedded in command-specific failure contexts like
/// `ProvisionFailureContext` and `ConfigureFailureContext`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseFailureContext {
    /// Human-readable error summary
    pub error_summary: String,

    /// When the failure occurred
    pub failed_at: DateTime<Utc>,

    /// When execution started
    pub execution_started_at: DateTime<Utc>,

    /// How long execution ran before failing
    pub execution_duration: Duration,

    /// Unique trace identifier
    pub trace_id: TraceId,

    /// Path to the detailed trace file (if generated)
    pub trace_file_path: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_base_failure_context() {
        let now = Utc::now();
        let trace_id = TraceId::new();

        let context = BaseFailureContext {
            error_summary: "Test error".to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(10),
            trace_id: trace_id.clone(),
            trace_file_path: None,
        };

        assert_eq!(context.error_summary, "Test error");
        assert_eq!(context.trace_id, trace_id);
        assert_eq!(context.trace_file_path, None);
    }

    #[test]
    fn it_should_serialize_base_failure_context_to_json() {
        let now = Utc::now();
        let trace_id = TraceId::new();

        let context = BaseFailureContext {
            error_summary: "Test error".to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(10),
            trace_id,
            trace_file_path: Some(PathBuf::from("/tmp/trace.log")),
        };

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("Test error"));
        assert!(json.contains("/tmp/trace.log"));
    }

    #[test]
    fn it_should_deserialize_base_failure_context_from_json() {
        let trace_id = TraceId::new();
        let json = format!(
            r#"{{
                "error_summary": "Deserialized error",
                "failed_at": "2025-10-07T12:00:00Z",
                "execution_started_at": "2025-10-07T11:59:00Z",
                "execution_duration": {{"secs": 60, "nanos": 0}},
                "trace_id": "{trace_id}",
                "trace_file_path": null
            }}"#
        );

        let context: BaseFailureContext = serde_json::from_str(&json).unwrap();
        assert_eq!(context.error_summary, "Deserialized error");
        assert_eq!(context.execution_duration, Duration::from_secs(60));
    }

    #[test]
    fn it_should_clone_base_failure_context() {
        let now = Utc::now();
        let context = BaseFailureContext {
            error_summary: "Original error".to_string(),
            failed_at: now,
            execution_started_at: now,
            execution_duration: Duration::from_secs(5),
            trace_id: TraceId::new(),
            trace_file_path: None,
        };

        let cloned = context.clone();
        assert_eq!(context.error_summary, cloned.error_summary);
        assert_eq!(context.trace_id, cloned.trace_id);
    }
}
