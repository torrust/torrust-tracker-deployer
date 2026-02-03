//! Backup Configuration DTO (Application Layer)
//!
//! This module contains the DTO type for backup configuration used in
//! environment creation. This type uses raw primitives (String, u32) for JSON
//! deserialization and converts to the rich domain type (`BackupConfig`).
//!
//! # Conversion Pattern
//!
//! Uses `TryFrom` for idiomatic Rust conversion from DTO to domain type.
//! See ADR: `docs/decisions/tryfrom-for-dto-to-domain-conversion.md`

use std::convert::TryFrom;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::backup::{BackupConfig, CronSchedule, RetentionDays};

/// Backup configuration section (DTO)
///
/// Optional configuration for automated backups. If present, backup support
/// is enabled with the specified schedule and retention policy.
///
/// # Examples
///
/// ```json
/// {
///     "schedule": "0 3 * * *",
///     "retention_days": 7
/// }
/// ```
///
/// All fields have defaults, so you can enable backup with minimal config:
///
/// ```json
/// {
///     "backup": {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BackupSection {
    /// Cron schedule for backups (5-field format: minute hour day month weekday)
    ///
    /// Default: "0 3 * * *" (3:00 AM daily)
    ///
    /// Examples:
    /// - "0 3 * * *" - 3:00 AM daily
    /// - "0 */6 * * *" - Every 6 hours
    /// - "0 0 * * 0" - Midnight every Sunday
    #[serde(default = "default_schedule")]
    pub schedule: String,

    /// Number of days to retain backups before automatic deletion
    ///
    /// Default: 7 days
    ///
    /// Must be greater than 0.
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

fn default_schedule() -> String {
    CronSchedule::default().as_str().to_string()
}

fn default_retention_days() -> u32 {
    RetentionDays::default().as_u32()
}

impl Default for BackupSection {
    fn default() -> Self {
        Self {
            schedule: default_schedule(),
            retention_days: default_retention_days(),
        }
    }
}

impl TryFrom<BackupSection> for BackupConfig {
    type Error = CreateConfigError;

    fn try_from(section: BackupSection) -> Result<Self, Self::Error> {
        let schedule = CronSchedule::new(section.schedule.clone()).map_err(|e| {
            CreateConfigError::InvalidBackupConfig(format!("Invalid cron schedule: {e}"))
        })?;

        let retention = RetentionDays::new(section.retention_days).map_err(|e| {
            CreateConfigError::InvalidBackupConfig(format!("Invalid retention days: {e}"))
        })?;

        Ok(BackupConfig::new(schedule, retention))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn it_should_have_default_values() {
        let section = BackupSection::default();
        assert_eq!(section.schedule, "0 3 * * *");
        assert_eq!(section.retention_days, 7);
    }

    #[test]
    fn it_should_deserialize_from_json_with_all_fields() {
        let json = r#"{
            "schedule": "0 */6 * * *",
            "retention_days": 30
        }"#;

        let section: BackupSection = serde_json::from_str(json).expect("Valid JSON");
        assert_eq!(section.schedule, "0 */6 * * *");
        assert_eq!(section.retention_days, 30);
    }

    #[test]
    fn it_should_deserialize_from_empty_json_with_defaults() {
        let json = "{}";

        let section: BackupSection = serde_json::from_str(json).expect("Valid JSON");
        assert_eq!(section.schedule, "0 3 * * *");
        assert_eq!(section.retention_days, 7);
    }

    #[test]
    fn it_should_convert_valid_section_to_backup_config() {
        let section = BackupSection {
            schedule: "0 3 * * *".to_string(),
            retention_days: 14,
        };

        let config: BackupConfig = section.try_into().expect("Valid backup config");
        assert_eq!(config.schedule().as_str(), "0 3 * * *");
        assert_eq!(config.retention_days().as_u32(), 14);
    }

    #[rstest]
    #[case("", "Invalid cron schedule")]
    #[case("0 3", "Invalid cron schedule")]
    #[case("0 3 * * * *", "Invalid cron schedule")]
    #[case("0 3 * * MON", "Invalid cron schedule")]
    fn it_should_reject_invalid_cron_schedule(#[case] schedule: &str, #[case] reason: &str) {
        let section = BackupSection {
            schedule: schedule.to_string(),
            retention_days: 7,
        };

        let result: Result<BackupConfig, _> = section.try_into();
        assert!(
            result.is_err(),
            "Schedule '{schedule}' ({reason}) should be rejected"
        );
    }

    #[test]
    fn it_should_reject_zero_retention_days() {
        let section = BackupSection {
            schedule: "0 3 * * *".to_string(),
            retention_days: 0,
        };

        let result: Result<BackupConfig, _> = section.try_into();
        assert!(result.is_err(), "Zero retention days should be rejected");
    }

    #[test]
    fn it_should_serialize_and_deserialize_correctly() {
        let original = BackupSection {
            schedule: "0 */6 * * *".to_string(),
            retention_days: 30,
        };

        let json = serde_json::to_string(&original).expect("Serialization should succeed");
        let deserialized: BackupSection =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(original, deserialized);
    }
}
