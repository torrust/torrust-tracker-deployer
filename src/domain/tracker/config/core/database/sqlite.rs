//! `SQLite` database configuration

use serde::{Deserialize, Deserializer, Serialize};

/// Error type for `SQLite` configuration validation
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SqliteConfigError {
    /// Database name cannot be empty
    #[error("SQLite database name cannot be empty")]
    EmptyDatabaseName,
}

impl SqliteConfigError {
    /// Returns detailed help text for resolving this error
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::EmptyDatabaseName => {
                "SQLite database name cannot be empty.\n\
                 \n\
                 The database_name field specifies the SQLite file name.\n\
                 This file will be created in the tracker's data directory.\n\
                 \n\
                 Examples of valid database names:\n\
                 - tracker.db\n\
                 - sqlite3.db\n\
                 - data.sqlite\n\
                 \n\
                 Fix:\n\
                 Set the database_name field in your database configuration:\n\
                 \n\
                 \"database\": {\n\
                   \"driver\": \"sqlite3\",\n\
                   \"database_name\": \"tracker.db\"\n\
                 }"
            }
        }
    }
}

/// `SQLite` database configuration
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SqliteConfig {
    /// Database file name (e.g., "tracker.db", "sqlite3.db")
    /// Path is relative to the tracker's data directory
    database_name: String,
}

impl SqliteConfig {
    /// Creates a new `SQLite` configuration with validation
    ///
    /// # Errors
    ///
    /// - `EmptyDatabaseName` if database name is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::SqliteConfig;
    ///
    /// let config = SqliteConfig::new("tracker.db").unwrap();
    /// assert_eq!(config.database_name(), "tracker.db");
    /// ```
    pub fn new(database_name: impl Into<String>) -> Result<Self, SqliteConfigError> {
        let name = database_name.into();
        if name.is_empty() {
            return Err(SqliteConfigError::EmptyDatabaseName);
        }
        Ok(Self {
            database_name: name,
        })
    }

    /// Returns the database file name
    #[must_use]
    pub fn database_name(&self) -> &str {
        &self.database_name
    }
}

/// Intermediate struct for deserialization
#[derive(Deserialize)]
struct SqliteConfigRaw {
    database_name: String,
}

impl<'de> Deserialize<'de> for SqliteConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = SqliteConfigRaw::deserialize(deserializer)?;
        Self::new(raw.database_name).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_sqlite_config_when_database_name_is_valid() {
        let config = SqliteConfig::new("test.db").unwrap();
        assert_eq!(config.database_name(), "test.db");
    }

    #[test]
    fn it_should_reject_empty_database_name_when_creating_sqlite_config() {
        let result = SqliteConfig::new("");
        assert!(matches!(result, Err(SqliteConfigError::EmptyDatabaseName)));
    }

    #[test]
    fn it_should_serialize_sqlite_config() {
        let config = SqliteConfig::new("tracker.db").unwrap();
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["database_name"], "tracker.db");
    }

    #[test]
    fn it_should_deserialize_sqlite_config_when_valid() {
        let json = r#"{"database_name": "tracker.db"}"#;
        let config: SqliteConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.database_name(), "tracker.db");
    }

    #[test]
    fn it_should_reject_deserialization_when_database_name_is_empty() {
        let json = r#"{"database_name": ""}"#;
        let result: Result<SqliteConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
