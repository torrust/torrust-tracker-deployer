/// Integration test to validate all AI training example configurations.
///
/// This test ensures that:
/// 1. All example configuration files exist
/// 2. All examples pass schema validation
/// 3. All examples are properly formatted JSON
/// 4. All examples can be rendered into deployment artifacts
///
/// Run with: `cargo test --test validate_ai_training_examples`
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

const EXAMPLES_DIR: &str = "docs/ai-training/dataset/environment-configs";
const FIXTURE_PRIVATE_KEY: &str = "fixtures/testing_rsa";
const FIXTURE_PUBLIC_KEY: &str = "fixtures/testing_rsa.pub";
const PLACEHOLDER_IP: &str = "203.0.113.1"; // RFC 5737 TEST-NET-1

#[test]
fn it_should_validate_all_ai_training_example_configurations() {
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

    // println!(
    //     "\nValidating {} example configurations...\n",
    //     example_files.len()
    // );

    let mut failed_validations = Vec::new();

    for example_file in &example_files {
        let file_name = example_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // print!("Validating {file_name}... ");

        match validate_configuration(example_file) {
            Ok(()) => {
                // println!("✓");
            }
            Err(error) => {
                println!("✗");
                failed_validations.push((file_name.to_string(), error));
            }
        }
    }

    if !failed_validations.is_empty() {
        eprintln!("\n❌ Failed validations:\n");
        for (file, error) in &failed_validations {
            eprintln!("  • {file}: {error}");
        }
        panic!(
            "\n{} out of {} example configurations failed validation",
            failed_validations.len(),
            example_files.len()
        );
    }

    // println!(
    //     "\n✅ All {} example configurations validated successfully!",
    //     example_files.len()
    // );
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

fn validate_configuration(config_path: &PathBuf) -> Result<(), String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--")
        .arg("validate")
        .arg("--env-file")
        .arg(config_path)
        .output()
        .map_err(|e| format!("Failed to execute validation command: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract the most relevant error message
        let error_message = extract_error_message(&stderr, &stdout);

        Err(error_message)
    }
}

fn extract_error_message(stderr: &str, stdout: &str) -> String {
    // Try to find specific error lines
    for line in stderr.lines().chain(stdout.lines()) {
        if line.contains("Error:") {
            return line.trim().to_string();
        }
        if line.contains("missing field") {
            return line.trim().to_string();
        }
        if line.contains("unknown variant") {
            return line.trim().to_string();
        }
    }

    // If no specific error found, return first non-empty line
    stderr
        .lines()
        .chain(stdout.lines())
        .find(|line| !line.trim().is_empty())
        .unwrap_or("Validation failed (no error details)")
        .trim()
        .to_string()
}

#[test]
fn it_should_verify_expected_number_of_examples() {
    // We expect exactly 15 example configurations (01-15)
    const EXPECTED_EXAMPLES: usize = 15;

    let examples_dir = PathBuf::from(EXAMPLES_DIR);
    let example_files = collect_example_files(&examples_dir);

    assert_eq!(
        example_files.len(),
        EXPECTED_EXAMPLES,
        "Expected {EXPECTED_EXAMPLES} example files, found {}. Files: {:?}",
        example_files.len(),
        example_files
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn it_should_verify_example_naming_convention() {
    let examples_dir = PathBuf::from(EXAMPLES_DIR);
    let example_files = collect_example_files(&examples_dir);

    assert!(
        !example_files.is_empty(),
        "Should find example configuration files in {EXAMPLES_DIR}"
    );

    // Compile regex once outside the loop
    let pattern =
        regex::Regex::new(r"^\d{2}-[a-z0-9-]+\.json$").expect("Regex pattern should be valid");

    for example_file in &example_files {
        let file_name = example_file
            .file_name()
            .and_then(|n| n.to_str())
            .expect("File should have a valid name");

        // Should match pattern: NN-descriptive-name.json
        assert!(
            pattern.is_match(file_name),
            "Example file '{file_name}' does not match naming convention: NN-descriptive-name.json"
        );
    }
}

#[test]
fn it_should_verify_all_examples_have_descriptions() {
    let examples_dir = PathBuf::from(EXAMPLES_DIR);
    let example_files = collect_example_files(&examples_dir);

    assert!(
        !example_files.is_empty(),
        "Should find example configuration files in {EXAMPLES_DIR}"
    );

    let mut missing_descriptions = Vec::new();

    for example_file in &example_files {
        let file_name = example_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let content =
            fs::read_to_string(example_file).expect("Should be able to read example file");

        let json: serde_json::Value =
            serde_json::from_str(&content).expect("Example file should contain valid JSON");

        if let Some(environment) = json.get("environment") {
            if let Some(description) = environment.get("description") {
                if description.is_null() || description.as_str().is_none_or(str::is_empty) {
                    missing_descriptions.push(file_name.to_string());
                }
            } else {
                missing_descriptions.push(file_name.to_string());
            }
        } else {
            missing_descriptions.push(file_name.to_string());
        }
    }

    assert!(
        missing_descriptions.is_empty(),
        "The following example files are missing descriptions: {missing_descriptions:?}"
    );
}

#[test]
fn it_should_render_all_ai_training_example_configurations() {
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

    let mut failed_renders = Vec::new();

    for example_file in &example_files {
        let file_name = example_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        match render_configuration(example_file) {
            Ok(()) => {
                // Render succeeded
            }
            Err(error) => {
                eprintln!("✗ {file_name}");
                failed_renders.push((file_name.to_string(), error));
            }
        }
    }

    if !failed_renders.is_empty() {
        eprintln!("\n❌ Failed renders:\n");
        for (file, error) in &failed_renders {
            eprintln!("  • {file}: {error}");
        }
        panic!(
            "\n{} out of {} example configurations failed to render",
            failed_renders.len(),
            example_files.len()
        );
    }
}

fn render_configuration(config_path: &PathBuf) -> Result<(), String> {
    // Create temporary directory for output
    let temp_output_dir =
        TempDir::new().map_err(|e| format!("Failed to create temp output directory: {e}"))?;

    // Read the original config
    let config_content =
        fs::read_to_string(config_path).map_err(|e| format!("Failed to read config file: {e}"))?;

    // Parse and modify SSH paths to use fixtures
    let mut config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config JSON: {e}"))?;

    // Convert fixture paths to absolute paths
    let workspace_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let abs_private_key = workspace_root.join(FIXTURE_PRIVATE_KEY);
    let abs_public_key = workspace_root.join(FIXTURE_PUBLIC_KEY);

    // Replace SSH key paths with absolute fixture paths
    if let Some(ssh_credentials) = config.get_mut("ssh_credentials") {
        if let Some(obj) = ssh_credentials.as_object_mut() {
            obj.insert(
                "private_key_path".to_string(),
                serde_json::Value::String(
                    abs_private_key
                        .to_str()
                        .ok_or("Failed to convert private key path to string")?
                        .to_string(),
                ),
            );
            obj.insert(
                "public_key_path".to_string(),
                serde_json::Value::String(
                    abs_public_key
                        .to_str()
                        .ok_or("Failed to convert public key path to string")?
                        .to_string(),
                ),
            );
        }
    }

    // Write modified config to temp file
    let temp_config_file = temp_output_dir.path().join("config.json");
    fs::write(
        &temp_config_file,
        serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize modified config: {e}"))?,
    )
    .map_err(|e| format!("Failed to write temp config file: {e}"))?;

    // Run render command
    let output = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--")
        .arg("render")
        .arg("--env-file")
        .arg(&temp_config_file)
        .arg("--instance-ip")
        .arg(PLACEHOLDER_IP)
        .arg("--output-dir")
        .arg(temp_output_dir.path())
        .arg("--force")
        .output()
        .map_err(|e| format!("Failed to execute render command: {e}"))?;

    if output.status.success() {
        // Verify that some key files were created
        let ansible_dir = temp_output_dir.path().join("ansible");
        let docker_compose_dir = temp_output_dir.path().join("docker-compose");

        if !ansible_dir.exists() {
            return Err("Render succeeded but ansible directory was not created".to_string());
        }

        if !docker_compose_dir.exists() {
            return Err(
                "Render succeeded but docker-compose directory was not created".to_string(),
            );
        }

        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // For render failures, show the full stderr as it contains detailed error info
        let error_message = if stderr.is_empty() {
            extract_error_message(&stderr, &stdout)
        } else {
            // Skip progress messages and extract actual errors
            let error_lines: Vec<&str> = stderr
                .lines()
                .filter(|line| {
                    line.contains("Error")
                        || line.contains("error")
                        || line.contains("failed")
                        || line.contains("✗")
                })
                .collect();

            if error_lines.is_empty() {
                // If no error lines found, show the last few lines of output
                stderr
                    .lines()
                    .rev()
                    .take(3)
                    .collect::<Vec<_>>()
                    .iter()
                    .rev()
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" | ")
            } else {
                error_lines.join(" | ")
            }
        };

        Err(error_message)
    }
}
