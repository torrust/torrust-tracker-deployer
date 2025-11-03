# Implement Installation Logic

**Issue**: [#117](https://github.com/torrust/torrust-tracker-deployer/issues/117)
**Parent Issue**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests  
**Depends On**: [#116](https://github.com/torrust/torrust-tracker-deployer/issues/116) - Create Docker Test Infrastructure (Issue 1-1-3)  
**Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution  
**Related**: [docs/e2e-testing.md](../e2e-testing.md)

## Overview

Implement the installation logic for all dependencies by converting existing bash scripts to Rust, add the `install` subcommand to the CLI binary, and extend the Docker test infrastructure to verify installations work correctly.

## Objectives

- [ ] Define `ToolInstaller` trait for installation abstraction
- [ ] Convert bash installation scripts to Rust implementations
- [ ] Add `install` subcommand to CLI binary
- [ ] Extend Docker tests to verify actual installation
- [ ] Test installation in clean Ubuntu 24.04 containers
- [ ] Ensure installation is idempotent and robust

## Context

This is **Phase 4** (final phase) of creating the dependency installation package. It adds actual installation capability, completing the package functionality.

### Why Installation Logic Last

Implementing installation after detection and Docker testing ensures:

1. **Detection works first** - We can test what's installed before we install it
2. **Docker infrastructure ready** - We can test installations in isolated containers
3. **CLI foundation exists** - We just add a new subcommand to existing structure
4. **Testing is easier** - Docker containers provide clean environments for testing installations

### Dependencies

- **Requires**: Issue 1-1-3 (Docker testing infrastructure) must be completed first
- **Uses**: Detection logic from Issue 1-1-1 and CLI from Issue 1-1-2
- **Completes**: The dependency installation package is ready for E2E integration (Issue 1-2)

## 🏗️ Architecture Requirements

**DDD Layers**: Domain (ToolInstaller trait), Infrastructure (installers), Presentation (install command)  
**Module Paths**:

- `src/installer/mod.rs` - ToolInstaller trait
- `src/installer/cargo_machete.rs` - Cargo-machete installer
- `src/installer/opentofu.rs` - OpenTofu installer
- `src/installer/ansible.rs` - Ansible installer
- `src/installer/lxd.rs` - LXD installer
- `src/bin/dependency-installer.rs` - Add install subcommand

### Directory Structure After This Phase

```text
packages/dependency-installer/
├── src/
│   ├── lib.rs
│   ├── manager.rs                  # DependencyManager (uses both traits)
│   ├── detector/
│   │   ├── mod.rs
│   │   ├── cargo_machete.rs
│   │   ├── opentofu.rs
│   │   ├── ansible.rs
│   │   └── lxd.rs
│   ├── installer/                  # ← NEW in this phase
│   │   ├── mod.rs
│   │   ├── cargo_machete.rs
│   │   ├── opentofu.rs
│   │   ├── ansible.rs
│   │   └── lxd.rs
│   ├── bin/
│   │   └── dependency-installer.rs # Add install command
│   └── error.rs
├── tests/
│   ├── docker_check_command.rs
│   └── docker_install_command.rs   # ← NEW tests in this phase
└── docker/
    └── ubuntu-24.04.Dockerfile
```

## Specifications

### ToolInstaller Trait

Define the trait in `src/installer/mod.rs`:

```rust
use async_trait::async_trait;
use crate::error::InstallationError;
use crate::detector::Dependency;

/// Trait for installing development dependencies
#[async_trait]
pub trait ToolInstaller: Send + Sync {
    /// Get the name of this installer
    fn name(&self) -> &str;

    /// Get the dependency this installer handles
    fn dependency(&self) -> Dependency;

    /// Install the tool
    ///
    /// This should be idempotent - calling it multiple times should be safe.
    /// If the tool is already installed, this should succeed without error.
    async fn install(&self) -> Result<(), InstallationError>;

    /// Check if the tool requires sudo/admin privileges
    fn requires_sudo(&self) -> bool {
        false
    }
}

// Re-export installer implementations
pub use cargo_machete::CargoMacheteInstaller;
pub use opentofu::OpenTofuInstaller;
pub use ansible::AnsibleInstaller;
pub use lxd::LxdInstaller;
```

### Example Installer: Cargo-machete

Convert `scripts/setup/install-cargo-machete.sh` to Rust in `src/installer/cargo_machete.rs`:

```rust
use async_trait::async_trait;
use crate::installer::ToolInstaller;
use crate::detector::Dependency;
use crate::error::InstallationError;
use std::process::Command;
use tracing::{info, debug};

pub struct CargoMacheteInstaller;

impl CargoMacheteInstaller {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolInstaller for CargoMacheteInstaller {
    fn name(&self) -> &str {
        "cargo-machete"
    }

    fn dependency(&self) -> Dependency {
        Dependency::CargoMachete
    }

    async fn install(&self) -> Result<(), InstallationError> {
        info!("Installing cargo-machete");

        // Equivalent to: cargo install cargo-machete
        let output = Command::new("cargo")
            .args(["install", "cargo-machete"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                tool: self.name().to_string(),
                command: "cargo install cargo-machete".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!("cargo install stderr: {}", stderr);

            // cargo install exits with 0 even if already installed
            // So any non-zero exit is a real error
            return Err(InstallationError::InstallFailed {
                tool: self.name().to_string(),
                message: stderr.to_string(),
            });
        }

        info!("Successfully installed cargo-machete");
        Ok(())
    }
}
```

### Example Installer: OpenTofu

Convert `scripts/setup/install-opentofu.sh` to Rust in `src/installer/opentofu.rs`:

```rust
use async_trait::async_trait;
use crate::installer::ToolInstaller;
use crate::detector::Dependency;
use crate::error::InstallationError;
use std::process::Command;
use tracing::{info, debug};

pub struct OpenTofuInstaller;

impl OpenTofuInstaller {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolInstaller for OpenTofuInstaller {
    fn name(&self) -> &str {
        "OpenTofu"
    }

    fn dependency(&self) -> Dependency {
        Dependency::OpenTofu
    }

    async fn install(&self) -> Result<(), InstallationError> {
        info!("Installing OpenTofu");

        // Step 1: Download installer script
        self.download_installer_script().await?;

        // Step 2: Make script executable
        self.make_executable().await?;

        // Step 3: Run installer
        self.run_installer().await?;

        // Step 4: Clean up
        self.cleanup().await?;

        info!("Successfully installed OpenTofu");
        Ok(())
    }

    fn requires_sudo(&self) -> bool {
        true  // OpenTofu installation requires sudo
    }
}

impl OpenTofuInstaller {
    async fn download_installer_script(&self) -> Result<(), InstallationError> {
        debug!("Downloading OpenTofu installer script");

        let output = Command::new("curl")
            .args([
                "--proto", "=https",
                "--tlsv1.2",
                "-fsSL",
                "https://get.opentofu.org/install-opentofu.sh",
                "-o", "/tmp/install-opentofu.sh"
            ])
            .output()
            .map_err(|e| InstallationError::DownloadFailed {
                tool: self.name().to_string(),
                url: "https://get.opentofu.org/install-opentofu.sh".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            return Err(InstallationError::DownloadFailed {
                tool: self.name().to_string(),
                url: "https://get.opentofu.org/install-opentofu.sh".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    async fn make_executable(&self) -> Result<(), InstallationError> {
        debug!("Making installer script executable");

        Command::new("chmod")
            .args(["+x", "/tmp/install-opentofu.sh"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                tool: self.name().to_string(),
                command: "chmod +x /tmp/install-opentofu.sh".to_string(),
                source: e,
            })?;

        Ok(())
    }

    async fn run_installer(&self) -> Result<(), InstallationError> {
        debug!("Running OpenTofu installer");

        let output = Command::new("sudo")
            .args([
                "/tmp/install-opentofu.sh",
                "--install-method", "deb"
            ])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                tool: self.name().to_string(),
                command: "sudo /tmp/install-opentofu.sh".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InstallationError::InstallFailed {
                tool: self.name().to_string(),
                message: stderr.to_string(),
            });
        }

        Ok(())
    }

    async fn cleanup(&self) -> Result<(), InstallationError> {
        debug!("Cleaning up installer script");

        let _ = Command::new("rm")
            .args(["-f", "/tmp/install-opentofu.sh"])
            .output();

        Ok(())
    }
}
```

### Update DependencyManager

Update `src/manager.rs` to support installation:

```rust
use crate::detector::{Dependency, ToolDetector};
use crate::installer::ToolInstaller;
use crate::error::{DetectionError, InstallationError};

pub struct DependencyManager {
    detectors: HashMap<Dependency, Box<dyn ToolDetector>>,
    installers: HashMap<Dependency, Box<dyn ToolInstaller>>,
}

impl DependencyManager {
    pub fn new() -> Self {
        let mut detectors = HashMap::new();
        let mut installers = HashMap::new();

        // Register detectors (from Phase 1)
        detectors.insert(Dependency::CargoMachete, Box::new(CargoMacheteDetector::new()));
        // ... other detectors

        // Register installers (Phase 4)
        installers.insert(Dependency::CargoMachete, Box::new(CargoMacheteInstaller::new()));
        installers.insert(Dependency::OpenTofu, Box::new(OpenTofuInstaller::new()));
        installers.insert(Dependency::Ansible, Box::new(AnsibleInstaller::new()));
        installers.insert(Dependency::Lxd, Box::new(LxdInstaller::new()));

        Self { detectors, installers }
    }

    /// Install a specific dependency
    pub async fn install(&self, dep: Dependency) -> Result<(), InstallationError> {
        let installer = self.installers.get(&dep)
            .ok_or_else(|| InstallationError::InstallerNotFound {
                tool: format!("{:?}", dep),
            })?;

        installer.install().await
    }

    /// Install all dependencies
    pub async fn install_all(&self) -> Result<Vec<InstallResult>, InstallationError> {
        let mut results = Vec::new();

        for dep in Dependency::all() {
            let result = match self.install(dep).await {
                Ok(_) => InstallResult {
                    dependency: dep,
                    success: true,
                    error: None,
                },
                Err(e) => InstallResult {
                    dependency: dep,
                    success: false,
                    error: Some(e.to_string()),
                },
            };

            results.push(result);
        }

        Ok(results)
    }
}

pub struct InstallResult {
    pub dependency: Dependency,
    pub success: bool,
    pub error: Option<String>,
}
```

### Add Install Command to CLI

Update `src/bin/dependency-installer.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    /// Check if dependencies are installed
    Check {
        #[arg(short, long)]
        tool: Option<String>,
    },

    /// List all available tools and their status
    List,

    /// Install dependencies
    Install {
        /// Specific tool to install (if omitted, installs all)
        #[arg(short, long)]
        tool: Option<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

fn handle_install(
    manager: &DependencyManager,
    tool: Option<String>,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match tool {
        Some(tool_name) => install_specific_tool(manager, &tool_name, yes),
        None => install_all_tools(manager, yes),
    }
}

async fn install_all_tools(
    manager: &DependencyManager,
    skip_confirm: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !skip_confirm {
        println!("This will install all required dependencies:");
        println!("- cargo-machete");
        println!("- OpenTofu");
        println!("- Ansible");
        println!("- LXD");
        println!();
        print!("Continue? [y/N] ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Installation cancelled");
            return Ok(());
        }
    }

    println!("Installing dependencies...\n");

    let results = manager.install_all().await?;
    let mut failed_count = 0;

    for result in &results {
        if result.success {
            println!("✓ {}: installed successfully", result.dependency);
        } else {
            println!("✗ {}: installation failed", result.dependency);
            if let Some(error) = &result.error {
                println!("  Error: {}", error);
            }
            failed_count += 1;
        }
    }

    println!();
    if failed_count > 0 {
        println!("Failed to install {} out of {} dependencies", failed_count, results.len());
        std::process::exit(1);
    } else {
        println!("All dependencies installed successfully");
        Ok(())
    }
}
```

### Extended Docker Tests

Create `tests/docker_install_command.rs`:

```rust
//! Integration tests for the install command using Docker containers.

use testcontainers::clients::Cli as DockerCli;

mod containers;
use containers::ubuntu::UbuntuContainer;

/// Test installing cargo-machete in a clean container
#[tokio::test]
async fn test_install_cargo_machete() {
    let docker = DockerCli::default();
    let binary_path = get_binary_path();

    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .start();

    // Verify cargo-machete is not installed
    let check_before = container.exec(&["dependency-installer", "check", "--tool", "cargo-machete"]);
    assert!(check_before.contains("not installed"));

    // Install cargo-machete
    let install_output = container.exec(&["dependency-installer", "install", "--tool", "cargo-machete", "-y"]);
    assert!(install_output.contains("installed successfully"));

    // Verify it's now installed
    let check_after = container.exec(&["dependency-installer", "check", "--tool", "cargo-machete"]);
    assert!(check_after.contains("installed"));

    let exit_code = container.exec_with_exit_code(&["dependency-installer", "check", "--tool", "cargo-machete"]);
    assert_eq!(exit_code, 0);
}

/// Test installing OpenTofu (requires sudo)
#[tokio::test]
async fn test_install_opentofu() {
    let docker = DockerCli::default();
    let binary_path = get_binary_path();

    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .with_sudo()  // Enable sudo for this test
        .start();

    // Install OpenTofu
    let install_output = container.exec(&["dependency-installer", "install", "--tool", "opentofu", "-y"]);
    assert!(install_output.contains("installed successfully"));

    // Verify installation
    let check_output = container.exec(&["dependency-installer", "check", "--tool", "opentofu"]);
    assert!(check_output.contains("installed"));
}

/// Test that installation is idempotent
#[tokio::test]
async fn test_install_idempotent() {
    let docker = DockerCli::default();
    let binary_path = get_binary_path();

    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .start();

    // Install once
    container.exec(&["dependency-installer", "install", "--tool", "cargo-machete", "-y"]);

    // Install again - should succeed without error
    let output = container.exec(&["dependency-installer", "install", "--tool", "cargo-machete", "-y"]);
    assert!(output.contains("installed successfully") || output.contains("already installed"));

    let exit_code = container.exec_with_exit_code(&["dependency-installer", "install", "--tool", "cargo-machete", "-y"]);
    assert_eq!(exit_code, 0);
}
```

## Implementation Tasks

### Installer Trait and Error Types

- [ ] Create `src/installer/mod.rs`
- [ ] Define `ToolInstaller` trait with `install()` method
- [ ] Add `requires_sudo()` method to trait
- [ ] Define `InstallationError` enum in `src/error.rs`:
  - [ ] `CommandFailed` variant
  - [ ] `DownloadFailed` variant
  - [ ] `InstallFailed` variant
  - [ ] `InstallerNotFound` variant
  - [ ] Implement `Display` and `Error` traits

### Convert Bash Scripts to Rust Installers

- [ ] Analyze each bash script in `scripts/setup/`:
  - [ ] `install-cargo-machete.sh`
  - [ ] `install-opentofu.sh`
  - [ ] `install-ansible.sh`
  - [ ] `install-lxd.sh`
- [ ] Create `src/installer/cargo_machete.rs`
  - [ ] Implement `CargoMacheteInstaller`
  - [ ] Convert script logic to Rust commands
  - [ ] Add error handling
  - [ ] Add logging
- [ ] Create `src/installer/opentofu.rs`
  - [ ] Implement `OpenTofuInstaller`
  - [ ] Handle multi-step installation (download, chmod, execute)
  - [ ] Implement cleanup
  - [ ] Mark as requiring sudo
- [ ] Create `src/installer/ansible.rs`
  - [ ] Implement `AnsibleInstaller`
  - [ ] Convert apt-get commands
  - [ ] Handle sudo requirements
- [ ] Create `src/installer/lxd.rs`
  - [ ] Implement `LxdInstaller`
  - [ ] Handle snap installation
  - [ ] Handle group configuration

### Update DependencyManager

- [ ] Add `installers` HashMap to `DependencyManager`
- [ ] Register all installers in `new()`
- [ ] Implement `install(&self, dep: Dependency)` method
- [ ] Implement `install_all(&self)` method
- [ ] Add `InstallResult` struct for results
- [ ] Test that both detection and installation work together

### Add Install Command to CLI

- [ ] Add `Install` variant to `Commands` enum
- [ ] Add `--yes` flag to skip confirmation
- [ ] Implement `handle_install()` function
- [ ] Implement `install_all_tools()` with confirmation prompt
- [ ] Implement `install_specific_tool()`
- [ ] Add progress indicators during installation
- [ ] Handle installation errors gracefully

### Docker Test Infrastructure Updates

- [ ] Update `tests/containers/ubuntu.rs`:
  - [ ] Add `with_sudo()` method to builder
  - [ ] Configure container to support sudo
- [ ] Create `tests/docker_install_command.rs`
- [ ] Implement test: `test_install_cargo_machete`
- [ ] Implement test: `test_install_opentofu`
- [ ] Implement test: `test_install_ansible`
- [ ] Implement test: `test_install_lxd`
- [ ] Implement test: `test_install_idempotent`
- [ ] Implement test: `test_install_all`

### Testing

- [ ] Unit test each installer independently:
  - [ ] Mock command execution
  - [ ] Test error handling
  - [ ] Test logging output
- [ ] Integration test in Docker:
  - [ ] Test each tool installation
  - [ ] Test idempotency
  - [ ] Test error scenarios
  - [ ] Verify installations with check command
- [ ] Run all tests: `cargo test`
- [ ] Manual testing:
  - [ ] Build binary: `cargo build --bin dependency-installer`
  - [ ] Test install command: `dependency-installer install --tool cargo-machete -y`
  - [ ] Verify installation worked

### Documentation

- [ ] Document each installer's requirements
- [ ] Add installation examples to README
- [ ] Document sudo requirements
- [ ] Update CLI help text with install command

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Manual installation testing successful

**ToolInstaller Trait**:

- [ ] Trait is well-defined with clear contract
- [ ] All installers implement the trait
- [ ] Error handling is comprehensive
- [ ] Logging provides visibility

**Installer Implementations**:

- [ ] Cargo-machete installer works
- [ ] OpenTofu installer works
- [ ] Ansible installer works
- [ ] LXD installer works
- [ ] All installers are idempotent
- [ ] Sudo requirements are handled correctly
- [ ] Installations can be verified with check command

**CLI Install Command**:

- [ ] `install` subcommand works for all tools
- [ ] `install --tool <name>` works for specific tools
- [ ] `install --yes` skips confirmation
- [ ] Confirmation prompt is clear
- [ ] Progress output is informative
- [ ] Exit codes are correct

**Docker Tests**:

- [ ] Tests verify actual installation in containers
- [ ] Tests verify idempotency
- [ ] Tests verify installations with check command
- [ ] Tests handle both sudo and non-sudo tools
- [ ] All tests pass consistently

**DependencyManager**:

- [ ] Manager integrates detectors and installers
- [ ] `install()` method works for single tool
- [ ] `install_all()` method works for all tools
- [ ] Results are tracked and reported

**Documentation**:

- [ ] README updated with install command usage
- [ ] Each installer is documented
- [ ] Sudo requirements are documented
- [ ] Examples are clear and complete

## Example Usage After Completion

```bash
# Install all dependencies
$ dependency-installer install
This will install all required dependencies:
- cargo-machete
- OpenTofu
- Ansible
- LXD

Continue? [y/N] y

Installing dependencies...

✓ cargo-machete: installed successfully
✓ OpenTofu: installed successfully
✓ Ansible: installed successfully
✓ LXD: installed successfully

All dependencies installed successfully

# Install specific tool
$ dependency-installer install --tool opentofu -y
Installing OpenTofu...
✓ OpenTofu: installed successfully

# Verify installation
$ dependency-installer check
Checking dependencies...

✓ cargo-machete: installed
✓ OpenTofu: installed
✓ Ansible: installed
✓ LXD: installed

All dependencies are installed
```

## Related Documentation

- [Issue 1-1-3](./116-1-1-3-create-docker-test-infrastructure.md) - Docker testing infrastructure extended here
- [Issue 1-1-2](./115-1-1-2-create-cli-binary-with-check-command.md) - CLI binary extended with install command
- [Issue 1-1-1](./114-1-1-1-create-detection-logic-package.md) - Detection logic used to verify installations
- [Parent Issue 1-1](./create-dependency-installation-package-for-e2e-tests.md) - Overall package specification
- [scripts/setup/](../../scripts/setup/) - Bash scripts to convert

## Notes

### Estimated Time

**4-5 hours** total for this phase (largest phase).

### Next Steps

After completing this phase:

1. The dependency-installer package is complete
2. **Issue 1-2**: Integrate the package with existing E2E tests
3. **Issue 1-3**: Update CI workflows to use the new binary

### Design Decisions

**Two-trait design**: Separating `ToolDetector` and `ToolInstaller` keeps concerns separated and allows reusing detection logic.

**Idempotent installations**: Each installer checks if the tool is already installed and handles re-installation gracefully.

**Confirmation prompt**: Following Unix conventions, we ask for confirmation before installing all tools but provide `-y` flag for automation.

**Docker testing**: Testing in Docker ensures installations work in clean environments and don't depend on developer machine state.

### Bash Script Conversion Strategy

For each bash script:

1. **Understand the logic** - Read and understand what the script does
2. **Identify commands** - List all shell commands used
3. **Map to Rust** - Use `std::process::Command` for shell commands
4. **Add error handling** - Wrap each command with proper error handling
5. **Add logging** - Use tracing for visibility
6. **Test thoroughly** - Verify in Docker containers

### Testing Strategy

- **Unit tests**: Mock command execution, test error handling
- **Integration tests**: Run actual installations in Docker
- **Verification tests**: Use check command to verify installations worked
- **Idempotency tests**: Install twice, verify both succeed
- **Error tests**: Test failure scenarios (network issues, permissions, etc.)

### Installation Requirements

Some tools require sudo:

- **OpenTofu**: Requires sudo for system-wide installation
- **Ansible**: Requires sudo for apt-get
- **LXD**: Requires sudo for snap and group configuration
- **Cargo-machete**: No sudo required (user-level cargo install)

The `requires_sudo()` method in the trait documents these requirements.
