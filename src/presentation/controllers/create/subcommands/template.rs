//! Template Generation Subcommand
//!
//! This module handles the template generation subcommand for creating
//! configuration file templates with placeholder values.

use std::path::Path;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::presentation::dispatch::ExecutionContext;

use super::environment::CreateSubcommandError;

/// Handle template generation
///
/// This function generates a configuration template file with placeholder values
/// that users can edit to create their own environment configurations.
///
/// # Arguments
///
/// * `output_path` - Path where the template file should be created
/// * `context` - Execution context providing access to application services
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateSubcommandError` on failure.
///
/// # Errors
///
/// Returns an error if template file creation fails.
///
/// # Panics
///
/// Panics if the tokio runtime cannot be created. This is a critical system
/// failure that prevents any async operations from running.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_template_generation(
    output_path: &Path,
    context: &ExecutionContext,
) -> Result<(), CreateSubcommandError> {
    // Lock user output for progress messages
    let user_output = context.user_output();
    let mut output = user_output
        .lock()
        .map_err(|_| CreateSubcommandError::UserOutputLockFailed)?;

    output.progress("Generating configuration template...");

    // Call existing domain method - template generation implemented in PR #48
    // This is async, so we need to use tokio runtime
    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            EnvironmentCreationConfig::generate_template_file(output_path)
                .await
                .map_err(|source| CreateSubcommandError::TemplateGenerationFailed { source })
        })?;

    output.success(&format!(
        "Configuration template generated: {}",
        output_path.display()
    ));

    output.blank_line();

    output.steps(
        "Next steps:",
        &[
            "Edit the template file and replace placeholder values:\n   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')\n   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key\n   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key",
            "Review default values:\n   - username: 'torrust' (can be changed if needed)\n   - port: 22 (standard SSH port)",
            &format!(
                "Create the environment:\n   torrust-tracker-deployer create environment --env-file {}",
                output_path.display()
            ),
        ],
    );

    Ok(())
}
