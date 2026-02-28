//! Views for Exists Command
//!
//! This module contains view components for rendering exists command output.
//!
//! # Architecture
//!
//! This module follows the Strategy Pattern for rendering:
//! - `ExistsResult`: The data DTO passed to all views
//! - `TextView`: Renders bare `true`/`false` text output
//! - `JsonView`: Renders bare `true`/`false` (valid JSON values)
//!
//! Both formats produce identical output since bare boolean is the
//! most natural representation in both human-readable text and JSON.
//!
//! # Structure
//!
//! - `view_data/`: Data structures (DTOs) passed to views
//!   - `exists_details.rs`: Re-exports the application DTO
//! - `views/`: View rendering implementations
//!   - `text_view.rs`: Human-readable text rendering
//!   - `json_view.rs`: Machine-readable JSON rendering

pub mod view_data;
pub mod views {
    pub mod json_view;
    pub mod text_view;

    // Re-export views for convenience
    pub use json_view::JsonView;
    pub use text_view::TextView;
}

// Re-export at module root for convenience
pub use view_data::ExistsResult;
pub use views::{JsonView, TextView};
