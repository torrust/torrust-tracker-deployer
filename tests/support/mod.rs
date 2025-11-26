//! Test Support Infrastructure
//!
//! This module provides reusable test utilities for black-box E2E testing.
//! It includes temporary workspace management, external process execution,
//! and environment state assertions.
//!
//! ## Shared vs Local Utilities
//!
//! - `ProcessRunner` - Re-exported from `src/testing/black_box` (shared with `src/bin/`)
//! - `TempWorkspace` - Local to `tests/` (only used by integration tests)
//! - `EnvironmentStateAssertions` - Local to `tests/` (only used by integration tests)

mod assertions;
mod temp_workspace;

pub use assertions::EnvironmentStateAssertions;
pub use temp_workspace::TempWorkspace;

// Re-export ProcessRunner from the library's testing module
// This allows tests to continue using `support::ProcessRunner` without changes
pub use torrust_tracker_deployer_lib::testing::e2e::black_box::ProcessRunner;
