# Codebase Architecture Overview

This document provides a comprehensive overview of the Rust codebase architecture, organizing all modules by their functional responsibilities and relationships within the deployment architecture.

## 🏗️ Three-Level Architecture Pattern

> **Architectural Foundation**: This architecture provides clear separation of concerns and enables scalable, maintainable code organization through distinct abstraction layers.

The project implements a **three-level architecture** for deployment automation:

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

- Validate remote server state and configuration
- Execute maintenance and setup tasks via SSH
- Can be wrapped into Steps for command composition

## 🔧 Supporting Systems

This architecture is supported by:

- **Command Wrappers** - Integration with external tools (`OpenTofu`, `Ansible`, `LXD`, `SSH`)
- **Template System** - Configuration template rendering and management
- **E2E Framework** - End-to-end testing and validation infrastructure

## 🔄 Architecture Flow & Command Orchestration

## Deployment Flow Pattern

The typical deployment flow follows this pattern:

1. **Commands** receive user input and orchestrate the deployment process
2. **Steps** execute specific deployment operations in sequence:
   - **Rendering** - Generate configuration files from templates
   - **Infrastructure** - Provision and manage infrastructure resources
   - **Connectivity** - Establish and verify network connections
   - **System** - Configure system-level settings
   - **Software** - Install and configure required software
   - **Validation** - Verify successful installation and configuration
   - **Application** - Deploy and manage applications
3. **Remote Actions** perform low-level operations on remote systems
4. **Command Wrappers** provide integration with external tools
5. **Template System** manages configuration generation throughout the process

### Command Orchestration Example

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

## 📚 Module Documentation

All modules include comprehensive `//!` documentation with:

- Clear module purpose descriptions
- Key features and functionality
- Integration points with other modules
- Usage context and examples where appropriate

## 🏢 Module Organization

### Core Infrastructure

**Root Level Files:**

- ✅ `src/main.rs` - Main binary entry point
- ✅ `src/command.rs` - Command execution utilities with error handling
- ✅ `src/container.rs` - Dependency injection container
- ✅ `src/logging.rs` - Logging configuration and utilities
- ✅ `src/lib.rs` - Library root module

**Binary Files:**

- ✅ `src/bin/e2e_tests.rs` - E2E testing binary
- ✅ `src/bin/linter.rs` - Code quality linting binary

### Level 1: High-Level Commands

**Command Modules:**

- ✅ `src/commands/mod.rs` - High-level deployment commands
- ✅ `src/commands/configure.rs` - Infrastructure configuration command
- ✅ `src/commands/provision.rs` - Infrastructure provisioning command
- ✅ `src/commands/test.rs` - Infrastructure testing command

### External Tool Integration

Command wrappers provide clean abstractions for integrating with external deployment tools, handling command execution, output parsing, and error management.

**Ansible Integration:**

Provides integration with `Ansible` for configuration management and software installation on remote systems.

- ✅ `src/ansible/mod.rs` - Ansible playbook integration and coordination
- ✅ `src/ansible/template/mod.rs` - Ansible template functionality module
- ✅ `src/ansible/template/renderer/mod.rs` - Ansible template rendering coordination
- ✅ `src/ansible/template/renderer/inventory.rs` - Ansible inventory template rendering

**Command Wrappers:**

- ✅ `src/command_wrappers/mod.rs` - Common wrapper utilities for external tools
- ✅ `src/command_wrappers/ansible.rs` - `Ansible` command execution wrapper

**SSH Wrappers:**

Enable secure remote access to provisioned systems for executing commands and file transfers.

- ✅ `src/command_wrappers/ssh/mod.rs` - SSH integration module and error handling
- ✅ `src/command_wrappers/ssh/client.rs` - SSH client implementation for remote operations
- ✅ `src/command_wrappers/ssh/connection.rs` - SSH connection configuration management
- ✅ `src/command_wrappers/ssh/credentials.rs` - SSH authentication credentials handling

**LXD Wrappers:**

Interface with LXD for container and virtual machine management, providing local development environments.

- ✅ `src/command_wrappers/lxd/mod.rs` - LXD container/VM management coordination
- ✅ `src/command_wrappers/lxd/client.rs` - LXD client implementation and command execution
- ✅ `src/command_wrappers/lxd/json_parser.rs` - Parse LXD JSON response data
- ✅ `src/command_wrappers/lxd/instance/mod.rs` - LXD instance type definitions
- ✅ `src/command_wrappers/lxd/instance/info.rs` - Instance information data structures
- ✅ `src/command_wrappers/lxd/instance/name.rs` - Instance name validation and formatting

**OpenTofu Wrappers:**

Interface with `OpenTofu` for infrastructure-as-code operations, managing infrastructure provisioning and state.

- ✅ `src/command_wrappers/opentofu/mod.rs` - `OpenTofu` infrastructure management coordination
- ✅ `src/command_wrappers/opentofu/client.rs` - `OpenTofu` client implementation and command execution
- ✅ `src/command_wrappers/opentofu/json_parser.rs` - Parse `OpenTofu` JSON output and state

### Configuration and Testing Framework

**Configuration Management:**

Handles application configuration loading, validation, and environment-specific settings management.

- ✅ `src/config/mod.rs` - Application configuration management and validation

**E2E Testing Infrastructure:**

Comprehensive end-to-end testing framework that validates complete deployment workflows from infrastructure provisioning to application deployment.

- ✅ `src/e2e/mod.rs` - End-to-end testing framework coordination and test execution
- ✅ `src/e2e/environment.rs` - Test environment setup and teardown management

**E2E Task Modules:**

Individual task modules that compose complete end-to-end test scenarios, validating different aspects of the deployment pipeline.

- ✅ `src/e2e/tasks/setup_ssh_key.rs` - SSH key generation and setup for secure access
- ✅ `src/e2e/tasks/configure_infrastructure.rs` - Infrastructure configuration validation
- ✅ `src/e2e/tasks/cleanup_infrastructure.rs` - Infrastructure cleanup and resource deallocation
- ✅ `src/e2e/tasks/validate_deployment.rs` - Complete deployment validation and health checks
- ✅ `src/e2e/tasks/provision_infrastructure.rs` - Infrastructure provisioning validation
- ✅ `src/e2e/tasks/clean_and_prepare_templates.rs` - Template cleanup and preparation for testing
- ✅ `src/e2e/tasks/preflight_cleanup.rs` - Pre-test environment cleanup and initialization

### Level 2: Granular Deployment Steps

Steps are the core building blocks of deployment workflows, providing reusable, composable operations that can be orchestrated by Commands. Each step category handles specific aspects of the deployment process.

**Infrastructure Steps:**

Manage the infrastructure lifecycle using `OpenTofu`, from planning and initialization to provisioning and information retrieval.

- ✅ `src/steps/infrastructure/mod.rs` - Infrastructure lifecycle management
- ✅ `src/steps/infrastructure/initialize.rs` - Initialize `OpenTofu` backend and providers
- ✅ `src/steps/infrastructure/apply.rs` - Apply infrastructure changes and provision resources
- ✅ `src/steps/infrastructure/get_instance_info.rs` - Retrieve provisioned instance information
- ✅ `src/steps/infrastructure/plan.rs` - Generate and validate infrastructure execution plans
- ✅ `src/steps/infrastructure/validate.rs` - Validate infrastructure configuration and state

**System-Level Steps:**

Handle system-level operations and waiting for system initialization processes to complete.

- ✅ `src/steps/system/mod.rs` - System-level configuration steps
- ✅ `src/steps/system/wait_cloud_init.rs` - Wait for cloud-init completion on remote systems

**Template Rendering Steps:**

Generate configuration files from templates, preparing tool-specific configurations for deployment.

- ✅ `src/steps/rendering/mod.rs` - Configuration template rendering coordination
- ✅ `src/steps/rendering/opentofu_templates.rs` - Generate `OpenTofu` configuration files
- ✅ `src/steps/rendering/ansible_templates.rs` - Generate `Ansible` playbook configurations

**Software Installation Steps:**

Install and configure required software on remote systems using `Ansible` playbooks.

- ✅ `src/steps/software/mod.rs` - Software installation and configuration coordination
- ✅ `src/steps/software/docker.rs` - Install Docker engine on remote systems
- ✅ `src/steps/software/docker_compose.rs` - Install Docker Compose tool

**Validation Steps:**

Verify successful installation and configuration of system components and software.

- ✅ `src/steps/validation/mod.rs` - System and software validation coordination
- ✅ `src/steps/validation/docker.rs` - Validate Docker engine installation and functionality
- ✅ `src/steps/validation/docker_compose.rs` - Verify Docker Compose installation
- ✅ `src/steps/validation/cloud_init.rs` - Confirm cloud-init process completion

**Connectivity Steps:**

Establish and verify network connections to remote systems, ensuring systems are accessible.

- ✅ `src/steps/connectivity/mod.rs` - Network connectivity operations coordination
- ✅ `src/steps/connectivity/wait_ssh_connectivity.rs` - Wait for SSH access to remote systems

**Application Steps:**

Handle application deployment and lifecycle management (prepared for future implementation).

- ✅ `src/steps/application/mod.rs` - Application deployment and lifecycle coordination

### Level 3: Remote System Operations

Remote Actions represent the lowest level of the architecture, performing direct operations on remote systems via SSH. These actions validate system state, install software, and execute maintenance tasks on provisioned infrastructure.

**Remote Actions:**

- ✅ `src/remote_actions/mod.rs` - Remote system operation definitions and traits
- ✅ `src/remote_actions/cloud_init.rs` - Validates cloud-init completion status
- ✅ `src/remote_actions/docker.rs` - Verifies Docker engine installation and status
- ✅ `src/remote_actions/docker_compose.rs` - Validates Docker Compose availability

### Template System

The template system provides dynamic configuration file generation using the Tera templating engine. It enables flexible, environment-specific configurations for both `OpenTofu` infrastructure definitions and `Ansible` playbooks.

**Template Engine:**

- ✅ `src/template/mod.rs` - Template system root module
- ✅ `src/template/engine.rs` - Tera template engine integration
- ✅ `src/template/file.rs` - Template file management
- ✅ `src/template/file_ops.rs` - File operations for templates
- ✅ `src/template/embedded.rs` - Embedded template resources

**Template Wrappers:**

Template wrappers provide specialized rendering logic for different tool configurations, handling tool-specific template variables and output formats.

- ✅ `src/template/wrappers/mod.rs` - Template wrapper utilities
- ✅ `src/template/wrappers/ansible/mod.rs` - Ansible template wrappers
- ✅ `src/template/wrappers/ansible/inventory/mod.rs` - Ansible inventory templates
- ✅ `src/template/wrappers/ansible/inventory/context/mod.rs` - Inventory context management
- ✅ `src/template/wrappers/tofu/mod.rs` - OpenTofu template wrappers
- ✅ `src/template/wrappers/tofu/lxd/mod.rs` - LXD-specific OpenTofu template wrappers

**Tofu Integration:**

Specialized integration for `OpenTofu` template processing, handling infrastructure-as-code template rendering with proper variable substitution.

- ✅ `src/tofu/mod.rs` - OpenTofu integration module
- ✅ `src/tofu/template/mod.rs` - OpenTofu template functionality module
- ✅ `src/tofu/template/renderer/mod.rs` - OpenTofu template rendering coordination
- ✅ `src/tofu/template/renderer/cloud_init.rs` - Cloud-init template rendering for OpenTofu

## 🔄 Architecture Flow

The typical deployment flow follows this pattern:

1. **Commands** receive user input and orchestrate the deployment process
2. **Steps** execute specific deployment operations in sequence:
   - **Rendering** - Generate configuration files from templates
   - **Infrastructure** - Provision and manage infrastructure resources
   - **Connectivity** - Establish and verify network connections
   - **System** - Configure system-level settings
   - **Software** - Install and configure required software
   - **Validation** - Verify successful installation and configuration
   - **Application** - Deploy and manage applications
3. **Remote Actions** perform low-level operations on remote systems
4. **Command Wrappers** provide integration with external tools
5. **Template System** manages configuration generation throughout the process

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

## 🚀 Recent Architecture Improvements

### Hierarchical Module Organization (September 2024)

Recent refactoring efforts have improved the module organization for both `Ansible` and `OpenTofu` integrations:

**Before:**

```text
src/ansible/template_renderer.rs
src/tofu/template_renderer.rs
src/tofu/cloud_init_template_renderer.rs
```

**After:**

```text
src/ansible/template/
├── mod.rs
└── renderer/
    ├── mod.rs (AnsibleTemplateRenderer)
    └── inventory.rs (InventoryTemplateRenderer)

src/tofu/template/
├── mod.rs
└── renderer/
    ├── mod.rs (TofuTemplateRenderer)
    └── cloud_init.rs (CloudInitTemplateRenderer)
```

**Benefits of the New Structure:**

- **Eliminated Redundant Naming**: Removed `_template_renderer` suffixes from file names
- **Logical Hierarchy**: Clear `template/renderer/` organization pattern
- **Better Separation**: Specialized renderers properly grouped by functionality
- **Consistent Pattern**: Same organizational approach across both modules
- **Improved Maintainability**: Easier to locate and modify specific template renderers

This refactoring maintains full backward compatibility while providing a cleaner, more maintainable codebase structure.

## 📊 Module Statistics

- **Total Modules**: 86 Rust files
- **Architecture Levels**: 3 (Commands → Steps → Remote Actions)
- **External Tool Integrations**: 4 (`OpenTofu`, `Ansible`, `LXD`, `SSH`)
- **Step Categories**: 7 (Infrastructure, System, Software, Validation, Connectivity, Application, Rendering)

## 💡 Key Design Principles

- **Separation of Concerns**: Each module has a single, well-defined responsibility
- **Composability**: Steps can be combined to create complex deployment workflows
- **Testability**: E2E framework enables comprehensive testing of deployment scenarios
- **External Tool Integration**: Clean abstraction layers for third-party tools
- **Template-Driven Configuration**: Flexible configuration management through templates
