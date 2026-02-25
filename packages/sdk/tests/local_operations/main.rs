//! SDK integration tests for local operations.
//!
//! These tests exercise the SDK public API exactly as an external consumer
//! would — importing only from `torrust_tracker_deployer_sdk`. They cover
//! local-only operations (create, show, list, exists, validate, destroy,
//! purge) against a temporary workspace directory.
//!
//! No infrastructure (LXD, Docker, SSH) is required.
//!
//! ## Structure
//!
//! One module per command, mirroring the CLI E2E tests in `tests/e2e/`:
//!
//! - `create` — create environment (typed builder + JSON file)
//! - `show` — show environment details + not-found error
//! - `list` — list environments (populated + empty workspace)
//! - `exists` — exists before/after create
//! - `validate` — validate config files (valid + invalid)
//! - `destroy` — destroy a created environment
//! - `purge` — purge environment completely
//! - `builder` — `DeployerBuilder` error cases
//! - `workflow` — chained operations (create → list → show → destroy → purge)

mod builder;
mod create;
mod destroy;
mod exists;
mod list;
mod purge;
mod show;
mod validate;
mod workflow;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use torrust_tracker_deployer_sdk::{Deployer, EnvironmentCreationConfig, EnvironmentName};

// ── Helpers ─────────────────────────────────────────────────────────

/// Absolute path to the repository root (two levels up from `packages/sdk/`).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("Failed to resolve repo root")
}

/// Absolute paths to the testing SSH key pair in `fixtures/`.
fn fixture_ssh_keys() -> (PathBuf, PathBuf) {
    let root = repo_root();
    (
        root.join("fixtures/testing_rsa"),
        root.join("fixtures/testing_rsa.pub"),
    )
}

/// Build a `Deployer` rooted in a fresh temp directory.
///
/// Returns `(deployer, temp_dir)`. The `TempDir` must be kept alive for
/// the duration of the test so the workspace is not deleted.
fn deployer_in_temp_dir() -> (Deployer, TempDir) {
    let workspace = TempDir::new().expect("Failed to create temp directory");
    let deployer = Deployer::builder()
        .working_dir(workspace.path())
        .build()
        .expect("Failed to build deployer");
    (deployer, workspace)
}

/// Build a minimal `EnvironmentCreationConfig` with the given name.
///
/// Uses the repository-root `fixtures/` SSH keys (absolute paths).
fn minimal_config(name: &str) -> EnvironmentCreationConfig {
    let (private_key, public_key) = fixture_ssh_keys();

    EnvironmentCreationConfig::builder()
        .name(name)
        .ssh_keys(private_key.to_string_lossy(), public_key.to_string_lossy())
        .provider_lxd("torrust-sdk-test")
        .sqlite("tracker.db")
        .api("0.0.0.0:1212", "MyAccessToken")
        .build()
        .expect("Failed to build config")
}

/// Create an environment with the given name and return its
/// `EnvironmentName`.
fn create_environment(deployer: &Deployer, name: &str) -> EnvironmentName {
    deployer
        .create_environment(minimal_config(name))
        .unwrap_or_else(|e| panic!("create_environment({name}) failed: {e}"))
}

/// Write a minimal valid environment JSON config to `dir/{filename}`.
///
/// Returns the absolute path to the written file.
fn write_config_json(dir: &Path, filename: &str, env_name: &str) -> PathBuf {
    let (private_key, public_key) = fixture_ssh_keys();

    let json = format!(
        r#"{{
  "environment": {{ "name": "{env_name}" }},
  "ssh_credentials": {{
    "private_key_path": "{private_key}",
    "public_key_path": "{public_key}"
  }},
  "provider": {{
    "provider": "lxd",
    "profile_name": "torrust-sdk-test"
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

    let path = dir.join(filename);
    std::fs::write(&path, json).expect("Failed to write config file");
    path
}

// ── Custom asserts ──────────────────────────────────────────────────

/// Assert that the named environment exists in the deployer workspace.
fn assert_environment_exists(deployer: &Deployer, name: &EnvironmentName) {
    assert!(
        deployer.exists(name).expect("exists() failed"),
        "expected environment '{name}' to exist"
    );
}

/// Assert that the named environment does NOT exist in the deployer workspace.
fn assert_environment_not_exists(deployer: &Deployer, name: &EnvironmentName) {
    assert!(
        !deployer.exists(name).expect("exists() failed"),
        "expected environment '{name}' to NOT exist"
    );
}
