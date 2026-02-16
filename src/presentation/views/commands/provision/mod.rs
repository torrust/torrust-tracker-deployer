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
//! # SOLID Principles
//!
//! - **Single Responsibility**: Each view has one job - render in its format
//! - **Open/Closed**: Add new formats by creating new view files, not modifying existing ones
//! - **Strategy Pattern**: Different rendering strategies for the same data
//!
//! # Adding New Formats
//!
//! To add a new output format (e.g., XML, YAML, CSV):
//! 1. Create a new file: `xml_view.rs`, `yaml_view.rs`, etc.
//! 2. Implement the view with `render(data: &ProvisionDetailsData) -> Result<String, Error>`
//! 3. Export it from this module
//! 4. No need to modify existing views or the DTO

pub mod connection_details;
pub mod dns_reminder;
pub mod json_view;
pub mod provision_details;
pub mod text_view;

// Re-export main types for convenience
pub use connection_details::ConnectionDetailsView;
pub use dns_reminder::DnsReminderView;
pub use json_view::JsonView;
pub use provision_details::ProvisionDetailsData;
pub use text_view::TextView;
