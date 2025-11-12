//! Testing utilities for `UserOutput` testing
//!
//! This module provides simplified testing infrastructure for capturing and asserting on output
//! in tests across the codebase. It is organized into focused submodules:
//!
//! - [`test_writer`] - Writer implementation for capturing output to shared buffers
//! - [`test_user_output`] - Test wrapper for `UserOutput` with buffer management
//! - [`test_wrapper`] - Generic wrapper providing convenient access to wrapped `UserOutput`
//!
//! # Quick Start
//!
//! For most tests, use `TestUserOutput::new()` to create a test output instance:
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::presentation::views::testing::TestUserOutput;
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//!
//! let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
//! test_output.output.progress("Processing...");
//!
//! assert_eq!(test_output.stderr(), "⏳ Processing...\n");
//! assert_eq!(test_output.stdout(), "");
//! ```
//!
//! For tests that need wrapped output (e.g., with `Arc<ReentrantMutex<RefCell<UserOutput>>>`):
//!
//! ```rust,ignore
//! use std::cell::RefCell;
//! use std::sync::Arc;
//! use parking_lot::ReentrantMutex;
//!
//! let test_user_output = TestUserOutput::new(VerbosityLevel::Normal);
//! let stdout_buf = Arc::clone(&test_user_output.stdout_buffer);
//! let stderr_buf = Arc::clone(&test_user_output.stderr_buffer);
//! let test_output = TestOutputWrapper::new(
//!     Arc::new(ReentrantMutex::new(RefCell::new(test_user_output.output))),
//!     stdout_buf,
//!     stderr_buf,
//! );
//! test_output.steps("Working...", &["Step 1"]);
//! assert_eq!(test_output.stderr(), "Working...\n  1. Step 1\n");
//! ```

pub mod test_user_output;
pub mod test_wrapper;
pub mod test_writer;

// Re-export main types for convenience
pub use test_user_output::TestUserOutput;
pub use test_wrapper::TestOutputWrapper;
pub use test_writer::TestWriter;
