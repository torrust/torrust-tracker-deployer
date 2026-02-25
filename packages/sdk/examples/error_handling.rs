//! Error handling example — matching on `SdkError` variants.
//!
//! Demonstrates how to handle errors returned by the SDK rather than using `?`
//! to propagate everything. Covers the most common patterns:
//!
//! 1. **Idempotent create** — skip creation if the environment already exists
//! 2. **Inspect individual error types** — extract context from typed variants
//! 3. **Unified `SdkError`** — wrapping multiple operations under one error type
//!
//! This example only uses operations that work locally (no infrastructure
//! required). No VMs are created.
//!
//! # Running
//!
//! ```bash
//! cargo run --example sdk_error_handling
//! ```

use std::path::PathBuf;

use torrust_tracker_deployer_sdk::CreateCommandHandlerError;
use torrust_tracker_deployer_sdk::{
    Deployer, EnvironmentCreationConfig, EnvironmentName, SdkError,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    println!("=== Torrust Tracker Deployer SDK — Error Handling Example ===\n");

    let deployer = Deployer::builder().working_dir(&workspace).build()?;

    let private_key = workspace.join("fixtures/testing_rsa");
    let public_key = workspace.join("fixtures/testing_rsa.pub");

    // -----------------------------------------------------------------------
    // Demo 1: Idempotent create — create the environment, then try again.
    // The second call should fail with EnvironmentAlreadyExists, which we
    // handle gracefully instead of crashing.
    // -----------------------------------------------------------------------
    println!("--- Demo 1: Idempotent create ---");

    let config = EnvironmentCreationConfig::builder()
        .name("sdk-error-demo")
        .ssh_keys(private_key.to_string_lossy(), public_key.to_string_lossy())
        .provider_lxd("torrust-sdk-error-demo")
        .sqlite("tracker.db")
        .udp("0.0.0.0:6969")
        .http("0.0.0.0:7070")
        .api("0.0.0.0:1212", "MyAccessToken")
        .build()?;

    let env_name = ensure_created(&deployer, config.clone())?;
    println!("  Environment ready: {env_name}");

    // Try creating the same environment again — should be handled, not panic.
    let env_name2 = ensure_created(&deployer, config)?;
    println!("  Second call returned (idempotent): {env_name2}\n");

    // -----------------------------------------------------------------------
    // Demo 2: Inspect a typed error — show variant context.
    // -----------------------------------------------------------------------
    println!("--- Demo 2: Inspect typed error context ---");

    let bad_config = EnvironmentCreationConfig::builder()
        .name("sdk-error-demo") // already exists
        .ssh_keys(private_key.to_string_lossy(), public_key.to_string_lossy())
        .provider_lxd("torrust-sdk-error-demo")
        .sqlite("tracker.db")
        .udp("0.0.0.0:6969")
        .http("0.0.0.0:7070")
        .api("0.0.0.0:1212", "MyAccessToken")
        .build()?;

    match deployer.create_environment(bad_config) {
        Ok(name) => println!("  Unexpected success: {name}"),
        Err(CreateCommandHandlerError::EnvironmentAlreadyExists { name }) => {
            println!("  Caught EnvironmentAlreadyExists: name = '{name}'");
            println!("  (This is expected — we intentionally triggered it.)");
        }
        Err(e) => println!("  Unexpected error: {e}"),
    }
    println!();

    // -----------------------------------------------------------------------
    // Demo 3: Unified SdkError — wrap multiple operations under one type.
    // -----------------------------------------------------------------------
    println!("--- Demo 3: Unified SdkError ---");

    let name = EnvironmentName::new("sdk-error-demo")?;
    match run_workflow(&deployer, &name) {
        Ok(()) => println!("  Workflow completed."),
        Err(SdkError::Show(e)) => println!("  Show failed: {e}"),
        Err(SdkError::List(e)) => println!("  List failed: {e}"),
        Err(e) => println!("  Other SDK error: {e}"),
    }
    println!();

    // -----------------------------------------------------------------------
    // Cleanup
    // -----------------------------------------------------------------------
    println!("--- Cleanup ---");
    deployer.purge(&name)?;
    println!("  Environment '{name}' purged.");

    println!("\n=== Example complete ===");
    Ok(())
}

/// Create an environment only if it does not already exist.
///
/// Returns the environment name on success (whether newly created or
/// pre-existing).
fn ensure_created(
    deployer: &Deployer,
    config: EnvironmentCreationConfig,
) -> Result<EnvironmentName, Box<dyn std::error::Error>> {
    match deployer.create_environment(config) {
        Ok(name) => {
            println!("  [create] Created new environment: {name}");
            Ok(name)
        }
        Err(CreateCommandHandlerError::EnvironmentAlreadyExists { name }) => {
            println!("  [create] Already exists — skipping: {name}");
            Ok(EnvironmentName::new(&name)?)
        }
        Err(e) => Err(Box::new(e)),
    }
}

/// A workflow function that uses `SdkError` as a single return type.
#[allow(clippy::result_large_err)]
fn run_workflow(deployer: &Deployer, name: &EnvironmentName) -> Result<(), SdkError> {
    let info = deployer.show(name)?;
    println!("  State: {}", info.state);

    let list = deployer.list()?;
    println!("  Total environments: {}", list.total_count);

    Ok(())
}
