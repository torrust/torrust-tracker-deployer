//! Template rendering delivery mechanisms
//!
//! This module contains template rendering delivery mechanisms and wrappers
//! that bridge domain template models with specific rendering implementations.

pub mod wrappers;

// Re-export wrapper types for convenience
pub use wrappers::*;
