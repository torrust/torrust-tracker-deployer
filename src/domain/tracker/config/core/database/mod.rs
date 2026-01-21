//! Database configuration for Tracker
//!
//! This module defines the database backend configuration options
//! for the Torrust Tracker.
//!
//! # Module Structure
//!
//! - `sqlite` - `SQLite` database configuration
//! - `mysql` - `MySQL` database configuration
//!
//! New database drivers can be added by creating a new module with
//! the driver's configuration struct and adding a variant to `DatabaseConfig`.

use serde::{Deserialize, Serialize};

mod mysql;
mod sqlite;

pub use mysql::{MysqlConfig, MysqlConfigError};
pub use sqlite::{SqliteConfig, SqliteConfigError};

/// `SQLite` driver name constant
pub const DRIVER_SQLITE: &str = "sqlite3";

/// `MySQL` driver name constant
pub const DRIVER_MYSQL: &str = "mysql";

/// Database configuration for Tracker
///
/// Supports multiple database backends. Currently implemented:
/// - `SQLite` (file-based, development and small deployments)
/// - `MySQL` (planned for production deployments)
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::{DatabaseConfig, SqliteConfig};
///
/// // SQLite configuration
/// let sqlite = DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap());
///
/// // MySQL configuration
/// // use torrust_tracker_deployer_lib::domain::tracker::MysqlConfig;
/// // use torrust_tracker_deployer_lib::shared::Password;
/// // let mysql = DatabaseConfig::Mysql(MysqlConfig::new(
/// //     "localhost", 3306, "tracker", "tracker_user", Password::from("secure_password")
/// // ).unwrap());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "driver", content = "config")]
pub enum DatabaseConfig {
    /// `SQLite` file-based database
    #[serde(rename = "sqlite3")]
    Sqlite(SqliteConfig),
    /// `MySQL` server-based database
    #[serde(rename = "mysql")]
    Mysql(MysqlConfig),
}

impl DatabaseConfig {
    /// Returns the database driver name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::{DatabaseConfig, SqliteConfig};
    ///
    /// let config = DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap());
    /// assert_eq!(config.driver_name(), "sqlite3");
    /// ```
    #[must_use]
    pub fn driver_name(&self) -> &str {
        match self {
            Self::Sqlite(..) => DRIVER_SQLITE,
            Self::Mysql(..) => DRIVER_MYSQL,
        }
    }

    /// Returns the database name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::{DatabaseConfig, SqliteConfig};
    ///
    /// let config = DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap());
    /// assert_eq!(config.database_name(), "tracker.db");
    /// ```
    #[must_use]
    pub fn database_name(&self) -> &str {
        match self {
            Self::Sqlite(config) => config.database_name(),
            Self::Mysql(config) => config.database_name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::Password;

    #[test]
    fn it_should_create_sqlite_database_config() {
        let config = DatabaseConfig::Sqlite(SqliteConfig::new("test.db").unwrap());

        assert_eq!(config.driver_name(), "sqlite3");
        assert_eq!(config.database_name(), "test.db");
    }

    #[test]
    fn it_should_serialize_sqlite_config() {
        let config = DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap());

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["driver"], "sqlite3");
        assert_eq!(json["config"]["database_name"], "tracker.db");
    }

    #[test]
    fn it_should_deserialize_sqlite_config() {
        let json = r#"{"driver": "sqlite3", "config": {"database_name": "tracker.db"}}"#;
        let config: DatabaseConfig = serde_json::from_str(json).unwrap();

        match config {
            DatabaseConfig::Sqlite(sqlite_config) => {
                assert_eq!(sqlite_config.database_name(), "tracker.db");
            }
            DatabaseConfig::Mysql(..) => panic!("Expected Sqlite variant"),
        }
    }

    #[test]
    fn it_should_create_mysql_database_config() {
        let config = DatabaseConfig::Mysql(
            MysqlConfig::new(
                "localhost",
                3306,
                "tracker",
                "tracker_user",
                Password::from("secure_password"),
            )
            .unwrap(),
        );

        assert_eq!(config.driver_name(), "mysql");
        assert_eq!(config.database_name(), "tracker");
    }

    #[test]
    fn it_should_serialize_mysql_config() {
        let config = DatabaseConfig::Mysql(
            MysqlConfig::new(
                "mysql",
                3306,
                "tracker",
                "tracker_user",
                Password::from("pass123"),
            )
            .unwrap(),
        );

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["driver"], "mysql");
        assert_eq!(json["config"]["host"], "mysql");
        assert_eq!(json["config"]["port"], 3306);
        assert_eq!(json["config"]["database_name"], "tracker");
        assert_eq!(json["config"]["username"], "tracker_user");
        assert_eq!(json["config"]["password"], "pass123");
    }

    #[test]
    fn it_should_deserialize_mysql_config() {
        let json = r#"{
            "driver": "mysql",
            "config": {
                "host": "localhost",
                "port": 3306,
                "database_name": "tracker",
                "username": "tracker_user",
                "password": "secure_password"
            }
        }"#;
        let config: DatabaseConfig = serde_json::from_str(json).unwrap();

        match config {
            DatabaseConfig::Mysql(mysql_config) => {
                assert_eq!(mysql_config.host(), "localhost");
                assert_eq!(mysql_config.port(), 3306);
                assert_eq!(mysql_config.database_name(), "tracker");
                assert_eq!(mysql_config.username(), "tracker_user");
                assert_eq!(mysql_config.password(), &Password::from("secure_password"));
            }
            DatabaseConfig::Sqlite(..) => panic!("Expected Mysql variant"),
        }
    }
}
