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
use std::net::IpAddr;
use tracing::info;

use crate::application::commands::TestCommand;
use crate::e2e::context::TestContext;

/// Validate deployment by running infrastructure validation tests
///
/// # Errors
///
/// Returns an error if:
/// - `TestCommand` execution fails
/// - Any validation check fails
pub async fn run_test_command(test_context: &TestContext, instance_ip: &IpAddr) -> Result<()> {
    info!("Starting deployment validation");

    // Ensure environment has the instance IP set (clone since we're borrowing)
    let environment = test_context
        .environment
        .clone()
        .with_instance_ip(*instance_ip);

    // Use TestCommand to handle all infrastructure validation steps
    let test_command = TestCommand::new();

    test_command
        .execute(&environment)
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to validate deployment")?;

    info!(status = "success", "All deployment validations passed");
    Ok(())
}
