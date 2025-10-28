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
    println!("=========================");
    println!();
    println!("This repository provides automated deployment infrastructure for Torrust tracker projects.");
    println!("The infrastructure includes VM provisioning with OpenTofu and configuration");
    println!("management with Ansible playbooks.");
    println!();
    println!("📋 Getting Started:");
    println!("   Please follow the instructions in the README.md file to:");
    println!("   1. Set up the required dependencies (OpenTofu, Ansible, LXD)");
    println!("   2. Provision the deployment infrastructure");
    println!("   3. Deploy and configure the services");
    println!();
    println!("🧪 Running E2E Tests:");
    println!("   Use the e2e tests binaries to run end-to-end tests:");
    println!("   cargo e2e-provision && cargo e2e-config");
    println!();
    println!("📖 For detailed instructions, see: README.md");
    println!();
    println!("💡 To see available commands, run: torrust-tracker-deployer --help");
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
    println!("   - Check that LXD is properly configured");
    println!();
    println!("2. Permission errors:");
    println!("   - Add your user to the lxd group: sudo usermod -aG lxd $USER");
    println!("   - Log out and log back in to apply group changes");
    println!("   - Verify permissions with: groups");
    println!();
    println!("3. Network connectivity issues:");
    println!("   - Check internet connectivity for image downloads");
    println!("   - Verify LXD daemon is running: lxd --version");
    println!("   - Test basic LXD functionality: lxc list");
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
