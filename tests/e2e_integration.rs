//! End-to-End Integration Tests
//!
//! This file provides the entry point for E2E integration tests that verify
//! the complete functionality of the Torrust Tracker Deployer commands.
//!
//! The tests are organized in separate modules:
//! - `e2e::create_command` - Tests for the create command
//! - `e2e::destroy_command` - Tests for the destroy command
//!
//! # Running Tests
//!
//! Run all E2E integration tests:
//! ```bash
//! cargo test --test e2e_integration
//! ```
//!
//! Run specific test module:
//! ```bash
//! cargo test --test e2e_integration create_command
//! cargo test --test e2e_integration destroy_command
//! ```

mod e2e;
mod support;
