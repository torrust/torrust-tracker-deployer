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
use crate::application::steps::system::InstallBackupCrontabStep;
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};

/// Release backup configuration to the remote host
///
/// This function orchestrates the complete backup release workflow:
/// 1. Renders backup configuration templates to build directory
/// 2. Creates backup storage directories on remote host
/// 3. Deploys configuration files to remote host via Ansible
///
/// The function returns early if backup is not configured in the environment.
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
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
    listener: Option<&dyn CommandProgressListener>,
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

    render_templates(environment, listener).await?;
    create_storage(environment, listener)?;
    deploy_config_to_remote(environment, listener)?;
    install_crontab(environment, listener)?;

    Ok(())
}

/// Render backup configuration templates to the build directory
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderBackupTemplates`) if rendering fails
#[allow(clippy::result_large_err)]
async fn render_templates(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderBackupTemplates;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Template source: {}/backup/",
            environment.templates_dir().display()
        ));
    }

    let step = RenderBackupTemplatesStep::new(
        Arc::new(environment.clone()),
        environment.templates_dir(),
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

    if let Some(l) = listener {
        l.on_detail("Rendering backup scripts and configuration from templates");
    }

    info!(
        command = "release",
        step = %current_step,
        "Backup configuration templates rendered successfully"
    );

    Ok(())
}

/// Create backup storage directories on the remote host
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::CreateBackupStorage`) if storage creation fails
#[allow(clippy::result_large_err)]
fn create_storage(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::CreateBackupStorage;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Ansible working directory: {}",
            environment.ansible_build_dir().display()
        ));
        l.on_debug("Executing playbook: ansible-playbook create-backup-storage.yml");
    }

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

    if let Some(l) = listener {
        l.on_detail("Creating storage directories: /opt/torrust/backup/{scripts,data,logs}");
    }

    info!(
        command = "release",
        step = %current_step,
        "Backup storage directories created successfully"
    );

    Ok(())
}

/// Deploy backup configuration files to the remote host via Ansible
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployBackupConfigToRemote`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_config_to_remote(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployBackupConfigToRemote;

    if let Some(l) = listener {
        l.on_debug("Executing playbook: ansible-playbook deploy-backup-config.yml");
    }

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

    if let Some(l) = listener {
        l.on_detail("Deploying backup scripts to /opt/torrust/backup/scripts");
    }

    info!(
        command = "release",
        step = %current_step,
        "Backup configuration deployed successfully"
    );

    Ok(())
}

/// Install backup crontab and maintenance script on the remote host
///
/// This installs the cron job that will execute backups on the configured schedule.
/// The cron daemon is always running, so the job will automatically execute on schedule.
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::InstallBackupCrontab`) if installation fails
#[allow(clippy::result_large_err)]
fn install_crontab(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::InstallBackupCrontab;

    if let Some(l) = listener {
        l.on_debug("Executing playbook: ansible-playbook install-backup-crontab.yml");
    }

    InstallBackupCrontabStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::InstallBackupCrontabFailed {
                    message: e.to_string(),
                    source: Box::new(e),
                    step: current_step,
                },
                current_step,
            )
        })?;

    if let Some(l) = listener {
        l.on_detail("Installing crontab for automated backups");
    }

    info!(
        command = "release",
        step = %current_step,
        "Backup crontab and maintenance script installed successfully"
    );

    Ok(())
}
