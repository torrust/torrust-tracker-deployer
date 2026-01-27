//! Tracker service release steps
//!
//! This module contains all steps required to release the Tracker service:
//! - Storage directory creation
//! - Database initialization
//! - Configuration template rendering
//! - Configuration deployment to remote

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::info;

use super::common::ansible_client;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::application::{
    CreateTrackerStorageStep, DeployTrackerConfigStep, InitTrackerDatabaseStep,
};
use crate::application::steps::rendering::RenderTrackerTemplatesStep;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};
use crate::domain::template::TemplateManager;
use crate::shared::SystemClock;

/// Release the Tracker service
///
/// Executes all steps required to release the Tracker:
/// 1. Create storage directories
/// 2. Initialize database
/// 3. Render configuration templates
/// 4. Deploy configuration to remote
///
/// # Errors
///
/// Returns a tuple of (error, step) if any tracker step fails
#[allow(clippy::result_large_err)]
pub fn release(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    create_storage(environment)?;
    init_database(environment)?;
    let tracker_build_dir = render_templates(environment)?;
    deploy_config_to_remote(environment, &tracker_build_dir)?;
    Ok(())
}

/// Create tracker storage directories on the remote host
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::CreateTrackerStorage`) if creation fails
#[allow(clippy::result_large_err)]
fn create_storage(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::CreateTrackerStorage;

    CreateTrackerStorageStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::TrackerStorageCreation {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    info!(
        command = "release",
        step = %current_step,
        "Tracker storage directories created successfully"
    );

    Ok(())
}

/// Initialize tracker database on the remote host
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::InitTrackerDatabase`) if initialization fails
#[allow(clippy::result_large_err)]
fn init_database(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::InitTrackerDatabase;

    InitTrackerDatabaseStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::TrackerDatabaseInit {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    info!(
        command = "release",
        step = %current_step,
        "Tracker database initialized successfully"
    );

    Ok(())
}

/// Render Tracker configuration templates to the build directory
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::RenderTrackerTemplates`) if rendering fails
#[allow(clippy::result_large_err)]
fn render_templates(
    environment: &Environment<Releasing>,
) -> StepResult<PathBuf, ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::RenderTrackerTemplates;

    let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));
    let clock = Arc::new(SystemClock);
    let step = RenderTrackerTemplatesStep::new(
        Arc::new(environment.clone()),
        template_manager,
        environment.build_dir().clone(),
        clock,
    );

    let tracker_build_dir = step.execute().map_err(|e| {
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
        tracker_build_dir = %tracker_build_dir.display(),
        "Tracker configuration templates rendered successfully"
    );

    Ok(tracker_build_dir)
}

/// Deploy tracker configuration to the remote host via Ansible
///
/// # Arguments
///
/// * `environment` - The environment in Releasing state
/// * `tracker_build_dir` - Path to the rendered tracker configuration
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::DeployTrackerConfigToRemote`) if deployment fails
#[allow(clippy::result_large_err)]
fn deploy_config_to_remote(
    environment: &Environment<Releasing>,
    tracker_build_dir: &Path,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::DeployTrackerConfigToRemote;

    DeployTrackerConfigStep::new(ansible_client(environment), tracker_build_dir.to_path_buf())
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::TrackerConfigDeployment {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    info!(
        command = "release",
        step = %current_step,
        "Tracker configuration deployed successfully"
    );

    Ok(())
}
