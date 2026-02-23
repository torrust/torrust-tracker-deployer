//! Basic SDK usage example.
//!
//! Demonstrates how to use the Torrust Tracker Deployer SDK to:
//! 1. Create a deployment environment using the typed builder
//! 2. List all environments in the workspace
//! 3. Show environment details
//! 4. Purge the environment (clean up)
//!
//! This example only uses operations that work locally (no infrastructure
//! required). It builds an environment config with the builder, creates the
//! local environment data, inspects it, and cleans up.
//!
//! # Running
//!
//! ```bash
//! cargo run --example sdk_basic_usage
//! ```
//!
//! The example name stays `sdk_basic_usage` (declared in `Cargo.toml`)
//! even though the file lives at `examples/sdk/basic_usage.rs`.

use std::path::PathBuf;

use torrust_tracker_deployer_lib::presentation::sdk::{Deployer, EnvironmentCreationConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 1. Initialize the deployer SDK
    println!("=== Torrust Tracker Deployer SDK — Basic Example ===\n");

    let deployer = Deployer::builder().working_dir(&workspace).build()?;
    println!(
        "[OK] Deployer initialized (workspace: {})\n",
        workspace.display()
    );

    // 2. Build the environment config with the typed builder — no JSON strings
    println!("--- Step 1: Create environment ---");
    let private_key = workspace.join("fixtures/testing_rsa");
    let public_key = workspace.join("fixtures/testing_rsa.pub");

    let config = EnvironmentCreationConfig::builder()
        .name("sdk-example")
        .ssh_keys(private_key.to_string_lossy(), public_key.to_string_lossy())
        .provider_lxd("torrust-sdk-example")
        .sqlite("tracker.db")
        .udp("0.0.0.0:6969")
        .http("0.0.0.0:7070")
        .api("0.0.0.0:1212", "MyAccessToken")
        .build()?;

    let environment = deployer.create_environment(config)?;
    println!("  Created: {}\n", environment.name());

    // 3. List all environments
    println!("--- Step 2: List environments ---");
    let env_list = deployer.list()?;
    println!("  Total: {} environment(s)", env_list.total_count);
    for summary in &env_list.environments {
        println!(
            "  - {} (state: {}, provider: {})",
            summary.name, summary.state, summary.provider
        );
    }
    println!();

    // 4. Show environment details — reuse name from the returned environment
    println!("--- Step 3: Show environment details ---");
    let env_name = environment.name();
    let info = deployer.show(env_name)?;
    println!("  Name:       {}", info.name);
    println!("  State:      {}", info.state);
    println!("  Provider:   {}", info.provider);
    println!("  Created at: {}", info.created_at);
    println!();

    // 5. Clean up — purge the environment (reuse same name)
    println!("--- Step 4: Purge environment ---");
    deployer.purge(env_name)?;
    println!("  Environment '{env_name}' purged.");
    println!();

    println!("=== Example complete ===");
    Ok(())
}
