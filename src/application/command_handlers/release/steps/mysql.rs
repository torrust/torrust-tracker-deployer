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
/// If `MySQL` is not configured as the tracker database, all steps are skipped.
///
/// # Errors
///
/// Returns a tuple of (error, step) if `MySQL` storage creation fails
#[allow(clippy::result_large_err)]
pub fn release(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    // Check if MySQL is configured (via tracker database driver)
    if !environment.context().user_inputs.tracker().uses_mysql() {
        info!(
            command = "release",
            service = "mysql",
            status = "skipped",
            "MySQL not configured - skipping all MySQL steps"
        );
        return Ok(());
    }

    create_storage(environment)?;
    Ok(())
}

/// Create `MySQL` storage directories on the remote host
///
/// # Errors
///
/// Returns a tuple of (error, `ReleaseStep::CreateMysqlStorage`) if creation fails
#[allow(clippy::result_large_err)]
fn create_storage(
    environment: &Environment<Releasing>,
) -> StepResult<(), ReleaseCommandHandlerError, ReleaseStep> {
    let current_step = ReleaseStep::CreateMysqlStorage;

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
