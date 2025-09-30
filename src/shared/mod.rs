//! Shared modules used across different parts of the application
//!
//! This module contains commonly used functionality that can be shared
//! between different layers of the application, including infrastructure,
//! e2e tests, and other components.

pub mod command;
pub mod executor;
pub mod port_checker;
pub mod ssh;
pub mod username;

// Re-export commonly used types for convenience
pub use command::{CommandError, CommandExecutor, CommandResult};
pub use port_checker::{PortChecker, PortCheckerError};
pub use username::{Username, UsernameError};
