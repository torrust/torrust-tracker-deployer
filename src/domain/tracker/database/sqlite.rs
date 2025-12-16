//! `SQLite` database configuration

use serde::{Deserialize, Serialize};

/// `SQLite` database configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SqliteConfig {
    /// Database file name (e.g., "tracker.db", "sqlite3.db")
    /// Path is relative to the tracker's data directory
    pub database_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_sqlite_config() {
        let config = SqliteConfig {
            database_name: "test.db".to_string(),
        };

        assert_eq!(config.database_name, "test.db");
    }

    #[test]
    fn it_should_serialize_sqlite_config() {
        let config = SqliteConfig {
            database_name: "tracker.db".to_string(),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["database_name"], "tracker.db");
    }

    #[test]
    fn it_should_deserialize_sqlite_config() {
        let json = r#"{"database_name": "tracker.db"}"#;
        let config: SqliteConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.database_name, "tracker.db");
    }
}
