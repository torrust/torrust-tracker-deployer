//! Create-from-JSON-file example.
//!
//! Demonstrates how to use `create_environment_from_file()` — the SDK method
//! that mirrors the CLI's `create environment --env-file <path>` flag.
//!
//! Users migrating from the CLI will already have JSON config files in their
//! `envs/` directory. This example shows the typical workflow:
//!
//! 1. **Validate** — check the config file is valid before creating
//! 2. **Create** — load the file and create the environment in one call
//! 3. **Show** — inspect the resulting environment
//! 4. **Purge** — clean up
//!
//! The example also shows how to handle `CreateEnvironmentFromFileError` to
//! distinguish load/parse failures from creation failures.
//!
//! # Running
//!
//! ```bash
//! cargo run --example sdk_create_from_json_file
//! ```

use std::io::Write as _;
use std::path::PathBuf;

use torrust_tracker_deployer_sdk::{ConfigLoadError, CreateEnvironmentFromFileError, Deployer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    println!("=== Torrust Tracker Deployer SDK — Create from JSON File ===\n");

    let deployer = Deployer::builder().working_dir(&workspace).build()?;
    println!(
        "[OK] Deployer initialized (workspace: {})\n",
        workspace.display()
    );

    let private_key = workspace.join("fixtures/testing_rsa");
    let public_key = workspace.join("fixtures/testing_rsa.pub");

    // ------------------------------------------------------------------
    // Write an environment config JSON file to a temp location.
    //
    // In real usage you would already have this file in your envs/ dir;
    // here we generate it at runtime so the example is self-contained and
    // the SSH key paths are always correct for the current workspace.
    // ------------------------------------------------------------------
    let json = format!(
        r#"{{
  "environment": {{ "name": "sdk-from-json-file" }},
  "ssh_credentials": {{
    "private_key_path": "{private_key}",
    "public_key_path": "{public_key}"
  }},
  "provider": {{
    "provider": "lxd",
    "profile_name": "torrust-sdk-from-json-file"
  }},
  "tracker": {{
    "core": {{
      "database": {{ "driver": "sqlite3", "database_name": "tracker.db" }},
      "private": false
    }},
    "udp_trackers": [{{ "bind_address": "0.0.0.0:6969" }}],
    "http_trackers": [{{ "bind_address": "0.0.0.0:7070" }}],
    "http_api": {{ "bind_address": "0.0.0.0:1212", "admin_token": "MyAccessToken" }},
    "health_check_api": {{ "bind_address": "0.0.0.0:1313" }}
  }}
}}"#,
        private_key = private_key.display(),
        public_key = public_key.display(),
    );

    let mut tmp = tempfile::NamedTempFile::new()?;
    tmp.write_all(json.as_bytes())?;
    let config_path = tmp.path();
    println!("Config file written to: {}\n", config_path.display());

    // ------------------------------------------------------------------
    // Step 1: Validate the config file before creating
    // ------------------------------------------------------------------
    println!("--- Step 1: Validate config ---");
    let validation = deployer.validate(config_path)?;
    println!("  Config is valid.");
    println!("  Environment name : {}", validation.environment_name);
    println!("  Provider         : {}\n", validation.provider);

    // ------------------------------------------------------------------
    // Step 2: Create environment from the JSON file
    // ------------------------------------------------------------------
    println!("--- Step 2: Create from file ---");
    let env_name = deployer.create_environment_from_file(config_path)?;
    println!("  Created: {env_name}\n");

    // ------------------------------------------------------------------
    // Step 3: Show environment details
    // ------------------------------------------------------------------
    println!("--- Step 3: Show environment ---");
    let info = deployer.show(&env_name)?;
    println!("  Name:       {}", info.name);
    println!("  State:      {}", info.state);
    println!("  Provider:   {}", info.provider);
    println!("  Created at: {}\n", info.created_at);

    // ------------------------------------------------------------------
    // Step 4: Demonstrate error handling for a missing file
    // ------------------------------------------------------------------
    println!("--- Step 4: Error — file not found ---");
    let bad_path = workspace.join("envs/non-existent-config.json");
    match deployer.create_environment_from_file(&bad_path) {
        Ok(_) => println!("  Unexpected success."),
        Err(CreateEnvironmentFromFileError::Load(ConfigLoadError::FileNotFound { path })) => {
            println!("  Caught FileNotFound: {}", path.display());
            println!("  (Expected — file does not exist.)\n");
        }
        Err(e) => println!("  Unexpected error: {e}\n"),
    }

    // ------------------------------------------------------------------
    // Cleanup
    // ------------------------------------------------------------------
    println!("--- Cleanup ---");
    deployer.purge(&env_name)?;
    println!("  Environment '{env_name}' purged.");

    println!("\n=== Example complete ===");
    Ok(())
}
