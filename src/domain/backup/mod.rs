//! Backup domain types for the Torrust Tracker Deployer.
//!
//! This module contains domain types related to backup configuration:
//! - `CronSchedule`: Validated cron schedule expression
//! - `RetentionDays`: Number of days to retain backups
//! - `BackupConfig`: Complete backup configuration

mod cron_schedule;
mod retention_days;

pub use cron_schedule::CronSchedule;
pub use retention_days::RetentionDays;

use serde::{Deserialize, Serialize};

/// Backup configuration for a deployed tracker instance.
///
/// Specifies when backups run (cron schedule) and how long to keep them (retention).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Cron schedule for when backups should run (e.g., "0 3 * * *" for 3:00 AM daily).
    schedule: CronSchedule,

    /// Number of days to retain backups before deletion.
    retention_days: RetentionDays,
}

impl BackupConfig {
    /// Creates a new backup configuration.
    ///
    /// # Arguments
    ///
    /// * `schedule` - Validated cron schedule
    /// * `retention_days` - Number of days to keep backups
    #[must_use]
    pub const fn new(schedule: CronSchedule, retention_days: RetentionDays) -> Self {
        Self {
            schedule,
            retention_days,
        }
    }

    /// Returns the cron schedule.
    #[must_use]
    pub const fn schedule(&self) -> &CronSchedule {
        &self.schedule
    }

    /// Returns the retention period in days.
    #[must_use]
    pub const fn retention_days(&self) -> &RetentionDays {
        &self.retention_days
    }
}

impl Default for BackupConfig {
    /// Default backup configuration:
    /// - Schedule: 3:00 AM daily ("0 3 * * *")
    /// - Retention: 7 days
    fn default() -> Self {
        Self {
            schedule: CronSchedule::default(),
            retention_days: RetentionDays::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_backup_config_with_valid_values() {
        let schedule = CronSchedule::new("0 3 * * *".to_string()).expect("valid cron schedule");
        let retention = RetentionDays::new(7).expect("valid retention days");

        let config = BackupConfig::new(schedule.clone(), retention);

        assert_eq!(config.schedule(), &schedule);
        assert_eq!(config.retention_days(), &retention);
    }

    #[test]
    fn it_should_provide_sensible_defaults() {
        let config = BackupConfig::default();

        assert_eq!(
            config.schedule().as_str(),
            "0 3 * * *",
            "default schedule should be 3:00 AM daily"
        );
        assert_eq!(
            config.retention_days().as_u32(),
            7,
            "default retention should be 7 days"
        );
    }

    #[test]
    fn it_should_serialize_and_deserialize_correctly() {
        let config = BackupConfig::default();

        let json = serde_json::to_string(&config).expect("serialization should succeed");
        let deserialized: BackupConfig =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(config, deserialized);
    }
}
