//! Core tracker configuration

use serde::{Deserialize, Deserializer, Serialize};

mod database;

pub use database::{
    DatabaseConfig, MysqlConfig, MysqlConfigError, SqliteConfig, SqliteConfigError,
};

/// Core tracker configuration options
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TrackerCoreConfig {
    /// Database configuration (`SQLite`, `MySQL`, etc.)
    database: DatabaseConfig,

    /// Tracker mode: true for private tracker, false for public
    private: bool,
}

impl TrackerCoreConfig {
    /// Creates a new core tracker configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::{
    ///     TrackerCoreConfig, DatabaseConfig, SqliteConfig,
    /// };
    ///
    /// let core = TrackerCoreConfig::new(
    ///     DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
    ///     true,
    /// );
    ///
    /// assert_eq!(core.database().database_name(), "tracker.db");
    /// assert!(core.private());
    /// ```
    #[must_use]
    pub fn new(database: DatabaseConfig, private: bool) -> Self {
        Self { database, private }
    }

    /// Returns a reference to the database configuration
    #[must_use]
    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }

    /// Returns whether this is a private tracker
    #[must_use]
    pub fn private(&self) -> bool {
        self.private
    }
}

/// Intermediate struct for deserialization
#[derive(Deserialize)]
struct TrackerCoreConfigRaw {
    database: DatabaseConfig,
    private: bool,
}

impl<'de> Deserialize<'de> for TrackerCoreConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = TrackerCoreConfigRaw::deserialize(deserializer)?;
        Ok(Self::new(raw.database, raw.private))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_core_config() {
        let core = TrackerCoreConfig::new(
            DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
            true,
        );

        assert_eq!(core.database().database_name(), "tracker.db");
        assert!(core.private());
    }

    #[test]
    fn it_should_serialize_core_config() {
        let core = TrackerCoreConfig::new(
            DatabaseConfig::Sqlite(SqliteConfig::new("test.db").unwrap()),
            false,
        );

        let json = serde_json::to_value(&core).unwrap();
        assert_eq!(json["private"], false);
    }
}
