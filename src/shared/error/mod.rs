//! Error handling utilities
//!
//! This module provides shared error handling utilities including
//! the `Traceable` trait for generating detailed error traces and
//! the `ErrorKind` enum for high-level error categorization.

pub mod kind;
pub mod traceable;

pub use kind::ErrorKind;
pub use traceable::Traceable;
