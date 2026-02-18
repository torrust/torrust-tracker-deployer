//! LXD Emergency Cleanup Tool for Torrust Tracker Deployer
//!
//! This binary provides an emergency cleanup tool for LXD deployment environments
//! when the normal cleanup process cannot be used. It removes all associated
//! infrastructure and artifacts, reusing the same cleanup logic as the E2E test suite.
//!
//! ## ⚠️ IMPORTANT: Normal Environment Removal Process
//!
//! **For normal operations, use the standard commands:**
//!
//! 1. **`destroy`** - Destroys the infrastructure (VM, profiles, etc.)
//! 2. **`purge`** - Removes the internal deployer data about the environment
//!
//! Example:
//! ```bash
//! cargo run -- destroy my-environment
//! cargo run -- purge my-environment
//! ```
//!
//! ## When to Use This Emergency Tool
//!
//! **ONLY use this tool when:**
//! - The environment's internal data is **missing** (manually deleted)
//! - The environment's internal data is **corrupted** and cannot be read
//! - E2E tests were interrupted and left orphaned LXD resources
//! - Emergency recovery is needed for LXD environments
//!
//! ## Usage
//!
//! Clean up a single LXD environment:
//!
//! ```bash
//! cargo run --bin lxd-cleanup -- cleanup-test
//! ```
//!
//! Clean up multiple LXD environments:
//!
//! ```bash
//! cargo run --bin lxd-cleanup -- env1 env2 env3
//! ```
//!
//! ## What Gets Cleaned
//!
//! For each LXD environment, this tool removes:
//! 1. **Build directory** - `./build/{environment_name}/`
//! 2. **Templates directory** - `./templates/{environment_name}/`
//! 3. **Data directory** - `./data/{environment_name}/`
//! 4. **`OpenTofu` infrastructure** - Runs `tofu destroy` if state exists
//! 5. **LXD resources** - Deletes VM instance and profile
//!
//! ## Safety
//!
//! This tool is destructive and will permanently delete:
//! - All infrastructure provisioned for the environment
//! - All generated configuration files
//! - All environment state data
//!
//! There is no confirmation prompt, so use with caution.

use anyhow::Result;
use clap::Parser;
use tracing::{error, info, warn};

use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput, LoggingBuilder};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::run_preflight_cleanup;

#[derive(Parser)]
#[command(name = "lxd-cleanup")]
#[command(about = "Emergency cleanup tool for LXD deployment environments")]
#[command(
    long_about = "⚠️  EMERGENCY TOOL - Use 'destroy' + 'purge' commands for normal operations.\n\n\
                        This tool is ONLY for emergency situations when environment data is missing or corrupted.\n\n\
                        Removes all infrastructure and artifacts for specified LXD environments.\n\
                        This includes build/templates/data directories, OpenTofu state, and LXD resources."
)]
struct Cli {
    /// LXD environment names to clean up
    #[arg(
        required = true,
        help = "One or more LXD environment names to clean up (e.g., cleanup-test, manual-test-mysql)"
    )]
    environments: Vec<String>,

    /// Logging format to use
    #[arg(
        long,
        default_value = "pretty",
        help = "Logging format: pretty, json, or compact"
    )]
    log_format: LogFormat,

    /// Show what would be cleaned without actually cleaning
    #[arg(
        long,
        help = "Dry run mode - show what would be cleaned without making changes"
    )]
    dry_run: bool,
}

/// Main entry point for the LXD emergency cleanup tool
///
/// ⚠️ WARNING: This is an emergency tool. For normal operations, use the
/// `destroy` and `purge` commands instead.
///
/// This function orchestrates the emergency cleanup process for one or more LXD environments:
/// 1. Initializes logging
/// 2. Validates environment names
/// 3. Executes cleanup for each environment using E2E preflight cleanup logic
/// 4. Reports results
///
/// Returns `Ok(())` if all cleanups succeed, `Err` otherwise.
///
/// # Errors
///
/// This function may return errors in the following cases:
/// - Invalid environment names provided
/// - Cleanup fails for any environment
/// - Permission issues accessing resources
#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging (use temp log dir since we're cleaning environments)
    LoggingBuilder::new(std::path::Path::new("./data/logs/manual-cleanup"))
        .with_format(cli.log_format.clone())
        .with_output(LogOutput::FileAndStderr)
        .init();

    info!(
        application = "torrust_tracker_deployer",
        tool = "lxd_cleanup",
        log_format = ?cli.log_format,
        dry_run = cli.dry_run,
        environment_count = cli.environments.len(),
        "Starting LXD emergency cleanup"
    );

    if cli.dry_run {
        warn!("🔍 DRY RUN MODE - No changes will be made");
        warn!("Remove --dry-run flag to perform actual cleanup");
    }

    let total_environments = cli.environments.len();
    let mut successful_cleanups = 0;
    let mut failed_cleanups = Vec::new();

    print_header(total_environments, cli.dry_run);

    for (index, environment_name) in cli.environments.iter().enumerate() {
        let env_num = index + 1;
        println!("[{env_num}/{total_environments}] Cleaning environment: {environment_name}");

        info!(
            operation = "environment_cleanup",
            environment = environment_name,
            progress = format!("{}/{}", env_num, total_environments),
            "Starting cleanup for environment"
        );

        if cli.dry_run {
            print_dry_run_info(environment_name);
            successful_cleanups += 1;
            continue;
        }

        // Perform actual cleanup using E2E preflight cleanup logic
        match run_preflight_cleanup(environment_name) {
            Ok(()) => {
                println!("  ✅ Successfully cleaned: {environment_name}");
                info!(
                    operation = "environment_cleanup",
                    environment = environment_name,
                    status = "success",
                    "Environment cleanup completed successfully"
                );
                successful_cleanups += 1;
            }
            Err(e) => {
                println!("  ❌ Failed to clean: {environment_name}");
                println!("     Error: {e}");
                error!(
                    operation = "environment_cleanup",
                    environment = environment_name,
                    status = "failed",
                    error = %e,
                    "Environment cleanup failed"
                );
                failed_cleanups.push((environment_name.clone(), e));
            }
        }
        println!();
    }

    print_summary(total_environments, successful_cleanups, &failed_cleanups);

    if !failed_cleanups.is_empty() {
        error!(
            operation = "lxd_cleanup",
            status = "partial_failure",
            successful = successful_cleanups,
            failed = failed_cleanups.len(),
            "LXD cleanup completed with failures"
        );

        return Err(anyhow::anyhow!(
            "Cleanup failed for {} environment(s): {}",
            failed_cleanups.len(),
            failed_cleanups
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    info!(
        operation = "lxd_cleanup",
        status = "success",
        cleaned_environments = successful_cleanups,
        "LXD cleanup completed successfully"
    );

    print_completion_message(cli.dry_run);

    Ok(())
}

/// Prints the tool header and initial information
fn print_header(total_environments: usize, dry_run: bool) {
    println!("\n========================================");
    println!("🧹 LXD Emergency Cleanup Tool");
    println!("========================================");
    println!("⚠️  For normal operations, use: 'destroy' + 'purge'");
    println!("LXD environments to clean: {total_environments}");
    if dry_run {
        println!("Mode: DRY RUN (no changes will be made)");
    }
    println!();
}

/// Prints what would be cleaned in dry run mode
fn print_dry_run_info(environment_name: &str) {
    println!("  ℹ️  Would clean:");
    println!("     - Build directory: ./build/{environment_name}");
    println!("     - Templates directory: ./templates/{environment_name}");
    println!("     - Data directory: ./data/{environment_name}");
    println!("     - OpenTofu state (if exists)");
    println!("     - LXD instance: torrust-tracker-vm-{environment_name}");
    println!("     - LXD profile: torrust-profile-{environment_name}");
    println!("  ✅ Dry run completed for: {environment_name}");
}

/// Prints the cleanup summary
fn print_summary(
    total_environments: usize,
    successful_cleanups: usize,
    failed_cleanups: &[(String, anyhow::Error)],
) {
    println!("========================================");
    println!("📊 Cleanup Summary");
    println!("========================================");
    println!("Total environments: {total_environments}");
    println!("✅ Successful: {successful_cleanups}");
    println!("❌ Failed: {}", failed_cleanups.len());
    println!();

    if !failed_cleanups.is_empty() {
        println!("Failed environments:");
        for (env_name, error) in failed_cleanups {
            println!("  - {env_name}: {error}");
        }
        println!();
    }
}

/// Prints the completion message
fn print_completion_message(dry_run: bool) {
    if dry_run {
        println!("✅ Dry run completed successfully");
        println!("   Run without --dry-run to perform actual cleanup");
    } else {
        println!("✅ All environments cleaned successfully");
    }
    println!();
}
