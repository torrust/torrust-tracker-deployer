//! Database configuration for Docker Compose templates

// External crates
use serde::Serialize;

/// `SQLite` driver name
pub const DRIVER_SQLITE: &str = "sqlite3";

/// `MySQL` driver name
pub const DRIVER_MYSQL: &str = "mysql";

/// Database configuration for docker-compose template
#[derive(Serialize, Debug, Clone)]
pub struct DatabaseConfig {
    /// Database driver: "sqlite3" or "mysql"
    pub driver: String,
    /// MySQL-specific configuration (only present when driver == "mysql")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql: Option<MysqlSetupConfig>,
}

impl DatabaseConfig {
    /// Get the database driver name
    #[must_use]
    pub fn driver(&self) -> &str {
        &self.driver
    }

    /// Get the `MySQL` setup configuration if present
    #[must_use]
    pub fn mysql(&self) -> Option<&MysqlSetupConfig> {
        self.mysql.as_ref()
    }
}

/// `MySQL` setup configuration for Docker Compose initialization
///
/// This configuration is used to set up a new `MySQL` database in Docker Compose.
/// It includes the root password needed for database initialization, unlike the
/// domain `MysqlConfig` which is used for connecting to an existing database.
///
/// Key differences from domain `MysqlConfig`:
/// - Includes `root_password` for database initialization
/// - Used for Docker Compose environment variable setup
/// - Does not include `host` (always the service name in Docker Compose)
#[derive(Serialize, Debug, Clone)]
pub struct MysqlSetupConfig {
    /// `MySQL` root password for database initialization
    pub root_password: String,
    /// `MySQL` database name to create
    pub database: String,
    /// `MySQL` user to create
    pub user: String,
    /// `MySQL` password for the created user
    pub password: String,
    /// `MySQL` port
    pub port: u16,
}
