//! Template Generation Subcommand
//!
//! This module handles the template generation subcommand for creating
//! configuration file templates with placeholder values.

pub mod errors;
pub mod handler;

// Re-export the main handler function
pub use handler::handle_template_generation;
