//! Docker Compose release steps
//!
//! This module contains all steps required to deploy Docker Compose:
//! - Template rendering
//! - Compose files deployment to remote

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::info;

use crate::adapters::ansible::AnsibleClient;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::{DeployComposeFilesStep, RenderDockerComposeTemplatesStep};
use crate::application::traits::CommandProgressListener;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::shared::clock::SystemClock;

/// Release Docker Compose configuration
///
/// Executes all steps required to deploy Docker Compose:
/// 1. Render Docker Compose templates
/// 2. Deploy compose files to remote
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, step) if any Docker Compose step fails
pub async fn release(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let compose_build_dir = render_templates(environment, listener).await?;
    deploy_files_to_remote(environment, &compose_build_dir, listener)?;
    Ok(())
}

/// Render Docker Compose templates to the build directory
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderDockerComposeTemplates`) if rendering fails
async fn render_templates(
    environment: &Environment<Releasing>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<PathBuf, ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderDockerComposeTemplates;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Template source: {}/docker-compose/",
            environment.templates_dir().display()
        ));
    }

    let clock = Arc::new(SystemClock);
    let step = RenderDockerComposeTemplatesStep::new(
        Arc::new(environment.clone()),
        environment.templates_dir(),
        environment.build_dir().clone(),
        clock,
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

    if let Some(l) = listener {
        l.on_detail("Rendering docker-compose.yml and .env from templates");
        l.on_debug(&format!("Template output: {}", compose_build_dir.display()));
    }

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
/// * `listener` - Optional progress listener for detail and debug reporting
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployComposeFilesToRemote`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_files_to_remote(
    environment: &Environment<Releasing>,
    compose_build_dir: &Path,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployComposeFilesToRemote;

    if let Some(l) = listener {
        l.on_debug(&format!(
            "Ansible working directory: {}",
            environment.ansible_build_dir().display()
        ));
        l.on_debug("Executing playbook: ansible-playbook deploy-compose-files.yml");
    }

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

    if let Some(l) = listener {
        l.on_detail("Deploying docker-compose.yml and .env to /opt/torrust");
    }

    info!(
        command = "release",
        compose_build_dir = %compose_build_dir.display(),
        instance_ip = ?environment.instance_ip(),
        "Compose files deployed to remote host successfully"
    );

    Ok(())
}
