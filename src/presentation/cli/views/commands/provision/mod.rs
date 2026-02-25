//! Views for Provision Command
//!
//! This module contains view components for rendering provision command output.
//!
//! # Architecture
//!
//! This module follows the Strategy Pattern for rendering:
//! - `ProvisionDetailsData`: The data DTO passed to all views
//! - `TextView`: Renders human-readable text output
//! - `JsonView`: Renders machine-readable JSON output
//!
//! # Structure
//!
//! - `view_data/`: Data structures (DTOs) passed to views
//!   - `provision_details.rs`: Main DTO with environment provision data
//!   - `connection_details.rs`: Helper DTO for connection information
//!   - `dns_reminder.rs`: Helper DTO for DNS reminder information
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
//! 2. Implement the view with `render(data: &ProvisionDetailsData) -> Result<String, Error>`
//! 3. Export it from this module
//! 4. No need to modify existing views or the DTO

pub mod view_data {
    pub mod connection_details;
    pub mod dns_reminder;
    pub mod provision_details;

    // Re-export main types for convenience
    pub use connection_details::ConnectionDetailsView;
    pub use dns_reminder::DnsReminderView;
    pub use provision_details::ProvisionDetailsData;
}

pub mod views {
    pub mod json_view;
    pub mod text_view;

    // Re-export main types for convenience
    pub use json_view::JsonView;
    pub use text_view::TextView;
}

// Re-export everything at the module level for backward compatibility
pub use view_data::{ConnectionDetailsView, DnsReminderView, ProvisionDetailsData};
pub use views::{JsonView, TextView};
