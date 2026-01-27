//! Template generation metadata for rendered configuration files.
//!
//! This module provides the `TemplateMetadata` struct that captures information about
//! when templates were generated. This metadata is embedded in rendered templates
//! to provide context for AI agents, developers, and system administrators.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};

/// Serializes `DateTime<Utc>` as ISO 8601 string for Tera templates
fn serialize_datetime_as_iso8601<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

/// Metadata about template generation.
///
/// This struct is designed to be flattened into template contexts using `#[serde(flatten)]`,
/// making the timestamp available at the top level in templates.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::metadata::TemplateMetadata;
/// use chrono::{TimeZone, Utc};
///
/// let timestamp = Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap();
/// let metadata = TemplateMetadata::new(timestamp);
/// assert_eq!(metadata.generated_at(), &timestamp);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateMetadata {
    /// Timestamp when the template was generated (UTC).
    #[serde(serialize_with = "serialize_datetime_as_iso8601")]
    generated_at: DateTime<Utc>,
}

impl TemplateMetadata {
    /// Creates a new `TemplateMetadata` with the given timestamp.
    ///
    /// # Arguments
    ///
    /// * `generated_at` - UTC timestamp when the template was generated
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::metadata::TemplateMetadata;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let timestamp = Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap();
    /// let metadata = TemplateMetadata::new(timestamp);
    /// ```
    #[must_use]
    pub fn new(generated_at: DateTime<Utc>) -> Self {
        Self { generated_at }
    }

    /// Returns the generation timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::metadata::TemplateMetadata;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let timestamp = Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap();
    /// let metadata = TemplateMetadata::new(timestamp);
    /// assert_eq!(metadata.generated_at(), &timestamp);
    /// ```
    #[must_use]
    pub fn generated_at(&self) -> &DateTime<Utc> {
        &self.generated_at
    }

    /// Returns the timestamp formatted as ISO 8601 string.
    ///
    /// Format: `YYYY-MM-DDTHH:MM:SSZ` (e.g., `2026-01-27T14:30:00Z`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::metadata::TemplateMetadata;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let timestamp = Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap();
    /// let metadata = TemplateMetadata::new(timestamp);
    /// assert_eq!(metadata.generated_at_iso8601(), "2026-01-27T14:30:00Z");
    /// ```
    #[must_use]
    pub fn generated_at_iso8601(&self) -> String {
        self.generated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn it_should_create_template_metadata_with_timestamp() {
        let timestamp = Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);

        assert_eq!(metadata.generated_at(), &timestamp);
    }

    #[test]
    fn it_should_format_timestamp_as_iso8601() {
        let timestamp = Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);

        assert_eq!(metadata.generated_at_iso8601(), "2026-01-27T14:30:00Z");
    }

    #[test]
    fn it_should_serialize_metadata_correctly() {
        let timestamp = Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let json = serde_json::to_string(&metadata).unwrap();

        assert!(json.contains("\"generated_at\""));
        assert!(json.contains("2026-01-27T14:30:00Z"));
    }

    #[test]
    fn it_should_deserialize_metadata_correctly() {
        let json = r#"{"generated_at":"2026-01-27T14:30:00Z"}"#;
        let metadata: TemplateMetadata = serde_json::from_str(json).unwrap();

        assert_eq!(
            metadata.generated_at(),
            &Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap()
        );
    }

    #[test]
    fn it_should_implement_clone() {
        let timestamp = Utc.with_ymd_and_hms(2026, 1, 27, 14, 30, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let cloned = metadata.clone();

        assert_eq!(metadata, cloned);
    }
}
