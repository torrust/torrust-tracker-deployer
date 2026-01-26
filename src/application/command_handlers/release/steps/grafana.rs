//! Grafana service release steps
//!
//! This module contains all steps required to release the Grafana service:
//! - Storage directory creation
//! - Provisioning template rendering
//! - Provisioning deployment to remote
//!
//! All steps are optional and only execute if Grafana is configured.

use std::sync::Arc;

use tracing::info;

use super::common::ansible_client;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::application::{
    CreateGrafanaStorageStep, DeployGrafanaProvisioningStep,
};
use crate::application::steps::rendering::RenderGrafanaTemplatesStep;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::domain::template::TemplateManager;

/// Release the Grafana service (if enabled)
///
/// Executes all steps required to release Grafana:
/// 1. Create storage directories
/// 2. Render provisioning templates
/// 3. Deploy provisioning to remote
///
/// If Grafana is not configured, all steps are skipped.
///
/// # Errors
///
/// Returns a tuple of (error, step) if any Grafana step fails
#[allow(clippy::result_large_err)]
pub fn release(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    create_storage(environment)?;
    render_templates(environment)?;
    deploy_provisioning_to_remote(environment)?;
    Ok(())
}

/// Create Grafana storage directories on the remote host (if enabled)
///
/// This step is optional and only executes if Grafana is configured in the environment.
/// If Grafana is not configured, the step is skipped without error.
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::CreateGrafanaStorage`) if creation fails
#[allow(clippy::result_large_err)]
fn create_storage(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::CreateGrafanaStorage;

    // Check if Grafana is configured
    if environment.context().user_inputs.grafana().is_none() {
        info!(
            command = "release",
            step = %current_step,
            status = "skipped",
            "Grafana not configured - skipping storage creation"
        );
        return Ok(());
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

    info!(
        command = "release",
        step = %current_step,
        "Grafana storage directories created successfully"
    );

    Ok(())
}

/// Render Grafana provisioning templates (if enabled)
///
/// This step is optional and only executes if Grafana is configured in the environment.
/// If Grafana is not configured, the step is skipped without error.
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderGrafanaTemplates`) if rendering fails
#[allow(clippy::result_large_err)]
fn render_templates(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderGrafanaTemplates;

    // Check if Grafana is configured
    if environment.context().user_inputs.grafana().is_none() {
        info!(
            command = "release",
            step = %current_step,
            status = "skipped",
            "Grafana not configured - skipping provisioning template rendering"
        );
        return Ok(());
    }

    // Check if Prometheus is configured (required for datasource)
    if environment.context().user_inputs.prometheus().is_none() {
        info!(
            command = "release",
            step = %current_step,
            status = "skipped",
            "Prometheus not configured - skipping Grafana provisioning (datasource requires Prometheus)"
        );
        return Ok(());
    }

    let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
    let step = RenderGrafanaTemplatesStep::new(
        Arc::new(environment.clone()),
        template_manager,
        environment.build_dir().clone(),
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
        "Grafana provisioning templates rendered successfully"
    );

    Ok(())
}

/// Deploy Grafana provisioning configuration to the remote host (if enabled)
///
/// This step is optional and only executes if Grafana is configured in the environment.
/// If Grafana is not configured, the step is skipped without error.
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployGrafanaProvisioning`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_provisioning_to_remote(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployGrafanaProvisioning;

    // Check if Grafana is configured
    if environment.context().user_inputs.grafana().is_none() {
        info!(
            command = "release",
            step = %current_step,
            status = "skipped",
            "Grafana not configured - skipping provisioning deployment"
        );
        return Ok(());
    }

    // Check if Prometheus is configured (required for datasource)
    if environment.context().user_inputs.prometheus().is_none() {
        info!(
            command = "release",
            step = %current_step,
            status = "skipped",
            "Prometheus not configured - skipping Grafana provisioning deployment"
        );
        return Ok(());
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

    info!(
        command = "release",
        step = %current_step,
        "Grafana provisioning configuration deployed successfully"
    );

    Ok(())
}
