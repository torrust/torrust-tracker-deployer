//! Hetzner provider-specific `OpenTofu` template functionality.
//!
//! This module contains template wrappers and utilities specific to the Hetzner Cloud provider.
//!
//! Note: cloud-init wrapper has been moved to `common::wrappers::cloud_init` since
//! the same cloud-init template is used by all providers.

pub mod wrappers;

pub use wrappers::variables;
