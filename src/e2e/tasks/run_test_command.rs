//! Deployment validation task for E2E testing
//!
//! This module provides comprehensive deployment validation functionality for E2E
//! testing. It verifies that deployed infrastructure is working correctly by
//! running a series of validation checks through the `TestCommand`.
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
//! - Comprehensive validation via `TestCommand` orchestration
//! - Async execution for efficient testing workflows
//! - Detailed logging and error reporting
//! - Integration with E2E testing pipeline
//!
//! This task ensures that the complete deployment workflow has resulted in a
//! functional environment ready for application deployment.

use anyhow::{Context, Result};
use tracing::info;

use crate::application::commands::TestCommand;
use crate::domain::environment::state::AnyEnvironmentState;
use crate::e2e::context::TestContext;

/// Validate deployment by running infrastructure validation tests
///
/// # Errors
///
/// Returns an error if:
/// - `TestCommand` execution fails
/// - Any validation check fails
/// - Environment does not have instance IP set
pub async fn run_test_command(test_context: &TestContext) -> Result<()> {
    info!("Starting deployment validation");

    // The environment in TestContext should already have the instance IP set after provisioning
    let instance_ip = test_context
        .environment
        .instance_ip()
        .context("Environment does not have instance IP set")?;

    info!(
        instance_ip = %instance_ip,
        "Validating deployment on instance"
    );

    // Use TestCommand to handle all infrastructure validation steps
    let test_command = TestCommand::new();

    // TestCommand::execute is generic over state, so we need to match on AnyEnvironmentState
    // and pass the typed environment. Since we're just reading, any state works.
    let result = match &test_context.environment {
        AnyEnvironmentState::Created(env) => test_command.execute(env).await,
        AnyEnvironmentState::Provisioning(env) => test_command.execute(env).await,
        AnyEnvironmentState::Provisioned(env) => test_command.execute(env).await,
        AnyEnvironmentState::Configuring(env) => test_command.execute(env).await,
        AnyEnvironmentState::Configured(env) => test_command.execute(env).await,
        AnyEnvironmentState::Releasing(env) => test_command.execute(env).await,
        AnyEnvironmentState::Released(env) => test_command.execute(env).await,
        AnyEnvironmentState::Running(env) => test_command.execute(env).await,
        AnyEnvironmentState::ProvisionFailed(env) => test_command.execute(env).await,
        AnyEnvironmentState::ConfigureFailed(env) => test_command.execute(env).await,
        AnyEnvironmentState::ReleaseFailed(env) => test_command.execute(env).await,
        AnyEnvironmentState::RunFailed(env) => test_command.execute(env).await,
        AnyEnvironmentState::Destroyed(env) => test_command.execute(env).await,
    };

    result
        .map_err(anyhow::Error::from)
        .context("Failed to validate deployment")?;

    info!(status = "success", "All deployment validations passed");
    Ok(())
}
