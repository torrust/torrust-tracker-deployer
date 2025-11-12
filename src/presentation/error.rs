//! Error Handling Module - Presentation Layer
//!
//! This module provides error handling functionality for the presentation layer,
//! specifically focusing on displaying errors to users in a helpful and actionable way.
//!
//! ## Purpose
//!
//! The error handling module is responsible for:
//! - **User-Friendly Error Display**: Converting internal errors to readable messages
//! - **Actionable Guidance**: Providing specific steps users can take to resolve issues
//! - **Fallback Handling**: Ensuring error messages are displayed even in degraded states
//! - **Consistent Formatting**: Maintaining consistent error output across all commands
//!
//! ## Design Principles
//!
//! - **Observability**: All errors include sufficient context for debugging
//! - **Actionability**: Error messages tell users how to fix problems
//! - **Reliability**: Error handling itself must not fail
//! - **Consistency**: All errors follow the same display patterns
//!
//! ## Module Integration
//!
//! This module integrates with:
//! - **`CommandError` Types** - Uses structured error types from `presentation::errors`
//! - **`UserOutput` Service** - Leverages user output for consistent formatting
//! - **Help System** - Displays detailed troubleshooting via `.help()` method
//!
//! ## Usage
//!
//! ```rust
//! use std::sync::Arc;
//! use std::cell::RefCell;
//! use parking_lot::ReentrantMutex;
//! use torrust_tracker_deployer_lib::presentation::{error, user_output};
//! use torrust_tracker_deployer_lib::presentation::errors::CommandError;
//!
//! # fn example(error: CommandError, user_output: Arc<ReentrantMutex<RefCell<user_output::UserOutput>>>) {
//! // Display error with detailed troubleshooting
//! error::handle_error(&error, &user_output);
//! # }
//! ```

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::presentation::errors::CommandError;
use crate::presentation::user_output::UserOutput;

/// Handle command errors with consistent user output
///
/// This function provides standardized error output for all command failures.
/// It displays the error message and detailed troubleshooting information
/// to help users resolve issues.
///
/// # Arguments
///
/// * `error` - The command error to handle and display
/// * `user_output` - Shared user output service for consistent output formatting
///
/// # Example
///
/// ```rust
/// use clap::Parser;
/// use std::sync::Arc;
/// use std::cell::RefCell;
/// use parking_lot::ReentrantMutex;
/// use torrust_tracker_deployer_lib::presentation::{error, input::cli, errors, user_output};
/// use torrust_tracker_deployer_lib::presentation::controllers::destroy::DestroySubcommandError;
/// use torrust_tracker_deployer_lib::domain::environment::name::EnvironmentNameError;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Example of handling a command error (simulated for testing)
/// let name_error = EnvironmentNameError::InvalidFormat {
///     attempted_name: "invalid_name".to_string(),
///     reason: "contains invalid characters: _".to_string(),
///     valid_examples: vec!["dev".to_string(), "staging".to_string()],
/// };
/// let sample_error = errors::CommandError::Destroy(
///     Box::new(DestroySubcommandError::InvalidEnvironmentName {
///         name: "invalid_name".to_string(),
///         source: name_error,
///     })
/// );
/// let user_output = Arc::new(ReentrantMutex::new(RefCell::new(user_output::UserOutput::new(user_output::VerbosityLevel::Normal))));
/// error::handle_error(&sample_error, &user_output);
/// # Ok(())
/// # }
/// ```
pub fn handle_error(error: &CommandError, user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>) {
    let help_text = error.help();

    // With ReentrantMutex, we can safely acquire the lock multiple times from the same thread
    let lock = user_output.lock();
    let mut output = lock.borrow_mut();
    output.error(&format!("{error}"));
    output.blank_line();
    output.info_block("For detailed troubleshooting:", &[help_text]);
}
