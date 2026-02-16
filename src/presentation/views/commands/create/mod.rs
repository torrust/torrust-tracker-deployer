//! Views for Create Command
//!
//! This module contains view components for the create command,
//! responsible for formatting and rendering output to users.
//!
//! # Architecture
//!
//! This module follows the Strategy Pattern for rendering:
//! - `EnvironmentDetailsData`: The data DTO passed to all views
//! - `TextView`: Renders human-readable text output
//! - `JsonView`: Renders machine-readable JSON output
//!
//! # Structure
//!
//! - `view_data/`: Data structures (DTOs) prepared for view rendering
//! - `views/`: View implementations (text, JSON, etc.)
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
//! 2. Implement the view with `render(data: &EnvironmentDetailsData) -> Result<String, Error>`
//! 3. Export it from the `views` module
//! 4. No need to modify existing views or the DTO

pub mod view_data {
    pub mod environment_details;
    pub use environment_details::EnvironmentDetailsData;
}

pub mod views {
    pub mod json_view;
    pub mod text_view;
    pub use json_view::JsonView;
    pub use text_view::TextView;
}

// Re-export main types for convenience
pub use view_data::EnvironmentDetailsData;
pub use views::{JsonView, TextView};
