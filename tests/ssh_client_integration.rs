//! SSH Client Integration Tests (Backwards Compatibility)
//!
//! This file provides backwards compatibility for the SSH client integration tests
//! that have been split into focused modules following project conventions.
//!
//! The tests are now organized in separate modules:
//! - `ssh_client::connectivity_tests` - SSH connectivity testing
//! - `ssh_client::command_execution_tests` - Remote command execution
//! - `ssh_client::configuration_tests` - Configuration validation
//!
//! All test utilities, constants, and helper functions are defined in the
//! `ssh_client::mod` module and re-exported for use across all test modules.
//!
//! # Running Tests
//!
//! Run all SSH integration tests:
//! ```bash
//! cargo test --test ssh_client_integration
//! ```
//!
//! Run specific test categories:
//! ```bash
//! cargo test ssh_client::connectivity_tests
//! cargo test ssh_client::command_execution_tests
//! cargo test ssh_client::configuration_tests
//! ```

mod ssh_client;
