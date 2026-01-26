//! Docker Compose release steps
//!
//! This module contains all steps required to deploy Docker Compose:
//! - Template rendering
//! - Compose files deployment to remote

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::info;

use crate::adapters::ansible::AnsibleClient;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::{DeployComposeFilesStep, RenderDockerComposeTemplatesStep};
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::domain::template::TemplateManager;

/// Release Docker Compose configuration
///
/// Executes all steps required to deploy Docker Compose:
/// 1. Render Docker Compose templates
/// 2. Deploy compose files to remote
///
/// # Errors
///
/// Returns a tuple of (error, step) if any Docker Compose step fails
pub async fn release(
    environment: &Environment<Releasing>,
    instance_ip: IpAddr,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let compose_build_dir = render_templates(environment).await?;
    deploy_files_to_remote(environment, &compose_build_dir, instance_ip)?;
    Ok(())
}

/// Render Docker Compose templates to the build directory
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderDockerComposeTemplates`) if rendering fails
async fn render_templates(
    environment: &Environment<Releasing>,
) -> StepResult<PathBuf, ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderDockerComposeTemplates;

    let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
    let step = RenderDockerComposeTemplatesStep::new(
        Arc::new(environment.clone()),
        template_manager,
        environment.build_dir().clone(),
    );

    let compose_build_dir = step.execute().await.map_err(|e| {
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
        compose_build_dir = %compose_build_dir.display(),
        "Docker Compose templates rendered successfully"
    );

    Ok(compose_build_dir)
}

/// Deploy compose files to the remote host via Ansible
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `compose_build_dir` - Path to the rendered compose files
/// * `instance_ip` - The target instance IP address
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployComposeFilesToRemote`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_files_to_remote(
    environment: &Environment<Releasing>,
    compose_build_dir: &Path,
    instance_ip: IpAddr,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployComposeFilesToRemote;

    let ansible_client = Arc::new(AnsibleClient::new(environment.ansible_build_dir()));
    let step = DeployComposeFilesStep::new(ansible_client, compose_build_dir.to_path_buf());

    step.execute().map_err(|e| {
        (
            ReleaseCommandHandlerError::ComposeFilesDeployment {
                message: e.to_string(),
                source: Box::new(e),
            },
            current_step,
        )
    })?;

    info!(
        command = "release",
        compose_build_dir = %compose_build_dir.display(),
        instance_ip = %instance_ip,
        "Compose files deployed to remote host successfully"
    );

    Ok(())
}
