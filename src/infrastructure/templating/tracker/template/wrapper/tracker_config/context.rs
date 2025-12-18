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

use crate::domain::environment::TrackerConfig;

/// Context for rendering tracker.toml.tera template
///
/// ## Current State (Phase 3)
///
/// This context contains fields for dynamic tracker configuration based on
/// the environment's tracker settings. Supports both `SQLite` and `MySQL` databases.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::tracker::TrackerContext;
/// use torrust_tracker_deployer_lib::domain::environment::{TrackerConfig, TrackerCoreConfig, DatabaseConfig, SqliteConfig, UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig};
///
/// let tracker_config = TrackerConfig {
///     core: TrackerCoreConfig {
///         database: DatabaseConfig::Sqlite(SqliteConfig {
///             database_name: "tracker.db".to_string(),
///         }),
///         private: true,
///     },
///     udp_trackers: vec![
///         UdpTrackerConfig { bind_address: "0.0.0.0:6868".parse().unwrap() },
///         UdpTrackerConfig { bind_address: "0.0.0.0:6969".parse().unwrap() },
///     ],
///     http_trackers: vec![
///         HttpTrackerConfig { bind_address: "0.0.0.0:7070".parse().unwrap() },
///     ],
///     http_api: HttpApiConfig {
///         bind_address: "0.0.0.0:1212".parse().unwrap(),
///         admin_token: "MyToken".to_string().into(),
///     },
/// };
/// let context = TrackerContext::from_config(&tracker_config);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct TrackerContext {
    /// Database driver: "sqlite3" or "mysql"
    pub database_driver: String,

    /// Database file name (e.g., "tracker.db", "sqlite3.db") - used for `SQLite`
    pub tracker_database_name: String,

    /// `MySQL` host (e.g., "mysql", "localhost") - only used when driver is "mysql"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql_host: Option<String>,

    /// `MySQL` port (typically 3306) - only used when driver is "mysql"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql_port: Option<u16>,

    /// `MySQL` database name - only used when driver is "mysql"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql_database: Option<String>,

    /// `MySQL` username - only used when driver is "mysql"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql_user: Option<String>,

    /// `MySQL` password - only used when driver is "mysql"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql_password: Option<String>,

    /// Whether tracker is in private mode
    pub tracker_core_private: bool,

    /// UDP tracker bind addresses
    pub udp_trackers: Vec<UdpTrackerEntry>,

    /// HTTP tracker bind addresses
    pub http_trackers: Vec<HttpTrackerEntry>,

    /// HTTP API bind address
    pub http_api_bind_address: String,
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
    pub fn from_config(config: &TrackerConfig) -> Self {
        use crate::domain::tracker::DatabaseConfig;

        let (mysql_host, mysql_port, mysql_database, mysql_user, mysql_password) =
            match &config.core.database {
                DatabaseConfig::Mysql(mysql_config) => (
                    Some(mysql_config.host.clone()),
                    Some(mysql_config.port),
                    Some(mysql_config.database_name.clone()),
                    Some(mysql_config.username.clone()),
                    Some(mysql_config.password.expose_secret().to_string()),
                ),
                DatabaseConfig::Sqlite(..) => (None, None, None, None, None),
            };

        Self {
            database_driver: config.core.database.driver_name().to_string(),
            tracker_database_name: config.core.database.database_name().to_string(),
            mysql_host,
            mysql_port,
            mysql_database,
            mysql_user,
            mysql_password,
            tracker_core_private: config.core.private,
            udp_trackers: config
                .udp_trackers
                .iter()
                .map(|t| UdpTrackerEntry {
                    bind_address: t.bind_address.to_string(),
                })
                .collect(),
            http_trackers: config
                .http_trackers
                .iter()
                .map(|t| HttpTrackerEntry {
                    bind_address: t.bind_address.to_string(),
                })
                .collect(),
            http_api_bind_address: config.http_api.bind_address.to_string(),
        }
    }

    /// Creates a default tracker context with hardcoded values
    ///
    /// Used when no tracker configuration is provided in environment.
    /// Provides backward compatibility with Phase 4 defaults.
    ///
    /// # Panics
    ///
    /// Panics if default IP addresses fail to parse (should never happen with valid constants).
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            database_driver: "sqlite3".to_string(),
            tracker_database_name: "sqlite3.db".to_string(),
            mysql_host: None,
            mysql_port: None,
            mysql_database: None,
            mysql_user: None,
            mysql_password: None,
            tracker_core_private: false,
            udp_trackers: vec![
                UdpTrackerEntry {
                    bind_address: "0.0.0.0:6868".parse().unwrap(),
                },
                UdpTrackerEntry {
                    bind_address: "0.0.0.0:6969".parse().unwrap(),
                },
            ],
            http_trackers: vec![HttpTrackerEntry {
                bind_address: "0.0.0.0:7070".parse().unwrap(),
            }],
            http_api_bind_address: "0.0.0.0:1212".parse().unwrap(),
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
        DatabaseConfig, HttpApiConfig, HttpTrackerConfig, MysqlConfig, SqliteConfig, TrackerConfig,
        TrackerCoreConfig, UdpTrackerConfig,
    };
    use crate::shared::Password;

    fn create_test_tracker_config() -> TrackerConfig {
        TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite(SqliteConfig {
                    database_name: "test_tracker.db".to_string(),
                }),
                private: true,
            },
            udp_trackers: vec![
                UdpTrackerConfig {
                    bind_address: "0.0.0.0:6868".parse().unwrap(),
                },
                UdpTrackerConfig {
                    bind_address: "0.0.0.0:6969".parse().unwrap(),
                },
            ],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "0.0.0.0:7070".parse().unwrap(),
            }],
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().unwrap(),
                admin_token: "test_admin_token".to_string().into(),
            },
        }
    }

    #[test]
    fn it_should_create_context_from_tracker_config() {
        let config = create_test_tracker_config();
        let context = TrackerContext::from_config(&config);

        assert_eq!(context.database_driver, "sqlite3");
        assert_eq!(context.tracker_database_name, "test_tracker.db");
        assert!(context.mysql_host.is_none());
        assert!(context.mysql_port.is_none());
        assert!(context.mysql_database.is_none());
        assert!(context.mysql_user.is_none());
        assert!(context.mysql_password.is_none());
        assert!(context.tracker_core_private);
        assert_eq!(context.udp_trackers.len(), 2);
        assert_eq!(context.udp_trackers[0].bind_address, "0.0.0.0:6868");
        assert_eq!(context.udp_trackers[1].bind_address, "0.0.0.0:6969");
        assert_eq!(context.http_trackers.len(), 1);
        assert_eq!(context.http_trackers[0].bind_address, "0.0.0.0:7070");
    }

    #[test]
    fn it_should_create_context_from_mysql_tracker_config() {
        let config = TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Mysql(MysqlConfig {
                    host: "mysql".to_string(),
                    port: 3306,
                    database_name: "tracker_db".to_string(),
                    username: "tracker_user".to_string(),
                    password: Password::from("secure_pass"),
                }),
                private: false,
            },
            udp_trackers: vec![UdpTrackerConfig {
                bind_address: "0.0.0.0:6969".parse().unwrap(),
            }],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "0.0.0.0:7070".parse().unwrap(),
            }],
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().unwrap(),
                admin_token: "test_token".to_string().into(),
            },
        };

        let context = TrackerContext::from_config(&config);

        assert_eq!(context.database_driver, "mysql");
        assert_eq!(context.tracker_database_name, "tracker_db");
        assert_eq!(context.mysql_host, Some("mysql".to_string()));
        assert_eq!(context.mysql_port, Some(3306));
        assert_eq!(context.mysql_database, Some("tracker_db".to_string()));
        assert_eq!(context.mysql_user, Some("tracker_user".to_string()));
        assert_eq!(context.mysql_password, Some("secure_pass".to_string()));
        assert!(!context.tracker_core_private);
    }

    #[test]
    fn it_should_create_default_context() {
        let context = TrackerContext::default_config();

        assert_eq!(context.database_driver, "sqlite3");
        assert_eq!(context.tracker_database_name, "sqlite3.db");
        assert!(context.mysql_host.is_none());
        assert!(context.mysql_port.is_none());
        assert!(context.mysql_database.is_none());
        assert!(context.mysql_user.is_none());
        assert!(context.mysql_password.is_none());
        assert!(!context.tracker_core_private);
        assert_eq!(context.udp_trackers.len(), 2);
        assert_eq!(context.http_trackers.len(), 1);
    }

    #[test]
    fn it_should_support_default_trait() {
        let context = TrackerContext::default();

        assert_eq!(context.database_driver, "sqlite3");
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
