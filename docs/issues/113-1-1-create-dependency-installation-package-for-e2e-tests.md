# Create Dependency Installation Package for E2E Tests

**Issue**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113)
**Parent Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution
**Related**: [docs/e2e-testing.md](../e2e-testing.md)

## Sub-Issues

This issue is split into 4 manageable sub-issues (one per phase):

1. **[#114](https://github.com/torrust/torrust-tracker-deployer/issues/114)** - [Create Detection Logic Package](./114-1-1-1-create-detection-logic-package.md) (Phase 1, 2-3 hours)
2. **[#115](https://github.com/torrust/torrust-tracker-deployer/issues/115)** - [Create CLI Binary with Check Command](./115-1-1-2-create-cli-binary-with-check-command.md) (Phase 2, 2-3 hours)
3. **[#116](https://github.com/torrust/torrust-tracker-deployer/issues/116)** - [Create Docker Test Infrastructure](./116-1-1-3-create-docker-test-infrastructure.md) (Phase 3, 2-3 hours)
4. **[#117](https://github.com/torrust/torrust-tracker-deployer/issues/117)** - [Implement Installation Logic](./117-1-1-4-implement-installation-logic.md) (Phase 4, 4-5 hours)

**Total Time**: 10-14 hours split into 4 focused tasks

## Overview

Convert the existing bash setup scripts in `scripts/setup/` into a Rust package that can detect and install required dependencies. This package will be usable both as a standalone binary and as a library, making it easier for AI agents (and humans) to set up development environments in ephemeral or fresh environments. The package can later be integrated with E2E tests and CI workflows (covered in separate issues).

## Objectives

- [ ] Convert bash scripts in `scripts/setup/` to a Rust package `packages/dependency-installer/`
- [ ] Create a reusable library that can detect and install external dependencies
- [ ] Provide a CLI binary for manual dependency management
- [ ] Make dependency management more accessible to AI coding agents (can be used as a library in future integration)

## Context

### Current Dependency Requirements

Development and testing workflows require the following external dependencies:

- **cargo-machete** - Required for pre-commit checks (detecting unused dependencies)
- **OpenTofu** - Required for infrastructure provisioning tests
- **Ansible** - Required for configuration management tests
- **LXD** - Required for VM-based testing

**Current Setup**: Dependencies are installed via bash scripts in `scripts/setup/`:

- `install-ansible.sh`
- `install-lxd-ci.sh`
- `install-opentofu.sh`

### Problem

**For human developers**: Installing dependencies once is straightforward, but the bash scripts may not be portable or reliable across different environments.

**For AI agents**: Agents often work in ephemeral environments without these tools pre-installed. We cannot use Docker for the full environment because:

- Dependencies are heavy
- LXD virtualization requires host-level setup that Docker cannot provide

**Current agent limitations**: Agents may skip tests or tasks if they encounter problems installing dependencies, reducing reliability.

### Solution Approach

Create a Rust package that:

1. **Detects** if a tool is already installed
2. **Installs** the tool if missing
3. **Works as a library** that can be called from other Rust code (e.g., E2E tests in future integration)
4. **Works as a binary** that can be executed manually or in CI workflows
5. **Remains agent-agnostic** (works with any AI agent, not just GitHub Copilot)

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (external tool management)
**Module Path**: `packages/dependency-installer/` (new workspace package)
**Pattern**: Library + Binary (similar to `packages/linting/`)

### Module Structure Requirements

- [ ] Follow workspace package conventions (see `packages/linting/` as reference)
- [ ] Use single binary with subcommands (like linting package)
- [ ] Implement proper logging with tracing crate (follow linting package pattern)
- [ ] Separate library and binary concerns
- [ ] Use trait-based abstractions for testability
- [ ] Clear error handling with actionable messages

### Architectural Constraints

- [ ] Library code must be testable without executing actual installations
- [ ] Binary must provide clear CLI interface with subcommands
- [ ] Use tracing crate for logging (following linting package pattern)
- [ ] Each installer must be independent (can run individually or as a group)
- [ ] Error messages must be actionable (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Follow observability principles with structured logging

### Anti-Patterns to Avoid

- ❌ Tightly coupling to specific CI environments
- ❌ Hard-coding paths or versions without configurability
- ❌ Making installation non-idempotent (should be safe to run multiple times)
- ❌ Silent failures or unclear error messages

## Specifications

### Package Structure

Create a new workspace package following the linting package pattern:

```text
packages/dependency-installer/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Library exports
│   ├── cli.rs                    # CLI implementation (main binary logic)
│   ├── manager.rs                # DependencyManager implementation
│   ├── detector/
│   │   ├── mod.rs                # ToolDetector trait and re-exports
│   │   ├── cargo_machete.rs      # CargoMacheteDetector
│   │   ├── opentofu.rs           # OpenTofuDetector
│   │   ├── ansible.rs            # AnsibleDetector
│   │   └── lxd.rs                # LxdDetector
│   ├── installer/
│   │   ├── mod.rs                # ToolInstaller trait and re-exports
│   │   ├── cargo_machete.rs      # CargoMacheteInstaller
│   │   ├── opentofu.rs           # OpenTofuInstaller
│   │   ├── ansible.rs            # AnsibleInstaller
│   │   └── lxd.rs                # LxdInstaller
│   ├── errors.rs                 # DetectionError and InstallationError
│   └── utils.rs                  # Command execution helpers
└── README.md
```

**Binary**: The main binary lives in `src/bin/dependency-installer.rs` (following the linter pattern)

### Logging with Tracing

Follow the linting package pattern for logging:

```rust
use tracing::{info, error, warn, debug};

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

**Logging Guidelines**:

- Use `info!()` for user-facing operations (checking tools, installing tools)
- Use `error!()` for failures with context
- Use `warn!()` for non-critical issues
- Use `debug!()` for detailed debugging information
- Include structured context in logs (tool name, operation, paths)

**Example Usage**:

```rust
impl ToolInstaller for OpenTofuInstaller {
    fn install(&self) -> Result<(), InstallerError> {
        info!(tool = "opentofu", "Checking if OpenTofu is installed");

        if self.is_installed()? {
            info!(tool = "opentofu", "OpenTofu is already installed, skipping");
            return Ok(());
        }

        info!(tool = "opentofu", "Installing OpenTofu");
        // Installation logic...
        info!(tool = "opentofu", "OpenTofu installed successfully");

        Ok(())
    }
}
```

### Library Interface

**Two-Trait Strategy**: Separate detection from installation for better separation of concerns and flexibility:

- **Phase 1**: Implement `ToolDetector` trait (detection only)
- **Phase 2**: Implement `ToolInstaller` trait (installation, depends on detector)

**Rationale**:

- Detection is always straightforward and universally implementable
- Installation complexity varies - some tools may be difficult to install purely in Rust
- Better separation of concerns - detection and installation are fundamentally different
- More flexible - can implement detection without committing to implement installation

```rust
/// Trait for detecting if a tool is installed
///
/// Phase 1: Implement this trait for all tools
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

/// Trait for installing tools
///
/// Phase 2: Implement this trait for all tools
///
/// Note: Types implementing ToolInstaller should also implement ToolDetector
pub trait ToolInstaller: ToolDetector {
    /// Install the tool if not already present
    ///
    /// This method should be idempotent - it should check if the tool
    /// is already installed before attempting installation.
    fn install(&self) -> Result<(), InstallationError>;

    /// Helper method that checks and installs if needed
    fn ensure_installed(&self) -> Result<(), InstallationError> {
        if self.is_installed().map_err(|e| InstallationError::DetectionFailed {
            tool: self.name().to_string(),
            source: Box::new(e),
        })? {
            info!(tool = self.name(), "Tool already installed, skipping");
            return Ok(());
        }

        info!(tool = self.name(), "Tool not found, installing");
        self.install()
    }
}

/// Main dependency manager
pub struct DependencyManager {
    detectors: Vec<Box<dyn ToolDetector>>,
    installers: Vec<Box<dyn ToolInstaller>>,
}

impl DependencyManager {
    /// Verify all dependencies are installed (detection only)
    ///
    /// Phase 1: Fully implement using detectors
    pub fn check_all(&self) -> Result<Vec<CheckResult>, DetectionError>;

    /// Install all missing dependencies
    ///
    /// Phase 2: Implement using installers
    pub fn install_all(&self) -> Result<Vec<InstallResult>, InstallationError>;

    /// Get specific detector by dependency type
    pub fn get_detector(&self, dep: Dependency) -> &dyn ToolDetector;

    /// Get specific installer by dependency type
    ///
    /// Phase 2: Implement this method
    pub fn get_installer(&self, dep: Dependency) -> Option<&dyn ToolInstaller>;
}

/// Dependency types
pub enum Dependency {
    CargoMachete,
    OpenTofu,
    Ansible,
    Lxd,
}

/// Error types for detection operations
///
/// Phase 1: Define and use this error type
#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("Failed to detect tool '{tool}': {source}")]
    DetectionFailed {
        tool: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Detection failed for tool '{tool}': {message}")]
    CommandFailed {
        tool: String,
        message: String,
    },
}

/// Error types for installation operations
///
/// Phase 2: Define and use this error type
#[derive(Debug, Error)]
pub enum InstallationError {
    #[error("Failed to detect tool '{tool}' before installation: {source}")]
    DetectionFailed {
        tool: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Installation failed for tool '{tool}': {source}")]
    InstallationFailed {
        tool: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Download failed for tool '{tool}': {source}")]
    DownloadFailed {
        tool: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Installation not yet implemented for tool '{tool}': {message}")]
    NotImplemented {
        tool: String,
        message: String,
    },
}
```

**Example Phase 1 Implementation** (detection only):

```rust
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

    fn required_version(&self) -> Option<&str> {
        None  // Using default implementation
    }
}
```

**Example Phase 2 Implementation** (with installation):

```rust
pub struct OpenTofuInstaller {
    detector: OpenTofuDetector,
}

// First implement ToolDetector by delegating to the detector
impl ToolDetector for OpenTofuInstaller {
    fn name(&self) -> &str {
        self.detector.name()
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        self.detector.is_installed()
    }

    fn required_version(&self) -> Option<&str> {
        self.detector.required_version()
    }
}

// Then implement ToolInstaller
impl ToolInstaller for OpenTofuInstaller {
    fn install(&self) -> Result<(), InstallationError> {
        info!(tool = "opentofu", "Installing OpenTofu");

        // Convert bash script logic to Rust:
        // 1. Download OpenTofu binary
        // 2. Extract archive
        // 3. Move to system path
        // 4. Verify installation

        // Implementation here...

        info!(tool = "opentofu", "OpenTofu installed successfully");
        Ok(())
    }

    // Can use default ensure_installed() implementation
}
```

### Binary Interface

The binary should support two modes:

#### Mode 1: Single Binary with Subcommands (Recommended)

```bash
# Check all dependencies
dependency-installer check

# Install all dependencies
dependency-installer install

# Check specific dependency
dependency-installer check --tool opentofu

# Install specific dependency
dependency-installer install --tool ansible

# List available tools
dependency-installer list

# Verbose output
dependency-installer check --verbose
```

**Benefits**:

- Single binary to manage
- Easier to discover available tools
- Consistent interface
- Better for CI workflows

#### Mode 2: Separate Binaries per Tool (Alternative)

```bash
# Separate binary per tool
install-cargo-machete
install-opentofu
install-ansible
install-lxd
```

**Benefits**:

- More granular control
- Can be used independently
- Simpler implementation per tool

**Recommendation**: Start with **Mode 1** (single binary with subcommands) for better discoverability and maintainability.

### CLI Arguments

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dependency-installer")]
#[command(about = "Manage development dependencies", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if dependencies are installed
    Check {
        /// Specific tool to check (if omitted, checks all)
        #[arg(short, long)]
        tool: Option<String>,
    },

    /// Install missing dependencies
    Install {
        /// Specific tool to install (if omitted, installs all missing)
        #[arg(short, long)]
        tool: Option<String>,

        /// Force reinstallation even if already installed
        #[arg(short, long)]
        force: bool,
    },

    /// List available tools
    List,
}
```

### Detection Logic

Each installer implements detection:

```rust
impl ToolInstaller for OpenTofuInstaller {
    fn is_installed(&self) -> Result<bool, InstallerError> {
        // Check if command exists
        let output = Command::new("which")
            .arg("tofu")
            .output()
            .map_err(|e| InstallerError::DetectionFailed {
                tool: "opentofu".to_string(),
                source: e
            })?;

        Ok(output.status.success())
    }

    fn name(&self) -> &str {
        "OpenTofu"
    }

    fn install(&self) -> Result<(), InstallerError> {
        // Download and install OpenTofu
        // Convert bash logic to Rust
        todo!("Implement installation")
    }
}
```

## Implementation Plan

**Scope**: This issue covers only the creation of the dependency installation package. Integration with E2E tests and CI workflows are separate issues.

### Commit Strategy

**Minimum requirement**: Commit at least once at the end of each phase when all acceptance criteria for that phase are met.

**Recommended approach**: Commit more frequently for small, logical steps whenever:

- Tests pass for the current work
- The change is production-ready (could be deployed safely)
- You've completed a logical unit of work (e.g., implemented one detector, added one test)

**Benefits of frequent commits**:

- Easier to review changes in smaller chunks
- Safer rollback points if something goes wrong
- Better git history that tells the story of development
- Follows continuous integration principles

**Example commit flow for Phase 1**:

1. "feat: create dependency-installer package structure" (initial setup)
2. "feat: add ToolDetector trait and DetectionError type" (core abstractions)
3. "feat: implement CargoMacheteDetector with tests" (first detector)
4. "feat: implement OpenTofuDetector with tests" (second detector)
5. "feat: implement AnsibleDetector and LxdDetector with tests" (remaining detectors)
6. "feat: implement DependencyManager detection functionality" (manager integration)

Each commit should pass all checks (`./scripts/pre-commit.sh`) before pushing.

### Phase 1: Create Package Structure and Detection Logic (2-3 hours)

**Goal**: Set up the package structure and implement tool detection using the `ToolDetector` trait.

**Implementation Strategy**: Define and implement the `ToolDetector` trait for all tools. The `ToolInstaller` trait will be defined in Phase 4.

- [ ] Create `packages/dependency-installer/` directory structure
- [ ] Add package to workspace `Cargo.toml`
- [ ] Add dependencies: `clap`, `tracing`, `tracing-subscriber`, `anyhow`, `thiserror`
- [ ] Implement `init_tracing()` following linting package pattern
- [ ] Define `ToolDetector` trait with detection methods only
- [ ] Define `DetectionError` error type
- [ ] Create detector structs for all four tools:
  - [ ] `CargoMacheteDetector`
  - [ ] `OpenTofuDetector`
  - [ ] `AnsibleDetector`
  - [ ] `LxdDetector`
- [ ] For each detector, implement `ToolDetector` trait:
  - [ ] `name()` - Return tool name
  - [ ] `is_installed()` - Check if command exists (using `which` or similar)
  - [ ] `required_version()` - Return `None` (can override later if needed)
- [ ] Implement `DependencyManager` with detection functionality:
  - [ ] `check_all()` - Verify all dependencies
  - [ ] `get_detector()` - Get specific detector
- [ ] Add structured logging to all detection operations (info level)
- [ ] Write unit tests for detection logic (mock command execution)

**Note**: This phase only implements detection. The CLI binary and Docker testing come in subsequent phases.

### Phase 2: Create CLI Binary with Check Command (2-3 hours)

**Goal**: Build the command-line interface that wraps the detection functionality, making it testable in Docker containers.

**Why CLI First**: Having a working binary allows us to test it directly in Docker containers (Phase 3) rather than writing complex test harness code around the library. This is more realistic since users will run the binary, not call library functions directly.

**Implementation Strategy**: Create a minimal but functional CLI with the `check` command. The `install` command will be added in Phase 4 after we have Docker testing infrastructure.

- [ ] Implement main binary at `src/bin/dependency-installer.rs`
- [ ] Initialize tracing in main binary (call `init_tracing()`)
- [ ] Define CLI structure with clap:
  - [ ] Add `check` subcommand (verify all or specific tool is installed)
  - [ ] Add `list` subcommand (show available tools)
  - [ ] Add `--verbose` flag for detailed output
  - [ ] Add `--tool` option for checking specific tools
- [ ] Implement `check` command functionality:
  - [ ] Create `DependencyManager` instance with all detectors
  - [ ] Call `check_all()` or `get_detector(tool).is_installed()`
  - [ ] Format output clearly (tool name: installed/not installed)
  - [ ] Exit with appropriate exit codes (0 = all installed, 1 = missing tools)
- [ ] Implement `list` command functionality:
  - [ ] Show all available tools with their current status
  - [ ] Include tool names and whether they're installed
- [ ] Add help messages and usage examples to CLI
- [ ] Test binary manually:
  - [ ] Run `dependency-installer check` (should detect installed tools)
  - [ ] Run `dependency-installer check --tool opentofu` (check specific tool)
  - [ ] Run `dependency-installer list` (show all tools)
  - [ ] Verify logging output is clear and informative
  - [ ] Test with `--verbose` flag

**Example Binary Usage After Phase 2**:

```bash
# Check all dependencies
$ dependency-installer check
Checking dependencies...
✓ cargo-machete: installed
✗ OpenTofu: not installed
✗ Ansible: not installed
✗ LXD: not installed

Missing 3 out of 4 required dependencies

# Check specific tool
$ dependency-installer check --tool opentofu
✗ OpenTofu: not installed

# List all tools
$ dependency-installer list
Available tools:
- cargo-machete (installed)
- OpenTofu (not installed)
- Ansible (not installed)
- LXD (not installed)
```

**Note**: The `install` subcommand will be added in Phase 4 after Docker infrastructure is ready for testing installations.

### Phase 3: Create Docker-Based Test Infrastructure (2-3 hours)

**Goal**: Set up Docker container infrastructure to verify the binary works correctly in clean environments where tools are missing.

**Why Docker**: Testing when tools are already installed on the developer's machine only verifies the "found" case. Docker provides clean Ubuntu images where we can test the "not found" case reliably. With the binary ready (Phase 2), we can now simply run it inside containers.

**Implementation Strategy**: Create Docker images and simple test utilities that execute the binary inside containers. No complex library integration needed - just run the binary and verify its output.

**Code Reuse Strategy**: The main project has existing Docker container helpers in `src/testing/e2e/containers/` that follow excellent patterns (builder pattern, type safety, testcontainers integration). Rather than creating dependencies between the new package and the main project, **copy and adapt these helpers into the new package** to keep it fully decoupled.

**Existing Helpers to Adapt** (from `src/testing/e2e/containers/`):

- `config_builder.rs` - Builder pattern for container configuration
- `image_builder.rs` - Build Docker images from Dockerfiles
- `executor.rs` - Trait for executing commands in containers

**Do NOT Copy**:

- `provisioned.rs` - Specific to provisioned instances (we'll create our own container type)

- [ ] Create `packages/dependency-installer/docker/` directory structure
- [ ] Create minimal Ubuntu-based Dockerfile for testing:
  - [ ] Location: `packages/dependency-installer/docker/Dockerfile`
  - [ ] Base: `ubuntu:24.04` (minimal LTS image)
  - [ ] Install only essential build tools (curl, ca-certificates, build-essential)
  - [ ] Do NOT pre-install target tools (OpenTofu, Ansible, LXD, cargo-machete)
  - [ ] Set up Rust toolchain (needed to build the binary)
  - [ ] Copy binary into container or build it inside
- [ ] Copy and adapt container helpers into package's test utilities:
  - [ ] Create `packages/dependency-installer/src/testing/containers/` module structure
  - [ ] Copy and adapt `ContainerConfigBuilder` from main project (simplify if needed)
  - [ ] Copy and adapt `ContainerImageBuilder` from main project (simplify if needed)
  - [ ] Copy and adapt `ContainerExecutor` trait from main project
  - [ ] Create `DependencyTesterContainer` following the pattern from `provisioned.rs`:
    - State machine pattern (Stopped → Running → Stopped)
    - Uses `testcontainers` crate for container lifecycle
    - Implements `ContainerExecutor` trait for running commands
    - Provides helpers for running the binary and capturing output
- [ ] Create test harness utilities:
  - [ ] `build_test_image()` - Build Docker image for testing
  - [ ] `run_binary_in_container()` - Execute the binary with arguments
  - [ ] `assert_tool_not_found()` - Verify binary reports tool as missing
  - [ ] `assert_tool_found()` - Verify binary reports tool as installed (for Phase 4)
- [ ] Document Docker testing approach in package README
- [ ] Write integration tests for the binary:
  - [ ] Test `check` command in clean container (all tools should be reported as missing)
  - [ ] Test `check --tool opentofu` in clean container (should report not installed)
  - [ ] Test `list` command in clean container (should show all tools as not installed)
  - [ ] Verify exit codes are correct (1 when tools are missing)
- [ ] Add structured logging to test utilities
- [ ] Ensure tests can run both locally and in CI

**Testing Binary in Docker** (simpler than library testing):

```rust
// Example integration test structure
#[test]
fn it_should_detect_missing_tools_in_clean_container() {
    let container = build_and_start_clean_container();

    // Simply run the binary - no complex library integration needed
    let output = container.execute_binary(&["check"]);

    assert!(output.contains("OpenTofu: not installed"));
    assert!(output.contains("Ansible: not installed"));
    assert!(output.contains("LXD: not installed"));
    assert!(output.contains("cargo-machete: not installed"));
    assert_eq!(output.exit_code, 1); // Missing dependencies
}

#[test]
fn it_should_check_specific_tool() {
    let container = build_and_start_clean_container();

    let output = container.execute_binary(&["check", "--tool", "opentofu"]);

    assert!(output.contains("OpenTofu: not installed"));
    assert_eq!(output.exit_code, 1);
}
```

**Notes**:

- Use `testcontainers` crate for Rust-based container management (similar to Docker E2E tests)
- Keep Docker images minimal to speed up test execution
- This infrastructure will be extended in Phase 4 to test installation logic
- Testing the binary is simpler and more realistic than testing library APIs

### Phase 4: Implement Installation Logic (4-5 hours)

**Goal**: Define the `ToolInstaller` trait, implement installation logic for all tools by converting bash scripts to Rust, and add the `install` command to the CLI binary.

**Implementation Strategy**: Create installer structs that implement both `ToolDetector` (by composition/delegation) and `ToolInstaller` traits. Use Docker test infrastructure from Phase 3 to verify installation works in clean environments by running the binary.

- [ ] Define `ToolInstaller` trait (extends `ToolDetector`)
- [ ] Define `InstallationError` error type
- [ ] Create installer structs for all four tools:
  - [ ] `CargoMacheteInstaller` (contains `CargoMacheteDetector`)
  - [ ] `OpenTofuInstaller` (contains `OpenTofuDetector`)
  - [ ] `AnsibleInstaller` (contains `AnsibleDetector`)
  - [ ] `LxdInstaller` (contains `LxdDetector`)
- [ ] For each installer, implement `ToolDetector` trait (delegate to detector)
- [ ] For each installer, implement `ToolInstaller` trait:
  - [ ] Convert `install-opentofu.sh` to Rust (download, extract, install to path)
  - [ ] Convert `install-ansible.sh` to Rust (package manager or pip)
  - [ ] Convert `install-lxd-ci.sh` to Rust (install and configure for CI)
  - [ ] Implement cargo-machete installer (`cargo install cargo-machete`)
- [ ] Update `DependencyManager` with installation functionality:
  - [ ] `install_all()` - Install all missing dependencies
  - [ ] `get_installer()` - Get specific installer (returns Option)
- [ ] Add structured logging to all installation operations (info, warn, error)
- [ ] Ensure installations are idempotent (use `ensure_installed()` helper)
- [ ] Add progress indicators for long-running installations
- [ ] Add `install` command to the CLI binary:
  - [ ] Add `install` subcommand with clap
  - [ ] Support `--tool` option for installing specific tools
  - [ ] Support `--force` flag for reinstallation
  - [ ] Call installers through `DependencyManager`
  - [ ] Provide clear output and progress indicators
  - [ ] Exit with appropriate exit codes
- [ ] Update `list` command to show installation status
- [ ] Test binary manually with new install command:
  - [ ] Run `dependency-installer install` (install all missing)
  - [ ] Run `dependency-installer install --tool opentofu` (install specific)
  - [ ] Run `dependency-installer install --force` (reinstall)
  - [ ] Verify idempotency (run install twice)
- [ ] Write integration tests using Docker infrastructure (run binary in containers):
  - [ ] Test `install` command in clean container (tools should be installed successfully)
  - [ ] Verify idempotency (run `install` twice, both should succeed)
  - [ ] Test `check` after `install` (tools should now be found)
  - [ ] Test `install --tool opentofu` (install specific tool)
  - [ ] Verify exit codes are correct

**Example Binary Usage After Phase 4**:

```bash
# Install all missing dependencies
$ dependency-installer install
Installing dependencies...
✓ cargo-machete: already installed
→ OpenTofu: downloading and installing...
✓ OpenTofu: installed successfully
→ Ansible: installing via apt...
✓ Ansible: installed successfully
→ LXD: installing and configuring...
✓ LXD: installed successfully

All dependencies installed successfully

# Install specific tool
$ dependency-installer install --tool opentofu
→ OpenTofu: downloading and installing...
✓ OpenTofu: installed successfully
```

**Note**: Installers implement both `ToolDetector` (via delegation) and `ToolInstaller` traits. Testing uses the Docker infrastructure from Phase 3, running the binary directly in containers.

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Package Structure**:

- [ ] New package exists at `packages/dependency-installer/`
- [ ] Package is registered in workspace `Cargo.toml`
- [ ] Package has library and binary targets
- [ ] Package follows conventions from `packages/linting/`
- [ ] Uses tracing crate for structured logging
- [ ] Logging follows linting package pattern

**Docker Test Infrastructure**:

- [ ] Docker test infrastructure exists in `packages/dependency-installer/docker/`
- [ ] Minimal Ubuntu-based Dockerfile is configured correctly
- [ ] Test harness utilities are implemented and documented
- [ ] Docker Compose configuration exists (if needed)
- [ ] Initial integration test skeleton passes

**Library Functionality**:

- [ ] `ToolDetector` trait is implemented for all four tools
- [ ] `ToolInstaller` trait is implemented for all four tools
- [ ] Detection logic works correctly (can detect installed tools)
- [ ] Installation logic works correctly (can install missing tools)
- [ ] Installations are idempotent (safe to run multiple times)
- [ ] Library can be used programmatically from tests

**Integration Testing with Docker**:

- [ ] Detection tests pass in clean Docker containers
- [ ] Installation tests pass in clean Docker containers
- [ ] Idempotency tests pass (install twice, both succeed)
- [ ] Post-installation detection tests pass (tools are found after install)

**Binary Functionality**:

- [ ] Binary has `check`, `install`, and `list` subcommands
- [ ] Can check all dependencies or specific tool
- [ ] Can install all dependencies or specific tool
- [ ] Provides clear output and progress indicators
- [ ] Error messages are actionable and helpful
- [ ] Help text is clear and includes examples

**Documentation**:

- [ ] Package README.md explains usage (library and binary)
- [ ] Code includes rustdoc comments for public APIs
- [ ] Error messages follow project conventions
- [ ] Binary help text explains all subcommands with examples

## Related Documentation

- [packages/linting/](../../packages/linting/) - Reference for package structure
- [docs/e2e-testing.md](../e2e-testing.md) - E2E testing documentation
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error handling guidelines
- [scripts/setup/](../../scripts/setup/) - Current bash scripts to convert
- [GitHub: Customize Agent Environment](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-environment) - Future Copilot integration

## Notes

### Estimated Time

**10-14 hours** total:

- Phase 1: 2-3 hours (package structure and detection logic)
- Phase 2: 2-3 hours (CLI binary with check command)
- Phase 3: 2-3 hours (Docker test infrastructure)
- Phase 4: 4-5 hours (installation logic)

### Design Decisions

**Binary before Docker testing**: We implement detection logic (Phase 1), then create the CLI binary (Phase 2) before setting up Docker infrastructure (Phase 3). This allows us to test the binary directly inside containers rather than writing complex test harness code around the library. Users will run the binary, so testing the binary is more realistic than testing library APIs directly.

**Code independence via copying**: Rather than creating internal dependencies between the new package and the main project's test utilities, we copy and adapt the Docker container helpers. This keeps the package fully decoupled and independently maintainable. The helpers (`config_builder.rs`, `image_builder.rs`, `executor.rs`) are concise, well-documented, and follow established patterns.

Benefits:

- Package remains independent (could be extracted as a separate crate in the future)
- No coupling to main project's internal test utilities
- Clearer package boundaries and responsibilities
- Easier to understand and maintain in isolation

**Reusable patterns, not reusable code**: We reuse the **patterns** (builder pattern, state machine, trait-based execution) but not the code itself through dependencies. This is similar to how `packages/linting/` doesn't depend on the main project - it's a standalone package.

**Single binary vs multiple binaries**: Recommend starting with a single binary with subcommands for better discoverability and maintenance. This can be changed later if needed.

**Library + Binary pattern**: Following the `packages/linting/` pattern ensures consistency across the codebase.

**Idempotency**: All installations must be idempotent - running the installer multiple times should be safe and not cause issues.

**Agent-agnostic approach**: While this enables GitHub Copilot agent support, it's designed to work with any AI agent or human developer.

### Future Enhancements

- Add support for custom installation paths
- Support different installation methods (apt, brew, cargo, manual download)
- Add version checking and upgrade capabilities
- Cache detection results for performance
- Add dry-run mode to see what would be installed

### Testing Strategy

- **Unit tests**: Mock command execution for detection and installation logic
- **Integration tests with Docker**: Test on actual clean Ubuntu containers using `testcontainers` crate
  - Detection tests: Verify tools are correctly identified as missing
  - Installation tests: Verify tools can be installed from scratch
  - Idempotency tests: Verify multiple installations don't cause errors
  - Post-installation tests: Verify detection works after installation
- **Package verification**: Verify the package API works correctly through test scenarios

**Docker Testing Approach**: Using Docker containers solves the "already installed" problem - we can test installation in truly clean environments where no tools are pre-installed. This provides confidence that the installer works correctly for end users.

### Conversion from Bash Scripts

Each bash script in `scripts/setup/` needs to be converted to Rust:

1. **`install-opentofu.sh`**: Download OpenTofu, extract, and install to system path
2. **`install-ansible.sh`**: Install Ansible via package manager or pip
3. **`install-lxd-ci.sh`**: Install and configure LXD for CI environments
4. **cargo-machete**: Simple `cargo install cargo-machete`

Focus on maintaining the same functionality while adding better error handling and progress reporting.
