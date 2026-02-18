//! Grafana service release steps
//!
//! This module contains all steps required to release the Grafana service:
//! - Storage directory creation
//! - Provisioning template rendering
//! - Provisioning deployment to remote
//!
//! All steps are optional and only execute if Grafana is configured.
//! Provisioning steps additionally require Prometheus for datasource configuration.

use std::sync::Arc;

use tracing::info;

use super::common::ansible_client;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::application::{
    CreateGrafanaStorageStep, DeployGrafanaProvisioningStep,
};
use crate::application::steps::rendering::RenderGrafanaTemplatesStep;
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::shared::clock::SystemClock;

/// Release the Grafana service (if enabled)
///
/// Executes all steps required to release Grafana:
/// 1. Create storage directories
/// 2. Render provisioning templates (requires Prometheus)
/// 3. Deploy provisioning to remote (requires Prometheus)
///
/// If Grafana is not configured, all steps are skipped.
/// Provisioning steps are skipped if Prometheus is not configured.
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, step) if any Grafana step fails
#[allow(clippy::result_large_err)]
pub fn release(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    // Check if Grafana is configured
    if environment.context().user_inputs.grafana().is_none() {
        info!(
            command = "release",
            service = "grafana",
            status = "skipped",
            "Grafana not configured - skipping all Grafana steps"
        );
        return Ok(());
    }

    create_storage(environment, listener)?;

    // Provisioning requires Prometheus for datasource configuration
    if environment.context().user_inputs.prometheus().is_none() {
        info!(
            command = "release",
            service = "grafana",
            status = "partial",
            "Prometheus not configured - skipping Grafana provisioning (datasource requires Prometheus)"
        );
        return Ok(());
    }

    render_templates(environment, listener)?;
    deploy_provisioning_to_remote(environment, listener)?;
    Ok(())
}

/// Create Grafana storage directories on the remote host
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::CreateGrafanaStorage`) if creation fails
#[allow(clippy::result_large_err)]
fn create_storage(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::CreateGrafanaStorage;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Ansible working directory: {}",
            environment.ansible_build_dir().display()
        ));
        l.on_debug("Executing playbook: ansible-playbook create-grafana-storage.yml");
    }

    CreateGrafanaStorageStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::GrafanaStorageCreation {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    if let Some(l) = listener {
        l.on_detail(
            "Creating storage directories: /opt/torrust/storage/grafana/{data,provisioning}",
        );
    }

    info!(
        command = "release",
        step = %current_step,
        "Grafana storage directories created successfully"
    );

    Ok(())
}

/// Render Grafana provisioning templates
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderGrafanaTemplates`) if rendering fails
#[allow(clippy::result_large_err)]
fn render_templates(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderGrafanaTemplates;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Template source: {}/grafana/",
            environment.templates_dir().display()
        ));
    }

    let clock = Arc::new(SystemClock);
    let step = RenderGrafanaTemplatesStep::new(
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
        l.on_detail("Rendering Grafana provisioning files (datasources, dashboards)");
    }

    info!(
        command = "release",
        step = %current_step,
        "Grafana provisioning templates rendered successfully"
    );

    Ok(())
}

/// Deploy Grafana provisioning configuration to the remote host
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployGrafanaProvisioning`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_provisioning_to_remote(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployGrafanaProvisioning;

    if let Some(l) = listener {
        l.on_debug("Executing playbook: ansible-playbook deploy-grafana-provisioning.yml");
    }

    DeployGrafanaProvisioningStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::GrafanaProvisioningDeployment {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    if let Some(l) = listener {
        l.on_detail("Deploying provisioning to /opt/torrust/storage/grafana/provisioning");
    }

    info!(
        command = "release",
        step = %current_step,
        "Grafana provisioning configuration deployed successfully"
    );

    Ok(())
}
