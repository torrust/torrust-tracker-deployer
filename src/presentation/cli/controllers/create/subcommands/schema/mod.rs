//! Schema Generation Subcommand
//!
//! This module handles the schema generation subcommand for creating
//! JSON Schema files from configuration types.

pub mod errors;
pub mod handler;

// Re-export the main handler and error types
pub use errors::CreateSchemaCommandError;
pub use handler::CreateSchemaCommandController;
