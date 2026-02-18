//! Prometheus service release steps
//!
//! This module contains all steps required to release the Prometheus service:
//! - Storage directory creation
//! - Configuration template rendering
//! - Configuration deployment to remote
//!
//! All steps are optional and only execute if Prometheus is configured.

use std::sync::Arc;

use tracing::info;

use super::common::ansible_client;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::application::{
    CreatePrometheusStorageStep, DeployPrometheusConfigStep,
};
use crate::application::steps::rendering::RenderPrometheusTemplatesStep;
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::shared::clock::SystemClock;

/// Release the Prometheus service (if enabled)
///
/// Executes all steps required to release Prometheus:
/// 1. Create storage directories
/// 2. Render configuration templates
/// 3. Deploy configuration to remote
///
/// If Prometheus is not configured, all steps are skipped.
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, step) if any Prometheus step fails
#[allow(clippy::result_large_err)]
pub fn release(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    // Check if Prometheus is configured
    if environment.context().user_inputs.prometheus().is_none() {
        info!(
            command = "release",
            service = "prometheus",
            status = "skipped",
            "Prometheus not configured - skipping all Prometheus steps"
        );
        return Ok(());
    }

    create_storage(environment, listener)?;
    render_templates(environment, listener)?;
    deploy_config_to_remote(environment, listener)?;
    Ok(())
}

/// Create Prometheus storage directories on the remote host
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::CreatePrometheusStorage`) if creation fails
#[allow(clippy::result_large_err)]
fn create_storage(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::CreatePrometheusStorage;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Ansible working directory: {}",
            environment.ansible_build_dir().display()
        ));
        l.on_debug("Executing playbook: ansible-playbook create-prometheus-storage.yml");
    }

    CreatePrometheusStorageStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::PrometheusStorageCreation {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    if let Some(l) = listener {
        l.on_detail("Creating storage directories: /opt/torrust/storage/prometheus/etc");
    }

    info!(
        command = "release",
        step = %current_step,
        "Prometheus storage directories created successfully"
    );

    Ok(())
}

/// Render Prometheus configuration templates to the build directory
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderPrometheusTemplates`) if rendering fails
#[allow(clippy::result_large_err)]
fn render_templates(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderPrometheusTemplates;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Template source: {}/prometheus/",
            environment.templates_dir().display()
        ));
    }

    let clock = Arc::new(SystemClock);
    let step = RenderPrometheusTemplatesStep::new(
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
        l.on_detail("Rendering prometheus.yml from template");
    }

    info!(
        command = "release",
        step = %current_step,
        "Prometheus configuration templates rendered successfully"
    );

    Ok(())
}

/// Deploy Prometheus configuration to the remote host via Ansible
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployPrometheusConfigToRemote`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_config_to_remote(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployPrometheusConfigToRemote;

    if let Some(l) = listener {
        l.on_debug("Executing playbook: ansible-playbook deploy-prometheus-config.yml");
    }

    DeployPrometheusConfigStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::PrometheusConfigDeployment {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    if let Some(l) = listener {
        l.on_detail("Deploying config to /opt/torrust/storage/prometheus/etc/prometheus.yml");
    }

    info!(
        command = "release",
        step = %current_step,
        "Prometheus configuration deployed successfully"
    );

    Ok(())
}
