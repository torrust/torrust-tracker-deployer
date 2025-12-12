//! Tracker Core configuration section (application DTO)
//!
//! This module provides the DTO for tracker core configuration,
//! used for JSON deserialization and validation before converting
//! to domain types.

use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::{DatabaseConfig, TrackerCoreConfig};

/// Database configuration section (application DTO)
///
/// Mirrors the domain `DatabaseConfig` enum but at the application layer.
/// Currently only `SQLite` is supported.
///
/// # Examples
///
/// ```json
/// {
///   "driver": "sqlite3",
///   "database_name": "tracker.db"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "driver")]
pub enum DatabaseSection {
    /// `SQLite` file-based database
    #[serde(rename = "sqlite3")]
    Sqlite {
        /// Database file name
        database_name: String,
    },
}

impl DatabaseSection {
    /// Converts this DTO to the domain `DatabaseConfig` type.
    ///
    /// # Errors
    ///
    /// This conversion currently cannot fail, but returns `Result`
    /// for consistency with other DTO conversions and to allow
    /// future validation.
    pub fn to_database_config(&self) -> Result<DatabaseConfig, CreateConfigError> {
        match self {
            Self::Sqlite { database_name } => Ok(DatabaseConfig::Sqlite {
                database_name: database_name.clone(),
            }),
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        Ok(TrackerCoreConfig {
            database: self.database.to_database_config()?,
            private: self.private,
        })
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
            config.database,
            DatabaseConfig::Sqlite {
                database_name: "tracker.db".to_string()
            }
        );
        assert!(!config.private);
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

        assert!(config.private);
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
}
