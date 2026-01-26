//! Release workflow orchestration
//!
//! This module orchestrates the complete release workflow by coordinating
//! all service-specific release steps in the correct order.

use super::errors::ReleaseCommandHandlerError;
use super::steps::{caddy, compose, grafana, mysql, prometheus, tracker};
use crate::application::command_handlers::common::StepResult;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Released, Releasing};

/// Execute the release workflow
///
/// Orchestrates the complete release workflow by calling each service's
/// release steps in the correct order. Each service module handles its
/// own conditional logic (e.g., skipping if not enabled).
///
/// # Errors
///
/// Returns a tuple of (error, `current_step`) if any release step fails
pub async fn execute(
    environment: &Environment<Releasing>,
) -> StepResult<Environment<Released>, ReleaseCommandHandlerError, ReleaseStep> {
    tracker::release(environment)?;
    prometheus::release(environment)?;
    grafana::release(environment)?;
    mysql::release(environment)?;
    caddy::release(environment)?;
    compose::release(environment).await?;

    Ok(environment.clone().released())
}
