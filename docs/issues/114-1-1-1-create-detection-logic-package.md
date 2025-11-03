# Create Detection Logic Package

**Issue**: [#114](https://github.com/torrust/torrust-tracker-deployer/issues/114)
**Parent Issue**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests
**Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution
**Related**: [docs/e2e-testing.md](../e2e-testing.md)

## Overview

Set up the `packages/dependency-installer/` package structure and implement tool detection using the `ToolDetector` trait. This phase focuses on creating the foundation for the dependency installer package and implementing detection logic for all four required tools (cargo-machete, OpenTofu, Ansible, LXD).

## Objectives

- [ ] Create workspace package structure at `packages/dependency-installer/`
- [ ] Define `ToolDetector` trait for checking if tools are installed
- [ ] Implement detectors for all 4 required tools
- [ ] Implement `DependencyManager` for orchestrating detection operations
- [ ] Add structured logging following linting package pattern
- [ ] Write unit tests for detection logic

## Context

This is **Phase 1** of creating the dependency installation package. It establishes the core abstractions and detection logic that will be used by:

- **Phase 2 (Issue 1-1-2)**: CLI binary that exposes detection functionality
- **Phase 3 (Issue 1-1-3)**: Docker testing infrastructure
- **Phase 4 (Issue 1-1-4)**: Installation logic and install command

### Required Dependencies

The package needs to detect these tools:

- **cargo-machete** - Required for pre-commit checks (detecting unused dependencies)
- **OpenTofu** - Required for infrastructure provisioning tests
- **Ansible** - Required for configuration management tests
- **LXD** - Required for VM-based testing

### Why Detection First

Detection is the foundation for everything else:

1. Simple and universally implementable (just check if command exists)
2. No system modifications required (safe to implement and test)
3. Can be tested without special infrastructure
4. Provides immediate value (users can check what's missing)

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (external tool management)
**Module Path**: `packages/dependency-installer/` (new workspace package)
**Pattern**: Library-first (binary comes in Phase 2)

### Package Structure

```text
packages/dependency-installer/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                    # Library exports
│   ├── manager.rs                # DependencyManager implementation
│   ├── detector/
│   │   ├── mod.rs                # ToolDetector trait and re-exports
│   │   ├── cargo_machete.rs      # CargoMacheteDetector
│   │   ├── opentofu.rs           # OpenTofuDetector
│   │   ├── ansible.rs            # AnsibleDetector
│   │   └── lxd.rs                # LxdDetector
│   ├── errors.rs                 # DetectionError type
│   └── utils.rs                  # Command execution helpers
└── tests/
    └── detector_tests.rs         # Unit tests for detectors
```

**Note**: The `installer/` directory and binary will be added in later phases.

## Specifications

### ToolDetector Trait

```rust
use thiserror::Error;

/// Trait for detecting if a tool is installed
pub trait ToolDetector {
    /// Get the tool name for display purposes
    fn name(&self) -> &str;

    /// Check if the tool is already installed
    fn is_installed(&self) -> Result<bool, DetectionError>;

    /// Get the required version (if applicable)
    fn required_version(&self) -> Option<&str> {
        None  // Default implementation
    }
}

/// Error types for detection operations
#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("Failed to detect tool '{tool}': {source}")]
    DetectionFailed {
        tool: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command execution failed for tool '{tool}': {message}")]
    CommandFailed {
        tool: String,
        message: String,
    },
}
```

### Example Detector Implementation

```rust
use tracing::info;
use std::process::Command;

pub struct OpenTofuDetector;

impl ToolDetector for OpenTofuDetector {
    fn name(&self) -> &str {
        "OpenTofu"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(tool = "opentofu", "Checking if OpenTofu is installed");

        let output = Command::new("which")
            .arg("tofu")
            .output()
            .map_err(|e| DetectionError::DetectionFailed {
                tool: self.name().to_string(),
                source: e
            })?;

        let installed = output.status.success();

        if installed {
            info!(tool = "opentofu", "OpenTofu is installed");
        } else {
            info!(tool = "opentofu", "OpenTofu is not installed");
        }

        Ok(installed)
    }
}
```

### DependencyManager

```rust
/// Main dependency manager for detection operations
pub struct DependencyManager {
    detectors: Vec<Box<dyn ToolDetector>>,
}

impl DependencyManager {
    pub fn new() -> Self {
        Self {
            detectors: vec![
                Box::new(CargoMacheteDetector),
                Box::new(OpenTofuDetector),
                Box::new(AnsibleDetector),
                Box::new(LxdDetector),
            ],
        }
    }

    /// Check all dependencies and return results
    pub fn check_all(&self) -> Result<Vec<CheckResult>, DetectionError> {
        self.detectors
            .iter()
            .map(|detector| {
                let installed = detector.is_installed()?;
                Ok(CheckResult {
                    tool: detector.name().to_string(),
                    installed,
                })
            })
            .collect()
    }

    /// Get specific detector by dependency type
    pub fn get_detector(&self, dep: Dependency) -> &dyn ToolDetector {
        match dep {
            Dependency::CargoMachete => &CargoMacheteDetector,
            Dependency::OpenTofu => &OpenTofuDetector,
            Dependency::Ansible => &AnsibleDetector,
            Dependency::Lxd => &LxdDetector,
        }
    }
}

pub struct CheckResult {
    pub tool: String,
    pub installed: bool,
}

pub enum Dependency {
    CargoMachete,
    OpenTofu,
    Ansible,
    Lxd,
}
```

### Logging Setup

```rust
/// Initialize tracing with default configuration
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .with_max_level(tracing::Level::INFO)
        .init();
}
```

### Command Execution Utilities

Create `src/command.rs` (prefer specific module name over generic "utils"):

````rust
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Failed to execute command '{command}': {source}")]
    ExecutionFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command '{command}' not found in PATH")]
    CommandNotFound { command: String },
}

/// Check if a command exists in the system PATH
///
/// # Examples
///
/// ```rust
/// use dependency_installer::command::command_exists;
///
/// // Check if 'cargo' is installed
/// let exists = command_exists("cargo")?;
/// assert!(exists);
///
/// // Check for non-existent command
/// let exists = command_exists("nonexistent-tool")?;
/// assert!(!exists);
/// ```
pub fn command_exists(command: &str) -> Result<bool, CommandError> {
    // Use 'which' on Unix-like systems to check if command exists
    let output = Command::new("which")
        .arg(command)
        .output()
        .map_err(|e| CommandError::ExecutionFailed {
            command: format!("which {}", command),
            source: e,
        })?;

    Ok(output.status.success())
}

/// Execute a command and return its stdout as a string
///
/// # Examples
///
/// ```rust
/// use dependency_installer::command::execute_command;
///
/// // Get cargo version
/// let version = execute_command("cargo", &["--version"])?;
/// println!("Cargo version: {}", version);
///
/// // Get ansible version
/// let version = execute_command("ansible", &["--version"])?;
/// ```
pub fn execute_command(command: &str, args: &[&str]) -> Result<String, CommandError> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CommandError::CommandNotFound {
                    command: command.to_string(),
                }
            } else {
                CommandError::ExecutionFailed {
                    command: format!("{} {}", command, args.join(" ")),
                    source: e,
                }
            }
        })?;

    if !output.status.success() {
        return Err(CommandError::ExecutionFailed {
            command: format!("{} {}", command, args.join(" ")),
            source: std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Command exited with status: {}", output.status),
            ),
        });
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| CommandError::ExecutionFailed {
            command: format!("{} {}", command, args.join(" ")),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })
}
````

**Commands needed for detection**:

- `which <tool>` - Check if tool exists in PATH
- `cargo-machete --version` - Verify cargo-machete installation
- `tofu --version` - Verify OpenTofu installation
- `ansible --version` - Verify Ansible installation
- `lxc --version` - Verify LXD installation

## Implementation Tasks

### Setup

- [ ] Create `packages/dependency-installer/` directory structure
- [ ] Create `Cargo.toml` with required dependencies:
  - [ ] `tracing` and `tracing-subscriber` for logging
  - [ ] `thiserror` for error handling
  - [ ] `anyhow` for general error handling (if needed)
- [ ] Add package to workspace `Cargo.toml`
- [ ] Create `README.md` with package overview
- [ ] Implement `init_tracing()` following linting package pattern

### Core Abstractions

- [ ] Define `ToolDetector` trait in `src/detector/mod.rs`
- [ ] Define `DetectionError` in `src/errors.rs`
- [ ] Define `Dependency` enum for tool types
- [ ] Define `CheckResult` struct for detection results

### Detector Implementations

For each tool, create a detector struct that implements `ToolDetector`:

- [ ] **CargoMacheteDetector** (`src/detector/cargo_machete.rs`):
  - [ ] Implement `name()` - returns "cargo-machete"
  - [ ] Implement `is_installed()` - check if `cargo-machete` command exists
  - [ ] Add structured logging (info level)
- [ ] **OpenTofuDetector** (`src/detector/opentofu.rs`):

  - [ ] Implement `name()` - returns "OpenTofu"
  - [ ] Implement `is_installed()` - check if `tofu` command exists
  - [ ] Add structured logging (info level)

- [ ] **AnsibleDetector** (`src/detector/ansible.rs`):

  - [ ] Implement `name()` - returns "Ansible"
  - [ ] Implement `is_installed()` - check if `ansible` command exists
  - [ ] Add structured logging (info level)

- [ ] **LxdDetector** (`src/detector/lxd.rs`):
  - [ ] Implement `name()` - returns "LXD"
  - [ ] Implement `is_installed()` - check if `lxd` or `lxc` command exists
  - [ ] Add structured logging (info level)

### Dependency Manager

- [ ] Implement `DependencyManager` in `src/manager.rs`:
  - [ ] `new()` - Create instance with all detectors
  - [ ] `check_all()` - Check all dependencies and return results
  - [ ] `get_detector()` - Get specific detector by dependency type
- [ ] Add structured logging to manager operations

### Utility Functions

- [ ] Create command execution helpers in `src/utils.rs`:
  - [ ] `command_exists(cmd: &str)` - Check if a command is available
  - [ ] Helper for running `which` or equivalent
  - [ ] Platform-specific detection if needed

### Testing

- [ ] Write unit tests in `tests/detector_tests.rs`:
  - [ ] Test each detector with mocked command execution
  - [ ] Test `DependencyManager::check_all()`
  - [ ] Test error handling for missing commands
  - [ ] Test that logging works correctly
- [ ] Ensure all tests pass

### Library Exports

- [ ] Update `src/lib.rs` to export public API:
  - [ ] Re-export `ToolDetector` trait
  - [ ] Re-export all detector types
  - [ ] Re-export `DependencyManager`
  - [ ] Re-export `DetectionError` and `CheckResult`
  - [ ] Re-export `init_tracing()`

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Package compiles without warnings
- [ ] All unit tests pass

**Package Structure**:

- [ ] Package exists at `packages/dependency-installer/`
- [ ] Package is registered in workspace `Cargo.toml`
- [ ] Package has only library target (no binary yet)
- [ ] Follows conventions from `packages/linting/`
- [ ] Uses tracing for structured logging

**Library Functionality**:

- [ ] `ToolDetector` trait is defined with clear documentation
- [ ] All 4 detectors implement `ToolDetector` trait
- [ ] Detection logic works correctly (can detect installed tools)
- [ ] `DependencyManager` can check all dependencies
- [ ] Error handling is clear and actionable
- [ ] Structured logging is present in all operations

**Testing**:

- [ ] Unit tests exist for all detectors
- [ ] Tests verify detection logic with mocked commands
- [ ] Tests verify `DependencyManager` functionality
- [ ] Tests verify error handling

**Documentation**:

- [ ] Package README explains purpose and usage
- [ ] Code includes rustdoc comments for public APIs
- [ ] Trait methods have clear documentation
- [ ] Error types are documented

## Related Documentation

- [packages/linting/](../../packages/linting/) - Reference for package structure
- [Parent Issue 1-1](./create-dependency-installation-package-for-e2e-tests.md) - Overall package specification
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error handling guidelines
- [docs/contributing/module-organization.md](../contributing/module-organization.md) - Module organization patterns

## Notes

### Estimated Time

**2-3 hours** total for this phase.

### Next Steps

After completing this phase:

1. **Issue 1-1-2**: Create CLI binary that uses this detection logic
2. **Issue 1-1-3**: Add Docker testing infrastructure
3. **Issue 1-1-4**: Implement installation logic

### Design Decisions

**Library-only in this phase**: We focus on the library implementation without the binary. The binary will be added in Phase 2, which allows us to keep this phase focused and manageable.

**Simple detection**: Detection only checks if commands exist using `which` or similar. No version checking or complex validation in this phase (can be added later if needed).

**Mock-based testing**: Use mocked command execution for unit tests since we can't assume tools are installed on the test machine.

### Detection Implementation Notes

**Command detection approaches**:

1. Use `which` command (Unix/Linux)
2. Use `where` command (Windows)
3. Use `Command::new("tool").arg("--version")` as fallback

Choose the most reliable approach for each platform. The `utils.rs` module should abstract these differences.
