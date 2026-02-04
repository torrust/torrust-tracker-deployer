//! Backup template context
//!
//! Defines the variables needed for backup.conf.tera template rendering.

use serde::Serialize;

use crate::domain::backup::BackupConfig;
use crate::infrastructure::templating::TemplateMetadata;

/// Database configuration for backup template
///
/// Represents the database type and connection details needed by the backup script.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackupDatabaseConfig {
    /// `MySQL` database configuration
    Mysql {
        host: String,
        port: u16,
        database: String,
        user: String,
        password: String,
    },
    /// `SQLite` database configuration
    Sqlite {
        /// Path to `SQLite` database file inside the container
        path: String,
    },
}

/// Context for rendering backup.conf.tera template
///
/// Contains all variables needed by the backup configuration template including
/// retention settings and database connection details.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::backup::BackupContext;
/// use torrust_tracker_deployer_lib::infrastructure::templating::backup::BackupDatabaseConfig;
/// use torrust_tracker_deployer_lib::infrastructure::templating::TemplateMetadata;
/// use torrust_tracker_deployer_lib::shared::clock::{Clock, SystemClock};
///
/// let clock = SystemClock;
/// let metadata = TemplateMetadata::new(clock.now());
/// let db_config = BackupDatabaseConfig::Sqlite {
///     path: "/data/storage/tracker/lib/tracker.db".to_string(),
/// };
/// let context = BackupContext::new(metadata, 7, db_config);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct BackupContext {
    /// Template generation metadata (timestamp, etc.)
    ///
    /// Flattened for template compatibility - serializes metadata at top level.
    #[serde(flatten)]
    pub metadata: TemplateMetadata,

    /// Number of days to retain backups before deletion
    pub retention_days: u32,

    /// Database configuration (`MySQL` or `SQLite`)
    #[serde(flatten)]
    pub database: BackupDatabaseConfig,
}

impl BackupContext {
    /// Creates a new backup context
    ///
    /// # Arguments
    ///
    /// * `metadata` - Template generation metadata
    /// * `retention_days` - Number of days to keep backups
    /// * `database` - Database configuration (`MySQL` or `SQLite`)
    #[must_use]
    pub const fn new(
        metadata: TemplateMetadata,
        retention_days: u32,
        database: BackupDatabaseConfig,
    ) -> Self {
        Self {
            metadata,
            retention_days,
            database,
        }
    }

    /// Creates backup context from domain configuration
    ///
    /// # Arguments
    ///
    /// * `metadata` - Template generation metadata
    /// * `backup_config` - Domain backup configuration
    /// * `database` - Database configuration derived from environment
    #[must_use]
    pub fn from_config(
        metadata: TemplateMetadata,
        backup_config: &BackupConfig,
        database: BackupDatabaseConfig,
    ) -> Self {
        Self {
            metadata,
            retention_days: backup_config.retention_days().as_u32(),
            database,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Utc;

    #[test]
    fn it_should_create_backup_context_with_mysql() {
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Mysql {
            host: "mysql".to_string(),
            port: 3306,
            database: "torrust_tracker".to_string(),
            user: "tracker_user".to_string(),
            password: "secret".to_string(),
        };

        let context = BackupContext::new(metadata.clone(), 7, db_config);

        assert_eq!(context.retention_days, 7);
        assert_eq!(context.metadata, metadata);
    }

    #[test]
    fn it_should_create_backup_context_with_sqlite() {
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Sqlite {
            path: "/data/storage/tracker/lib/tracker.db".to_string(),
        };

        let context = BackupContext::new(metadata.clone(), 14, db_config);

        assert_eq!(context.retention_days, 14);
        assert_eq!(context.metadata, metadata);
    }
}
