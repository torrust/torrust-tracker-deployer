//! Full LXD deployment example using the SDK.
//!
//! Demonstrates the complete deployment lifecycle:
//!
//! 1. **Create** — Build an environment configuration using the typed builder
//! 2. **Provision** — Spin up an LXD VM instance via `OpenTofu`
//! 3. **Configure** — Install software via Ansible playbooks
//! 4. **Release** — Deploy tracker configuration files
//! 5. **Run** — Start Docker Compose services
//! 6. **Test** — Verify the deployment is healthy
//! 7. **Destroy** — Tear down the LXD VM
//! 8. **Purge** — Remove local data and build artifacts
//!
//! # Requirements
//!
//! - LXD must be installed and configured locally
//! - SSH keys in `fixtures/testing_rsa{,.pub}`
//! - `OpenTofu`, Ansible, and Docker must be available
//!
//! # Running
//!
//! ```bash
//! cargo run --example sdk_full_deployment
//! ```
//!
//! **Warning**: This example creates real LXD infrastructure. It will be
//! destroyed and purged at the end, but if the process is interrupted you
//! may need to clean up manually with `lxc delete --force <instance>`.

use std::path::PathBuf;
use std::sync::Arc;

use torrust_tracker_deployer_sdk::CommandProgressListener;
use torrust_tracker_deployer_sdk::{Deployer, EnvironmentCreationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    println!("=== Torrust Tracker Deployer SDK — Full LXD Deployment ===\n");

    // ------------------------------------------------------------------
    // 1. Initialize the SDK — attach the progress listener at build time
    // ------------------------------------------------------------------
    let deployer = Deployer::builder()
        .working_dir(&workspace)
        .progress_listener(Arc::new(PrintProgressListener))
        .build()?;
    let workspace_display = workspace.display();
    println!("[OK] Deployer initialized (workspace: {workspace_display})\n");

    let private_key = workspace.join("fixtures/testing_rsa");
    let public_key = workspace.join("fixtures/testing_rsa.pub");

    // ------------------------------------------------------------------
    // 2. Create environment
    // ------------------------------------------------------------------
    println!("--- Step 1: Create environment ---");
    let config = EnvironmentCreationConfig::builder()
        .name("sdk-full-deploy")
        .ssh_keys(private_key.to_string_lossy(), public_key.to_string_lossy())
        .provider_lxd("torrust-sdk-full-deploy")
        .sqlite("tracker.db")
        .udp("0.0.0.0:6969")
        .http("0.0.0.0:7070")
        .api("0.0.0.0:1212", "MyAccessToken")
        .health_check("127.0.0.1:1313")
        .build()?;

    let env_name = deployer.create_environment(config)?;
    println!("  Created: {env_name}\n");

    // ------------------------------------------------------------------
    // 3. Provision — create the LXD VM via OpenTofu
    // ------------------------------------------------------------------
    println!("--- Step 2: Provision infrastructure ---");
    deployer.provision(&env_name).await?;
    println!("  Provisioning complete.\n");

    // ------------------------------------------------------------------
    // 4. Configure — run Ansible playbooks
    // ------------------------------------------------------------------
    println!("--- Step 3: Configure environment ---");
    deployer.configure(&env_name)?;
    println!("  Configuration complete.\n");

    // ------------------------------------------------------------------
    // 5. Release — deploy tracker files
    // ------------------------------------------------------------------
    println!("--- Step 4: Release software ---");
    deployer.release(&env_name).await?;
    println!("  Release complete.\n");

    // ------------------------------------------------------------------
    // 6. Run — start Docker Compose services
    // ------------------------------------------------------------------
    println!("--- Step 5: Run services ---");
    deployer.run_services(&env_name)?;
    println!("  Services started.\n");

    // ------------------------------------------------------------------
    // 7. Test — verify the deployment
    // ------------------------------------------------------------------
    println!("--- Step 6: Test deployment ---");
    let test_result = deployer.test(&env_name).await?;
    let instance_ip = test_result.instance_ip;
    println!("  Instance IP: {instance_ip}");
    if test_result.dns_warnings.is_empty() {
        println!("  No DNS warnings.");
    } else {
        for w in &test_result.dns_warnings {
            println!("  DNS warning: {w:?}");
        }
    }
    println!();

    // ------------------------------------------------------------------
    // 8. Destroy — tear down the LXD VM
    // ------------------------------------------------------------------
    println!("--- Step 7: Destroy infrastructure ---");
    deployer.destroy(&env_name)?;
    println!("  Infrastructure destroyed.\n");

    // ------------------------------------------------------------------
    // 9. Purge — remove local data
    // ------------------------------------------------------------------
    println!("--- Step 8: Purge local data ---");
    deployer.purge(&env_name)?;
    println!("  Environment '{env_name}' purged.\n");

    println!("=== Full deployment lifecycle complete ===");
    Ok(())
}

/// A simple progress listener that prints steps to stdout.
///
/// This is a lightweight alternative to the CLI's `VerboseProgressListener`
/// that does not depend on any presentation-layer types.
struct PrintProgressListener;

impl CommandProgressListener for PrintProgressListener {
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str) {
        println!("  [{step_number}/{total_steps}] {description}...");
    }

    fn on_step_completed(&self, step_number: usize, description: &str) {
        println!("  [{step_number}] {description} — done");
    }

    fn on_detail(&self, message: &str) {
        println!("     → {message}");
    }

    fn on_debug(&self, message: &str) {
        println!("     [debug] {message}");
    }
}
