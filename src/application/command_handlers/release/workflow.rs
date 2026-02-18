//! Release workflow orchestration
//!
//! This module orchestrates the complete release workflow by coordinating
//! all service-specific release steps in the correct order.

use super::errors::ReleaseCommandHandlerError;
use super::handler::TOTAL_RELEASE_STEPS;
use super::steps::{backup, caddy, compose, grafana, mysql, prometheus, tracker};
use crate::application::command_handlers::common::StepResult;
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Released, Releasing};

/// Execute the release workflow
///
/// Orchestrates the complete release workflow by calling each service's
/// release steps in the correct order. Each service module handles its
/// own conditional logic (e.g., skipping if not enabled).
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for step-level reporting
///
/// # Errors
///
/// Returns a tuple of (error, `current_step`) if any release step fails
pub async fn execute(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<Environment<Released>, ReleaseCommandHandlerError, ReleaseStep> {
    // Step 1/7: Release Tracker service
    notify_step_started(listener, 1, "Releasing Tracker service");
    tracker::release(environment, listener)?;

    // Step 2/7: Release Prometheus service
    notify_step_started(listener, 2, "Releasing Prometheus service");
    prometheus::release(environment, listener)?;

    // Step 3/7: Release Grafana service
    notify_step_started(listener, 3, "Releasing Grafana service");
    grafana::release(environment, listener)?;

    // Step 4/7: Release MySQL service
    notify_step_started(listener, 4, "Releasing MySQL service");
    mysql::release(environment, listener)?;

    // Step 5/7: Release Backup service
    notify_step_started(listener, 5, "Releasing Backup service");
    backup::release(environment, listener).await?;

    // Step 6/7: Release Caddy service
    notify_step_started(listener, 6, "Releasing Caddy service");
    caddy::release(environment, listener)?;

    // Step 7/7: Deploy Docker Compose configuration
    notify_step_started(listener, 7, "Deploying Docker Compose configuration");
    compose::release(environment, listener).await?;

    Ok(environment.clone().released())
}

/// Notify the progress listener that a step has started.
///
/// This is a convenience helper that handles the `Option` check,
/// keeping the step-reporting code in the workflow clean.
fn notify_step_started(
    listener: Option<&dyn CommandProgressListener>,
    step_number: usize,
    description: &str,
) {
    if let Some(l) = listener {
        l.on_step_started(step_number, TOTAL_RELEASE_STEPS, description);
    }
}
