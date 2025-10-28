//! Template Generation Subcommand
//!
//! This module handles the template generation subcommand for creating
//! configuration file templates with placeholder values.

use std::path::Path;

use crate::domain::config::EnvironmentCreationConfig;
use crate::presentation::user_output::{UserOutput, VerbosityLevel};

use super::super::errors::CreateSubcommandError;

/// Handle template generation
///
/// This function generates a configuration template file with placeholder values
/// that users can edit to create their own environment configurations.
///
/// # Arguments
///
/// * `output_path` - Path where the template file should be created
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateSubcommandError` on failure.
///
/// # Errors
///
/// Returns an error if template file creation fails.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_template_generation(output_path: &Path) -> Result<(), CreateSubcommandError> {
    // Create user output for progress messages
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    output.progress("Generating configuration template...");

    // Call existing domain method - template generation implemented in PR #48
    // This is async, so we need to use tokio runtime
    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            EnvironmentCreationConfig::generate_template_file(output_path)
                .await
                .map_err(CreateSubcommandError::TemplateGenerationFailed)
        })?;

    output.success(&format!(
        "Configuration template generated: {}",
        output_path.display()
    ));
    println!();
    println!("Next steps:");
    println!("1. Edit the template file and replace placeholder values:");
    println!("   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')");
    println!("   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key");
    println!("   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key");
    println!("2. Review default values:");
    println!("   - username: 'torrust' (can be changed if needed)");
    println!("   - port: 22 (standard SSH port)");
    println!("3. Create the environment:");
    println!(
        "   torrust-tracker-deployer create environment --env-file {}",
        output_path.display()
    );

    Ok(())
}
