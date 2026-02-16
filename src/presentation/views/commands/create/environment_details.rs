//! Environment Details Data Transfer Object
//!
//! This module contains the presentation DTO for environment creation details.
//! It serves as the data structure passed to view renderers (TextView, JsonView, etc.).
//!
//! # Architecture
//!
//! This follows the Strategy Pattern where:
//! - This DTO is the data passed to all rendering strategies
//! - Different views (TextView, JsonView) consume this data
//! - Adding new formats doesn't modify this DTO or existing views
//!
//! # SOLID Principles
//!
//! - **Single Responsibility**: This file only defines the data structure
//! - **Open/Closed**: New formats extend by adding views, not modifying this
//! - **Separation of Concerns**: Data definition separate from rendering logic

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;

use crate::domain::environment::state::Created;
use crate::domain::environment::Environment;

/// Environment details data for rendering
///
/// This struct holds all the data needed to render environment creation
/// information for display to the user. It is consumed by view renderers
/// (TextView, JsonView) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples domain models from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentDetailsData {
    /// Name of the created environment
    pub environment_name: String,
    /// Name of the instance that will be created
    pub instance_name: String,
    /// Path to the data directory
    pub data_dir: PathBuf,
    /// Path to the build directory
    pub build_dir: PathBuf,
    /// Timestamp when the environment was created (ISO 8601 format in JSON)
    pub created_at: DateTime<Utc>,
}

/// Conversion from domain model to presentation DTO
///
/// This `From` trait implementation is placed in the presentation layer
/// (not in the domain layer) to maintain proper DDD layering:
///
/// - Domain layer should not depend on presentation layer DTOs
/// - Presentation layer can depend on domain models (allowed)
/// - This keeps the domain clean and focused on business logic
///
/// Alternative approaches considered:
/// - Adding method to `Environment<Created>`: Would violate DDD by making
///   domain depend on presentation DTOs
/// - Keeping mapping in controller: Works but less idiomatic than `From` trait
impl From<&Environment<Created>> for EnvironmentDetailsData {
    fn from(environment: &Environment<Created>) -> Self {
        Self {
            environment_name: environment.name().as_str().to_string(),
            instance_name: environment.instance_name().as_str().to_string(),
            data_dir: environment.data_dir().clone(),
            build_dir: environment.build_dir().clone(),
            created_at: environment.created_at(),
        }
    }
}
