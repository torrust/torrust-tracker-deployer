//! Backup release step module
//!
//! This module handles the release workflow for backup configuration files.
//! It orchestrates the rendering and deployment of backup configuration
//! when backup is enabled in the environment.

use std::sync::Arc;

use tracing::info;

use super::common::ansible_client;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::application::{CreateBackupStorageStep, DeployBackupConfigStep};
use crate::application::steps::rendering::RenderBackupTemplatesStep;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::domain::template::TemplateManager;

/// Release backup configuration to the remote host
///
/// This function orchestrates the complete backup release workflow:
/// 1. Renders backup configuration templates to build directory
/// 2. Creates backup storage directories on remote host
/// 3. Deploys configuration files to remote host via Ansible
///
/// The function returns early if backup is not configured in the environment.
///
/// # Errors
///
/// Returns `ReleaseCommandHandlerError` if:
/// - Template rendering fails
/// - Storage creation fails
/// - Configuration deployment fails
#[allow(clippy::result_large_err)]
pub async fn release(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    // Check if backup is configured
    if environment.context().user_inputs.backup().is_none() {
        info!(
            command = "release",
            service = "backup",
            status = "skipped",
            "Backup not configured - skipping all backup steps"
        );
        return Ok(());
    }

    render_templates(environment).await?;
    create_storage(environment)?;
    deploy_config_to_remote(environment)?;

    Ok(())
}

/// Render backup configuration templates to the build directory
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderBackupTemplates`) if rendering fails
#[allow(clippy::result_large_err)]
async fn render_templates(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderBackupTemplates;

    let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
    let step = RenderBackupTemplatesStep::new(
        Arc::new(environment.clone()),
        template_manager,
        environment.build_dir().clone(),
    );

    step.execute().await.map_err(|e| {
        (
            ReleaseCommandHandlerError::RenderBackupTemplatesFailed {
                message: e.to_string(),
                source: Box::new(e),
                step: current_step,
            },
            current_step,
        )
    })?;

    info!(
        command = "release",
        step = %current_step,
        "Backup configuration templates rendered successfully"
    );

    Ok(())
}

/// Create backup storage directories on the remote host
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::CreateBackupStorage`) if storage creation fails
#[allow(clippy::result_large_err)]
fn create_storage(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::CreateBackupStorage;

    CreateBackupStorageStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::CreateBackupStorageFailed {
                    message: e.to_string(),
                    source: Box::new(e),
                    step: current_step,
                },
                current_step,
            )
        })?;

    info!(
        command = "release",
        step = %current_step,
        "Backup storage directories created successfully"
    );

    Ok(())
}

/// Deploy backup configuration files to the remote host via Ansible
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployBackupConfigToRemote`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_config_to_remote(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployBackupConfigToRemote;

    DeployBackupConfigStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::DeployBackupConfigFailed {
                    message: e.to_string(),
                    source: Box::new(e),
                    step: current_step,
                },
                current_step,
            )
        })?;

    info!(
        command = "release",
        step = %current_step,
        "Backup configuration deployed successfully"
    );

    Ok(())
}
