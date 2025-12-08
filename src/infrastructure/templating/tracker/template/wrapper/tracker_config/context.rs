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
/// ## Current State (Phase 6)
///
/// This context contains fields for dynamic tracker configuration based on
/// the environment's tracker settings.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::tracker::TrackerContext;
/// use torrust_tracker_deployer_lib::domain::environment::{TrackerConfig, TrackerCoreConfig, DatabaseConfig, UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig};
///
/// let tracker_config = TrackerConfig {
///     core: TrackerCoreConfig {
///         database: DatabaseConfig::Sqlite {
///             database_name: "tracker.db".to_string(),
///         },
///         private: true,
///     },
///     udp_trackers: vec![
///         UdpTrackerConfig { bind_address: "0.0.0.0:6868".to_string() },
///         UdpTrackerConfig { bind_address: "0.0.0.0:6969".to_string() },
///     ],
///     http_trackers: vec![
///         HttpTrackerConfig { bind_address: "0.0.0.0:7070".to_string() },
///     ],
///     http_api: HttpApiConfig {
///         admin_token: "MyToken".to_string(),
///     },
/// };
/// let context = TrackerContext::from_config(&tracker_config);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct TrackerContext {
    /// Database file name (e.g., "tracker.db", "sqlite3.db")
    pub tracker_database_name: String,

    /// Whether tracker is in private mode
    pub tracker_core_private: bool,

    /// UDP tracker bind addresses
    pub udp_trackers: Vec<UdpTrackerEntry>,

    /// HTTP tracker bind addresses
    pub http_trackers: Vec<HttpTrackerEntry>,
}

/// UDP tracker entry for template rendering
#[derive(Debug, Clone, Serialize)]
pub struct UdpTrackerEntry {
    pub bind_address: String,
}

/// HTTP tracker entry for template rendering
#[derive(Debug, Clone, Serialize)]
pub struct HttpTrackerEntry {
    pub bind_address: String,
}

impl TrackerContext {
    /// Creates a new tracker context from tracker configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The tracker configuration from environment
    #[must_use]
    pub fn from_config(config: &crate::domain::environment::TrackerConfig) -> Self {
        Self {
            tracker_database_name: config.core.database.database_name().to_string(),
            tracker_core_private: config.core.private,
            udp_trackers: config
                .udp_trackers
                .iter()
                .map(|t| UdpTrackerEntry {
                    bind_address: t.bind_address.clone(),
                })
                .collect(),
            http_trackers: config
                .http_trackers
                .iter()
                .map(|t| HttpTrackerEntry {
                    bind_address: t.bind_address.clone(),
                })
                .collect(),
        }
    }

    /// Creates a default tracker context with hardcoded values
    ///
    /// Used when no tracker configuration is provided in environment.
    /// Provides backward compatibility with Phase 4 defaults.
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            tracker_database_name: "sqlite3.db".to_string(),
            tracker_core_private: false,
            udp_trackers: vec![
                UdpTrackerEntry {
                    bind_address: "0.0.0.0:6868".to_string(),
                },
                UdpTrackerEntry {
                    bind_address: "0.0.0.0:6969".to_string(),
                },
            ],
            http_trackers: vec![HttpTrackerEntry {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
        }
    }
}

impl Default for TrackerContext {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::environment::{
        DatabaseConfig, HttpApiConfig, HttpTrackerConfig, TrackerConfig, TrackerCoreConfig,
        UdpTrackerConfig,
    };

    fn create_test_tracker_config() -> TrackerConfig {
        TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite {
                    database_name: "test_tracker.db".to_string(),
                },
                private: true,
            },
            udp_trackers: vec![
                UdpTrackerConfig {
                    bind_address: "0.0.0.0:6868".to_string(),
                },
                UdpTrackerConfig {
                    bind_address: "0.0.0.0:6969".to_string(),
                },
            ],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
            http_api: HttpApiConfig {
                admin_token: "test_admin_token".to_string(),
            },
        }
    }

    #[test]
    fn it_should_create_context_from_tracker_config() {
        let config = create_test_tracker_config();
        let context = TrackerContext::from_config(&config);

        assert_eq!(context.tracker_database_name, "test_tracker.db");
        assert!(context.tracker_core_private);
        assert_eq!(context.udp_trackers.len(), 2);
        assert_eq!(context.udp_trackers[0].bind_address, "0.0.0.0:6868");
        assert_eq!(context.udp_trackers[1].bind_address, "0.0.0.0:6969");
        assert_eq!(context.http_trackers.len(), 1);
        assert_eq!(context.http_trackers[0].bind_address, "0.0.0.0:7070");
    }

    #[test]
    fn it_should_create_default_context() {
        let context = TrackerContext::default_config();

        assert_eq!(context.tracker_database_name, "sqlite3.db");
        assert!(!context.tracker_core_private);
        assert_eq!(context.udp_trackers.len(), 2);
        assert_eq!(context.http_trackers.len(), 1);
    }

    #[test]
    fn it_should_support_default_trait() {
        let context = TrackerContext::default();

        assert_eq!(context.tracker_database_name, "sqlite3.db");
        assert!(!context.tracker_core_private);
    }

    #[test]
    fn it_should_be_cloneable() {
        let config = create_test_tracker_config();
        let context = TrackerContext::from_config(&config);
        let cloned = context.clone();

        assert_eq!(context.tracker_database_name, cloned.tracker_database_name);
        assert_eq!(context.tracker_core_private, cloned.tracker_core_private);
        assert_eq!(context.udp_trackers.len(), cloned.udp_trackers.len());
        assert_eq!(context.http_trackers.len(), cloned.http_trackers.len());
    }

    #[test]
    fn it_should_support_debug_formatting() {
        let context = TrackerContext::default_config();
        let debug_output = format!("{context:?}");

        assert!(debug_output.contains("TrackerContext"));
        assert!(debug_output.contains("tracker_database_name"));
    }
}
