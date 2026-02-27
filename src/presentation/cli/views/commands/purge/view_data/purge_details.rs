//! Purge Details Data Transfer Object
//!
//! This module contains the presentation DTO for purge command details.
//! It serves as the data structure passed to view renderers (`TextView`, `JsonView`, etc.).
//!
//! # Architecture
//!
//! This follows the Strategy Pattern where:
//! - This DTO is the data passed to all rendering strategies
//! - Different views (`TextView`, `JsonView`) consume this data
//! - Adding new formats doesn't modify this DTO or existing views
//!
//! # SOLID Principles
//!
//! - **Single Responsibility**: This file only defines the data structure
//! - **Open/Closed**: New formats extend by adding views, not modifying this
//! - **Separation of Concerns**: Data definition separate from rendering logic

use serde::Serialize;

/// Purge details data for rendering
///
/// This struct holds all the data needed to render purge command
/// information for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples application types from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
///
/// # Named Constructor
///
/// `PurgeDetailsData` is built from just the environment name string since
/// the application-layer `PurgeCommandHandler::execute()` returns `()` on success —
/// there is no result struct. `purged: true` is always set because this DTO is only
/// constructed on the success path.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PurgeDetailsData {
    /// Name of the environment that was purged
    pub environment_name: String,
    /// Always `true` when the command exits successfully
    pub purged: bool,
}

impl PurgeDetailsData {
    /// Construct a `PurgeDetailsData` from an environment name
    ///
    /// This named constructor always sets `purged: true` because it is only
    /// called on the success path — purge failures result in an error return,
    /// never a `PurgeDetailsData`.
    ///
    /// # Arguments
    ///
    /// * `environment_name` - Name of the environment that was successfully purged
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::purge::PurgeDetailsData;
    ///
    /// let data = PurgeDetailsData::from_environment_name("my-env");
    ///
    /// assert_eq!(data.environment_name, "my-env");
    /// assert!(data.purged);
    /// ```
    #[must_use]
    pub fn from_environment_name(environment_name: &str) -> Self {
        Self {
            environment_name: environment_name.to_string(),
            purged: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_build_dto_from_environment_name() {
        // Act
        let data = PurgeDetailsData::from_environment_name("test-env");

        // Assert
        assert_eq!(data.environment_name, "test-env");
        assert!(data.purged);
    }

    #[test]
    fn it_should_always_set_purged_to_true() {
        // Arrange — no purge failure can produce a PurgeDetailsData
        // (only success paths call from_environment_name)
        let data = PurgeDetailsData::from_environment_name("test-env");

        // Assert
        assert!(data.purged, "purged should always be true on success path");
    }
}
