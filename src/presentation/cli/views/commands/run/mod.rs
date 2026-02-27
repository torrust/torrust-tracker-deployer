//! Views for the Run Command
//!
//! This module provides different rendering strategies for run command output.
//! Following the Strategy Pattern, each view (`TextView`, `JsonView`) implements
//! a different output format for the same underlying data (`ServiceInfo` and `GrafanaInfo` DTOs).

pub mod view_data;
mod views;

pub use view_data::RunDetailsData;
pub use views::JsonView;
pub use views::TextView;
