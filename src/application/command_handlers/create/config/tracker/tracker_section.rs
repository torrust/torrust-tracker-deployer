//! Tracker configuration section (application DTO)
//!
//! This module provides the aggregated DTO for complete tracker configuration,
//! used for JSON deserialization and validation before converting to domain types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    HealthCheckApiSection, HttpApiSection, HttpTrackerSection, TrackerCoreSection,
    UdpTrackerSection,
};
use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::{
    HealthCheckApiConfig, HttpApiConfig, HttpTrackerConfig, TrackerConfig, UdpTrackerConfig,
};

/// Tracker configuration section (application DTO)
///
/// Aggregates all tracker configuration sections: core, UDP trackers,
/// HTTP trackers, and HTTP API.
///
/// # Examples
///
/// ```json
/// {
///   "core": {
///     "database": {
///       "driver": "sqlite3",
///       "database_name": "tracker.db"
///     },
///     "private": false
///   },
///   "udp_trackers": [
///     { "bind_address": "0.0.0.0:6969" }
///   ],
///   "http_trackers": [
///     { "bind_address": "0.0.0.0:7070" }
///   ],
///   "http_api": {
///     "bind_address": "0.0.0.0:1212",
///     "admin_token": "MyAccessToken"
///   },
///   "health_check_api": {
///     "bind_address": "127.0.0.1:1313"
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TrackerSection {
    /// Core tracker configuration (database, privacy mode)
    pub core: TrackerCoreSection,
    /// UDP tracker instances
    pub udp_trackers: Vec<UdpTrackerSection>,
    /// HTTP tracker instances
    pub http_trackers: Vec<HttpTrackerSection>,
    /// HTTP API configuration
    pub http_api: HttpApiSection,
    /// Health Check API configuration
    pub health_check_api: HealthCheckApiSection,
}

impl TrackerSection {
    /// Converts this DTO to the domain `TrackerConfig` type.
    ///
    /// # Errors
    ///
    /// Returns error if any of the nested sections fail validation:
    /// - Invalid bind address formats
    /// - Invalid database configuration
    /// - Socket address conflicts (multiple services on same IP:Port:Protocol)
    pub fn to_tracker_config(&self) -> Result<TrackerConfig, CreateConfigError> {
        let core = self.core.to_tracker_core_config()?;

        let udp_trackers: Result<Vec<UdpTrackerConfig>, CreateConfigError> = self
            .udp_trackers
            .iter()
            .map(UdpTrackerSection::to_udp_tracker_config)
            .collect();

        let http_trackers: Result<Vec<HttpTrackerConfig>, CreateConfigError> = self
            .http_trackers
            .iter()
            .map(HttpTrackerSection::to_http_tracker_config)
            .collect();

        let http_api: HttpApiConfig = self.http_api.to_http_api_config()?;

        let health_check_api: HealthCheckApiConfig =
            self.health_check_api.to_health_check_api_config()?;

        let config = TrackerConfig {
            core,
            udp_trackers: udp_trackers?,
            http_trackers: http_trackers?,
            http_api,
            health_check_api,
        };

        // Validate socket address uniqueness
        config.validate().map_err(CreateConfigError::from)?;

        Ok(config)
    }
}

impl Default for TrackerSection {
    /// Returns a default tracker configuration DTO suitable for development and testing
    ///
    /// # Default Values
    ///
    /// - Database: `SQLite` with filename "tracker.db"
    /// - Mode: Public tracker (private = false)
    /// - UDP trackers: One instance on "0.0.0.0:6969"
    /// - HTTP trackers: One instance on "0.0.0.0:7070"
    /// - HTTP API: Bind address "0.0.0.0:1212"
    /// - Admin token: `MyAccessToken`
    fn default() -> Self {
        Self {
            core: TrackerCoreSection {
                database: super::tracker_core_section::DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![UdpTrackerSection {
                bind_address: "0.0.0.0:6969".to_string(),
            }],
            http_trackers: vec![HttpTrackerSection {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:1212".to_string(),
                admin_token: "MyAccessToken".to_string(),
            },
            health_check_api: HealthCheckApiSection::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::*;
    use crate::application::command_handlers::create::config::tracker::tracker_core_section::DatabaseSection;
    use crate::domain::tracker::{DatabaseConfig, SqliteConfig};

    #[test]
    fn it_should_convert_to_domain_config_when_transforming_tracker_section() {
        let section = TrackerSection {
            core: TrackerCoreSection {
                database: DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![UdpTrackerSection {
                bind_address: "0.0.0.0:6969".to_string(),
            }],
            http_trackers: vec![HttpTrackerSection {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:1212".to_string(),
                admin_token: "MyAccessToken".to_string(),
            },
            health_check_api: HealthCheckApiSection::default(),
        };

        let config = section.to_tracker_config().unwrap();

        assert_eq!(
            config.core.database,
            DatabaseConfig::Sqlite(SqliteConfig {
                database_name: "tracker.db".to_string()
            })
        );
        assert!(!config.core.private);
        assert_eq!(config.udp_trackers.len(), 1);
        assert_eq!(config.http_trackers.len(), 1);
        assert_eq!(
            config.http_api.bind_address,
            "0.0.0.0:1212".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn it_should_handle_multiple_tracker_instances_when_configured() {
        let section = TrackerSection {
            core: TrackerCoreSection {
                database: DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![
                UdpTrackerSection {
                    bind_address: "0.0.0.0:6969".to_string(),
                },
                UdpTrackerSection {
                    bind_address: "0.0.0.0:6970".to_string(),
                },
            ],
            http_trackers: vec![
                HttpTrackerSection {
                    bind_address: "0.0.0.0:7070".to_string(),
                },
                HttpTrackerSection {
                    bind_address: "0.0.0.0:7071".to_string(),
                },
            ],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:1212".to_string(),
                admin_token: "MyAccessToken".to_string(),
            },
            health_check_api: HealthCheckApiSection::default(),
        };

        let config = section.to_tracker_config().unwrap();

        assert_eq!(config.udp_trackers.len(), 2);
        assert_eq!(config.http_trackers.len(), 2);
    }

    #[test]
    fn it_should_fail_when_bind_address_is_invalid() {
        let section = TrackerSection {
            core: TrackerCoreSection {
                database: DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![UdpTrackerSection {
                bind_address: "invalid".to_string(),
            }],
            http_trackers: vec![],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:1212".to_string(),
                admin_token: "MyAccessToken".to_string(),
            },
            health_check_api: HealthCheckApiSection::default(),
        };

        let result = section.to_tracker_config();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::InvalidBindAddress { .. }
        ));
    }

    #[test]
    fn it_should_serialize_to_json_when_converting_section() {
        let section = TrackerSection {
            core: TrackerCoreSection {
                database: DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![UdpTrackerSection {
                bind_address: "0.0.0.0:6969".to_string(),
            }],
            http_trackers: vec![HttpTrackerSection {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:1212".to_string(),
                admin_token: "MyAccessToken".to_string(),
            },
            health_check_api: HealthCheckApiSection::default(),
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("\"driver\":\"sqlite3\""));
        assert!(json.contains("\"udp_trackers\""));
        assert!(json.contains("\"http_trackers\""));
        assert!(json.contains("\"http_api\""));
    }

    #[test]
    fn it_should_deserialize_from_json_when_parsing_section() {
        let json = r#"{
            "core": {
                "database": {
                    "driver": "sqlite3",
                    "database_name": "tracker.db"
                },
                "private": true
            },
            "udp_trackers": [
                { "bind_address": "0.0.0.0:6969" }
            ],
            "http_trackers": [
                { "bind_address": "0.0.0.0:7070" }
            ],
            "http_api": {
                "bind_address": "0.0.0.0:1212",
                "admin_token": "MyAccessToken"
            },
            "health_check_api": {
                "bind_address": "127.0.0.1:1313"
            }
        }"#;

        let section: TrackerSection = serde_json::from_str(json).unwrap();

        assert!(section.core.private);
        assert_eq!(section.udp_trackers.len(), 1);
        assert_eq!(section.http_trackers.len(), 1);
    }

    #[test]
    fn it_should_reject_configuration_with_duplicate_socket_addresses() {
        // HTTP tracker and API on same port (TCP protocol conflict)
        let section = TrackerSection {
            core: TrackerCoreSection {
                database: DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![],
            http_trackers: vec![HttpTrackerSection {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:7070".to_string(),
                admin_token: "token".to_string(),
            },
            health_check_api: HealthCheckApiSection::default(),
        };

        let result = section.to_tracker_config();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::TrackerConfigValidation(_)
        ));
    }

    #[test]
    fn it_should_accept_udp_and_tcp_on_same_port() {
        // UDP and TCP can share the same port (different protocol spaces)
        let section = TrackerSection {
            core: TrackerCoreSection {
                database: DatabaseSection::Sqlite {
                    database_name: "tracker.db".to_string(),
                },
                private: false,
            },
            udp_trackers: vec![UdpTrackerSection {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
            http_trackers: vec![HttpTrackerSection {
                bind_address: "0.0.0.0:7070".to_string(),
            }],
            http_api: HttpApiSection {
                bind_address: "0.0.0.0:1212".to_string(),
                admin_token: "token".to_string(),
            },
            health_check_api: HealthCheckApiSection::default(),
        };

        let result = section.to_tracker_config();
        assert!(result.is_ok());
    }
}
