//! LXD instance data types and utilities
//!
//! This module provides data types and utilities for working with LXD instances
//! (containers and virtual machines), including instance information and naming.
//!
//! ## Components
//!
//! - `info` - Instance information structures containing runtime data
//! - `name` - Instance naming utilities and validation
//!
//! These types are used throughout the LXD integration to represent and
//! manipulate instance data in a type-safe manner.

pub mod info;
pub mod name;

pub use info::InstanceInfo;
pub use name::InstanceName;
