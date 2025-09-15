# E2E Tests Refactoring Plan - Three-Level Architecture

> **📋 Architectural Refactoring Plan**  
> This document outlines the refactoring of `src/bin/e2e_tests.rs` into a production-ready three-level architecture that will serve as the foundation for the Torrust Tracker Deploy console application.

## 🎯 Refactoring Goals

- **Extract E2E logic** into production command handlers
- **Create modular architecture** with clear separation of concerns
- **Enable console application** with proper CLI subcommand structure
- **Improve scalability** and maintainability for future development

## 🏗️ Three-Level Architecture

The refactoring will implement a clean three-level abstraction:

### Level 1: Commands

**Direct mapping to console commands** - Top-level operations that users invoke

- Maps to CLI subcommands (`torrust-deploy provision`, `torrust-deploy configure`, etc.)
- Handles command-line arguments and user interaction
- Orchestrates multiple steps to achieve command objectives
- Manages command-specific error handling and reporting

### Level 2: Steps

**Reusable building blocks** - Modular operations that can be composed into commands

- Independent, testable units of work
- Can be reused across multiple commands
- Handle specific deployment tasks (template rendering, infrastructure operations, etc.)
- Pass context and state between operations

### Level 3: Remote Actions

**Operations executed on remote servers** - SSH-based actions on provisioned infrastructure

- Already partially implemented in `src/actions/`
- Validate remote server state and configuration
- Execute maintenance and setup tasks
- Can be wrapped into Steps for command composition

## 📁 Proposed Directory Structure

### Current `src/` Structure (To Be Refactored)

```text
src/
├── bin/
│   ├── e2e_tests.rs           # 427 lines - needs refactoring
│   └── linter.rs              # Keep as-is
├── command.rs                 # Keep - command execution utilities
├── config.rs                  # Keep - configuration management
├── container.rs               # Rename to services.rs
├── lib.rs                     # Keep - library exports
├── main.rs                    # Keep - main binary entry point
├── actions/                   # Keep - Level 3 (Remote Actions)
│   ├── mod.rs
│   ├── cloud_init.rs
│   ├── docker.rs
│   └── docker_compose.rs
├── ansible/                   # Keep - template rendering utilities
│   ├── mod.rs
│   └── template_renderer.rs
├── command_wrappers/          # Keep - external tool wrappers
│   ├── mod.rs
│   ├── ansible.rs
│   ├── ssh.rs
│   ├── lxd/
│   └── opentofu/
├── template/                  # Keep - template management
│   ├── mod.rs
│   ├── embedded.rs
│   ├── engine.rs
│   ├── file_ops.rs
│   ├── file.rs
│   └── wrappers/
└── tofu/                      # Keep - OpenTofu utilities
    ├── mod.rs
    └── template_renderer.rs
```

### New `src/` Structure (After Refactoring)

```text
src/
├── bin/
│   ├── e2e_tests.rs           # Minimal orchestration (~50 lines)
│   ├── linter.rs              # Keep as-is
│   └── torrust_deploy.rs      # NEW: Main console application
├── lib.rs                     # Updated exports
├── main.rs                    # Keep - delegates to console app
├──
├── commands/                  # NEW: Level 1 (Commands)
│   ├── mod.rs
│   ├── check.rs               # Tool validation command
│   ├── create.rs              # Environment creation command
│   ├── provision.rs           # Infrastructure provisioning command
│   ├── configure.rs           # System configuration command
│   ├── release.rs             # Application deployment command
│   ├── run.rs                 # Service startup command
│   ├── test.rs                # Validation command
│   ├── status.rs              # Environment info command
│   ├── destroy.rs             # Cleanup command
│   └── context.rs             # Shared deployment context
├──
├── steps/                     # NEW: Level 2 (Steps)
│   ├── mod.rs
│   ├── template_steps.rs      # Template rendering steps
│   ├── infrastructure_steps.rs # Infrastructure operations
│   ├── connectivity_steps.rs  # Network and SSH steps
│   ├── remote_execution_steps.rs # Remote command execution
│   ├── validation_steps.rs    # Validation and health checks
│   └── application_steps.rs   # Application management steps
├──
├── actions/                   # Level 3 (Remote Actions) - Enhanced
│   ├── mod.rs                 # Enhanced trait definition
│   ├── cloud_init.rs          # Keep - cloud-init validation
│   ├── docker.rs              # Keep - Docker validation
│   ├── docker_compose.rs      # Keep - Docker Compose validation
│   ├── tracker.rs             # NEW: Torrust Tracker validation
│   ├── database.rs            # NEW: Database connectivity validation
│   ├── firewall.rs            # NEW: Firewall rules validation
│   └── service_health.rs      # NEW: Generic service health checks
├──
├── core/                      # Renamed and enhanced core utilities
│   ├── mod.rs
│   ├── config.rs              # Moved from root - configuration management
│   ├── services.rs            # Renamed from container.rs - DI container
│   ├── context.rs             # NEW: Shared execution context
│   ├── error.rs               # NEW: Comprehensive error types
│   └── progress.rs            # NEW: Progress reporting system
├──
├── command.rs                 # Keep - command execution utilities
├── command_wrappers/          # Keep - external tool wrappers
│   ├── mod.rs
│   ├── ansible.rs
│   ├── ssh.rs
│   ├── lxd/
│   └── opentofu/
├── template/                  # Keep - template management
│   ├── mod.rs
│   ├── embedded.rs
│   ├── engine.rs
│   ├── file_ops.rs
│   ├── file.rs
│   └── wrappers/
├── ansible/                   # Keep - Ansible-specific utilities
│   ├── mod.rs
│   └── template_renderer.rs
└── tofu/                      # Keep - OpenTofu-specific utilities
    ├── mod.rs
    └── template_renderer.rs
```

## 🔄 Command to Steps Mapping

```rust
// Level 1: Commands and their constituent Level 2: Steps

ProvisionCommand:
├── ✅ RenderOpenTofuTemplatesStep   // Template rendering - COMPLETED
├── ✅ InitializeInfrastructureStep  // tofu init - COMPLETED
├── ✅ PlanInfrastructureStep        // tofu plan - COMPLETED
├── ✅ ApplyInfrastructureStep       // tofu apply - COMPLETED
├── ✅ GetInstanceInfoStep           // Extract IP from tofu outputs - COMPLETED
├── ✅ WaitForSSHConnectivityStep    // Validate SSH access - COMPLETED
└── ✅ WaitForCloudInitStep          // Wait for cloud-init completion - COMPLETED

ConfigureCommand:
├── ✅ RenderAnsibleTemplatesStep    // Template rendering with runtime vars - COMPLETED
├── ✅ InstallDockerStep             // Docker installation via Ansible - COMPLETED
├── ✅ InstallDockerComposeStep      // Docker Compose installation via Ansible - COMPLETED
└── ValidateRemoteServicesStep       // Validate installations

ReleaseCommand:
├── GenerateComposeConfigStep        // Create docker-compose.yml
├── TransferFilesStep                // Copy files to remote
├── DeployApplicationStep            // Deploy application via Ansible
└── ValidateDeploymentStep           // Validate deployment

RunCommand:
├── StartServicesStep                // docker-compose up
├── WaitForServicesHealthStep        // Wait for containers to be healthy
└── GetServiceStatusStep             // Report service endpoints

TestCommand:
├── ✅ ValidateDockerInstallationStep   // Validate Docker is working - COMPLETED
├── ValidateDockerComposeInstallationStep // Validate Docker Compose is working
├── ValidateTrackerStep              // Validate Torrust Tracker functionality
└── ValidateRemoteServicesStep       // Overall validation

DestroyCommand:
├── StopServicesStep                 // Graceful service shutdown (if running)
└── DestroyInfrastructureStep        // tofu destroy
```

## 📋 Implementation Type Hierarchy

### Level 1: Command Types

```rust
// Core command trait
trait Command {
    async fn execute(&self, context: &DeploymentContext) -> Result<(), CommandError>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

// Command implementations
struct ProvisionCommand { /* ... */ }
struct ConfigureCommand { /* ... */ }
struct ReleaseCommand { /* ... */ }
struct RunCommand { /* ... */ }
struct TestCommand { /* ... */ }
struct StatusCommand { /* ... */ }
struct DestroyCommand { /* ... */ }
struct CheckCommand { /* ... */ }
struct CreateCommand { /* ... */ }
```

### Level 2: Step Types

```rust
// Core step trait
trait Step {
    async fn execute(&self, context: &mut DeploymentContext) -> Result<StepOutput, StepError>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn requires(&self) -> Vec<&'static str>; // Prerequisites
}

// Step implementations by category
// Template Steps
struct RenderOpenTofuTemplatesStep { /* ... */ }
struct RenderAnsibleTemplatesStep { /* ... */ }

// Infrastructure Steps
struct InitializeInfrastructureStep { /* ... */ }
struct PlanInfrastructureStep { /* ... */ }
struct ApplyInfrastructureStep { /* ... */ }
struct DestroyInfrastructureStep { /* ... */ }
struct GetInstanceInfoStep { /* ... */ }

// Connectivity Steps
struct WaitForSSHConnectivityStep { /* ... */ }
struct ValidateNetworkConnectivityStep { /* ... */ }

// Remote Execution Steps
struct WaitForCloudInitStep { /* ... */ }
struct InstallDockerStep { /* ... */ }
struct InstallDockerComposeStep { /* ... */ }
struct DeployApplicationStep { /* ... */ }
struct TransferFilesStep { /* ... */ }

// Application Steps
struct GenerateComposeConfigStep { /* ... */ }
struct StartServicesStep { /* ... */ }
struct StopServicesStep { /* ... */ }
struct GetServiceStatusStep { /* ... */ }

// Validation Steps
struct ValidateToolsStep { /* ... */ }
struct ValidateRemoteServicesStep { /* ... */ }
struct ValidateDeploymentStep { /* ... */ }
struct ValidateDockerInstallationStep { /* ... */ }
struct ValidateDockerComposeInstallationStep { /* ... */ }
struct ValidateTrackerStep { /* ... */ }
```

### Level 3: Remote Action Types (Enhanced)

```rust
// Enhanced remote action trait (existing)
trait RemoteAction {
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn timeout(&self) -> Duration;
}

// Existing remote actions (keep)
struct CloudInitValidator { /* ... */ }
struct DockerValidator { /* ... */ }
struct DockerComposeValidator { /* ... */ }

// New remote actions
struct TrackerValidator { /* ... */ }
struct DatabaseValidator { /* ... */ }
struct FirewallValidator { /* ... */ }
struct ServiceHealthValidator { /* ... */ }
```

## 🔧 Directory Renames and Changes

### Files to Rename

- `src/container.rs` → `src/core/services.rs`
- `src/config.rs` → `src/core/config.rs`

### Files to Create

- `src/bin/torrust_deploy.rs` - Main console application
- `src/commands/` - All command implementations
- `src/steps/` - All step implementations
- `src/core/context.rs` - Shared execution context
- `src/core/error.rs` - Comprehensive error types
- `src/core/progress.rs` - Progress reporting
- `src/actions/tracker.rs` - Torrust Tracker validation
- `src/actions/database.rs` - Database validation
- `src/actions/firewall.rs` - Firewall validation
- `src/actions/service_health.rs` - Service health checks

### Files to Refactor

- `src/bin/e2e_tests.rs` - Reduce from 427 lines to ~50 lines, use new architecture
- `src/actions/mod.rs` - Enhanced trait definitions
- `src/lib.rs` - Updated exports for new structure

## 🚀 Migration Strategy

### Phase 1: Core Infrastructure (Week 1-2)

1. **Create directory structure** and move existing files
2. **Implement core types** (Command, Step, enhanced RemoteAction traits)
3. **Create DeploymentContext** for state sharing between components
4. **Implement basic error and progress systems**

### Phase 2: Extract Commands (Week 3-4)

1. **Implement ProvisionCommand** by extracting logic from E2E tests
2. **Implement ConfigureCommand** by extracting configuration logic
3. **Implement DestroyCommand** by extracting cleanup logic
4. **Create supporting steps** as needed for these commands

### Phase 3: Console Application (Week 5-6)

1. **Create main console application** with CLI framework
2. **Implement remaining commands** (CheckCommand, StatusCommand, etc.)
3. **Add application-specific steps** for release and run commands
4. **Refactor E2E tests** to use new architecture

### Phase 4: Enhanced Features (Week 7-8)

1. **Add multi-environment support**
2. **Implement state persistence** between command invocations
3. **Add Torrust Tracker-specific remote actions**
4. **Complete application deployment pipeline**

## 📊 Expected Benefits

### Code Quality

- **Reduced complexity**: Large monolithic E2E test broken into focused components
- **Better testability**: Each command and step can be unit tested independently
- **Clear separation**: Command orchestration, step execution, remote validation are distinct
- **Reusable components**: Steps can be shared across commands

### Maintainability

- **Modular structure**: Changes to one command don't affect others
- **Clear interfaces**: Well-defined traits for commands, steps, and remote actions
- **Easy extension**: Adding new commands/steps/actions follows established patterns
- **Better error handling**: Comprehensive error types with context

### Production Readiness

- **Console application**: Ready-to-use CLI with proper subcommand structure
- **State management**: Context passing enables complex workflows
- **Progress reporting**: User-friendly feedback during long-running operations
- **Configuration system**: Support for different environments and settings

This architecture transformation will convert the E2E test logic into a production-ready console application while maintaining all existing functionality and enabling future expansion.

### 1. Stage-Based Execution System Enhancement

#### Current State (Good)

The current implementation already provides a **well-structured 4-stage execution framework**:

- ✅ Clear stage separation with dedicated methods
- ✅ Logical flow from template rendering → provisioning → configuration → validation
- ✅ Good error handling with `anyhow` context
- ✅ Async operations where appropriate

#### Remaining Enhancement Opportunities

- **Stage Abstraction**: Convert methods to independent stage components
- **Progress Tracking**: Add structured progress reporting for long-running stages
- **Stage Context**: Shared context passing between stages
- **Stage Validation**: Pre/post conditions for each stage

#### Recommended Improvements

1. **Extract Stage Orchestrator**

```rust
trait ExecutionStage {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn execute(&self, context: &mut StageContext) -> Result<()>;
}

struct StageOrchestrator {
    stages: Vec<Box<dyn ExecutionStage>>,
    progress_reporter: Box<dyn ProgressReporter>,
}
```

1. **Individual Stage Implementations**

```rust
struct TemplateRenderingStage {
    template_manager: TemplateManager,
}

struct InfrastructureProvisioningStage {
    opentofu_client: OpenTofuClient,
}

struct ConfigurationManagementStage {
    ansible_client: AnsibleClient,
    ssh_client: SshClient,
}
```

### 2. Configuration Management Enhancement

#### Current State (Basic Implementation)

The current implementation has a **solid foundation** with `Config` and `Services`:

- ✅ Centralized configuration with `Config` struct
- ✅ Dependency injection pattern with `Services`
- ✅ CLI argument parsing with `clap`
- ✅ Path management for templates, build directories, SSH keys

#### Remaining Enhancement Opportunities

- **External Configuration Files**: TOML/YAML configuration files
- **Environment-Specific Settings**: Development, staging, production configurations
- **Timeout Configuration**: Externalized timeout values for SSH, cloud-init, deployments
- **Provider Configuration**: Pluggable provider settings (not just LXD)

#### Recommended Improvements

1. **Configuration File Support**

```rust
#[derive(Deserialize, Debug)]
struct E2EConfig {
    infrastructure: InfrastructureConfig,
    ssh: SshConfig,
    validation: ValidationConfig,
    timeouts: TimeoutConfig,
    templates: TemplateConfig,
}

#[derive(Deserialize, Debug)]
struct TimeoutConfig {
    ssh_connection: Duration,
    cloud_init_wait: Duration,
    playbook_execution: Duration,
}
```

1. **CLI Override System**

```rust
impl E2EConfig {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
    fn with_cli_overrides(self, cli: &Cli) -> Self;
    fn merge_with_defaults(self) -> Self;
}
```

### 3. Enhanced Error Context and Reporting

#### Current Issues

- **Generic Error Handling**: Heavy reliance on `anyhow::Error` without specific error types
- **Limited Error Context**: Errors don't provide enough information for debugging
- **No Test Reporting**: No structured test results or metrics collection

#### Recommended Improvements

1. **Comprehensive Error Types**

```rust
#[derive(Debug, thiserror::Error)]
enum E2ETestError {
    #[error("Infrastructure provisioning failed: {source}")]
    ProvisioningFailed {
        source: anyhow::Error,
        provider: String,
        stage: String,
    },

    #[error("Template rendering failed for {template}: {source}")]
    TemplateRenderingFailed {
        template: String,
        source: anyhow::Error,
    },

    #[error("Validation failed for {validator}: {source}")]
    ValidationFailed {
        validator: String,
        source: anyhow::Error,
    },
}
```

1. **Test Result Reporting**

```rust
#[derive(Debug)]
struct TestReport {
    stages: HashMap<String, StageResult>,
    total_duration: Duration,
    success: bool,
    errors: Vec<E2ETestError>,
}

struct StageResult {
    name: String,
    duration: Duration,
    success: bool,
    error: Option<E2ETestError>,
}
```

### 4. Parallel Operations and Performance Enhancement

#### Current State (Sequential Execution)

The current implementation executes most operations sequentially:

- ✅ Async/await pattern implemented where appropriate
- ❌ Validation steps run sequentially (could be parallel)
- ❌ Template processing is sequential
- ❌ No batching of similar operations

#### Recommended Improvements

1. **Parallel Validation Execution**

```rust
async fn run_parallel_validations(&self, container_ip: &str) -> Result<()> {
    let validators = vec![
        CloudInitValidator::new(&self.config.ssh_key_path, &self.config.ssh_username, container_ip),
        DockerValidator::new(&self.config.ssh_key_path, &self.config.ssh_username, container_ip),
        DockerComposeValidator::new(&self.config.ssh_key_path, &self.config.ssh_username, container_ip),
    ];

    let validation_tasks: Vec<_> = validators
        .into_iter()
        .map(|validator| validator.execute(container_ip))
        .collect();

    futures::future::try_join_all(validation_tasks).await?;
    Ok(())
}
```

### 5. Code Quality Improvements

#### Identified Technical Debt

Based on current code analysis, the following improvements would enhance code quality:

1. **Method Size Reduction**: Several methods exceed 50+ lines and could be broken down:

   - `provision_infrastructure()` (~45 lines) - could extract IP retrieval logic
   - `render_configuration_templates()` (~25 lines) - good size but context creation could be extracted
   - `run_full_deployment_test()` (~50 lines) - could extract stage coordination

2. **Error Handling Consistency**: Mix of `anyhow::Error` and `map_err(|e| anyhow::anyhow!(e))` patterns could be more consistent

3. **Duplicate IP Retrieval Logic**: Currently gets IP from both OpenTofu and LXD - consolidate or make the validation more explicit

4. **Hard-coded Values Still Present**:
   - SSH connection timeouts
   - Cloud-init wait durations
   - Instance name "torrust-vm" is hard-coded
   - Playbook names are hard-coded strings

## 🏗️ Updated Architecture Proposal

**Current State (Improved):**

- ✅ `CommandExecutor` - src/command.rs
- ✅ Client abstractions - src/command_wrappers/ (OpenTofuClient, SshClient, AnsibleClient, LxdClient)
- ✅ Validation system - src/actions/ with `RemoteAction` trait
- ✅ Template management - src/template/ with dedicated renderers in src/tofu/, src/ansible/
- ✅ Configuration pattern - src/config.rs and src/container.rs for dependency injection

**Proposed Further Structure:**

```text
src/
├── bin/
│   └── e2e_tests.rs (minimal orchestration - currently 427 lines)
├── e2e/                           # NEW: E2E-specific modules
│   ├── mod.rs
│   ├── config.rs                  # E2E configuration management
│   ├── orchestrator.rs            # Stage-based execution
│   ├── progress.rs                # Progress reporting
│   ├── metrics.rs                 # Metrics collection
│   ├── error.rs                   # Comprehensive error types
│   └── stages/
│       ├── mod.rs
│       ├── template_rendering.rs
│       ├── infrastructure_provisioning.rs
│       ├── configuration_management.rs
│       └── validation.rs
├── command.rs                     # ✅ exists
├── config.rs                      # ✅ exists
├── container.rs                   # ✅ exists (Services)
├── command_wrappers/              # ✅ exists
│   ├── opentofu/
│   ├── ssh.rs
│   ├── ansible.rs
│   └── lxd/
├── actions/                       # ✅ exists - validation system
├── template/                      # ✅ exists
├── tofu/                          # ✅ exists - template renderer
└── ansible/                       # ✅ exists - template renderer
```

## 📈 Updated Implementation Roadmap

### 🎯 Phase 1: Enhanced Configuration and Observability (High Priority)

> **Goal**: Improve configuration management and add better observability

- [ ] **Task 1.1**: External Configuration Files

  - Support TOML/YAML configuration files for environment-specific settings
  - Extract hard-coded timeouts (SSH connection: 30s, cloud-init: 300s, etc.)
  - Add environment profiles (dev, staging, production)
  - Implement configuration validation and defaults

- [ ] **Task 1.2**: Enhanced Progress Reporting

  - Replace println! with structured progress reporting
  - Add progress indicators for long-running operations (cloud-init wait, playbook execution)
  - Implement operation timing and metrics collection
  - Add stage-level progress tracking

### 🏗️ Phase 2: Stage Architecture Enhancement (Medium Priority)

> **Goal**: Enhance the current 4-stage architecture with better abstractions

- [ ] **Task 2.1**: Stage Trait and Context System

  - Create `ExecutionStage` trait for stage abstraction
  - Implement `StageContext` for shared state between stages
  - Add pre/post validation hooks for stages
  - Create dedicated stage implementations

- [ ] **Task 2.2**: Enhanced Error Context System

  - Create comprehensive `E2ETestError` enum with stage context
  - Add detailed error messages with resolution suggestions
  - Implement error reporting and aggregation by stage
  - Better error recovery and rollback mechanisms

### 🎨 Phase 3: Performance and Extensibility (Lower Priority)

> **Goal**: Add parallel operations and extensibility features

- [ ] **Task 3.1**: Parallel Operations

  - Implement concurrent validation execution (all validators run in parallel)
  - Add parallel template processing where safe
  - Optimize I/O bound operations with proper async coordination

- [ ] **Task 3.2**: TestEnvironment Simplification

  - Extract `TestEnvironment` orchestration to dedicated orchestrator
  - Simplify `TestEnvironment` to pure dependency injection container
  - Break down remaining large methods (> 50 lines)

- [ ] **Task 3.3**: Provider Extensibility

  - Abstract provider-specific code behind traits
  - Add support for additional providers beyond LXD
  - Make provider selection configurable

## ✅ Completed Tasks

> **Instructions**: These tasks have been completed since the original refactoring plan

### 📅 September 2025 - Major Refactoring Phase

- **✅ Command Abstraction (Originally Task 1.1)**: `CommandExecutor` extracted with proper error handling and timeout support in `src/command.rs`

- **✅ Infrastructure Provider (Originally Task 2.1)**: `OpenTofuClient` implemented in `src/command_wrappers/opentofu/` with consistent interface for init, apply, destroy operations

- **✅ SSH Client Wrapper (Originally Task 2.2)**: `SshClient` implemented in `src/command_wrappers/ssh.rs` with connection management, security settings, and async connectivity checking

- **✅ Configuration Management Client**: `AnsibleClient` implemented in `src/command_wrappers/ansible.rs` for playbook execution and configuration management

- **✅ LXD Integration**: `LxdClient` implemented in `src/command_wrappers/lxd/client.rs` for container management and IP address retrieval

- **✅ Validation System (Originally Task 3.1)**: `RemoteAction` trait implemented in `src/actions/mod.rs` with specific validators:

  - `CloudInitValidator` for cloud-init completion validation
  - `DockerValidator` for Docker installation validation
  - `DockerComposeValidator` for Docker Compose validation

- **✅ Template Integration**: `TemplateManager` integrated with dedicated renderers:

  - `TofuTemplateRenderer` in `src/tofu/` for OpenTofu templates
  - `AnsibleTemplateRenderer` in `src/ansible/` for Ansible templates

- **✅ Async Operations (Originally Task 3.3)**: Main execution flow converted to async/await pattern with proper async I/O operations

- **✅ Error Handling Foundation**: `CommandError` type implemented with structured error reporting and `anyhow` integration

- **✅ Configuration Architecture**: `Config` and `Services` pattern implemented for dependency injection and configuration management

- **✅ 4-Stage Execution Framework**: Well-defined execution stages implemented:

  - Stage 1: Render provision templates (OpenTofu) to build/
  - Stage 2: Provision infrastructure from build directory
  - Stage 3: Render configuration templates (Ansible) with runtime variables
  - Stage 4: Run Ansible playbooks from build/

- **✅ Embedded Template System**: `TemplateManager` with embedded resources and reset functionality for fresh template extraction

- **✅ Error Recovery**: Emergency cleanup with `emergency_destroy` function and proper Drop trait implementation

### 🔢 Module Statistics Improvement

- **Lines of Code**: Reduced from 735 → 427 lines (42% reduction)
- **External Dependencies**: Abstracted behind client interfaces in `src/command_wrappers/`
- **Error Handling**: Structured error types with `anyhow` integration
- **Code Organization**: Separated concerns with dedicated client modules and dependency injection pattern
- **Template System**: Dedicated renderers for different infrastructure components

## 🎯 Expected Benefits from Further Refactoring

**Already Achieved:**

- ✅ **Clean Architecture**: Well-structured 4-stage execution framework
- ✅ **Better Abstraction**: Client libraries provide clean interfaces to external tools
- ✅ **Improved Testability**: Individual clients can be unit tested in isolation
- ✅ **Enhanced Extensibility**: Easy to add new validators through `RemoteAction` trait
- ✅ **Structured Error Handling**: `CommandError` and `anyhow` integration provide detailed error context
- ✅ **Async Performance**: Non-blocking I/O operations improve responsiveness
- ✅ **Template System**: Embedded templates with proper management and rendering
- ✅ **Configuration Foundation**: `Config` and `Services` dependency injection pattern

**Still To Achieve:**

- **Configuration Flexibility**: External configuration files for environment-specific settings
- **Enhanced Observability**: Structured progress reporting and metrics collection
- **Parallel Execution**: Concurrent validation and template operations for improved performance
- **Stage Abstraction**: Independent stage components with trait-based architecture
- **Better Error Context**: More detailed error reporting with stage-specific context and resolution suggestions
- **Provider Extensibility**: Support for infrastructure providers beyond LXD

## 🔍 Updated Analysis Summary

### Current Module Statistics (After Improvements)

- **Lines of code**: 427 lines (reduced from 735, -42%)
- **Methods in TestEnvironment**: 8 methods (reduced from 15+)
- **Direct command executions**: 0 (abstracted behind client interfaces)
- **Client abstractions**: 4 dedicated clients (OpenTofu, SSH, Ansible, LXD) in `src/command_wrappers/`
- **Validation system**: 3 validators with `RemoteAction` trait in `src/actions/`
- **Template renderers**: 2 specialized renderers (`TofuTemplateRenderer`, `AnsibleTemplateRenderer`)
- **Configuration architecture**: `Config` + `Services` dependency injection pattern
- **Hard-coded values**: Still present but significantly reduced

### Remaining Code Smells

1. **God Class Pattern**:

   - `TestEnvironment` still orchestrates everything (437 lines)
   - Handles dependency injection, template rendering, orchestration, and cleanup

2. **Large Methods**:

   - `render_runtime_templates()`: ~80 lines
   - `run_full_deployment_test()`: ~60 lines

3. **Configuration Issues**:

   - All timeouts, paths, and settings still hard-coded
   - No environment-specific configuration support

4. **Limited Observability**:
   - Basic println! logging without structured progress
   - No metrics collection or detailed error reporting

### Progress Assessment

**Before Refactoring (Original State):**

- Maintainability: 3/10
- Testability: 2/10
- Readability: 4/10
- Extensibility: 2/10

**Current State (After Major Improvements):**

- Maintainability: 6/10 (improved client abstractions)
- Testability: 7/10 (clients can be unit tested)
- Readability: 6/10 (cleaner structure, async operations)
- Extensibility: 8/10 (validation system, client interfaces)

**Target State (After Full Refactoring):**

- Maintainability: 9/10 (stage-based architecture)
- Testability: 9/10 (isolated components)
- Readability: 8/10 (configuration-driven, clear stages)
- Extensibility: 9/10 (pluggable stages and configuration)

### Key Achievements

- ✅ **42% code reduction** while maintaining all functionality
- ✅ **Zero direct command calls** - all abstracted behind clients in `src/command_wrappers/`
- ✅ **Extensible validation system** with trait-based architecture in `src/actions/`
- ✅ **Async/await conversion** for better performance
- ✅ **Structured error handling** foundation with `CommandError` and `anyhow` integration
- ✅ **Template architecture** with specialized renderers for different components
- ✅ **Dependency injection** pattern with `Config` and `Services`

---

_Report updated on September 10, 2025_  
_Analysis of: `/src/bin/e2e_tests.rs` (427 lines, improved from 735 lines)_
