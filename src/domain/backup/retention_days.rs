//! Backup retention period in days.

use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Number of days to retain backups before deletion.
///
/// Must be at least 1 day. Values of 0 are rejected to prevent
/// accidental deletion of all backups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RetentionDays(NonZeroU32);

/// Errors that can occur when creating `RetentionDays`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RetentionDaysError {
    /// Retention days must be at least 1
    #[error("Retention days must be at least 1 (got 0)")]
    Zero,
}

impl RetentionDays {
    /// Creates a new retention period.
    ///
    /// # Errors
    ///
    /// Returns an error if `days` is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_tracker_deployer::domain::backup::RetentionDays;
    ///
    /// let retention = RetentionDays::new(7)?;
    /// assert_eq!(retention.as_u32(), 7);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(days: u32) -> Result<Self, RetentionDaysError> {
        NonZeroU32::new(days)
            .map(Self)
            .ok_or(RetentionDaysError::Zero)
    }

    /// Returns the retention period as a u32.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0.get()
    }
}

impl Default for RetentionDays {
    /// Default retention: 7 days
    fn default() -> Self {
        Self(NonZeroU32::new(7).expect("7 is non-zero"))
    }
}

impl<'de> Deserialize<'de> for RetentionDays {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let days = u32::deserialize(deserializer)?;
        Self::new(days).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(1)]
    #[case(7)]
    #[case(14)]
    #[case(30)]
    #[case(90)]
    #[case(365)]
    fn it_should_accept_valid_retention_days(#[case] days: u32) {
        let result = RetentionDays::new(days);
        assert!(result.is_ok(), "Retention days {days} should be valid");
        assert_eq!(result.unwrap().as_u32(), days);
    }

    #[test]
    fn it_should_reject_zero_days() {
        let result = RetentionDays::new(0);
        assert_eq!(result, Err(RetentionDaysError::Zero));
    }

    #[test]
    fn it_should_use_sensible_default() {
        let retention = RetentionDays::default();
        assert_eq!(retention.as_u32(), 7);
    }

    #[test]
    fn it_should_deserialize_valid_retention_days() {
        let json = "7";
        let retention: RetentionDays = serde_json::from_str(json).expect("valid retention");
        assert_eq!(retention.as_u32(), 7);
    }

    #[test]
    fn it_should_reject_zero_during_deserialization() {
        let json = "0";
        let result: Result<RetentionDays, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Zero should fail deserialization");
    }

    #[test]
    fn it_should_serialize_and_deserialize_correctly() {
        let original = RetentionDays::new(7).expect("valid retention");
        let json = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: RetentionDays =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(original, deserialized);
    }
}
