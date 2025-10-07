//! Testing utilities and fixtures
//!
//! This module provides reusable test fixtures and helpers for testing
//! various components of the application.

pub mod fixtures;
pub mod mock_clock;

// Re-export commonly used testing types
pub use mock_clock::MockClock;
