//! Tracker template context
//!
//! Defines the variables needed for tracker.toml.tera template rendering.
//!
//! ## Phase 4 vs Phase 6
//!
//! - **Phase 4**: All values are hardcoded in the template. This context exists
//!   but contains no fields - it's used with an empty Tera context.
//! - **Phase 6**: Will add fields for dynamic configuration (database path,
//!   tracker ports, API settings, etc.)

use serde::Serialize;

/// Context for rendering tracker.toml.tera template
///
/// ## Current State (Phase 4)
///
/// This context is currently empty because Phase 4 uses hardcoded values in
/// the template file. No variable substitution is performed.
///
/// ## Future State (Phase 6)
///
/// Will be extended to include:
/// - Database configuration (driver, path)
/// - Tracker bindings (UDP/HTTP addresses and ports)
/// - HTTP API configuration
/// - Logging settings
/// - Core tracker policies
///
/// # Example (Future Phase 6)
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::infrastructure::templating::tracker::TrackerContext;
///
/// let context = TrackerContext {
///     database_driver: "sqlite3".to_string(),
///     database_path: "/var/lib/torrust/tracker/database/sqlite3.db".to_string(),
///     udp_trackers: vec![
///         "0.0.0.0:6868".to_string(),
///         "0.0.0.0:6969".to_string(),
///     ],
///     http_trackers: vec!["0.0.0.0:7070".to_string()],
///     api_bind_address: "0.0.0.0:1212".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct TrackerContext {
    // Phase 4: No fields - all values hardcoded in template
    // Phase 6: Will add fields for dynamic configuration
}

impl TrackerContext {
    /// Creates a new empty tracker context for Phase 4
    ///
    /// In Phase 4, all configuration values are hardcoded in the template,
    /// so this context contains no fields.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TrackerContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_empty_context_for_phase_4() {
        let context = TrackerContext::new();

        // Phase 4: Context should be empty (no fields)
        let json = serde_json::to_value(&context).expect("Failed to serialize");
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn it_should_support_default_trait() {
        let context = TrackerContext::default();
        let json = serde_json::to_value(&context).expect("Failed to serialize");
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn it_should_be_cloneable() {
        let context = TrackerContext::new();
        let cloned = context.clone();

        let original_json = serde_json::to_value(&context).expect("Failed to serialize");
        let cloned_json = serde_json::to_value(&cloned).expect("Failed to serialize");

        assert_eq!(original_json, cloned_json);
    }

    #[test]
    fn it_should_support_debug_formatting() {
        let context = TrackerContext::new();
        let debug_output = format!("{context:?}");

        assert!(debug_output.contains("TrackerContext"));
    }
}
