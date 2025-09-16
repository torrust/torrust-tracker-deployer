//! Template preparation task for E2E testing
//!
//! This module provides functionality to clean and prepare the templates directory
//! for testing, ensuring that tests use fresh embedded templates rather than any
//! potentially modified or corrupted templates from previous test runs.
//!
//! ## Key Operations
//!
//! - Reset templates directory to clean state
//! - Extract fresh embedded templates
//! - Ensure test isolation by removing previous template modifications
//!
//! This task is typically executed early in the E2E test workflow to guarantee
//! consistent starting conditions for all tests.

use anyhow::Result;
use tracing::info;

use crate::container::Services;

/// Clean and prepare templates directory to ensure fresh embedded templates
///
/// # Errors
///
/// Returns an error if the template manager fails to reset the templates directory
pub fn clean_and_prepare_templates(services: &Services) -> Result<()> {
    // Clean templates directory to ensure we use fresh templates from embedded resources
    info!(
        operation = "clean_templates",
        "Cleaning templates directory to ensure fresh embedded templates"
    );

    services
        .template_manager
        .reset_templates_dir()
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}
