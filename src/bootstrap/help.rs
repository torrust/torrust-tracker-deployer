//! Help and Usage Information Module
//!
//! This module provides help and usage information for the Torrust Tracker Deployer
//! application. It contains functions to display helpful information to users
//! when they need guidance on how to use the application.
//!
//! ## Design Principles
//!
//! - **Independent**: No dependencies on presentation layer or CLI structures
//! - **User-Focused**: Clear, actionable guidance for users
//! - **Comprehensive**: Covers getting started, examples, and next steps

/// Display helpful information to the user when no command is provided
///
/// This function shows getting started information, usage examples, and
/// helpful links when the user runs the application without any subcommands.
/// It provides a friendly introduction to the application and guides users
/// toward productive next steps.
///
/// # Output
///
/// Prints directly to stdout with formatted, user-friendly content including:
/// - Application overview and purpose
/// - Getting started instructions
/// - Testing guidance
/// - Documentation references
/// - Next steps for users
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::bootstrap::help;
///
/// // Display help when user runs app without arguments
/// help::display_getting_started();
/// ```
pub fn display_getting_started() {
    println!("🏗️  Torrust Tracker Deployer");
    println!("===========================");
    println!();
    println!("Automated deployment tool for the Torrust Tracker application.");
    println!("Manage complete deployment lifecycles from environment creation to verification.");
    println!();
    println!("📋 Quick Start:");
    println!("   1. Check dependencies:");
    println!(
        "      cargo run --package torrust-dependency-installer --bin dependency-installer check"
    );
    println!();
    println!("   2. Create and deploy an environment:");
    println!("      torrust-tracker-deployer create template my-env.json");
    println!("      # Edit my-env.json with your SSH keys");
    println!("      torrust-tracker-deployer create environment --env-file my-env.json");
    println!("      torrust-tracker-deployer provision my-environment");
    println!("      torrust-tracker-deployer configure my-environment");
    println!("      torrust-tracker-deployer test my-environment");
    println!();
    println!("   3. Clean up when done:");
    println!("      torrust-tracker-deployer destroy my-environment");
    println!();
    println!("📖 Documentation:");
    println!("   - Quick Start Guide: docs/user-guide/quick-start.md");
    println!("   - Command Reference: docs/user-guide/commands/README.md");
    println!("   - Main README: README.md");
    println!();
    println!("💡 To see all available commands: torrust-tracker-deployer --help");
}

/// Display troubleshooting information for common issues
///
/// This function provides guidance for common problems users might encounter
/// when setting up or using the Torrust Tracker Deployer.
///
/// # Output
///
/// Prints troubleshooting guidance to stdout including:
/// - Common setup issues and solutions
/// - Dependency verification steps
/// - Configuration validation tips
/// - Where to get additional help
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::bootstrap::help;
///
/// // Display troubleshooting info when user encounters issues
/// help::display_troubleshooting();
/// ```
pub fn display_troubleshooting() {
    println!("🔧 Troubleshooting Guide");
    println!("=======================");
    println!();
    println!("Common issues and solutions:");
    println!();
    println!("1. Dependencies not found:");
    println!("   - Ensure OpenTofu is installed and in PATH");
    println!("   - Verify Ansible is installed and accessible");
    println!("   - Check that your infrastructure provider is properly configured");
    println!();
    println!("2. Permission errors:");
    println!("   - Ensure you have the necessary permissions for your provider");
    println!("   - For LXD: Add your user to the lxd group");
    println!("   - For Hetzner: Verify your API token has correct permissions");
    println!();
    println!("3. Network connectivity issues:");
    println!("   - Check internet connectivity for image downloads");
    println!("   - Verify your provider is accessible");
    println!("   - Test provider-specific connectivity");
    println!();
    println!("4. Configuration problems:");
    println!("   - Validate YAML/JSON syntax in configuration files");
    println!("   - Check that environment names follow naming conventions");
    println!("   - Ensure SSH keys are properly configured");
    println!();
    println!("📖 For more help:");
    println!("   - Check the docs/ directory for detailed guides");
    println!("   - Review the README.md for setup instructions");
    println!("   - Open an issue on GitHub for additional support");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_display_getting_started_without_panicking() {
        // Test that the function runs without panicking
        // We can't easily test stdout content in unit tests,
        // but we can ensure the function doesn't crash
        display_getting_started();
    }

    #[test]
    fn it_should_display_troubleshooting_without_panicking() {
        // Test that the function runs without panicking
        display_troubleshooting();
    }
}
