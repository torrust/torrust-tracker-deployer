//! Validate config example.
//!
//! Demonstrates how to use `validate()` as a standalone pre-flight check —
//! useful for CI pipelines, shell scripts, or agent workflows that need to
//! verify a config file is correct before committing to a full deployment.
//!
//! Scenarios covered:
//!
//! 1. **Valid config** — print the parsed summary (name, provider, features)
//! 2. **Invalid JSON** — catch the parse error before any state is mutated
//! 3. **Invalid domain rules** — catch e.g. a missing required bind address
//!
//! # Running
//!
//! ```bash
//! cargo run --example sdk_validate_config
//! ```

use std::io::Write as _;
use std::path::PathBuf;

use torrust_tracker_deployer_sdk::Deployer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    println!("=== Torrust Tracker Deployer SDK — Validate Config Example ===\n");

    let deployer = Deployer::builder().working_dir(&workspace).build()?;

    let private_key = workspace.join("fixtures/testing_rsa");
    let public_key = workspace.join("fixtures/testing_rsa.pub");

    // ------------------------------------------------------------------
    // Scenario 1: Valid configuration
    // ------------------------------------------------------------------
    println!("--- Scenario 1: Valid config ---");

    let valid_json = format!(
        r#"{{
  "environment": {{ "name": "sdk-validate-demo" }},
  "ssh_credentials": {{
    "private_key_path": "{private_key}",
    "public_key_path": "{public_key}"
  }},
  "provider": {{
    "provider": "lxd",
    "profile_name": "torrust-sdk-validate-demo"
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

    let valid_file = write_temp_json(&valid_json)?;
    match deployer.validate(valid_file.path()) {
        Ok(result) => {
            println!("  ✓ Config is valid.");
            println!("  Environment : {}", result.environment_name);
            println!("  Provider    : {}", result.provider);
            println!("  Prometheus  : {}", yes_no(result.has_prometheus));
            println!("  Grafana     : {}", yes_no(result.has_grafana));
            println!("  HTTPS       : {}", yes_no(result.has_https));
            println!("  Backup      : {}", yes_no(result.has_backup));
        }
        Err(e) => println!("  ✗ Unexpected error: {e}"),
    }
    println!();

    // ------------------------------------------------------------------
    // Scenario 2: Malformed JSON
    // ------------------------------------------------------------------
    println!("--- Scenario 2: Malformed JSON ---");

    let bad_json = r#"{ "environment": { "name": "broken" MISSING_COMMA "extra": 1 } }"#;
    let bad_json_file = write_temp_json(bad_json)?;
    match deployer.validate(bad_json_file.path()) {
        Ok(_) => println!("  Unexpected success."),
        Err(e) => println!("  ✗ Caught parse error (expected): {e}"),
    }
    println!();

    // ------------------------------------------------------------------
    // Scenario 3: Structurally valid JSON but missing required fields
    // ------------------------------------------------------------------
    println!("--- Scenario 3: Missing required fields ---");

    let incomplete_json = format!(
        r#"{{
  "environment": {{ "name": "sdk-incomplete" }},
  "ssh_credentials": {{
    "private_key_path": "{private_key}",
    "public_key_path": "{public_key}"
  }},
  "provider": {{
    "provider": "lxd",
    "profile_name": "torrust-sdk-incomplete"
  }},
  "tracker": {{
    "core": {{
      "database": {{ "driver": "sqlite3", "database_name": "tracker.db" }},
      "private": false
    }},
    "udp_trackers": [],
    "http_trackers": [],
    "http_api": {{ "bind_address": "0.0.0.0:1212", "admin_token": "MyAccessToken" }},
    "health_check_api": {{ "bind_address": "0.0.0.0:1313" }}
  }}
}}"#,
        private_key = private_key.display(),
        public_key = public_key.display(),
    );

    let incomplete_file = write_temp_json(&incomplete_json)?;
    match deployer.validate(incomplete_file.path()) {
        Ok(result) => {
            // No UDP/HTTP trackers is currently allowed — print the result.
            println!("  ✓ Config accepted (no trackers configured).");
            println!("  Environment : {}", result.environment_name);
        }
        Err(e) => println!("  ✗ Caught validation error (expected): {e}"),
    }

    println!("\n=== Example complete ===");
    Ok(())
}

fn yes_no(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

fn write_temp_json(json: &str) -> Result<tempfile::NamedTempFile, std::io::Error> {
    let mut f = tempfile::NamedTempFile::new()?;
    f.write_all(json.as_bytes())?;
    Ok(f)
}
