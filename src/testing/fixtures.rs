//! Test fixtures for persistence and serialization testing
//!
//! This module provides reusable test entities and builders for testing
//! JSON serialization, deserialization, and file persistence operations.

use serde::{Deserialize, Serialize};

/// Test entity for JSON serialization tests
///
/// This is a simple entity used across multiple tests to verify
/// serialization, deserialization, and persistence operations.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deploy::testing::fixtures::TestEntity;
///
/// // Create with default values
/// let entity = TestEntity::default();
/// assert_eq!(entity.id, "test-id");
/// assert_eq!(entity.value, 42);
///
/// // Create with custom values
/// let entity = TestEntity::new("custom-id", 100);
/// assert_eq!(entity.id, "custom-id");
/// assert_eq!(entity.value, 100);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestEntity {
    pub id: String,
    pub value: i32,
}

impl Default for TestEntity {
    /// Create a test entity with default values
    ///
    /// Default values:
    /// - id: "test-id"
    /// - value: 42
    fn default() -> Self {
        Self {
            id: "test-id".to_string(),
            value: 42,
        }
    }
}

impl TestEntity {
    /// Create a test entity with custom values
    ///
    /// # Arguments
    ///
    /// * `id` - The entity ID (can be any type that converts to String)
    /// * `value` - The entity value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::testing::fixtures::TestEntity;
    ///
    /// let entity = TestEntity::new("my-id", 100);
    /// assert_eq!(entity.id, "my-id");
    /// assert_eq!(entity.value, 100);
    /// ```
    #[must_use]
    pub fn new(id: impl Into<String>, value: i32) -> Self {
        Self {
            id: id.into(),
            value,
        }
    }
}
