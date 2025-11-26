//! Black-box testing utilities
//!
//! This module provides utilities for black-box testing of the CLI application.
//! These utilities run the production binary as an external process, testing
//! the public interface exactly as end-users would use it.
//!
//! ## Module Structure
//!
//! - [`ProcessRunner`] - Executes CLI commands as external processes
//! - [`ProcessResult`] - Wraps execution results with convenient accessors
//! - `tasks` - Reusable tasks for black-box E2E testing
//!
//! ## Usage
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::testing::e2e::black_box::{ProcessRunner, ProcessResult};
//!
//! let result = ProcessRunner::new()
//!     .run_create_command("./config.json")?;
//!
//! assert!(result.success());
//! ```

mod process_runner;
pub mod tasks;

pub use process_runner::{ProcessResult, ProcessRunner};
