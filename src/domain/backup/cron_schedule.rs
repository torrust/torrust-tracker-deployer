//! Validated cron schedule expression.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Validated cron schedule expression (5-field format).
///
/// Validates that the cron expression follows the standard 5-field format:
/// `minute hour day month weekday`
///
/// Examples:
/// - `"0 3 * * *"` - 3:00 AM daily
/// - `"0 */6 * * *"` - Every 6 hours
/// - `"0 0 * * 0"` - Midnight every Sunday
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CronSchedule(String);

/// Errors that can occur when creating a `CronSchedule`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CronScheduleError {
    /// Cron schedule is empty
    #[error("Cron schedule cannot be empty")]
    Empty,

    /// Cron schedule has wrong number of fields
    #[error("Cron schedule must have 5 fields (minute hour day month weekday), got {0} fields")]
    InvalidFieldCount(usize),

    /// Cron schedule contains invalid characters
    #[error("Cron schedule contains invalid characters: {0}")]
    InvalidCharacters(String),
}

impl CronSchedule {
    /// Creates a new validated cron schedule.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The schedule is empty
    /// - The schedule doesn't have exactly 5 fields
    /// - The schedule contains invalid characters
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_tracker_deployer_lib::domain::backup::CronSchedule;
    ///
    /// let schedule = CronSchedule::new("0 3 * * *".to_string())?;
    /// assert_eq!(schedule.as_str(), "0 3 * * *");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(schedule: String) -> Result<Self, CronScheduleError> {
        if schedule.trim().is_empty() {
            return Err(CronScheduleError::Empty);
        }

        // Validate characters first (before splitting, to catch injection attempts)
        let valid_chars = |c: char| c.is_ascii_digit() || matches!(c, '*' | '-' | '/' | ',' | ' ');
        if let Some(invalid) = schedule.chars().find(|c| !valid_chars(*c)) {
            return Err(CronScheduleError::InvalidCharacters(format!(
                "found '{invalid}'"
            )));
        }

        // Validate field count (5 fields: minute hour day month weekday)
        let fields: Vec<&str> = schedule.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(CronScheduleError::InvalidFieldCount(fields.len()));
        }

        Ok(Self(schedule))
    }

    /// Returns the cron schedule as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CronSchedule {
    /// Default cron schedule: 3:00 AM daily ("0 3 * * *")
    fn default() -> Self {
        Self("0 3 * * *".to_string())
    }
}

impl<'de> Deserialize<'de> for CronSchedule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let schedule = String::deserialize(deserializer)?;
        Self::new(schedule).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("0 3 * * *", "3:00 AM daily")]
    #[case("0 */6 * * *", "Every 6 hours")]
    #[case("0 0 * * 0", "Midnight every Sunday")]
    #[case("30 2 1 * *", "2:30 AM on the 1st of every month")]
    #[case("0 0 1,15 * *", "Midnight on 1st and 15th")]
    #[case("*/15 * * * *", "Every 15 minutes")]
    #[case("0 9-17 * * 1-5", "9 AM to 5 PM, Monday to Friday")]
    fn it_should_accept_valid_cron_schedules(#[case] schedule: &str, #[case] description: &str) {
        let result = CronSchedule::new(schedule.to_string());
        assert!(
            result.is_ok(),
            "Schedule '{schedule}' ({description}) should be valid, got error: {result:?}"
        );
    }

    #[rstest]
    #[case("")]
    #[case("   ")]
    fn it_should_reject_empty_schedule(#[case] schedule: &str) {
        let result = CronSchedule::new(schedule.to_string());
        assert_eq!(result, Err(CronScheduleError::Empty));
    }

    #[rstest]
    #[case("0 3 *", 3)]
    #[case("0 3", 2)]
    #[case("0 3 * * * *", 6)]
    #[case("0 3 * * * * 2026", 7)]
    fn it_should_reject_wrong_field_count(#[case] schedule: &str, #[case] expected_count: usize) {
        let result = CronSchedule::new(schedule.to_string());
        assert_eq!(
            result,
            Err(CronScheduleError::InvalidFieldCount(expected_count)),
            "Schedule '{schedule}' should be rejected"
        );
    }

    #[rstest]
    #[case("0 3 * * * #comment", "Contains #")]
    #[case("0 3 * * MON", "Contains letters")]
    #[case("0 3 * * ?", "Contains ?")]
    #[case("0 3 * * *; rm -rf /", "Command injection attempt")]
    fn it_should_reject_invalid_characters(#[case] schedule: &str, #[case] reason: &str) {
        let result = CronSchedule::new(schedule.to_string());
        assert!(
            matches!(result, Err(CronScheduleError::InvalidCharacters(_))),
            "Schedule '{schedule}' ({reason}) should be rejected as invalid characters, got: {result:?}"
        );
    }

    #[test]
    fn it_should_return_schedule_as_string() {
        let schedule = CronSchedule::new("0 3 * * *".to_string()).expect("valid schedule");
        assert_eq!(schedule.as_str(), "0 3 * * *");
    }

    #[test]
    fn it_should_use_sensible_default() {
        let schedule = CronSchedule::default();
        assert_eq!(schedule.as_str(), "0 3 * * *");
    }

    #[test]
    fn it_should_deserialize_valid_cron_schedule() {
        let json = r#""0 3 * * *""#;
        let schedule: CronSchedule = serde_json::from_str(json).expect("valid schedule");
        assert_eq!(schedule.as_str(), "0 3 * * *");
    }

    #[rstest]
    #[case(r#""""#, "Empty")]
    #[case(r#""0 3""#, "Too few fields")]
    #[case(r#""0 3 * * * *""#, "Too many fields")]
    #[case(r#""0 3 * * MON""#, "Invalid characters")]
    fn it_should_reject_invalid_schedule_during_deserialization(
        #[case] json: &str,
        #[case] reason: &str,
    ) {
        let result: Result<CronSchedule, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "JSON '{json}' ({reason}) should fail deserialization"
        );
    }

    #[test]
    fn it_should_serialize_and_deserialize_correctly() {
        let original = CronSchedule::new("0 3 * * *".to_string()).expect("valid schedule");
        let json = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: CronSchedule =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(original, deserialized);
    }
}
