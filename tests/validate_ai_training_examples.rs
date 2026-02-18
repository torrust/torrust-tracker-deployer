/// Integration test to validate all AI training example configurations.
///
/// This test ensures that:
/// 1. All example configuration files exist
/// 2. All examples pass schema validation
/// 3. All examples are properly formatted JSON
/// 4. All examples can be rendered into deployment artifacts
///
/// Run with: `cargo test --test validate_ai_training_examples`
mod support;

use std::fs;
use std::path::{Path, PathBuf};
use support::ProcessRunner;
use tempfile::TempDir;

const EXAMPLES_DIR: &str = "docs/ai-training/dataset/environment-configs";
const FIXTURE_PRIVATE_KEY: &str = "fixtures/testing_rsa";
const FIXTURE_PUBLIC_KEY: &str = "fixtures/testing_rsa.pub";
const PLACEHOLDER_IP: &str = "203.0.113.1"; // RFC 5737 TEST-NET-1

// Helper Functions

/// Extract file name from path, returning "unknown" if extraction fails
fn get_file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
}

/// Setup test examples by collecting all JSON files from the examples directory
fn setup_test_examples() -> Vec<PathBuf> {
    let examples_dir = PathBuf::from(EXAMPLES_DIR);
    assert!(
        examples_dir.exists(),
        "Examples directory should exist: {EXAMPLES_DIR}"
    );

    let example_files = collect_example_files(&examples_dir);
    assert!(
        !example_files.is_empty(),
        "Should find example configuration files in {EXAMPLES_DIR}"
    );

    example_files
}

/// Convert a path to a string slice with better error context
fn path_to_str<'a>(path: &'a Path, context: &str) -> Result<&'a str, String> {
    path.to_str()
        .ok_or_else(|| format!("Invalid UTF-8 in {context}: {}", path.display()))
}

/// Report failures and panic with a summary
fn report_failures_and_panic(failures: &[(String, String)], total: usize, operation: &str) {
    eprintln!("\n❌ Failed {operation}s:\n");
    for (file, error) in failures {
        eprintln!("  • {file}: {error}");
    }
    panic!(
        "\n{} out of {total} example configurations failed {operation}",
        failures.len()
    );
}

// Test Functions

#[test]
fn it_should_validate_all_ai_training_example_configurations() {
    let example_files = setup_test_examples();
    let mut failures = Vec::new();

    for example_file in &example_files {
        let file_name = get_file_name(example_file);

        if let Err(error) = validate_configuration(example_file) {
            println!("✗");
            failures.push((file_name.to_string(), error));
        }
    }

    if !failures.is_empty() {
        report_failures_and_panic(&failures, example_files.len(), "validation");
    }
}

fn collect_example_files(examples_dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

fn validate_configuration(config_path: &Path) -> Result<(), String> {
    let config_path_str = path_to_str(config_path, "config path")?;

    // Create a temporary workspace for this validation
    let temp_workspace =
        TempDir::new().map_err(|e| format!("Failed to create temp workspace: {e}"))?;
    let log_dir = temp_workspace.path().join("logs");

    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .log_dir(&log_dir)
        .run_validate_command(config_path_str)
        .map_err(|e| format!("Failed to run validate command: {e}"))?;

    if result.success() {
        Ok(())
    } else {
        Err(format!(
            "Validation failed with exit code: {:?}\nstderr: {}",
            result.exit_code(),
            result.stderr()
        ))
    }
}

#[test]
fn it_should_verify_expected_number_of_examples() {
    const EXPECTED_EXAMPLES: usize = 15;

    let example_files = setup_test_examples();

    assert_eq!(
        example_files.len(),
        EXPECTED_EXAMPLES,
        "Expected {EXPECTED_EXAMPLES} example files, found {}. Files: {:?}",
        example_files.len(),
        example_files
            .iter()
            .map(|p| get_file_name(p))
            .collect::<Vec<_>>()
    );
}

#[test]
fn it_should_verify_example_naming_convention() {
    let example_files = setup_test_examples();
    let pattern =
        regex::Regex::new(r"^\d{2}-[a-z0-9-]+\.json$").expect("Regex pattern should be valid");

    for example_file in &example_files {
        let file_name = get_file_name(example_file);

        assert!(
            pattern.is_match(file_name),
            "Example file '{file_name}' does not match naming convention: NN-descriptive-name.json"
        );
    }
}

#[test]
fn it_should_verify_all_examples_have_descriptions() {
    let example_files = setup_test_examples();
    let mut missing_descriptions = Vec::new();

    for example_file in &example_files {
        let file_name = get_file_name(example_file);
        let content =
            fs::read_to_string(example_file).expect("Should be able to read example file");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("Example file should contain valid JSON");

        if !has_valid_description(&json) {
            missing_descriptions.push(file_name.to_string());
        }
    }

    assert!(
        missing_descriptions.is_empty(),
        "The following example files are missing descriptions: {missing_descriptions:?}"
    );
}

fn has_valid_description(json: &serde_json::Value) -> bool {
    json.get("environment")
        .and_then(|env| env.get("description"))
        .and_then(|desc| desc.as_str())
        .is_some_and(|s| !s.is_empty())
}

#[test]
fn it_should_render_all_ai_training_example_configurations() {
    let example_files = setup_test_examples();
    let mut failures = Vec::new();

    for example_file in &example_files {
        let file_name = get_file_name(example_file);

        if let Err(error) = render_configuration(example_file) {
            eprintln!("✗ {file_name}");
            failures.push((file_name.to_string(), error));
        }
    }

    if !failures.is_empty() {
        report_failures_and_panic(&failures, example_files.len(), "render");
    }
}

fn render_configuration(config_path: &Path) -> Result<(), String> {
    let temp_workspace =
        TempDir::new().map_err(|e| format!("Failed to create temp workspace: {e}"))?;

    let temp_config_file = prepare_test_config(config_path, &temp_workspace)?;
    let output_dir = temp_workspace.path().join("output");

    execute_render_command(&temp_config_file, &output_dir, &temp_workspace)?;
    verify_render_output_directories(&output_dir)?;

    Ok(())
}

/// Prepare a test configuration by replacing SSH credentials with fixture paths
fn prepare_test_config(config_path: &Path, temp_workspace: &TempDir) -> Result<PathBuf, String> {
    let config_content =
        fs::read_to_string(config_path).map_err(|e| format!("Failed to read config file: {e}"))?;

    let mut config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {e}"))?;

    replace_ssh_credentials_with_fixtures(&mut config)?;

    let temp_config_file = temp_workspace.path().join("config.json");
    fs::write(
        &temp_config_file,
        serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize modified config: {e}"))?,
    )
    .map_err(|e| format!("Failed to write temp config file: {e}"))?;

    Ok(temp_config_file)
}

/// Replace SSH credentials in config with absolute paths to test fixtures
fn replace_ssh_credentials_with_fixtures(config: &mut serde_json::Value) -> Result<(), String> {
    let workspace_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let abs_private_key = workspace_root.join(FIXTURE_PRIVATE_KEY);
    let abs_public_key = workspace_root.join(FIXTURE_PUBLIC_KEY);

    if let Some(ssh_credentials) = config.get_mut("ssh_credentials") {
        if let Some(credentials_map) = ssh_credentials.as_object_mut() {
            credentials_map.insert(
                "private_key_path".to_string(),
                serde_json::Value::String(
                    path_to_str(&abs_private_key, "private key path")?.to_string(),
                ),
            );
            credentials_map.insert(
                "public_key_path".to_string(),
                serde_json::Value::String(
                    path_to_str(&abs_public_key, "public key path")?.to_string(),
                ),
            );
        }
    }

    Ok(())
}

/// Execute the render command using `ProcessRunner`
fn execute_render_command(
    temp_config_file: &Path,
    output_dir: &Path,
    temp_workspace: &TempDir,
) -> Result<(), String> {
    let temp_config_file_str = path_to_str(temp_config_file, "temp config file path")?;
    let output_dir_str = path_to_str(output_dir, "output directory path")?;

    // Create log directory in temp workspace
    let log_dir = temp_workspace.path().join("logs");

    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .log_dir(&log_dir)
        .run_render_command_with_config_file(temp_config_file_str, PLACEHOLDER_IP, output_dir_str)
        .map_err(|e| format!("Failed to run render command: {e}"))?;

    if !result.success() {
        return Err(format!(
            "Render command failed with exit code: {:?}\nstderr: {}",
            result.exit_code(),
            result.stderr()
        ));
    }

    Ok(())
}

/// Verify that the render command created the expected output directories
fn verify_render_output_directories(output_dir: &Path) -> Result<(), String> {
    let ansible_dir = output_dir.join("ansible");
    let docker_compose_dir = output_dir.join("docker-compose");

    if !ansible_dir.exists() {
        return Err("Render succeeded but ansible directory was not created".to_string());
    }

    if !docker_compose_dir.exists() {
        return Err("Render succeeded but docker-compose directory was not created".to_string());
    }

    Ok(())
}
