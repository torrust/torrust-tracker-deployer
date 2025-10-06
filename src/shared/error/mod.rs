//! Error handling utilities
//!
//! This module provides shared error handling utilities including
//! the `Traceable` trait for generating detailed error traces.

pub mod traceable;

pub use traceable::Traceable;
