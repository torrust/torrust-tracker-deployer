//! LXD container and VM management wrapper
//!
//! This module provides a comprehensive interface for managing LXD containers and virtual
//! machines, including instance lifecycle management, information retrieval, and JSON
//! output parsing.
//!
//! ## Module Structure
//!
//! - `client` - Main `LxdClient` for executing LXD commands
//! - `instance` - Instance information and naming utilities
//! - `json_parser` - JSON output parsing for LXD command responses
//!
//! The module abstracts LXD's command-line interface and provides type-safe Rust APIs
//! for common container and VM operations.

pub mod client;
pub mod instance;
pub mod json_parser;

// Re-export public types for external use
pub use client::LxdClient;
pub use instance::{InstanceInfo, InstanceName};
