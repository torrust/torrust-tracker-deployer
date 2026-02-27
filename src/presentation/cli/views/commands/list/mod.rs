//! Views for List Command
//!
//! This module contains view components for rendering list command output.
//!
//! # Architecture
//!
//! This module follows the Strategy Pattern for rendering:
//! - `TextView`: Renders human-readable text table output
//! - `JsonView`: Renders machine-readable JSON output
//!
//! # Structure
//!
//! - `views/`: View rendering implementations
//!   - `text_view.rs`: Human-readable table rendering
//!   - `json_view.rs`: JSON output for automation workflows

pub mod view_data;
pub mod views {
    pub mod json_view;
    pub mod text_view;

    // Re-export main types for convenience
    pub use json_view::JsonView;
    pub use text_view::TextView;
}

// Re-export everything at the module level for backward compatibility
pub use view_data::EnvironmentList;
pub use views::{JsonView, TextView};
