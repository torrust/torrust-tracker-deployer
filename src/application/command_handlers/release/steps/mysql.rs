//! `MySQL` service release steps
//!
//! This module contains all steps required to release the `MySQL` service:
//! - Storage directory creation
//!
//! All steps are optional and only execute if `MySQL` is configured as the tracker database.

use tracing::info;

use super::common::ansible_client;
use crate::application::command_handlers::common::StepResult;
use crate::application::command_handlers::release::errors::ReleaseCommandHandlerError;
use crate::application::steps::application::CreateMysqlStorageStep;
use crate::domain::environment::state::ReleaseStep;
use crate::domain::environment::{Environment, Releasing};

/// Release the `MySQL` service (if enabled)
///
/// Executes all steps required to release `MySQL`:
/// 1. Create `MySQL` storage directories
///
/// If `MySQL` is not configured as the tracker database, this step is skipped.
///
/// # Errors
///
/// Returns a tuple of (error, step) if `MySQL` storage creation fails
#[allow(clippy::result_large_err)]
pub fn release(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    create_storage(environment)?;
    Ok(())
}

/// Create `MySQL` storage directories on the remote host (if enabled)
///
/// This step is optional and only executes if `MySQL` is configured as the tracker database.
/// If `MySQL` is not configured, the step is skipped without error.
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::CreateMysqlStorage`) if creation fails
#[allow(clippy::result_large_err)]
fn create_storage(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::CreateMysqlStorage;

    // Check if MySQL is configured (via tracker database driver)
    if !environment.context().user_inputs.tracker().uses_mysql() {
        info!(
            command = "release",
            step = %current_step,
            status = "skipped",
            "MySQL not configured - skipping storage creation"
        );
        return Ok(());
    }

    CreateMysqlStorageStep::new(ansible_client(environment))
        .execute()
        .map_err(|e| {
            (
                ReleaseCommandHandlerError::MysqlStorageCreation {
                    message: e.to_string(),
                    source: Box::new(e),
                },
                current_step,
            )
        })?;

    info!(
        command = "release",
        step = %current_step,
        "MySQL storage directories created successfully"
    );

    Ok(())
}
