//! Hetzner provider-specific `OpenTofu` template functionality.
//!
//! This module contains template wrappers and utilities specific to the Hetzner Cloud provider.

pub mod wrappers;

pub use wrappers::cloud_init;
pub use wrappers::variables;
