//! Infrastructure Layer (DDD)
//!
//! This module contains infrastructure concerns including low-level command execution
//! and external tool adapters. The infrastructure layer provides technical capabilities
//! and integration points for the application layer.
//!
//! ## Components
//!
//! - `executor` - Low-level shell command execution utilities
//! - `adapters` - External tool integration adapters (Ansible, LXD, `OpenTofu`, SSH)

pub mod adapters;
pub mod executor;

// Re-export commonly used types for convenience
pub use executor::{CommandError, CommandExecutor};
