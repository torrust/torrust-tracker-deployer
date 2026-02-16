//! Views for the Show Command
//!
//! This module provides different rendering strategies for environment information.
//! Following the Strategy Pattern, each view (TextView, JsonView) implements
//! a different output format for the same underlying data (EnvironmentInfo DTO).

mod json_view;
mod text_view;

// Helper modules for TextView (text-based rendering components)
mod basic;
mod grafana;
mod https_hint;
mod infrastructure;
mod next_step;
mod prometheus;
mod tracker_services;

pub use json_view::JsonView;
pub use text_view::TextView;
