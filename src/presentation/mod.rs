//! Presentation Layer
//!
//! This layer handles user-facing output and presentation concerns following DDD architecture.
//! It manages how information is presented to users, separate from internal logging and
//! application logic.
//!
//! ## Responsibilities
//!
//! - **User Output**: Managing user-facing messages, progress updates, and result presentation
//! - **Output Channels**: Implementing proper stdout/stderr separation for CLI applications
//! - **Verbosity Control**: Handling different levels of output detail based on user preferences
//! - **Output Formatting**: Structuring output for both human consumption and automation/piping
//!
//! ## Design Principles
//!
//! - **Channel Separation**: Following Unix conventions with stdout for results and stderr for operational messages
//! - **Automation Friendly**: Supporting clean piping and redirection for scripting
//! - **User Experience**: Providing clear, actionable feedback without interfering with result data
//! - **Verbosity Levels**: Respecting user preferences for output detail

pub mod user_output;

// Re-export commonly used presentation types for convenience
pub use user_output::{UserOutput, VerbosityLevel};
