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
    // Future: MySQL support
    // #[serde(rename = "mysql")]
    // Mysql {
    //     host: String,
    //     port: u16,
    //     database_name: String,
    //     username: String,
    //     password: String,
    // },
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
            // Self::Mysql { .. } => "mysql",
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
            Self::Sqlite { database_name } => database_name,
            // Self::Mysql { database_name, .. } => database_name,
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
            } // _ => panic!("Expected Sqlite variant"),
        }
    }
}
