//! Environment Creation Subcommand
//!
//! This module handles the environment creation subcommand for creating
//! deployment environments from configuration files.

pub mod config_loader;
pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export the main handler function, error types, and config loader
pub use config_loader::ConfigLoader;
pub use errors::{ConfigFormat, CreateEnvironmentCommandError};
pub use handler::handle;
