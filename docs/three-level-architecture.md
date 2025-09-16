# Three-Level Architecture

> **🏗️ Architectural Foundation**  
> This document outlines the three-level architecture pattern used in the Torrust Tracker Deploy project. This architecture provides clear separation of concerns and enables scalable, maintainable code organization.

## 🏗️ Architecture Overview

The three-level architecture implements a clean abstraction pattern that separates different concerns into distinct layers:

### Level 1: Commands

**Direct mapping to console commands** - Top-level operations that users invoke

- Orchestrates multiple steps to achieve command objectives
- Manages command-specific error handling and reporting
- Currently implemented: `ProvisionCommand`, `ConfigureCommand`, `TestCommand`

### Level 2: Steps

**Reusable building blocks** - Modular operations that can be composed into commands

- Independent, testable units of work
- Can be reused across multiple commands
- Handle specific deployment tasks (template rendering, infrastructure operations, etc.)
- Organized by operation type in dedicated directories

### Level 3: Remote Actions

**Operations executed on remote servers** - SSH-based actions on provisioned infrastructure

- Implemented in `src/remote_actions/`
- Validate remote server state and configuration
- Execute maintenance and setup tasks via SSH
- Can be wrapped into Steps for command composition

## 📁 Directory Structure

The architecture is reflected in the current source code organization:

```text
src/
├── commands/                  # Level 1 (Commands) - Currently Implemented
│   ├── mod.rs
│   ├── provision.rs           # ✅ Infrastructure provisioning command
│   ├── configure.rs           # ✅ System configuration command
│   └── test.rs                # ✅ Validation command
├──
├── steps/                     # Level 2 (Steps) - Currently Implemented
│   ├── mod.rs
│   ├── rendering/             # ✅ Template rendering steps
│   ├── infrastructure/        # ✅ Infrastructure operations
│   ├── connectivity/          # ✅ Network and SSH steps
│   ├── system/                # ✅ Remote system execution
│   ├── validation/            # ✅ Validation and health checks
│   ├── application/           # Directory structure ready
│   └── software/              # ✅ Software installation steps
├──
├── remote_actions/            # Level 3 (Remote Actions) - Currently Implemented
│   ├── mod.rs                 # ✅ RemoteAction trait definition
│   ├── cloud_init.rs          # ✅ Cloud-init validation
│   ├── docker.rs              # ✅ Docker validation
│   └── docker_compose.rs      # ✅ Docker Compose validation
```

## 📋 Implementation Type Hierarchy

### Level 1: Command Types (Currently Implemented)

```rust
// Commands are concrete structs, no common trait yet
struct ProvisionCommand { /* ... */ }  // ✅ Implemented
struct ConfigureCommand { /* ... */ }  // ✅ Implemented
struct TestCommand { /* ... */ }       // ✅ Implemented
```

### Level 2: Step Types (Currently Implemented)

```rust
// Steps are individual structs, no common trait yet
// Template Steps
struct RenderOpenTofuTemplatesStep { /* ... */ }    // ✅ Implemented
struct RenderAnsibleTemplatesStep { /* ... */ }     // ✅ Implemented

// Infrastructure Steps
struct InitializeInfrastructureStep { /* ... */ }   // ✅ Implemented
struct PlanInfrastructureStep { /* ... */ }         // ✅ Implemented
struct ApplyInfrastructureStep { /* ... */ }        // ✅ Implemented
struct GetInstanceInfoStep { /* ... */ }            // ✅ Implemented

// Connectivity Steps
struct WaitForSSHConnectivityStep { /* ... */ }     // ✅ Implemented

// System Steps
struct WaitForCloudInitStep { /* ... */ }           // ✅ Implemented

// Software Steps
struct InstallDockerStep { /* ... */ }              // ✅ Implemented
struct InstallDockerComposeStep { /* ... */ }       // ✅ Implemented

// Validation Steps
struct ValidateCloudInitCompletionStep { /* ... */ } // ✅ Implemented
struct ValidateDockerInstallationStep { /* ... */ }  // ✅ Implemented
struct ValidateDockerComposeInstallationStep { /* ... */ } // ✅ Implemented
```

### Level 3: Remote Action Types (Currently Implemented)

```rust
// RemoteAction trait (implemented)
trait RemoteAction {
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError>;
    fn name(&self) -> &'static str;
}

// Remote action implementations
struct CloudInitValidator { /* ... */ }      // ✅ Implemented
struct DockerValidator { /* ... */ }         // ✅ Implemented
struct DockerComposeValidator { /* ... */ }  // ✅ Implemented
```

## 📊 Architecture Benefits

### Code Quality

- **Reduced complexity**: Large operations broken into focused components
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

## 🔄 Current Command Orchestration Pattern

Commands orchestrate multiple steps to achieve their objectives. Here's how `ProvisionCommand` works:

```rust
impl ProvisionCommand {
    pub async fn execute(&mut self) -> Result<(), ProvisionCommandError> {
        // Execute steps in sequence
        self.render_opentofu_templates().await?;
        self.initialize_infrastructure().await?;
        self.plan_infrastructure().await?;
        self.apply_infrastructure().await?;
        let instance_info = self.get_instance_info().await?;
        self.render_ansible_templates(&instance_info.ip_address).await?;
        self.wait_for_ssh_connectivity(&instance_info.ip_address).await?;
        self.wait_for_cloud_init(&instance_info.ip_address).await?;

        Ok(())
    }

    // Each method delegates to corresponding Step structs
    async fn render_opentofu_templates(&self) -> Result<(), ProvisionTemplateError> {
        RenderOpenTofuTemplatesStep::new(&self.tofu_renderer, &self.config)
            .execute().await
    }
    // ... other step delegations
}
```

## 🎯 Current Usage Examples

### Command Level Usage

```rust
// Command execution through the current E2E test structure
let mut provision_command = ProvisionCommand::new(
    tofu_renderer,
    ansible_renderer,
    config,
    opentofu_client,
    ansible_client,
);
provision_command.execute().await?;
```

### Step Level Usage

```rust
// Individual step execution
let step = RenderOpenTofuTemplatesStep::new(&tofu_renderer, &config);
step.execute().await?;
```

### Remote Action Level Usage

```rust
// Remote validation
let validator = DockerValidator::new(&ssh_credentials);
validator.execute(&server_ip).await?;
```

## 📋 TODO: Not Yet Implemented

- [ ] **Common Command trait** - Commands use different method signatures
- [ ] **Common Step trait** - Steps have varying interfaces
- [ ] **Shared DeploymentContext** - No context passing between steps
- [ ] **CLI subcommands** - No main console application yet
- [ ] **Additional commands**: `check`, `create`, `release`, `run`, `status`, `destroy`
- [ ] **Additional remote actions**: tracker validation, database validation, etc.

This three-level architecture provides a solid foundation for building scalable, maintainable deployment automation systems while maintaining clear separation of concerns at each level.
