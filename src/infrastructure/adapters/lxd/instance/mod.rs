//! LXD instance data types and utilities
//!
//! This module provides data types and utilities for working with LXD instances
//! (containers and virtual machines), including instance information.
//!
//! ## Components
//!
//! - `info` - Instance information structures containing runtime data
//!
//! Instance naming utilities are provided by the domain layer's `InstanceName` type.
//!
//! These types are used throughout the LXD integration to represent and
//! manipulate instance data in a type-safe manner.

pub mod info;

pub use info::InstanceInfo;
// Re-export InstanceName from domain layer for convenience
pub use crate::domain::InstanceName;
