//! Tracker Core configuration section (application DTO)
//!
//! This module provides the DTO for tracker core configuration,
//! used for JSON deserialization and validation before converting
//! to domain types.
//!
//! # Conversion Pattern
//!
//! Uses `TryFrom` for idiomatic Rust conversion from DTO to domain type.
//! See ADR: `docs/decisions/tryfrom-for-dto-to-domain-conversion.md`

use std::convert::TryFrom;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::{DatabaseConfig, MysqlConfig, SqliteConfig, TrackerCoreConfig};
use crate::shared::{generate_random_password, Password, PlainPassword};

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
        /// Optional `MySQL` root password
        ///
        /// When provided, used as `MYSQL_ROOT_PASSWORD` in the rendered `.env` file.
        /// When absent, a cryptographically random password is generated at environment creation time.
        #[serde(default)]
        root_password: Option<PlainPassword>,
    },
}

impl TryFrom<DatabaseSection> for DatabaseConfig {
    type Error = CreateConfigError;

    fn try_from(section: DatabaseSection) -> Result<Self, Self::Error> {
        match section {
            DatabaseSection::Sqlite { database_name } => {
                let config = SqliteConfig::new(database_name)?;
                Ok(Self::Sqlite(config))
            }
            DatabaseSection::Mysql {
                host,
                port,
                database_name,
                username,
                password,
                root_password,
            } => {
                let root_password = root_password
                    .map_or_else(generate_random_password, |p| Password::from(p.as_str()));
                let config = MysqlConfig::new(
                    host,
                    port,
                    database_name,
                    username,
                    Password::from(password.as_str()),
                    root_password,
                )?;
                Ok(Self::Mysql(config))
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

impl TryFrom<TrackerCoreSection> for TrackerCoreConfig {
    type Error = CreateConfigError;

    fn try_from(section: TrackerCoreSection) -> Result<Self, Self::Error> {
        let database_config: DatabaseConfig = section.database.try_into()?;
        Ok(Self::new(database_config, section.private))
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

        let config: TrackerCoreConfig = section.try_into().unwrap();

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

        let config: TrackerCoreConfig = section.try_into().unwrap();

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
                root_password: None,
            },
            private: false,
        };

        let config: TrackerCoreConfig = section.try_into().unwrap();

        let DatabaseConfig::Mysql(mysql) = config.database() else {
            panic!("expected MySQL config");
        };
        assert_eq!(mysql.host(), "localhost");
        assert_eq!(mysql.port(), 3306);
        assert_eq!(mysql.database_name(), "tracker");
        assert_eq!(mysql.username(), "tracker_user");
        assert_eq!(mysql.password().expose_secret(), "secure_password");
        // root_password is generated randomly — just verify it is non-empty
        assert!(!mysql.root_password().expose_secret().is_empty());
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
                root_password: None,
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
                root_password: None,
            }
        );
        assert!(!section.private);
    }
}
