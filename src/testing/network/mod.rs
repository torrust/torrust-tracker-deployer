//! Network Testing Utilities
//!
//! This module provides network-related testing utilities for checking port
//! connectivity and port usage. These utilities are used exclusively in tests
//! to verify network connectivity and diagnose port conflicts.
//!
//! ## Modules
//!
//! - `port_checker` - TCP port connectivity checking
//! - `port_usage_checker` - Port usage and process identification

pub mod port_checker;
pub mod port_usage_checker;

// Re-export commonly used types for convenience
pub use port_checker::{PortChecker, PortCheckerError};
pub use port_usage_checker::{PortUsageChecker, PortUsageError};
