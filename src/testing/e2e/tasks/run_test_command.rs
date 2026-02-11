//! Deployment validation task for E2E testing
//!
//! This module provides comprehensive deployment validation functionality for E2E
//! testing. It verifies that deployed infrastructure is working correctly by
//! running a series of validation checks through the `TestCommandHandler`.
//!
//! ## Validation Areas
//!
//! - Infrastructure service validation (Docker, Docker Compose)
//! - System initialization validation (cloud-init completion)
//! - Network connectivity and SSH access verification
//! - Application deployment readiness checks
//!
//! ## Key Features
//!
//! - Comprehensive validation via `TestCommandHandler` orchestration
//! - Async execution for efficient testing workflows
//! - Detailed logging and error reporting
//! - Integration with E2E testing pipeline
//!
//! This task ensures that the complete deployment workflow has resulted in a
//! functional environment ready for application deployment.

use thiserror::Error;
use tracing::info;

use crate::application::command_handlers::test::TestCommandHandlerError;
use crate::application::command_handlers::TestCommandHandler;
use crate::testing::e2e::context::TestContext;

/// Errors that can occur during the test/validation task
#[derive(Debug, Error)]
pub enum TestTaskError {
    /// Environment does not have an instance IP set
    #[error(
        "Environment does not have instance IP set, state: {state_type}
Tip: Ensure the environment is provisioned before running validation tests"
    )]
    MissingInstanceIp { state_type: String },

    /// Test command execution failed
    #[error(
        "Failed to validate deployment: {source}
Tip: Check that all required services are installed and running on the instance"
    )]
    ValidationFailed {
        #[source]
        source: TestCommandHandlerError,
    },
}

impl TestTaskError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::testing::e2e::tasks::run_test_command::TestTaskError;
    /// let error = TestTaskError::MissingInstanceIp {
    ///     state_type: "Created".to_string(),
    /// };
    /// println!("{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::MissingInstanceIp { .. } => {
                "Missing Instance IP - Detailed Troubleshooting:

1. Verify provisioning completed:
   - Check environment.json for instance_ip field
   - Ensure provision command completed successfully
   - Look for OpenTofu output errors

2. If provisioning succeeded but IP is missing:
   - Check OpenTofu outputs configuration
   - Verify the instance was actually created
   - Check LXD/provider for instance status

3. Recovery steps:
   - Re-run the provision command
   - Or destroy and recreate the environment

For more information, see the E2E testing documentation."
            }

            Self::ValidationFailed { .. } => {
                "Validation Failed - Detailed Troubleshooting:

1. Check which validation failed:
   - Cloud-init completion check
   - Docker installation check
   - Docker Compose installation check

2. Verify instance is accessible:
   - Test SSH connection manually
   - Check instance is running (not stopped or crashed)
   - Verify network connectivity

3. Check service installation:
   - SSH into instance and verify services manually
   - Check configuration playbooks completed successfully
   - Review Ansible logs for installation errors

4. Common issues:
   - Cloud-init still running (wait a few minutes)
   - Docker daemon not started
   - Services installed but not in PATH
   - Incomplete configuration phase

5. Recovery steps:
   - Wait for cloud-init to complete (check with: cloud-init status)
   - Re-run configuration command if services missing
   - Check instance system logs (journalctl)

For more information, see docs/e2e-testing/."
            }
        }
    }
}

/// Validate deployment by running infrastructure validation tests
///
/// # Errors
///
/// Returns an error if:
/// - Environment does not have instance IP set
/// - `TestCommandHandler` execution fails
/// - Any validation check fails
pub async fn run_test_command(test_context: &TestContext) -> Result<(), TestTaskError> {
    info!("Starting deployment validation");

    // The environment in TestContext should already have the instance IP set after provisioning
    let instance_ip =
        test_context
            .environment
            .instance_ip()
            .ok_or_else(|| TestTaskError::MissingInstanceIp {
                state_type: test_context.environment.state_name().to_string(),
            })?;

    info!(
        instance_ip = %instance_ip,
        "Validating deployment on instance"
    );

    // Create repository for the environment
    let repository = test_context.create_repository();

    // Use TestCommandHandler to handle all infrastructure validation steps
    let test_command_handler = TestCommandHandler::new(repository);

    // TestCommandHandler now accepts EnvironmentName instead of Environment
    let result = test_command_handler
        .execute(test_context.environment.name())
        .await;

    let test_result = result.map_err(|source| TestTaskError::ValidationFailed { source })?;

    if test_result.has_dns_warnings() {
        info!(
            dns_warnings = test_result.dns_warnings.len(),
            "DNS warnings detected (advisory only)"
        );
    }

    info!(status = "success", "All deployment validations passed");
    Ok(())
}
