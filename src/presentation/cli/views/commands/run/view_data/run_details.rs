//! Run Details Data Transfer Object
//!
//! This module contains the presentation DTO for the run command output.
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

use crate::application::command_handlers::show::info::{GrafanaInfo, ServiceInfo};

/// Run details data for rendering
///
/// This struct holds all the data needed to render run command output
/// for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// The `state` field is always `"Running"` — this command only executes
/// when services are being started successfully.
#[derive(Debug, Serialize)]
pub struct RunDetailsData {
    /// Name of the environment
    pub environment_name: String,

    /// Current state — always `"Running"` for this command
    pub state: String,

    /// Tracker and API service endpoint information
    pub services: ServiceInfo,

    /// Optional Grafana dashboard information
    pub grafana: Option<GrafanaInfo>,
}

impl RunDetailsData {
    /// Create a new `RunDetailsData`
    ///
    /// The `state` field is always set to `"Running"`.
    ///
    /// # Arguments
    ///
    /// * `environment_name` - Name of the environment
    /// * `services` - Tracker and API service endpoint information
    /// * `grafana` - Optional Grafana dashboard information
    #[must_use]
    pub fn new(
        environment_name: String,
        services: ServiceInfo,
        grafana: Option<GrafanaInfo>,
    ) -> Self {
        Self {
            environment_name,
            state: "Running".to_string(),
            services,
            grafana,
        }
    }
}
