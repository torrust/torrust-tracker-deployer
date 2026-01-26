//! Release workflow orchestration
//!
//! This module orchestrates the complete release workflow by coordinating
//! all service-specific release steps in the correct order.

use std::net::IpAddr;

use super::errors::ReleaseCommandHandlerError;
use super::steps;
use crate::application::command_handlers::common::StepResult;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Released, Releasing};

/// Execute the release workflow with step tracking
///
/// This function orchestrates the complete release workflow, organized by service:
///
/// 1. **Tracker**: Storage creation, database init, config rendering, deployment
/// 2. **Prometheus**: Storage creation, config rendering, deployment (if enabled)
/// 3. **Grafana**: Storage creation, provisioning rendering, deployment (if enabled)
/// 4. **`MySQL`**: Storage creation (if enabled)
/// 5. **Caddy**: Config rendering, deployment (if HTTPS enabled)
/// 6. **Docker Compose**: Template rendering, deployment
///
/// If an error occurs, it returns both the error and the step that was being
/// executed, enabling accurate failure context generation.
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `instance_ip` - The validated instance IP address (used for Docker Compose deployment logging)
///
/// # Errors
///
/// Returns a tuple of (error, `current_step`) if any release step fails
pub async fn execute(
    environment: &Environment<Releasing>,
    instance_ip: IpAddr,
) -> StepResult<Environment<Released>, ReleaseCommandHandlerError, ReleaseStep> {
    // Tracker service steps
    steps::tracker::release(environment)?;

    // Prometheus service steps (if enabled)
    steps::prometheus::release(environment)?;

    // Grafana service steps (if enabled)
    steps::grafana::release(environment)?;

    // MySQL service steps (if enabled)
    steps::mysql::release(environment)?;

    // Caddy service steps (if HTTPS enabled)
    steps::caddy::release(environment)?;

    // Docker Compose deployment
    steps::compose::release(environment, instance_ip).await?;

    let released = environment.clone().released();

    Ok(released)
}
