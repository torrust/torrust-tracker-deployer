//! LXD instance name re-export from domain layer
//!
//! This module re-exports the `InstanceName` type from the domain layer
//! to maintain backward compatibility with existing LXD adapter imports.

// Re-export the domain type
pub use crate::domain::InstanceName;
