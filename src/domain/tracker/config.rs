//! Tracker configuration domain types
//!
//! This module contains the main tracker configuration and component types
//! used for deploying the Torrust Tracker.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::{DatabaseConfig, SqliteConfig};

/// Tracker deployment configuration
///
/// This structure mirrors the real tracker configuration but only includes
/// user-configurable fields that are exposed via the environment.json file.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::{
///     TrackerConfig, TrackerCoreConfig, DatabaseConfig, SqliteConfig,
///     UdpTrackerConfig, HttpTrackerConfig, HttpApiConfig
/// };
///
/// let tracker_config = TrackerConfig {
///     core: TrackerCoreConfig {
///         database: DatabaseConfig::Sqlite(SqliteConfig {
///             database_name: "tracker.db".to_string(),
///         }),
///         private: false,
///     },
///     udp_trackers: vec![
///         UdpTrackerConfig { bind_address: "0.0.0.0:6969".parse().unwrap() },
///     ],
///     http_trackers: vec![
///         HttpTrackerConfig { bind_address: "0.0.0.0:7070".parse().unwrap() },
///     ],
///     http_api: HttpApiConfig {
///         bind_address: "0.0.0.0:1212".parse().unwrap(),
///         admin_token: "MyAccessToken".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerConfig {
    /// Core tracker configuration
    pub core: TrackerCoreConfig,

    /// UDP tracker instances
    pub udp_trackers: Vec<UdpTrackerConfig>,

    /// HTTP tracker instances
    pub http_trackers: Vec<HttpTrackerConfig>,

    /// HTTP API configuration
    pub http_api: HttpApiConfig,
}

/// Core tracker configuration options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerCoreConfig {
    /// Database configuration (`SQLite`, `MySQL`, etc.)
    pub database: DatabaseConfig,

    /// Tracker mode: true for private tracker, false for public
    pub private: bool,
}

/// UDP tracker bind configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UdpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:6868")
    #[serde(
        serialize_with = "serialize_socket_addr",
        deserialize_with = "deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,
}

/// HTTP tracker bind configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:7070")
    #[serde(
        serialize_with = "serialize_socket_addr",
        deserialize_with = "deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,
}

/// HTTP API configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpApiConfig {
    /// Bind address (e.g., "0.0.0.0:1212")
    #[serde(
        serialize_with = "serialize_socket_addr",
        deserialize_with = "deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,

    /// Admin access token for HTTP API authentication
    pub admin_token: String,
}

impl Default for TrackerConfig {
    /// Returns a default tracker configuration suitable for development and testing
    ///
    /// # Default Values
    ///
    /// - Database: `SQLite` with filename "tracker.db"
    /// - Mode: Public tracker (private = false)
    /// - UDP trackers: One instance on port 6969
    /// - HTTP trackers: One instance on port 7070
    /// - HTTP API: Bind address 0.0.0.0:1212
    /// - Admin token: `MyAccessToken`
    fn default() -> Self {
        Self {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite(SqliteConfig {
                    database_name: "tracker.db".to_string(),
                }),
                private: false,
            },
            udp_trackers: vec![UdpTrackerConfig {
                bind_address: "0.0.0.0:6969".parse().expect("valid address"),
            }],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "0.0.0.0:7070".parse().expect("valid address"),
            }],
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().expect("valid address"),
                admin_token: "MyAccessToken".to_string(),
            },
        }
    }
}

fn serialize_socket_addr<S>(addr: &SocketAddr, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&addr.to_string())
}

fn deserialize_socket_addr<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_tracker_config() {
        let config = TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite(SqliteConfig {
                    database_name: "tracker.db".to_string(),
                }),
                private: true,
            },
            udp_trackers: vec![UdpTrackerConfig {
                bind_address: "0.0.0.0:6868".parse().unwrap(),
            }],
            http_trackers: vec![HttpTrackerConfig {
                bind_address: "0.0.0.0:7070".parse().unwrap(),
            }],
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().unwrap(),
                admin_token: "test_token".to_string(),
            },
        };

        assert_eq!(config.core.database.database_name(), "tracker.db");
        assert!(config.core.private);
        assert_eq!(config.udp_trackers.len(), 1);
        assert_eq!(config.http_trackers.len(), 1);
    }

    #[test]
    fn it_should_serialize_tracker_config() {
        let config = TrackerConfig {
            core: TrackerCoreConfig {
                database: DatabaseConfig::Sqlite(SqliteConfig {
                    database_name: "test.db".to_string(),
                }),
                private: false,
            },
            udp_trackers: vec![],
            http_trackers: vec![],
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().unwrap(),
                admin_token: "token123".to_string(),
            },
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["core"]["private"], false);
        assert_eq!(json["http_api"]["admin_token"], "token123");
    }

    #[test]
    fn it_should_create_default_tracker_config() {
        let config = TrackerConfig::default();

        // Verify default database configuration
        assert_eq!(config.core.database.database_name(), "tracker.db");
        assert_eq!(config.core.database.driver_name(), "sqlite3");

        // Verify public tracker mode
        assert!(!config.core.private);

        // Verify UDP trackers (1 instance)
        assert_eq!(config.udp_trackers.len(), 1);
        assert_eq!(
            config.udp_trackers[0].bind_address,
            "0.0.0.0:6969".parse::<SocketAddr>().unwrap()
        );

        // Verify HTTP trackers (1 instance)
        assert_eq!(config.http_trackers.len(), 1);
        assert_eq!(
            config.http_trackers[0].bind_address,
            "0.0.0.0:7070".parse::<SocketAddr>().unwrap()
        );

        // Verify HTTP API configuration
        assert_eq!(
            config.http_api.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.http_api.admin_token, "MyAccessToken");
    }
}
