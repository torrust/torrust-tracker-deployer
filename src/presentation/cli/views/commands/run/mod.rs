//! Views for the Run Command
//!
//! This module provides different rendering strategies for run command output.
//! Following the Strategy Pattern, each view (`TextView`, `JsonView`) implements
//! a different output format for the same underlying data (`ServiceInfo` and `GrafanaInfo` DTOs).

mod json_view;
mod text_view;

pub use json_view::JsonView;
pub use text_view::TextView;
