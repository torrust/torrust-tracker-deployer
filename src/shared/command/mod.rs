//! Command execution utilities with error handling and logging
//!
//! This module provides utilities for executing external commands with proper error handling,
//! logging, and output capture. It supports both verbose and quiet execution modes and provides
//! structured error types for different failure scenarios.
//!
//! ## Key Features
//!
//! - Structured error handling with detailed context
//! - Optional verbose output logging
//! - Working directory support
//! - Comprehensive error categorization (startup vs execution failures)

pub mod error;
pub mod executor;
pub mod result;

// Re-export the main types for convenience
pub use error::CommandError;
pub use executor::CommandExecutor;
pub use result::CommandResult;
