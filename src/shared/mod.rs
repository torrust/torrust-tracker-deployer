//! Shared modules used across different parts of the application
//!
//! This module contains commonly used functionality that can be shared
//! between different layers of the application, including infrastructure,
//! e2e tests, and other components.

pub mod clock;
pub mod command;
pub mod error;
pub mod secrets;
pub mod username;

// Re-export commonly used types for convenience
pub use clock::{Clock, SystemClock};
pub use command::{CommandError, CommandExecutor, CommandResult};
pub use error::{ErrorKind, Traceable};
pub use secrets::{ApiToken, ExposeSecret, Password, PlainApiToken, PlainPassword};
pub use username::{Username, UsernameError};
