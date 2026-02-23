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

#![allow(dead_code, unused_imports)]

mod assertions;
mod temp_workspace;

pub use assertions::EnvironmentStateAssertions;
pub use temp_workspace::TempWorkspace;

// Re-export ProcessRunner from the library's testing module
pub use torrust_tracker_deployer_lib::testing::e2e::ProcessRunner;

/// Returns a [`ProcessRunner`] configured to use the pre-built production binary.
///
/// Cargo sets `CARGO_BIN_EXE_torrust-tracker-deployer` to the absolute path of the
/// compiled binary when building integration tests. Cargo guarantees the binary is
/// built as a prerequisite automatically — no race conditions.
///
/// # Panics
///
/// Panics at compile time if the binary name does not exist in the workspace.
pub fn process_runner() -> ProcessRunner {
    ProcessRunner::new().with_binary(env!("CARGO_BIN_EXE_torrust-tracker-deployer"))
}
