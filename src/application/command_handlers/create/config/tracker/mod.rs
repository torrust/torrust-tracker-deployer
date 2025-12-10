//! Tracker Configuration DTOs (Application Layer)
//!
//! This module contains DTO types for tracker configuration used in
//! environment creation. These types use raw primitives (String) for
//! JSON deserialization and convert to rich domain types (`SocketAddr`).

mod http_tracker_section;
mod udp_tracker_section;

pub use http_tracker_section::HttpTrackerSection;
pub use udp_tracker_section::UdpTrackerSection;
