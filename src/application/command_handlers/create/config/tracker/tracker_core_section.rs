//! Tracker Core configuration section (application DTO)
//!
//! This module provides the DTO for tracker core configuration,
//! used for JSON deserialization and validation before converting
//! to domain types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::{DatabaseConfig, MysqlConfig, SqliteConfig, TrackerCoreConfig};
use crate::shared::{Password, PlainPassword};

/// Database configuration section (application DTO)
///
/// Mirrors the domain `DatabaseConfig` enum but at the application layer.
/// Supports both `SQLite` and `MySQL` database backends.
///
/// # Examples
///
/// ```json
/// {
///   "driver": "sqlite3",
///   "database_name": "tracker.db"
/// }
/// ```
///
/// ```json
/// {
///   "driver": "mysql",
///   "host": "localhost",
///   "port": 3306,
///   "database_name": "tracker",
///   "username": "tracker_user",
///   "password": "secure_password"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "driver")]
pub enum DatabaseSection {
    /// `SQLite` file-based database
    #[serde(rename = "sqlite3")]
    Sqlite {
        /// Database file name
        database_name: String,
    },
    /// `MySQL` server-based database
    #[serde(rename = "mysql")]
    Mysql {
        /// `MySQL` server host
        host: String,
        /// `MySQL` server port
        port: u16,
        /// Database name
        database_name: String,
        /// Database username
        username: String,
        /// Database password (plain text during DTO serialization/deserialization)
        ///
        /// Uses `PlainPassword` type alias to explicitly mark this as a temporarily visible secret.
        /// Converted to secure `Password` type in `to_database_config()` at the DTO-to-domain boundary.
        password: PlainPassword,
    },
}

impl DatabaseSection {
    /// Converts this DTO to the domain `DatabaseConfig` type.
    ///
    /// # Errors
    ///
    /// - `SqliteConfigInvalid` if `SQLite` database name validation fails
    /// - `MysqlConfigInvalid` if `MySQL` configuration validation fails
    pub fn to_database_config(&self) -> Result<DatabaseConfig, CreateConfigError> {
        match self {
            Self::Sqlite { database_name } => {
                let config = SqliteConfig::new(database_name.clone())?;
                Ok(DatabaseConfig::Sqlite(config))
            }
            Self::Mysql {
                host,
                port,
                database_name,
                username,
                password,
            } => {
                let config = MysqlConfig::new(
                    host.clone(),
                    *port,
                    database_name.clone(),
                    username.clone(),
                    Password::from(password.as_str()),
                )?;
                Ok(DatabaseConfig::Mysql(config))
            }
        }
    }
}

/// Tracker core configuration section (application DTO)
///
/// Contains core tracker settings like database and privacy mode.
///
/// # Examples
///
/// ```json
/// {
///   "database": {
///     "driver": "sqlite3",
///     "database_name": "tracker.db"
///   },
///   "private": false
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TrackerCoreSection {
    /// Database configuration
    pub database: DatabaseSection,
    /// Privacy mode: true for private tracker, false for public
    pub private: bool,
}

impl TrackerCoreSection {
    /// Converts this DTO to the domain `TrackerCoreConfig` type.
    ///
    /// # Errors
    ///
    /// Returns error if database validation fails.
    pub fn to_tracker_core_config(&self) -> Result<TrackerCoreConfig, CreateConfigError> {
        Ok(TrackerCoreConfig::new(
            self.database.to_database_config()?,
            self.private,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_to_domain_config_when_transforming_tracker_core_section() {
        let section = TrackerCoreSection {
            database: DatabaseSection::Sqlite {
                database_name: "tracker.db".to_string(),
            },
            private: false,
        };

        let config = section.to_tracker_core_config().unwrap();

        assert_eq!(
            *config.database(),
            DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap())
        );
        assert!(!config.private());
    }

    #[test]
    fn it_should_handle_private_mode_flag_when_configuring_tracker() {
        let section = TrackerCoreSection {
            database: DatabaseSection::Sqlite {
                database_name: "private.db".to_string(),
            },
            private: true,
        };

        let config = section.to_tracker_core_config().unwrap();

        assert!(config.private());
    }

    #[test]
    fn it_should_serialize_to_json_when_converting_core_section() {
        let section = TrackerCoreSection {
            database: DatabaseSection::Sqlite {
                database_name: "tracker.db".to_string(),
            },
            private: false,
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("\"driver\":\"sqlite3\""));
        assert!(json.contains("\"database_name\":\"tracker.db\""));
        assert!(json.contains("\"private\":false"));
    }

    #[test]
    fn it_should_deserialize_from_json_when_parsing_core_section() {
        let json = r#"{
            "database": {
                "driver": "sqlite3",
                "database_name": "tracker.db"
            },
            "private": true
        }"#;

        let section: TrackerCoreSection = serde_json::from_str(json).unwrap();

        assert_eq!(
            section.database,
            DatabaseSection::Sqlite {
                database_name: "tracker.db".to_string()
            }
        );
        assert!(section.private);
    }

    #[test]
    fn it_should_convert_mysql_to_domain_config_when_transforming_tracker_core_section() {
        let section = TrackerCoreSection {
            database: DatabaseSection::Mysql {
                host: "localhost".to_string(),
                port: 3306,
                database_name: "tracker".to_string(),
                username: "tracker_user".to_string(),
                password: "secure_password".to_string(),
            },
            private: false,
        };

        let config = section.to_tracker_core_config().unwrap();

        assert_eq!(
            *config.database(),
            DatabaseConfig::Mysql(
                MysqlConfig::new(
                    "localhost",
                    3306,
                    "tracker",
                    "tracker_user",
                    Password::from("secure_password"),
                )
                .unwrap()
            )
        );
        assert!(!config.private());
    }

    #[test]
    fn it_should_serialize_mysql_to_json_when_converting_core_section() {
        let section = TrackerCoreSection {
            database: DatabaseSection::Mysql {
                host: "mysql".to_string(),
                port: 3306,
                database_name: "tracker".to_string(),
                username: "tracker_user".to_string(),
                password: "pass123".to_string(),
            },
            private: false,
        };

        let json = serde_json::to_string(&section).unwrap();
        assert!(json.contains("\"driver\":\"mysql\""));
        assert!(json.contains("\"host\":\"mysql\""));
        assert!(json.contains("\"port\":3306"));
        assert!(json.contains("\"database_name\":\"tracker\""));
        assert!(json.contains("\"username\":\"tracker_user\""));
        assert!(json.contains("\"password\":\"pass123\""));
    }

    #[test]
    fn it_should_deserialize_mysql_from_json_when_parsing_core_section() {
        let json = r#"{
            "database": {
                "driver": "mysql",
                "host": "localhost",
                "port": 3306,
                "database_name": "tracker",
                "username": "tracker_user",
                "password": "secure_password"
            },
            "private": false
        }"#;

        let section: TrackerCoreSection = serde_json::from_str(json).unwrap();

        assert_eq!(
            section.database,
            DatabaseSection::Mysql {
                host: "localhost".to_string(),
                port: 3306,
                database_name: "tracker".to_string(),
                username: "tracker_user".to_string(),
                password: "secure_password".to_string(),
            }
        );
        assert!(!section.private);
    }
}
