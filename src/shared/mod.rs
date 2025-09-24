//! Shared modules used across different parts of the application
//!
//! This module contains commonly used functionality that can be shared
//! between different layers of the application, including infrastructure,
//! e2e tests, and other components.

pub mod executor;
pub mod ssh;

// Re-export commonly used types for convenience
pub use executor::{CommandError, CommandExecutor};
