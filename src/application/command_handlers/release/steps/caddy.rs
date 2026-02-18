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
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::shared::clock::SystemClock;

/// Release the Caddy service (if HTTPS enabled)
///
/// Executes all steps required to release Caddy:
/// 1. Render configuration templates
/// 2. Deploy configuration to remote
///
/// If HTTPS is not configured, all steps are skipped.
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, step) if any Caddy step fails
#[allow(clippy::result_large_err)]
pub fn release(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
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

    render_templates(environment, listener)?;
    deploy_config_to_remote(environment, listener)?;
    Ok(())
}

/// Render Caddy configuration templates
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderCaddyTemplates`) if rendering fails
#[allow(clippy::result_large_err)]
fn render_templates(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderCaddyTemplates;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Template source: {}/caddy/",
            environment.templates_dir().display()
        ));
    }

    let clock = Arc::new(SystemClock);
    let step = RenderCaddyTemplatesStep::new(
        Arc::new(environment.clone()),
        environment.templates_dir(),
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

    if let Some(l) = listener {
        l.on_detail("Rendering Caddyfile from template");
    }

    info!(
        command = "release",
        step = %current_step,
        "Caddy configuration templates rendered successfully"
    );

    Ok(())
}

/// Deploy Caddy configuration to the remote host
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployCaddyConfigToRemote`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_config_to_remote(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployCaddyConfigToRemote;

    if let Some(l) = listener {
        l.on_debug("Executing playbook: ansible-playbook deploy-caddy-config.yml");
    }

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

    if let Some(l) = listener {
        l.on_detail("Deploying Caddyfile to /opt/torrust/Caddyfile");
    }

    info!(
        command = "release",
        step = %current_step,
        "Caddy configuration deployed to remote successfully"
    );

    Ok(())
}
