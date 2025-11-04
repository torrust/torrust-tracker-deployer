# Implement Installation Logic# Implement Installation Logic

**Issue**: [#117](https://github.com/torrust/torrust-tracker-deployer/issues/117) **Issue**: [#117](https://github.com/torrust/torrust-tracker-deployer/issues/117)

**Parent Issue**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests **Parent Issue**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests

**Depends On**: [#116](https://github.com/torrust/torrust-tracker-deployer/issues/116) - Create Docker Test Infrastructure (Issue 1-1-3) **Depends On**: [#116](https://github.com/torrust/torrust-tracker-deployer/issues/116) - Create Docker Test Infrastructure (Issue 1-1-3)

**Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution **Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution

**Related**: [docs/e2e-testing.md](../e2e-testing.md)**Related**: [docs/e2e-testing.md](../e2e-testing.md)

## Overview## Overview

Implement the installation logic for all dependencies by converting existing bash scripts to Rust, add the `install` subcommand to the CLI binary, and extend the Docker test infrastructure to verify installations work correctly.Implement the installation logic for all dependencies by converting existing bash scripts to Rust, add the \`install\` subcommand to the CLI binary, and extend the Docker test infrastructure to verify installations work correctly.

**Design Note**: This package uses **structured logging only** (via the tracing crate) for automation-focused design. There are no user-facing `println!()` statements - all output is through structured logs suitable for CI/CD pipelines.**Note**: This package uses **structured logging only** (via the tracing crate) for automation-focused design. There are no user-facing \`println!()\` statements - all output is through structured logs suitable for CI/CD pipelines.

## Objectives## Objectives

- [ ] Define `DependencyInstaller` trait for installation abstraction- [ ] Define \`DependencyInstaller\` trait for installation abstraction

- [ ] Convert bash installation scripts to Rust implementations- [ ] Convert bash installation scripts to Rust implementations

- [ ] Add `install` subcommand to CLI binary using handler-based architecture- [ ] Add \`install\` subcommand to CLI binary using handler-based architecture

- [ ] Extend Docker tests to verify actual installation- [ ] Extend Docker tests to verify actual installation

- [ ] Test installation in clean Ubuntu 24.04 containers- [ ] Test installation in clean Ubuntu 24.04 containers

- [ ] Ensure installation is idempotent and robust- [ ] Ensure installation is idempotent and robust

- [ ] Use structured logging (tracing) for observability- [ ] Use structured logging (tracing) for observability

## Context## Context

This is **Phase 4** (final phase) of creating the dependency installation package. It adds actual installation capability, completing the package functionality.This is **Phase 4** (final phase) of creating the dependency installation package. It adds actual installation capability, completing the package functionality.

### Why Installation Logic Last### Why Installation Logic Last

Implementing installation after detection and Docker testing ensures:Implementing installation after detection and Docker testing ensures:

1. **Detection works first** - We can test what's installed before we install it1. **Detection works first** - We can test what's installed before we install it

2. **Docker infrastructure ready** - We can test installations in isolated containers2. **Docker infrastructure ready** - We can test installations in isolated containers

3. **CLI foundation exists** - We just add a new subcommand to existing structure3. **CLI foundation exists** - We just add a new subcommand to existing structure

4. **Testing is easier** - Docker containers provide clean environments for testing installations4. **Testing is easier** - Docker containers provide clean environments for testing installations

### Dependencies### Dependencies

- **Requires**: Issue 1-1-3 (Docker testing infrastructure) must be completed first- **Requires**: Issue 1-1-3 (Docker testing infrastructure) must be completed first

- **Uses**: Detection logic from Issue 1-1-1 and CLI from Issue 1-1-2- **Uses**: Detection logic from Issue 1-1-1 and CLI from Issue 1-1-2

- **Completes**: The dependency installation package is ready for E2E integration (Issue 1-2)- **Completes**: The dependency installation package is ready for E2E integration (Issue 1-2)

## 🏗️ Architecture Requirements## 🏗️ Architecture Requirements

**DDD Layers**: Domain (DependencyInstaller trait), Infrastructure (installers), Presentation (install command handler) **DDD Layers**: Domain (DependencyInstaller trait), Infrastructure (installers), Presentation (install command handler)

**Module Paths**:**Module Paths**:

- `src/installer/mod.rs` - DependencyInstaller trait and error types- \`src/installer/mod.rs\` - DependencyInstaller trait

- `src/installer/cargo_machete.rs` - Cargo-machete installer- \`src/installer/cargo_machete.rs\` - Cargo-machete installer

- `src/installer/opentofu.rs` - OpenTofu installer- \`src/installer/opentofu.rs\` - OpenTofu installer

- `src/installer/ansible.rs` - Ansible installer- \`src/installer/ansible.rs\` - Ansible installer

- `src/installer/lxd.rs` - LXD installer- \`src/installer/lxd.rs\` - LXD installer

- `src/handlers/install.rs` - Install command handler- \`src/handlers/install.rs\` - Install command handler

- `src/bin/dependency-installer.rs` - Update with install subcommand- \`src/bin/dependency-installer.rs\` - Update with install subcommand

### Directory Structure After This PhaseSee the full specification in the backup file for complete implementation details.

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
│   │   ├── mod.rs                  # Trait + error types
│   │   ├── cargo_machete.rs
│   │   ├── opentofu.rs
│   │   ├── ansible.rs
│   │   └── lxd.rs
│   ├── handlers/                   # Existing, extend with install
│   │   ├── mod.rs
│   │   ├── check.rs                # Existing
│   │   ├── list.rs                 # Existing
│   │   └── install.rs              # ← NEW
│   ├── bin/
│   │   └── dependency-installer.rs # Add install command
│   ├── command.rs                  # Existing utilities
│   ├── cli.rs                      # Update with Install command
│   ├── app.rs                      # Update to handle install
│   └── logging.rs                  # Existing
├── tests/
│   ├── docker_check_command.rs     # Existing
│   └── docker_install_command.rs   # ← NEW tests in this phase
└── docker/
    └── ubuntu-24.04.Dockerfile     # May need sudo support
```

## Specifications

### DependencyInstaller Trait

Define the trait in `src/installer/mod.rs`:

```rust
use async_trait::async_trait;
use crate::Dependency;

/// Trait for installing development dependencies
#[async_trait]
pub trait DependencyInstaller: Send + Sync {
    /// Get the name of this installer for logging
    fn name(&self) -> &str;

    /// Get the dependency this installer handles
    fn dependency(&self) -> Dependency;

    /// Install the dependency
    ///
    /// This should be idempotent - calling it multiple times should be safe.
    /// If the dependency is already installed, this should succeed without error.
    ///
    /// # Errors
    ///
    /// Returns an error if installation fails
    async fn install(&self) -> Result<(), InstallationError>;

    /// Check if the dependency requires sudo/admin privileges
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

### InstallationError Type

Add to `src/installer/mod.rs`:

```rust
use thiserror::Error;

/// Error types for installation operations
#[derive(Debug, Error)]
pub enum InstallationError {
    #[error("Failed to execute command for dependency '{dependency}': {command}")]
    CommandFailed {
        dependency: Dependency,
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to download installer for dependency '{dependency}' from {url}")]
    DownloadFailed {
        dependency: Dependency,
        url: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Installation failed for dependency '{dependency}': {message}")]
    InstallFailed {
        dependency: Dependency,
        message: String,
    },

    #[error("Installer not found for dependency '{dependency}'")]
    InstallerNotFound { dependency: Dependency },
}
```

### Example Installer: Cargo-machete

Convert `scripts/setup/install-cargo-machete.sh` to Rust in `src/installer/cargo_machete.rs`:

```rust
use async_trait::async_trait;
use crate::installer::{DependencyInstaller, InstallationError};
use crate::Dependency;
use std::process::Command;
use tracing::{info, debug};

pub struct CargoMacheteInstaller;

impl CargoMacheteInstaller {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DependencyInstaller for CargoMacheteInstaller {
    fn name(&self) -> &str {
        "cargo-machete"
    }

    fn dependency(&self) -> Dependency {
        Dependency::CargoMachete
    }

    async fn install(&self) -> Result<(), InstallationError> {
        info!(dependency = self.name(), "Installing dependency");

        // Equivalent to: cargo install cargo-machete
        let output = Command::new("cargo")
            .args(["install", "cargo-machete"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                dependency: self.dependency(),
                command: "cargo install cargo-machete".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!(stderr = %stderr, "Installation command stderr");

            return Err(InstallationError::InstallFailed {
                dependency: self.dependency(),
                message: stderr.to_string(),
            });
        }

        info!(
            dependency = self.name(),
            status = "installed",
            "Installation complete"
        );
        Ok(())
    }
}
```

### Example Installer: OpenTofu

Convert `scripts/setup/install-opentofu.sh` to Rust in `src/installer/opentofu.rs`:

```rust
use async_trait::async_trait;
use crate::installer::{DependencyInstaller, InstallationError};
use crate::Dependency;
use std::process::Command;
use tracing::{info, debug};

pub struct OpenTofuInstaller;

impl OpenTofuInstaller {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DependencyInstaller for OpenTofuInstaller {
    fn name(&self) -> &str {
        "OpenTofu"
    }

    fn dependency(&self) -> Dependency {
        Dependency::OpenTofu
    }

    async fn install(&self) -> Result<(), InstallationError> {
        info!(dependency = self.name(), "Installing dependency");

        // Step 1: Download installer script
        self.download_installer_script().await?;

        // Step 2: Make script executable
        self.make_executable().await?;

        // Step 3: Run installer
        self.run_installer().await?;

        // Step 4: Clean up
        self.cleanup().await?;

        info!(
            dependency = self.name(),
            status = "installed",
            "Installation complete"
        );
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
                dependency: self.dependency(),
                url: "https://get.opentofu.org/install-opentofu.sh".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            return Err(InstallationError::DownloadFailed {
                dependency: self.dependency(),
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
                dependency: self.dependency(),
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
                dependency: self.dependency(),
                command: "sudo /tmp/install-opentofu.sh".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InstallationError::InstallFailed {
                dependency: self.dependency(),
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
// Add to existing DependencyManager implementation

impl DependencyManager {
    /// Get a specific installer by dependency type
    ///
    /// Note: This creates a new installer instance on each call, which is acceptable
    /// since installers are lightweight and stateless.
    #[must_use]
    pub fn get_installer(&self, dep: Dependency) -> Box<dyn DependencyInstaller> {
        match dep {
            Dependency::CargoMachete => Box::new(CargoMacheteInstaller::new()),
            Dependency::OpenTofu => Box::new(OpenTofuInstaller::new()),
            Dependency::Ansible => Box::new(AnsibleInstaller::new()),
            Dependency::Lxd => Box::new(LxdInstaller::new()),
        }
    }

    /// Install a specific dependency
    ///
    /// # Errors
    ///
    /// Returns an error if installation fails
    pub async fn install(&self, dep: Dependency) -> Result<(), InstallationError> {
        let installer = self.get_installer(dep);
        installer.install().await
    }

    /// Install all dependencies
    ///
    /// # Errors
    ///
    /// Returns an error if any installation operation fails
    pub async fn install_all(&self) -> Result<Vec<InstallResult>, InstallationError> {
        let mut results = Vec::new();

        for dep in Dependency::all() {
            let result = match self.install(*dep).await {
                Ok(()) => InstallResult {
                    dependency: *dep,
                    success: true,
                    error: None,
                },
                Err(e) => InstallResult {
                    dependency: *dep,
                    success: false,
                    error: Some(e.to_string()),
                },
            };

            results.push(result);
        }

        Ok(results)
    }
}

/// Result of installing a single dependency
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub dependency: Dependency,
    pub success: bool,
    pub error: Option<String>,
}
```

### Add Install Command Handler

Create `src/handlers/install.rs`:

```rust
//! Install command handler
//!
//! This module handles installing dependencies.

// External crates
use thiserror::Error;
use tracing::{error, info};

// Internal crate
use crate::installer::InstallationError;
use crate::{Dependency, DependencyManager};

// ============================================================================
// PUBLIC API - Functions
// ============================================================================

/// Handle the install command
///
/// # Errors
///
/// Returns an error if:
/// - Installation fails
/// - Internal error occurs during installation
pub async fn handle_install(
    manager: &DependencyManager,
    dependency: Option<Dependency>,
) -> Result<(), InstallError> {
    match dependency {
        Some(dep) => install_specific_dependency(manager, dep).await?,
        None => install_all_dependencies(manager).await?,
    }

    Ok(())
}

// ============================================================================
// PRIVATE - Helper Functions
// ============================================================================

async fn install_all_dependencies(
    manager: &DependencyManager,
) -> Result<(), InstallAllDependenciesError> {
    info!("Installing all dependencies");

    let results = manager.install_all().await?;

    let mut failed_count = 0;

    for result in &results {
        if result.success {
            info!(
                dependency = %result.dependency,
                status = "installed",
                "Installation successful"
            );
        } else {
            error!(
                dependency = %result.dependency,
                status = "failed",
                error = result.error.as_deref(),
                "Installation failed"
            );
            failed_count += 1;
        }
    }

    if failed_count > 0 {
        info!(
            failed_count,
            total_count = results.len(),
            "Installation completed with failures"
        );
        Err(InstallAllDependenciesError::SomeInstallationsFailed {
            failed_count,
            total_count: results.len(),
        })
    } else {
        info!("All dependencies installed successfully");
        Ok(())
    }
}

async fn install_specific_dependency(
    manager: &DependencyManager,
    dependency: Dependency,
) -> Result<(), InstallSpecificDependencyError> {
    info!(dependency = %dependency, "Installing specific dependency");

    manager.install(dependency).await?;

    info!(
        dependency = %dependency,
        status = "installed",
        "Installation complete"
    );

    Ok(())
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("Failed to install all dependencies: {0}")]
    InstallAll(#[from] InstallAllDependenciesError),

    #[error("Failed to install specific dependency: {0}")]
    InstallSpecific(#[from] InstallSpecificDependencyError),
}

#[derive(Debug, Error)]
pub enum InstallAllDependenciesError {
    #[error("Failed to install dependencies: {failed_count} out of {total_count} failed")]
    SomeInstallationsFailed {
        failed_count: usize,
        total_count: usize,
    },

    #[error("Installation process failed: {0}")]
    InstallationFailed(#[from] InstallationError),
}

#[derive(Debug, Error)]
pub enum InstallSpecificDependencyError {
    #[error("Installation failed: {0}")]
    InstallationFailed(#[from] InstallationError),
}
```

### Update CLI and App

Update `src/cli.rs` to add the Install command:

```rust
// Add to Commands enum
#[derive(Subcommand)]
pub enum Commands {
    /// Check if dependencies are installed
    Check {
        /// Specific dependency to check (if omitted, checks all)
        #[arg(short = 'd', long)]
        dependency: Option<Dependency>,
    },

    /// List all available dependencies and their status
    List,

    /// Install dependencies
    Install {
        /// Specific dependency to install (if omitted, installs all)
        #[arg(short = 'd', long)]
        dependency: Option<Dependency>,
    },
}
```

Update `src/app.rs` to handle the new command:

```rust
use crate::handlers::{check, install, list};

pub async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    crate::logging::init_logging(cli.log_level, cli.verbose);

    let manager = DependencyManager::new();

    match cli.command {
        Commands::Check { dependency } => {
            check::handle_check(&manager, dependency)?;
        }
        Commands::List => {
            list::handle_list(&manager)?;
        }
        Commands::Install { dependency } => {
            install::handle_install(&manager, dependency).await?;
        }
    }

    Ok(())
}
```

Update `src/handlers/mod.rs`:

```rust
pub mod check;
pub mod install;  // Add this line
pub mod list;
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
    let check_result = container.exec(&[
        "dependency-installer",
        "check",
        "--dependency",
        "cargo-machete",
    ]);
    assert!(check_result.contains("not installed"));

    // Install cargo-machete
    let install_result = container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
    ]);
    assert!(
        install_result.contains("Installation complete")
            || install_result.contains("installed")
    );

    // Verify it's now installed
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "check",
        "--dependency",
        "cargo-machete",
    ]);
    assert_eq!(exit_code, 0);
}

/// Test installing OpenTofu (requires sudo)
#[tokio::test]
async fn test_install_opentofu() {
    let docker = DockerCli::default();
    let binary_path = get_binary_path();

    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .with_sudo() // Enable sudo for this test
        .start();

    // Install OpenTofu
    let install_result = container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "opentofu",
    ]);
    assert!(
        install_result.contains("Installation complete")
            || install_result.contains("installed")
    );

    // Verify installation
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "check",
        "--dependency",
        "opentofu",
    ]);
    assert_eq!(exit_code, 0);
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
    container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
    ]);

    // Install again - should succeed without error
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
    ]);
    assert_eq!(exit_code, 0);
}

/// Test installing all dependencies
#[tokio::test]
async fn test_install_all() {
    let docker = DockerCli::default();
    let binary_path = get_binary_path();

    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .with_sudo()
        .start();

    // Install all dependencies
    let install_result = container.exec(&["dependency-installer", "install"]);

    // Check that we got success messages
    assert!(
        install_result.contains("Installation complete")
            || install_result.contains("All dependencies installed")
    );

    // Verify all are installed
    let exit_code =
        container.exec_with_exit_code(&["dependency-installer", "check"]);
    assert_eq!(exit_code, 0);
}

fn get_binary_path() -> std::path::PathBuf {
    std::path::PathBuf::from("target/debug/dependency-installer")
}
```

## Implementation Tasks

### Phase 1: Installer Trait and Error Types

- [ ] Create `src/installer/mod.rs`
- [ ] Define `DependencyInstaller` trait with:
  - [ ] `name()` method
  - [ ] `dependency()` method
  - [ ] `install()` async method
  - [ ] `requires_sudo()` method with default implementation
- [ ] Define `InstallationError` enum with thiserror:
  - [ ] `CommandFailed` variant with dependency, command, source
  - [ ] `DownloadFailed` variant with dependency, url, source
  - [ ] `InstallFailed` variant with dependency, message
  - [ ] `InstallerNotFound` variant with dependency

### Phase 2: Convert Bash Scripts to Rust Installers

Analyze existing bash scripts in `scripts/setup/` and convert to Rust:

- [ ] **Cargo-machete** (`install-cargo-machete.sh`):

  - [ ] Create `src/installer/cargo_machete.rs`
  - [ ] Implement `CargoMacheteInstaller` struct
  - [ ] Implement `DependencyInstaller` trait
  - [ ] Use `cargo install cargo-machete` command
  - [ ] Add structured logging with tracing
  - [ ] Handle errors with `InstallationError`

- [ ] **OpenTofu** (`install-opentofu.sh`):

  - [ ] Create `src/installer/opentofu.rs`
  - [ ] Implement `OpenTofuInstaller` struct
  - [ ] Implement multi-step installation:
    - [ ] Download installer script with curl
    - [ ] Make script executable with chmod
    - [ ] Run installer with sudo
    - [ ] Clean up temporary files
  - [ ] Mark as `requires_sudo() = true`
  - [ ] Add structured logging for each step

- [ ] **Ansible** (`install-ansible.sh`):

  - [ ] Create `src/installer/ansible.rs`
  - [ ] Implement `AnsibleInstaller` struct
  - [ ] Convert apt-get commands to Rust
  - [ ] Mark as `requires_sudo() = true`
  - [ ] Handle package manager operations

- [ ] **LXD** (`install-lxd.sh`):
  - [ ] Create `src/installer/lxd.rs`
  - [ ] Implement `LxdInstaller` struct
  - [ ] Handle snap installation
  - [ ] Configure user groups if needed
  - [ ] Mark as `requires_sudo() = true`

### Phase 3: Update DependencyManager

- [ ] Add `get_installer(&self, dep: Dependency)` method
- [ ] Implement `install(&self, dep: Dependency)` async method
- [ ] Implement `install_all(&self)` async method
- [ ] Define `InstallResult` struct with:
  - [ ] `dependency` field
  - [ ] `success` field
  - [ ] `error` field (Option<String>)
- [ ] Test integration between detection and installation

### Phase 4: Add Install Command Handler

- [ ] Create `src/handlers/install.rs`
- [ ] Implement `handle_install()` function
- [ ] Implement `install_all_dependencies()` helper
- [ ] Implement `install_specific_dependency()` helper
- [ ] Define error types with thiserror:
  - [ ] `InstallError` enum
  - [ ] `InstallAllDependenciesError` enum
  - [ ] `InstallSpecificDependencyError` enum
- [ ] Add structured logging with proper fields
- [ ] Update `src/handlers/mod.rs` to export install module

### Phase 5: Update CLI and App

- [ ] Update `src/cli.rs`:
  - [ ] Add `Install` variant to `Commands` enum
  - [ ] Add `--dependency` flag (optional, installs all if omitted)
- [ ] Update `src/app.rs`:
  - [ ] Add `install` handler import
  - [ ] Add match arm for `Install` command
  - [ ] Call `install::handle_install().await`
- [ ] Ensure async support is properly configured
- [ ] Update CLI help text

### Phase 6: Docker Test Infrastructure

- [ ] Update `tests/containers/ubuntu.rs`:

  - [ ] Add `with_sudo()` method to builder
  - [ ] Configure container to support sudo operations
  - [ ] Test sudo configuration

- [ ] Create `tests/docker_install_command.rs`:
  - [ ] Implement `test_install_cargo_machete`
  - [ ] Implement `test_install_opentofu`
  - [ ] Implement `test_install_ansible`
  - [ ] Implement `test_install_lxd`
  - [ ] Implement `test_install_idempotent`
  - [ ] Implement `test_install_all`
  - [ ] Implement helper function `get_binary_path()`

### Phase 7: Testing and Validation

- [ ] **Unit tests** for each installer:

  - [ ] Mock command execution where possible
  - [ ] Test error handling paths
  - [ ] Verify structured logging output

- [ ] **Integration tests** in Docker:

  - [ ] Test each dependency installation
  - [ ] Verify idempotency (install twice, both succeed)
  - [ ] Test error scenarios
  - [ ] Verify with check command after installation

- [ ] **Manual testing**:

  - [ ] Build binary: `cargo build --bin dependency-installer`
  - [ ] Test specific dependency: `dependency-installer install --dependency cargo-machete`
  - [ ] Test all dependencies: `dependency-installer install`
  - [ ] Verify with check command

- [ ] Run complete test suite: `cargo test`

### Phase 8: Documentation

- [ ] Update `packages/dependency-installer/README.md`:

  - [ ] Add install command examples
  - [ ] Document exit codes (0=success, 1=failures, 2=invalid args, 3=internal error)
  - [ ] Show structured logging output examples
  - [ ] Document --dependency flag
  - [ ] Document log level control

- [ ] Document each installer:

  - [ ] List requirements (Rust, curl, sudo, etc.)
  - [ ] Document sudo requirements
  - [ ] Note any platform-specific behavior

- [ ] Update package documentation:
  - [ ] Add module-level docs for installer module
  - [ ] Document structured logging patterns
  - [ ] Add usage examples

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Manual installation testing successful
- [ ] No clippy warnings
- [ ] Code is properly formatted with rustfmt

**DependencyInstaller Trait**:

- [ ] Trait is well-defined with clear contract
- [ ] All 4 installers implement the trait correctly
- [ ] Error handling uses `InstallationError` consistently
- [ ] Structured logging provides visibility at appropriate levels

**Installer Implementations**:

- [ ] Cargo-machete installer works in Docker
- [ ] OpenTofu installer works in Docker (with sudo)
- [ ] Ansible installer works in Docker (with sudo)
- [ ] LXD installer works in Docker (with sudo)
- [ ] All installers are idempotent (can run multiple times)
- [ ] Sudo requirements are correctly marked
- [ ] Installations can be verified with check command
- [ ] Error messages are clear and actionable

**CLI Install Command**:

- [ ] `install` subcommand works for all dependencies
- [ ] `install --dependency <name>` works for specific dependencies
- [ ] Structured logging output is informative and properly formatted
- [ ] Exit codes are correct:
  - 0 for success
  - 1 for installation failures
  - 2 for invalid arguments
- [ ] Help text is accurate and helpful

**Docker Tests**:

- [ ] Tests verify actual installation in clean containers
- [ ] Tests verify idempotency (install twice succeeds)
- [ ] Tests verify installations using check command
- [ ] Tests handle both sudo and non-sudo dependencies
- [ ] All tests pass consistently
- [ ] Container cleanup works properly

**DependencyManager**:

- [ ] Manager successfully integrates detectors and installers
- [ ] `get_installer()` returns correct installer for each dependency
- [ ] `install()` method works for single dependency
- [ ] `install_all()` method works for all dependencies
- [ ] Results are properly tracked and reported
- [ ] Error handling is comprehensive

**Documentation**:

- [ ] README updated with install command usage
- [ ] Each installer has clear documentation
- [ ] Sudo requirements are clearly documented
- [ ] Structured logging patterns are documented
- [ ] Examples are complete and tested

## Example Usage After Completion

### Installing All Dependencies

```bash
$ dependency-installer install
2025-11-04T10:15:20.123456Z  INFO install: Installing all dependencies
2025-11-04T10:15:21.234567Z  INFO install: Installing dependency dependency="cargo-machete"
2025-11-04T10:15:25.345678Z  INFO install: Installation successful dependency="cargo-machete" status="installed"
2025-11-04T10:15:26.456789Z  INFO install: Installing dependency dependency="OpenTofu"
2025-11-04T10:15:35.567890Z  INFO install: Installation successful dependency="OpenTofu" status="installed"
2025-11-04T10:15:36.678901Z  INFO install: Installing dependency dependency="Ansible"
2025-11-04T10:15:45.789012Z  INFO install: Installation successful dependency="Ansible" status="installed"
2025-11-04T10:15:46.890123Z  INFO install: Installing dependency dependency="LXD"
2025-11-04T10:15:55.901234Z  INFO install: Installation successful dependency="LXD" status="installed"
2025-11-04T10:15:56.012345Z  INFO install: All dependencies installed successfully

$ echo $?
0  # Success
```

### Installing Specific Dependency

```bash
$ dependency-installer install --dependency opentofu
2025-11-04T10:20:10.123456Z  INFO install: Installing specific dependency dependency="opentofu"
2025-11-04T10:20:15.234567Z  INFO install: Installation complete dependency="opentofu" status="installed"

$ echo $?
0  # Success
```

### Installing with Verbose Logging

```bash
$ dependency-installer install --dependency opentofu --verbose
2025-11-04T10:25:10.123456Z  INFO install: Installing specific dependency dependency="opentofu"
2025-11-04T10:25:11.234567Z DEBUG opentofu_installer: Downloading installer script
2025-11-04T10:25:13.345678Z DEBUG opentofu_installer: Making script executable
2025-11-04T10:25:14.456789Z DEBUG opentofu_installer: Running installer with sudo
2025-11-04T10:25:20.567890Z DEBUG opentofu_installer: Cleaning up installer script
2025-11-04T10:25:20.678901Z  INFO install: Installation complete dependency="opentofu" status="installed"
```

### Verifying Installation

```bash
$ dependency-installer check
2025-11-04T10:30:10.123456Z  INFO check: Checking all dependencies
2025-11-04T10:30:10.234567Z  INFO check: Dependency check result dependency="cargo-machete" status="installed"
2025-11-04T10:30:10.345678Z  INFO check: Dependency check result dependency="OpenTofu" status="installed"
2025-11-04T10:30:10.456789Z  INFO check: Dependency check result dependency="Ansible" status="installed"
2025-11-04T10:30:10.567890Z  INFO check: Dependency check result dependency="LXD" status="installed"
2025-11-04T10:30:10.678901Z  INFO check: All dependencies are installed

$ echo $?
0  # All installed
```

### Handling Errors

```bash
$ dependency-installer install --dependency nonexistent
Error: Invalid value 'nonexistent' for '--dependency <DEPENDENCY>': Unknown dependency: nonexistent. Available: cargo-machete, opentofu, ansible, lxd

$ echo $?
2  # Invalid argument
```

## Logging Best Practices

This tool uses structured logging with the `tracing` crate for automation-focused observability.

### Log Levels

- **ERROR**: Installation failures, unrecoverable errors
- **WARN**: Non-critical issues (already installed, skipped optional steps)
- **INFO**: High-level progress (start installation, completion, status updates)
- **DEBUG**: Detailed operation steps (download, chmod, execute, cleanup)
- **TRACE**: Very detailed execution (command output, internal state transitions)

### Structured Fields

Always include relevant context for observability:

```rust
// ✅ Good: Rich context with structured fields
info!(
    dependency = "OpenTofu",
    status = "installed",
    version = "1.6.0",
    duration_ms = 15234,
    "Installation complete"
);

// ❌ Bad: No context, just a message
info!("Installation complete");
```

### Controlling Output

Users control log verbosity through CLI flags or environment variables:

```bash
# Default (INFO and above)
dependency-installer install

# Verbose (DEBUG and above) - shows detailed steps
dependency-installer install --verbose

# Specific level
dependency-installer install --log-level trace  # Most detailed
dependency-installer install --log-level error  # Minimal output

# Environment variable (overrides CLI flags)
RUST_LOG=debug dependency-installer install

# Complex filtering with RUST_LOG
RUST_LOG=dependency_installer=debug,installer=trace dependency-installer install
```

## Related Documentation

- [Issue 1-1-3](./116-1-1-3-create-docker-test-infrastructure.md) - Docker testing infrastructure extended here
- [Issue 1-1-2](./115-1-1-2-create-cli-binary-with-check-command.md) - CLI binary extended with install command
- [Issue 1-1-1](./114-1-1-1-create-detection-logic-package.md) - Detection logic used to verify installations
- [Parent Issue 1-1](./113-create-dependency-installation-package-for-e2e-tests.md) - Overall package specification
- [scripts/setup/](../../scripts/setup/) - Original bash scripts to convert
- [packages/dependency-installer/README.md](../../packages/dependency-installer/README.md) - Package README

## Notes

### Time Estimate

**4-5 hours** total for this phase (largest of the 4 phases).

### Next Steps After Completion

1. The dependency-installer package is **complete and fully functional**
2. **Issue 1-2**: Integrate the package with existing E2E tests
3. **Issue 1-3**: Update CI workflows to use the new binary instead of bash scripts

### Key Design Decisions

**Two-trait design**: Separating `DependencyDetector` and `DependencyInstaller` keeps concerns separated, allows reusing detection logic, and follows single responsibility principle.

**Idempotent installations**: Each installer must handle being run multiple times safely. Already-installed dependencies should not cause errors.

**Structured logging only**: No interactive prompts or user-facing `println!()` output. Designed for CI/CD automation where structured logs enable programmatic parsing and filtering.

**Handler-based architecture**: Following existing pattern in the codebase with dedicated handler modules for each command.

**Docker testing required**: Testing in Docker ensures installations work in clean environments and don't depend on developer machine state.

### Bash Script Conversion Strategy

For each bash script in `scripts/setup/`:

1. **Understand the logic** - Read and document what each script does
2. **Identify commands** - List all shell commands used (curl, chmod, sudo, apt-get, snap, etc.)
3. **Map to Rust** - Use `std::process::Command` for executing shell commands
4. **Add error handling** - Wrap each command with proper `InstallationError` variants
5. **Add structured logging** - Use tracing at appropriate levels (info, debug)
6. **Test in Docker** - Verify the Rust version works identically to bash version

### Testing Strategy

- **Unit tests**: Test individual installer methods, mock command execution where feasible
- **Integration tests**: Run actual installations in Docker containers (Ubuntu 24.04)
- **Verification tests**: Use `check` command to verify installations succeeded
- **Idempotency tests**: Install twice, verify both succeed without errors
- **Error scenario tests**: Test network failures, permission errors, missing dependencies

### Installation Requirements

Dependencies have different installation requirements:

| Dependency    | Requires Sudo | Installation Method | Notes                         |
| ------------- | ------------- | ------------------- | ----------------------------- |
| cargo-machete | No            | `cargo install`     | User-level, no system changes |
| OpenTofu      | Yes           | Curl script + deb   | System-wide installation      |
| Ansible       | Yes           | `apt-get`           | System package manager        |
| LXD           | Yes           | `snap` + groups     | System service + user config  |

The `requires_sudo()` trait method documents these requirements for users and tests.
