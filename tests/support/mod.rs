//! Test Support Infrastructure
//!
//! This module provides reusable test utilities for black-box E2E testing.
//! It includes temporary workspace management, external process execution,
//! and environment state assertions.

mod assertions;
mod process_runner;
mod temp_workspace;

pub use assertions::EnvironmentStateAssertions;
pub use process_runner::ProcessRunner;
pub use temp_workspace::TempWorkspace;
