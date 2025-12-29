//! Core tracker configuration

use serde::{Deserialize, Serialize};

use crate::domain::tracker::DatabaseConfig;

/// Core tracker configuration options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerCoreConfig {
    /// Database configuration (`SQLite`, `MySQL`, etc.)
    pub database: DatabaseConfig,

    /// Tracker mode: true for private tracker, false for public
    pub private: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tracker::{DatabaseConfig, SqliteConfig};

    #[test]
    fn it_should_create_core_config() {
        let core = TrackerCoreConfig {
            database: DatabaseConfig::Sqlite(SqliteConfig {
                database_name: "tracker.db".to_string(),
            }),
            private: true,
        };

        assert_eq!(core.database.database_name(), "tracker.db");
        assert!(core.private);
    }

    #[test]
    fn it_should_serialize_core_config() {
        let core = TrackerCoreConfig {
            database: DatabaseConfig::Sqlite(SqliteConfig {
                database_name: "test.db".to_string(),
            }),
            private: false,
        };

        let json = serde_json::to_value(&core).unwrap();
        assert_eq!(json["private"], false);
    }
}
