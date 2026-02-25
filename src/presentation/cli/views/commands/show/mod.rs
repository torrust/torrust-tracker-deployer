//! Views for Show Command
//!
//! This module contains view components for rendering show command output.
//!
//! # Architecture
//!
//! This module follows the Strategy Pattern for rendering:
//! - `TextView`: Renders human-readable text output with environment details
//! - `JsonView`: Renders machine-readable JSON output for automation
//!
//! # Structure
//!
//! - `views/`: View rendering implementations
//!   - `text_view.rs`: Main `TextView` with composition of helper views
//!   - `json_view.rs`: Main `JsonView` for JSON serialization
//!   - Helper views: basic, infrastructure, `tracker_services`, prometheus, grafana, `https_hint`, `next_step`

pub mod views;

// Re-export main types for convenience
pub use views::{JsonView, TextView};
