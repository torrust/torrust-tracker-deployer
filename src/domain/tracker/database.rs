//! Database configuration for Tracker
//!
//! This module defines the database backend configuration options
//! for the Torrust Tracker.

use serde::{Deserialize, Serialize};

/// Database configuration for Tracker
///
/// Supports multiple database backends. Currently implemented:
/// - `SQLite` (file-based, development and small deployments)
/// - `MySQL` (planned for production deployments)
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::DatabaseConfig;
///
/// // SQLite configuration
/// let sqlite = DatabaseConfig::Sqlite {
///     database_name: "tracker.db".to_string(),
/// };
///
/// // MySQL configuration (future)
/// // let mysql = DatabaseConfig::Mysql {
/// //     host: "localhost".to_string(),
/// //     port: 3306,
/// //     database_name: "tracker".to_string(),
/// //     username: "tracker_user".to_string(),
/// //     password: "secure_password".to_string(),
/// // };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "driver")]
pub enum DatabaseConfig {
    /// `SQLite` file-based database
    #[serde(rename = "sqlite3")]
    Sqlite {
        /// Database file name (e.g., "tracker.db", "sqlite3.db")
        /// Path is relative to the tracker's data directory
        database_name: String,
    },
    /// `MySQL` server-based database
    #[serde(rename = "mysql")]
    Mysql {
        /// `MySQL` server host (e.g., "localhost", "mysql")
        host: String,
        /// `MySQL` server port (typically 3306)
        port: u16,
        /// Database name
        database_name: String,
        /// Database username
        username: String,
        /// Database password
        password: String,
    },
}

impl DatabaseConfig {
    /// Returns the database driver name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::DatabaseConfig;
    ///
    /// let config = DatabaseConfig::Sqlite {
    ///     database_name: "tracker.db".to_string(),
    /// };
    /// assert_eq!(config.driver_name(), "sqlite3");
    /// ```
    #[must_use]
    pub fn driver_name(&self) -> &str {
        match self {
            Self::Sqlite { .. } => "sqlite3",
            Self::Mysql { .. } => "mysql",
        }
    }

    /// Returns the database name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::DatabaseConfig;
    ///
    /// let config = DatabaseConfig::Sqlite {
    ///     database_name: "tracker.db".to_string(),
    /// };
    /// assert_eq!(config.database_name(), "tracker.db");
    /// ```
    #[must_use]
    pub fn database_name(&self) -> &str {
        match self {
            Self::Sqlite { database_name } | Self::Mysql { database_name, .. } => database_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_sqlite_database_config() {
        let config = DatabaseConfig::Sqlite {
            database_name: "test.db".to_string(),
        };

        assert_eq!(config.driver_name(), "sqlite3");
        assert_eq!(config.database_name(), "test.db");
    }

    #[test]
    fn it_should_serialize_sqlite_config() {
        let config = DatabaseConfig::Sqlite {
            database_name: "tracker.db".to_string(),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["driver"], "sqlite3");
        assert_eq!(json["database_name"], "tracker.db");
    }

    #[test]
    fn it_should_deserialize_sqlite_config() {
        let json = r#"{"driver": "sqlite3", "database_name": "tracker.db"}"#;
        let config: DatabaseConfig = serde_json::from_str(json).unwrap();

        match config {
            DatabaseConfig::Sqlite { database_name } => {
                assert_eq!(database_name, "tracker.db");
            }
            DatabaseConfig::Mysql { .. } => panic!("Expected Sqlite variant"),
        }
    }

    #[test]
    fn it_should_create_mysql_database_config() {
        let config = DatabaseConfig::Mysql {
            host: "localhost".to_string(),
            port: 3306,
            database_name: "tracker".to_string(),
            username: "tracker_user".to_string(),
            password: "secure_password".to_string(),
        };

        assert_eq!(config.driver_name(), "mysql");
        assert_eq!(config.database_name(), "tracker");
    }

    #[test]
    fn it_should_serialize_mysql_config() {
        let config = DatabaseConfig::Mysql {
            host: "mysql".to_string(),
            port: 3306,
            database_name: "tracker".to_string(),
            username: "tracker_user".to_string(),
            password: "pass123".to_string(),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["driver"], "mysql");
        assert_eq!(json["host"], "mysql");
        assert_eq!(json["port"], 3306);
        assert_eq!(json["database_name"], "tracker");
        assert_eq!(json["username"], "tracker_user");
        assert_eq!(json["password"], "pass123");
    }

    #[test]
    fn it_should_deserialize_mysql_config() {
        let json = r#"{
            "driver": "mysql",
            "host": "localhost",
            "port": 3306,
            "database_name": "tracker",
            "username": "tracker_user",
            "password": "secure_password"
        }"#;
        let config: DatabaseConfig = serde_json::from_str(json).unwrap();

        match config {
            DatabaseConfig::Mysql {
                host,
                port,
                database_name,
                username,
                password,
            } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 3306);
                assert_eq!(database_name, "tracker");
                assert_eq!(username, "tracker_user");
                assert_eq!(password, "secure_password");
            }
            DatabaseConfig::Sqlite { .. } => panic!("Expected Mysql variant"),
        }
    }
}
