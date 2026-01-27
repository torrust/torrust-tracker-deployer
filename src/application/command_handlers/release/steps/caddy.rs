//! Caddy service release steps
//!
//! This module contains all steps required to release the Caddy service:
//! - Configuration template rendering
//! - Configuration deployment to remote
//!
//! All steps are optional and only execute if HTTPS is configured.

use std::sync::Arc;

use tracing::info;

use super::common::ansible_client;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::application::DeployCaddyConfigStep;
use crate::application::steps::rendering::RenderCaddyTemplatesStep;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::domain::template::TemplateManager;
use crate::shared::clock::SystemClock;

/// Release the Caddy service (if HTTPS enabled)
///
/// Executes all steps required to release Caddy:
/// 1. Render configuration templates
/// 2. Deploy configuration to remote
///
/// If HTTPS is not configured, all steps are skipped.
///
/// # Errors
///
/// Returns a tuple of (error, step) if any Caddy step fails
#[allow(clippy::result_large_err)]
pub fn release(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    // Check if HTTPS is configured
    if environment.context().user_inputs.https().is_none() {
        info!(
            command = "release",
            service = "caddy",
            status = "skipped",
            "HTTPS not configured - skipping all Caddy steps"
        );
        return Ok(());
    }

    render_templates(environment)?;
    deploy_config_to_remote(environment)?;
    Ok(())
}

/// Render Caddy configuration templates
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderCaddyTemplates`) if rendering fails
#[allow(clippy::result_large_err)]
fn render_templates(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderCaddyTemplates;

    let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
    let clock = Arc::new(SystemClock);
    let step = RenderCaddyTemplatesStep::new(
        Arc::new(environment.clone()),
        template_manager,
        environment.build_dir().clone(),
        clock,
    );

    step.execute().map_err(|e| {
        (
            ReleaseCommandHandlerError::TemplateRendering {
                message: e.to_string(),
                source: Box::new(e),
            },
            current_step,
        )
    })?;

    info!(
        command = "release",
        step = %current_step,
        "Caddy configuration templates rendered successfully"
    );

    Ok(())
}

/// Deploy Caddy configuration to the remote host
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployCaddyConfigToRemote`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_config_to_remote(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployCaddyConfigToRemote;

    DeployCaddyConfigStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::CaddyConfigDeployment {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    info!(
        command = "release",
        step = %current_step,
        "Caddy configuration deployed to remote successfully"
    );

    Ok(())
}
