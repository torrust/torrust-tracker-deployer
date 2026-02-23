//! Views for Configure Command
//!
//! This module contains view components for rendering configure command output.
//!
//! # Architecture
//!
//! This module follows the Strategy Pattern for rendering:
//! - `ConfigureDetailsData`: The data DTO passed to all views
//! - `TextView`: Renders human-readable text output
//! - `JsonView`: Renders machine-readable JSON output
//!
//! # Structure
//!
//! - `view_data/`: Data structures (DTOs) passed to views
//!   - `configure_details.rs`: Main DTO with environment configure data
//! - `views/`: View rendering implementations
//!   - `text_view.rs`: Human-readable text rendering
//!   - `json_view.rs`: Machine-readable JSON rendering
//!
//! # SOLID Principles
//!
//! - **Single Responsibility**: Each view has one job - render in its format
//! - **Open/Closed**: Add new formats by creating new view files, not modifying existing ones
//! - **Strategy Pattern**: Different rendering strategies for the same data
//!
//! # Adding New Formats
//!
//! To add a new output format (e.g., XML, YAML, CSV):
//! 1. Create a new file in `views/`: `xml_view.rs`, `yaml_view.rs`, etc.
//! 2. Implement the view with `render(data: &ConfigureDetailsData) -> String`
//! 3. Export it from this module
//! 4. No need to modify existing views or the DTO

pub mod view_data {
    pub mod configure_details;

    // Re-export main types for convenience
    pub use configure_details::ConfigureDetailsData;
}

pub mod views {
    pub mod json_view;
    pub mod text_view;

    // Re-export views for convenience
    pub use json_view::JsonView;
    pub use text_view::TextView;
}

// Re-export at module root for convenience
pub use view_data::ConfigureDetailsData;
pub use views::{JsonView, TextView};
