//! `OpenTofu` template wrappers
//!
//! Organized by provider (e.g., lxd/)

pub mod lxd;

// Re-export LXD templates for easier access
pub use lxd::{CloudInitTemplate, MainTfTemplate};
