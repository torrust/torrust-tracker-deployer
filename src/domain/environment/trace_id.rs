//! Trace identifier for linking errors to trace files
//!
//! The `TraceId` provides a unique identifier for each error trace,
//! enabling correlation between error contexts stored in state and
//! detailed trace files.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for error traces
///
/// Uses UUID v4 to ensure uniqueness across all environments and time periods.
/// The newtype pattern provides type safety and prevents mixing trace IDs
/// with other UUIDs in the system.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer::domain::environment::TraceId;
///
/// let trace_id = TraceId::new();
/// println!("Trace ID: {}", trace_id);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceId(Uuid);

impl TraceId {
    /// Generate a new unique trace identifier
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer::domain::environment::TraceId;
    ///
    /// let id1 = TraceId::new();
    /// let id2 = TraceId::new();
    /// assert_ne!(id1, id2);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID value
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer::domain::environment::TraceId;
    ///
    /// let trace_id = TraceId::new();
    /// let uuid = trace_id.inner();
    /// println!("UUID: {}", uuid);
    /// ```
    #[must_use]
    pub fn inner(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_unique_trace_ids() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn it_should_serialize_trace_id_to_json() {
        let trace_id = TraceId::new();
        let json = serde_json::to_string(&trace_id).unwrap();
        assert!(json.contains('-')); // UUIDs contain hyphens
    }

    #[test]
    fn it_should_deserialize_trace_id_from_json() {
        let uuid = Uuid::new_v4();
        let json = format!("\"{uuid}\"");
        let trace_id: TraceId = serde_json::from_str(&json).unwrap();
        assert_eq!(trace_id.inner(), &uuid);
    }

    #[test]
    fn it_should_display_trace_id_as_uuid_string() {
        let uuid = Uuid::new_v4();
        let trace_id = TraceId(uuid);
        assert_eq!(trace_id.to_string(), uuid.to_string());
    }

    #[test]
    fn it_should_provide_access_to_inner_uuid() {
        let uuid = Uuid::new_v4();
        let trace_id = TraceId(uuid);
        assert_eq!(trace_id.inner(), &uuid);
    }

    #[test]
    fn it_should_create_default_trace_id() {
        let id1 = TraceId::default();
        let id2 = TraceId::default();
        assert_ne!(id1, id2); // Each default should be unique
    }
}
