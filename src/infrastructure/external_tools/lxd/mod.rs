//! LXD integration for container and VM management
//!
//! This module provides LXD-specific functionality for the deployment system.
//! LXD is used as a virtualization provider for creating and managing instances.
//!
//! ## Components
//!
//! - `adapter` - LXD command-line tool wrapper and client implementation

pub mod adapter;

pub use adapter::client::LxdClient;
