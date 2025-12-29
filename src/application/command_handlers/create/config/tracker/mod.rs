//! Tracker Configuration DTOs (Application Layer)
//!
//! This module contains DTO types for tracker configuration used in
//! environment creation. These types use raw primitives (String) for
//! JSON deserialization and convert to rich domain types (`SocketAddr`).

mod health_check_api_section;
mod http_api_section;
mod http_tracker_section;
mod tracker_core_section;
mod tracker_section;
mod udp_tracker_section;

pub use health_check_api_section::HealthCheckApiSection;
pub use http_api_section::HttpApiSection;
pub use http_tracker_section::HttpTrackerSection;
pub use tracker_core_section::{DatabaseSection, TrackerCoreSection};
pub use tracker_section::TrackerSection;
pub use udp_tracker_section::UdpTrackerSection;
