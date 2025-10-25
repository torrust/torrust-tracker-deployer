# E2E Black Box Test for Create Command

**Issue**: [#38](https://github.com/torrust/torrust-tracker-deployer/issues/38)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command
**Depends On**: [#37](https://github.com/torrust/torrust-tracker-deployer/issues/37) - CLI Presentation Layer
**Related**: [E2E Testing Guide](../e2e-testing.md), [Testing Conventions](../contributing/testing.md)

## Overview

Implement a true black-box end-to-end test for the create command that tests the production application as an external process. Unlike other E2E tests that mock parts of the system, this test exercises the complete application workflow from configuration file to persisted environment state.

## Goals

- [ ] Create a true black-box E2E test that runs the production application as an external process
- [ ] Test the complete workflow: config file → command execution → environment persistence
- [ ] Use temporary directories for complete test isolation
- [ ] Verify environment state is correctly persisted in the data folder
- [ ] Add support for `--working-dir` optional argument to specify application working directory
- [ ] Establish pattern for black-box testing that can be extended to other commands

## 🏗️ Architecture Requirements

**Test Type**: Black Box E2E Test
**Test Location**: `tests/e2e_create_command.rs` (separate from existing E2E infrastructure)
**Pattern**: External Process Testing + Temporary Directory Isolation
**Dependencies**: Production binary, temporary file system operations

### Test Structure

```text
tests/
├── e2e_create_command.rs     # New black-box E2E test
└── support/                  # Test support utilities
    ├── mod.rs
    ├── temp_workspace.rs     # Temporary workspace management
    ├── process_runner.rs     # External process execution
    └── assertions.rs         # Environment state assertions
```

## Specifications

### Black Box Test Implementation

```rust
// tests/e2e_create_command.rs
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use serde_json::Value;

mod support;
use support::{TempWorkspace, ProcessRunner, EnvironmentStateAssertions};

#[test]
fn it_should_create_environment_from_config_file_black_box() {
    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config();
    temp_workspace.write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Act: Run production application as external process
    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    // Assert: Verify command succeeded
    assert!(result.success(), "Create command failed: {}", result.stderr());

    // Assert: Verify environment state was persisted
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-environment");
    env_assertions.assert_environment_state_is("test-environment", "Created");
    env_assertions.assert_data_directory_structure("test-environment");
    env_assertions.assert_trace_directory_exists("test-environment");
}

#[test]
fn it_should_fail_gracefully_with_invalid_config() {
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create invalid configuration
    let invalid_config = r#"{"invalid": "config"}"#;
    temp_workspace.write_file("invalid.json", invalid_config)
        .expect("Failed to write invalid config");

    // Run command and expect failure
    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./invalid.json")
        .expect("Failed to run create command");

    // Assert command failed with helpful error message
    assert!(!result.success(), "Command should have failed with invalid config");
    assert!(result.stderr().contains("Configuration validation failed"));
}

#[test]
fn it_should_fail_when_config_file_not_found() {
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Run command with non-existent config file
    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./nonexistent.json")
        .expect("Failed to run create command");

    // Assert command failed with file not found error
    assert!(!result.success(), "Command should have failed with missing file");
    assert!(result.stderr().contains("Failed to parse configuration file"));
}

fn create_test_environment_config() -> String {
    serde_json::json!({
        "environment": {
            "name": "test-environment"
        },
        "ssh_credentials": {
            "private_key_path": "~/.ssh/id_rsa",
            "public_key_path": "~/.ssh/id_rsa.pub",
            "username": "torrust",
            "port": 22
        }
    }).to_string()
}
```

### Test Support Infrastructure

#### Temporary Workspace Management

```rust
// tests/support/temp_workspace.rs
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use std::fs;
use anyhow::Result;

/// Manages a temporary workspace for black-box testing
pub struct TempWorkspace {
    temp_dir: TempDir,
}

impl TempWorkspace {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self { temp_dir })
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn write_config_file(&self, filename: &str, config: &str) -> Result<()> {
        let file_path = self.temp_dir.path().join(filename);
        fs::write(file_path, config)?;
        Ok(())
    }

    pub fn write_file(&self, filename: &str, content: &str) -> Result<()> {
        let file_path = self.temp_dir.path().join(filename);
        fs::write(file_path, content)?;
        Ok(())
    }

    pub fn data_dir(&self) -> PathBuf {
        self.temp_dir.path().join("data")
    }

    pub fn build_dir(&self) -> PathBuf {
        self.temp_dir.path().join("build")
    }
}
```

#### External Process Execution

```rust
// tests/support/process_runner.rs
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use anyhow::{Result, Context};

/// Runs the production application as an external process
pub struct ProcessRunner {
    working_dir: Option<PathBuf>,
}

impl ProcessRunner {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn working_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.working_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    pub fn run_create_command(&self, config_file: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--", "create", "--env-file", config_file]);

        // Set working directory if specified
        if let Some(working_dir) = &self.working_dir {
            cmd.current_dir(working_dir);
        }

        let output = cmd.output()
            .context("Failed to execute create command")?;

        Ok(ProcessResult::new(output))
    }

    // Future: Add support for --working-dir argument
    pub fn run_create_command_with_working_dir(&self, config_file: &str, app_working_dir: &Path) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");
        cmd.args(&[
            "run", "--", "create",
            "--env-file", config_file,
            "--working-dir", app_working_dir.to_str().unwrap()
        ]);

        if let Some(test_working_dir) = &self.working_dir {
            cmd.current_dir(test_working_dir);
        }

        let output = cmd.output()
            .context("Failed to execute create command with working dir")?;

        Ok(ProcessResult::new(output))
    }
}

/// Wrapper around process execution results
pub struct ProcessResult {
    output: Output,
}

impl ProcessResult {
    fn new(output: Output) -> Self {
        Self { output }
    }

    pub fn success(&self) -> bool {
        self.output.status.success()
    }

    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).to_string()
    }

    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).to_string()
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.output.status.code()
    }
}
```

#### Environment State Assertions

```rust
// tests/support/assertions.rs
use std::path::{Path, PathBuf};
use std::fs;
use serde_json::Value;
use anyhow::{Result, Context};

/// Assertions for verifying environment state after command execution
pub struct EnvironmentStateAssertions {
    workspace_path: PathBuf,
}

impl EnvironmentStateAssertions {
    pub fn new<P: AsRef<Path>>(workspace_path: P) -> Self {
        Self {
            workspace_path: workspace_path.as_ref().to_path_buf(),
        }
    }

    pub fn assert_environment_exists(&self, env_name: &str) {
        let env_file_path = self.environment_json_path(env_name);
        assert!(
            env_file_path.exists(),
            "Environment file should exist at: {}",
            env_file_path.display()
        );
    }

    pub fn assert_environment_state_is(&self, env_name: &str, expected_state: &str) {
        let env_data = self.read_environment_json(env_name)
            .expect("Failed to read environment JSON");

        // Parse the environment state structure
        let state_key = env_data.as_object()
            .expect("Environment JSON should be an object")
            .keys()
            .next()
            .expect("Environment should have a state key");

        assert_eq!(
            state_key, expected_state,
            "Environment state should be '{}', but was '{}'",
            expected_state, state_key
        );
    }

    pub fn assert_data_directory_structure(&self, env_name: &str) {
        let data_dir = self.workspace_path.join("data").join(env_name);
        assert!(
            data_dir.exists(),
            "Data directory should exist at: {}",
            data_dir.display()
        );

        let env_json = data_dir.join("environment.json");
        assert!(
            env_json.exists(),
            "Environment JSON should exist at: {}",
            env_json.display()
        );
    }

    pub fn assert_trace_directory_exists(&self, env_name: &str) {
        let traces_dir = self.workspace_path
            .join("data")
            .join(env_name)
            .join("traces");

        assert!(
            traces_dir.exists(),
            "Traces directory should exist at: {}",
            traces_dir.display()
        );
    }

    pub fn assert_build_directory_structure(&self, env_name: &str) {
        let build_dir = self.workspace_path.join("build").join(env_name);
        assert!(
            build_dir.exists(),
            "Build directory should exist at: {}",
            build_dir.display()
        );
    }

    fn environment_json_path(&self, env_name: &str) -> PathBuf {
        self.workspace_path
            .join("data")
            .join(env_name)
            .join("environment.json")
    }

    fn read_environment_json(&self, env_name: &str) -> Result<Value> {
        let env_file_path = self.environment_json_path(env_name);
        let content = fs::read_to_string(&env_file_path)
            .context(format!("Failed to read environment file: {}", env_file_path.display()))?;

        let json: Value = serde_json::from_str(&content)
            .context("Failed to parse environment JSON")?;

        Ok(json)
    }
}
```

### Test Support Module

```rust
// tests/support/mod.rs
mod temp_workspace;
mod process_runner;
mod assertions;

pub use temp_workspace::TempWorkspace;
pub use process_runner::{ProcessRunner, ProcessResult};
pub use assertions::EnvironmentStateAssertions;
```

## Implementation Plan

### Phase 1: Test Infrastructure (2 hours)

- [ ] Create `tests/e2e_create_command.rs` file for black-box tests
- [ ] Implement `TempWorkspace` for temporary directory management
- [ ] Implement `ProcessRunner` for external process execution
- [ ] Implement `EnvironmentStateAssertions` for result verification
- [ ] Create test support module structure

### Phase 2: Core Black Box Tests (2 hours)

- [ ] Implement successful environment creation test
- [ ] Test with invalid configuration file
- [ ] Test with missing configuration file
- [ ] Test environment state persistence verification
- [ ] Test data and trace directory structure creation

### Phase 3: Future Enhancement - Working Directory Support (1 hour)

- [ ] Add `--working-dir` argument to CLI argument parsing
- [ ] Update application to respect working directory for data/build folders
- [ ] Add test for `--working-dir` functionality
- [ ] Update process runner to support new argument

### Phase 4: Test Integration (1 hour)

- [ ] Integrate black-box tests into CI pipeline
- [ ] Add test documentation and usage examples
- [ ] Ensure tests run reliably in different environments
- [ ] Add test cleanup and error handling

## Future Enhancements

### Working Directory Support

The `--working-dir` optional argument will be valuable for:

- **Testing**: Running application from source directory but creating data in temp directory
- **Deployment**: Running application from one location but storing data elsewhere
- **Development**: Flexible workspace organization

```bash
# Example usage with --working-dir
torrust-tracker-deployer create environment --env-file ./config.json --working-dir /tmp/test-workspace

# This would:
# - Read config.json from current directory
# - Create data/ and build/ directories in /tmp/test-workspace
# - Run application logic from current directory
```

### Extended Black Box Testing

This pattern can be extended to other commands:

- **Provision Command**: Test complete provisioning workflow
- **Configure Command**: Test configuration application
- **Destroy Command**: Test cleanup and state removal
- **Status Command**: Test environment status reporting

## Acceptance Criteria

- [ ] Black-box E2E test that runs production application as external process
- [ ] Test creates environment from configuration file and verifies persistence
- [ ] Test handles invalid configuration gracefully with helpful error messages
- [ ] Test verifies correct data directory structure creation
- [ ] Test uses temporary directories for complete isolation
- [ ] All tests pass reliably in CI environment
- [ ] Test support infrastructure is reusable for other commands
- [ ] Documentation includes examples and usage patterns

## Testing Strategy

### Test Categories

1. **Happy Path Tests**:

   - Valid configuration creates environment successfully
   - Environment state is correctly persisted
   - Directory structure is created properly

2. **Error Handling Tests**:

   - Invalid configuration format
   - Missing configuration file
   - File permission issues
   - Invalid environment names

3. **Integration Tests**:
   - Complete workflow from config to persistence
   - External process execution reliability
   - Temporary workspace cleanup

### Test Isolation

- Each test uses its own temporary directory
- No shared state between tests
- Complete cleanup after each test
- No dependency on external services or infrastructure

## Related Documentation

- [E2E Testing Guide](../e2e-testing.md) - Overview of E2E testing approach
- [Testing Conventions](../contributing/testing.md) - Testing standards and patterns
- [CLI Subcommand Structure](../codebase-architecture.md#presentation-layer) - CLI architecture
- [Environment State Management](../codebase-architecture.md#domain-layer) - Environment lifecycle

## Notes

### Benefits of Black Box Testing

- **Real-world validation**: Tests the actual user experience
- **Integration verification**: Ensures all components work together
- **Regression detection**: Catches issues that unit tests might miss
- **Documentation**: Serves as executable documentation of expected behavior

### Difference from Existing E2E Tests

Unlike existing E2E tests that mock infrastructure components, this test:

- **Runs actual production binary**: No mocking or test doubles
- **External process execution**: Tests CLI interface exactly as users would
- **Complete isolation**: Uses temporary workspaces with no shared state
- **Full workflow coverage**: Tests from configuration file to persisted state

This complements existing E2E tests by providing true black-box validation while maintaining fast, isolated test execution.
