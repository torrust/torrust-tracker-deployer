//! Constants for command handlers
//!
//! This module provides centralized configuration constants used across command handlers.
//! These constants help eliminate duplicate magic numbers and make configuration explicit.
//!
//! ## Constants
//!
//! - [`DEFAULT_LOCK_TIMEOUT`]: Default timeout for file lock operations in repository
//! - [`DEFAULT_VERBOSITY`]: Default verbosity level for user output
//!
//! ## Benefits
//!
//! - Eliminates duplicate magic numbers across command handlers
//! - Makes configuration values explicit and discoverable
//! - Easier to adjust behavior without searching through code
//! - Better documentation for why specific values are used
//!
//! ## Usage Example
//!
//! ```rust
//! use torrust_tracker_deployer_lib::presentation::controllers::constants::{DEFAULT_LOCK_TIMEOUT, DEFAULT_VERBOSITY};
//! use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
//! use torrust_tracker_deployer_lib::presentation::views::UserOutput;
//!
//! // Use default lock timeout for repository operations
//! let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
//!
//! // Use default verbosity for user output
//! let mut output = UserOutput::new(DEFAULT_VERBOSITY);
//! ```

use std::time::Duration;

use crate::presentation::views::VerbosityLevel;

/// Default timeout for file lock operations in repository
///
/// This timeout is used when acquiring file locks during repository operations
/// to prevent indefinite blocking. A 30-second timeout provides a balance between:
///
/// - Allowing sufficient time for legitimate operations to complete
/// - Preventing indefinite hangs in case of issues
/// - Providing reasonable user experience (users won't wait forever)
///
/// This value is used across all command handlers that interact with the repository.
pub const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(30);

/// Default verbosity level for user output
///
/// This verbosity level is used as the default for user-facing output across all commands.
/// `VerbosityLevel::Normal` provides essential progress and results without overwhelming
/// users with details, making it suitable for typical command execution.
///
/// Users can override this default through command-line flags if they need:
/// - Less output (Quiet) for automation/scripting
/// - More output (Verbose/Debug) for troubleshooting
///
/// This value is used across all command handlers for consistent user experience.
pub const DEFAULT_VERBOSITY: VerbosityLevel = VerbosityLevel::Normal;
