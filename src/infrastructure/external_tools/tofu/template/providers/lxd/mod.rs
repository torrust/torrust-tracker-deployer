//! LXD provider-specific `OpenTofu` template functionality.
//!
//! This module contains template wrappers and utilities specific to the LXD provider.

pub mod wrappers;

pub use wrappers::cloud_init;
pub use wrappers::variables;
