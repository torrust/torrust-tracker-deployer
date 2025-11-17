//! Error types for the Provision Subcommand
//!
//! This module defines error types that can occur during CLI provision command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::application::command_handlers::provision::errors::ProvisionCommandHandlerError;
use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::views::progress::ProgressReporterError;

/// Provision command specific errors
///
/// This enum contains all error variants specific to the provision command,
/// including environment validation, repository access, and provisioning failures.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum ProvisionSubcommandError {
    // ===== Environment Validation Errors =====
    /// Environment name validation failed
    ///
    /// The provided environment name doesn't meet the validation requirements.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Invalid environment name '{name}': {source}
Tip: Environment names must be 1-63 characters, start with letter/digit, contain only letters/digits/hyphens")]
    InvalidEnvironmentName {
        name: String,
        #[source]
        source: EnvironmentNameError,
    },

    /// Environment not found or inaccessible
    ///
    /// The environment couldn't be loaded from persistent storage.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Environment '{name}' not found or inaccessible from data directory '{data_dir}'
Tip: Check if environment exists: ls -la {data_dir}/"
    )]
    EnvironmentNotAccessible { name: String, data_dir: String },

    /// Environment is not in Created state
    ///
    /// The environment is in the wrong state for provisioning.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Environment '{name}' is in '{current_state}' state, but 'Created' state is required for provisioning
Tip: Only environments in 'Created' state can be provisioned"
    )]
    InvalidEnvironmentState { name: String, current_state: String },

    // ===== Repository Access Errors =====
    /// Repository operation failed
    ///
    /// Failed to create or access the environment repository.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to access environment repository at '{data_dir}': {reason}
Tip: Check directory permissions and disk space"
    )]
    RepositoryAccessFailed { data_dir: String, reason: String },

    // ===== Provision Operation Errors =====
    /// Provision operation failed
    ///
    /// The provisioning process encountered an error during execution.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to provision environment '{name}': {source}
Tip: Check logs and try running with --log-output file-and-stderr for more details"
    )]
    ProvisionOperationFailed {
        name: String,
        #[source]
        source: Box<ProvisionCommandHandlerError>,
    },

    // ===== Internal Errors =====
    /// Progress reporting failed
    ///
    /// Failed to report progress to the user due to an internal error.
    /// This indicates a critical internal error.
    #[error(
        "Failed to report progress: {source}
Tip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    ProgressReportingFailed {
        #[source]
        source: ProgressReporterError,
    },
}

// ============================================================================
// ERROR CONVERSIONS
// ============================================================================

impl From<ProgressReporterError> for ProvisionSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl ProvisionSubcommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// Using with Container and `ExecutionContext` (recommended):
    ///
    /// ```rust
    /// use std::path::Path;
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use torrust_tracker_deployer_lib::presentation::controllers::provision;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal);
    /// let context = ExecutionContext::new(Arc::new(container));
    ///
    /// if let Err(e) = provision::handle("test-env", Path::new("."), &context) {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// ```
    ///
    /// Direct usage (for testing):
    ///
    /// ```rust
    /// use std::path::{Path, PathBuf};
    /// use std::sync::Arc;
    /// use std::time::Duration;
    /// use parking_lot::ReentrantMutex;
    /// use std::cell::RefCell;
    /// use torrust_tracker_deployer_lib::presentation::controllers::provision;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    /// use torrust_tracker_deployer_lib::shared::clock::SystemClock;
    ///
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let repository_factory = Arc::new(RepositoryFactory::new(Duration::from_secs(30)));
    /// let clock = Arc::new(SystemClock);
    /// if let Err(e) = provision::handle_provision_command("test-env", Path::new("."), repository_factory, clock, &output) {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// ```
    #[must_use]
    #[allow(clippy::too_many_lines)] // Help text is comprehensive for user guidance
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEnvironmentName { .. } => {
                "Invalid Environment Name - Detailed Troubleshooting:

1. Check environment name format:
   - Length: Must be 1-63 characters
   - Start: Must begin with a letter or digit
   - Characters: Only letters, digits, and hyphens allowed
   - No special characters: Avoid spaces, underscores, dots

2. Valid examples:
   - 'production'
   - 'staging-01'
   - 'dev-environment'

3. Invalid examples:
   - 'prod_01' (underscore not allowed)
   - '-production' (cannot start with hyphen)
   - 'prod.env' (dots not allowed)

For more information, see environment naming documentation."
            }

            Self::EnvironmentNotAccessible { .. } => {
                "Environment Not Accessible - Detailed Troubleshooting:

1. Verify environment exists:
   - List environments: ls -la data/
   - Check for environment.json file in data/<environment-name>/

2. Check file permissions:
   - Read permission: chmod +r data/<environment-name>/environment.json
   - Directory access: chmod +rx data/<environment-name>/

3. Verify data directory:
   - Ensure data/ directory exists
   - Check disk space: df -h
   - Verify no file locks: lsof data/

4. Common solutions:
   - Create environment first: torrust-tracker-deployer create environment -f <config-file>
   - Check working directory: pwd
   - Verify correct environment name

For more information, see the create command documentation."
            }

            Self::InvalidEnvironmentState { .. } => {
                "Invalid Environment State - Detailed Troubleshooting:

The provision command requires an environment in 'Created' state.

1. Check current state:
   - View environment file: cat data/<environment-name>/environment.json
   - Look for 'state' field

2. State transitions:
   - Created → Provisioning → Provisioned (success path)
   - Created → Provisioning → ProvisionFailed (error path)

3. If environment is already Provisioned:
   - Environment is ready to use
   - No need to provision again
   - Proceed to next deployment steps

4. If environment is in ProvisionFailed state:
   - Review error logs to understand failure
   - Fix underlying issues (network, permissions, etc.)
   - Create a new environment and try again

5. If environment is in unexpected state:
   - Consider creating a new environment
   - Report issue if state seems invalid

For more information, see environment lifecycle documentation."
            }

            Self::RepositoryAccessFailed { .. } => {
                "Repository Access Failed - Detailed Troubleshooting:

1. Check directory permissions:
   - Read/write access: ls -la data/
   - Fix permissions: chmod -R u+rw data/

2. Verify disk space:
   - Check available space: df -h
   - Free up space if needed

3. Check for file locks:
   - List open files: lsof data/
   - Kill processes if safe to do so

4. Verify directory exists:
   - Create if missing: mkdir -p data/
   - Check parent directory permissions

5. Common issues:
   - Running multiple instances simultaneously
   - File system mounted read-only
   - Insufficient permissions

For persistent issues, check system logs and file system health."
            }

            Self::ProvisionOperationFailed { .. } => {
                "Provision Operation Failed - Detailed Troubleshooting:

1. Review detailed error logs:
   - Run with verbose output: --log-output file-and-stderr
   - Check log files in data/logs/

2. Common failure points:
   - OpenTofu initialization or apply failures
   - LXD/VM provisioning issues
   - SSH connectivity problems
   - Cloud-init timeout or failures

3. Infrastructure-specific troubleshooting:
   
   OpenTofu/LXD issues:
   - Check LXD status: lxc list
   - Verify LXD daemon: systemctl status lxd
   - Review OpenTofu state: cd build/tofu/lxd && tofu state list
   
   SSH connectivity:
   - Verify SSH keys exist: ls -la ~/.ssh/
   - Check VM is running: lxc list
   - Test SSH manually: ssh -i <key> <user>@<ip>
   
   Cloud-init:
   - Check cloud-init status: lxc exec <vm> -- cloud-init status
   - View cloud-init logs: lxc exec <vm> -- cat /var/log/cloud-init.log

4. Recovery steps:
   - Review error messages and logs
   - Fix underlying issues
   - Destroy failed environment: torrust-tracker-deployer destroy <env-name>
   - Create new environment and retry

For more information, see the provisioning troubleshooting guide."
            }

            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Critical Internal Error:

This is a critical internal error that should not occur during normal operation.

1. Immediate actions:
   - Save full error output
   - Copy log files from data/logs/
   - Note the exact command that was running

2. Report the issue:
   - Create GitHub issue with full details
   - Include: command, error output, logs, system info
   - Describe steps to reproduce

3. Temporary workarounds:
   - Try running command again
   - Restart application
   - Check for system resource issues (memory, file descriptors)

This error indicates a bug in the progress reporting system.
Please report it so we can fix it."
            }
        }
    }
}
