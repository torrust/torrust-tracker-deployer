# Codebase Architecture Overview

This document provides a comprehensive overview of the Rust codebase architecture, organizing all modules by their functional responsibilities and relationships within the deployment architecture.

## 🎨 Domain-Driven Design (DDD) Architecture

The project follows **Domain-Driven Design** principles with a layered architecture that enforces clear separation of concerns and dependency rules:

### Layer Structure

```text
┌─────────────────────────────────────┐
│      Application Layer              │
│  (Commands, Use Cases, Steps)       │
│  src/application/                   │
└────────────┬────────────────────────┘
             │ depends on
             ↓
┌─────────────────────────────────────┐
│       Domain Layer                  │
│  (Business Logic, Entities,         │
│   Value Objects)                    │
│  src/domain/                        │
└─────────────────────────────────────┘
             ↑
             │ depends on
             │
┌─────────────────────────────────────┐
│    Infrastructure Layer             │
│  (External Tools, File System,      │
│   SSH, Templates, Trace Writers)    │
│  src/infrastructure/                │
└─────────────────────────────────────┘
```

### Layer Responsibilities

**Domain Layer** (`src/domain/`):

- **Purpose**: Core business logic and domain entities
- **Contains**: Entities (Environment), Value Objects (EnvironmentName, TraceId), State Machine (type-state pattern)
- **Rules**: No dependencies on infrastructure or application layers
- **Example**: `Environment<S>` entity with type-state pattern for deployment lifecycle

**Application Layer** (`src/application/`):

- **Purpose**: Use cases and command orchestration
- **Contains**: Commands, Steps, Application services
- **Rules**: Depends on domain layer, coordinates infrastructure services
- **Example**: `ProvisionCommand` orchestrating provisioning workflow

**Infrastructure Layer** (`src/infrastructure/`):

- **Purpose**: External integrations and technical implementations
- **Contains**: File system operations, SSH clients, OpenTofu/Ansible wrappers, trace writers
- **Rules**: Implements domain interfaces, depends on domain layer
- **Example**: `ProvisionTraceWriter` implementing trace file persistence

### Dependency Rule

The fundamental rule is that **dependencies flow inward**:

- Infrastructure → Domain (✅ Correct)
- Application → Domain (✅ Correct)
- Domain → Infrastructure (❌ Forbidden)
- Domain → Application (❌ Forbidden)

This ensures the domain layer remains pure business logic, free from technical implementation details.

## 🏗️ Three-Level Architecture Pattern

> **Architectural Foundation**: This architecture provides clear separation of concerns and enables scalable, maintainable code organization through distinct abstraction layers.

The project implements a **three-level architecture** for deployment automation:

### Level 1: Commands

**Direct mapping to console commands** - Top-level operations that users invoke

- Orchestrates multiple steps to achieve command objectives
- Manages command-specific error handling and reporting
- Currently implemented: `ProvisionCommand`, `ConfigureCommand`

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

- **External Tool Adapters** - Integration with external tools (`OpenTofu`, `Ansible`, `LXD`, `SSH`)
- **Template System** - Configuration template rendering and management
- **E2E Framework** - End-to-end testing and validation infrastructure

## 🔄 Architecture Flow & Command Orchestration

### Deployment Flow Pattern

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
4. **External Tool Adapters** provide integration with external tools
5. **Template System** manages configuration generation throughout the process

### Command Orchestration Example

Commands orchestrate multiple steps to achieve their objectives. Here's how `ProvisionCommand` works:

```rust
impl ProvisionCommand {
    pub async fn execute(&mut self) -> Result<Environment<Provisioned>, ProvisionCommandError> {
        // Execute steps in sequence
        self.render_opentofu_templates().await?;
        self.initialize_infrastructure().await?;
        self.plan_infrastructure().await?;
        self.apply_infrastructure().await?;
        let instance_info = self.get_instance_info().await?;
        self.render_ansible_templates(&instance_info.ip_address).await?;
        self.wait_for_ssh_connectivity(&instance_info.ip_address).await?;
        self.wait_for_cloud_init(&instance_info.ip_address).await?;

        Ok(provisioned_environment)
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
- ✅ `src/container.rs` - Dependency injection container
- ✅ `src/logging.rs` - Logging configuration and utilities
- ✅ `src/lib.rs` - Library root module

**Binary Files:**

- ✅ `src/bin/linter.rs` - Code quality linting binary
- ✅ `src/bin/e2e-config-tests.rs` - E2E configuration tests
- ✅ `src/bin/e2e-provision-tests.rs` - E2E provisioning tests
- ✅ `src/bin/e2e-tests-full.rs` - Full E2E test suite

### Domain Layer

**Core Domain Entities:**

- ✅ `src/domain/mod.rs` - Domain layer root module
- ✅ `src/domain/environment/mod.rs` - Environment entity and aggregate root
- ✅ `src/domain/environment/name.rs` - Environment name value object
- ✅ `src/domain/environment/trace_id.rs` - Trace identifier value object
- ✅ `src/domain/environment/repository.rs` - Environment repository trait
- ✅ `src/domain/environment/state/` - Environment state machine (type-state pattern)
- ✅ `src/domain/instance_name.rs` - Instance name value object
- ✅ `src/domain/profile_name.rs` - Profile name value object

**Domain Template System:**

- ✅ `src/domain/template/mod.rs` - Template domain module
- ✅ `src/domain/template/engine.rs` - Template engine abstraction
- ✅ `src/domain/template/file.rs` - Template file domain entity
- ✅ `src/domain/template/file_ops.rs` - Template file operations

### Application Layer

**Level 1: High-Level Commands:**

- ✅ `src/application/mod.rs` - Application layer root module
- ✅ `src/application/commands/mod.rs` - Command coordination
- ✅ `src/application/commands/provision.rs` - Infrastructure provisioning command
- ✅ `src/application/commands/configure.rs` - Infrastructure configuration command

**Level 2: Granular Deployment Steps:**

Steps are the core building blocks of deployment workflows, providing reusable, composable operations.

**Infrastructure Steps:**

- ✅ `src/application/steps/infrastructure/mod.rs` - Infrastructure lifecycle management
- ✅ `src/application/steps/infrastructure/initialize.rs` - Initialize OpenTofu backend
- ✅ `src/application/steps/infrastructure/apply.rs` - Apply infrastructure changes
- ✅ `src/application/steps/infrastructure/get_instance_info.rs` - Retrieve instance information
- ✅ `src/application/steps/infrastructure/plan.rs` - Generate execution plans
- ✅ `src/application/steps/infrastructure/validate.rs` - Validate infrastructure configuration

**System-Level Steps:**

- ✅ `src/application/steps/system/mod.rs` - System-level configuration steps
- ✅ `src/application/steps/system/wait_cloud_init.rs` - Wait for cloud-init completion

**Template Rendering Steps:**

- ✅ `src/application/steps/rendering/mod.rs` - Template rendering coordination
- ✅ `src/application/steps/rendering/opentofu_templates.rs` - Generate OpenTofu configurations
- ✅ `src/application/steps/rendering/ansible_templates.rs` - Generate Ansible configurations

**Software Installation Steps:**

- ✅ `src/application/steps/software/mod.rs` - Software installation coordination
- ✅ `src/application/steps/software/docker.rs` - Install Docker engine
- ✅ `src/application/steps/software/docker_compose.rs` - Install Docker Compose

**Validation Steps:**

- ✅ `src/application/steps/validation/mod.rs` - System and software validation
- ✅ `src/application/steps/validation/docker.rs` - Validate Docker installation
- ✅ `src/application/steps/validation/docker_compose.rs` - Verify Docker Compose
- ✅ `src/application/steps/validation/cloud_init.rs` - Confirm cloud-init completion

**Connectivity Steps:**

- ✅ `src/application/steps/connectivity/mod.rs` - Network connectivity operations
- ✅ `src/application/steps/connectivity/wait_ssh_connectivity.rs` - Wait for SSH access

**Application Steps:**

- ✅ `src/application/steps/application/mod.rs` - Application deployment coordination

### Infrastructure Layer

**External Tool Adapters:**

Generic CLI wrappers for external tools (moved from shared/ and infrastructure/):

- ✅ `src/adapters/mod.rs` - Adapter module root with re-exports
- ✅ `src/adapters/ansible/mod.rs` - Ansible CLI wrapper
- ✅ `src/adapters/docker/` - Docker CLI wrapper
- ✅ `src/adapters/lxd/` - LXD CLI wrapper (client, instance management, JSON parsing)
- ✅ `src/adapters/ssh/` - SSH client wrapper
- ✅ `src/adapters/tofu/` - OpenTofu CLI wrapper (client, JSON parsing)

**External Tool Configuration (Application-Specific):**

Application-specific template rendering and configuration for external tools:

**Ansible Configuration:**

- ✅ `src/infrastructure/external_tools/ansible/mod.rs` - Ansible integration root
- ✅ `src/infrastructure/external_tools/ansible/template/mod.rs` - Ansible templates
- ✅ `src/infrastructure/external_tools/ansible/template/renderer/mod.rs` - Template rendering
- ✅ `src/infrastructure/external_tools/ansible/template/renderer/inventory.rs` - Inventory rendering
- ✅ `src/infrastructure/external_tools/ansible/template/wrappers/inventory/` - Inventory template wrappers

**OpenTofu Configuration:**

- ✅ `src/infrastructure/external_tools/tofu/mod.rs` - OpenTofu integration root
- ✅ `src/infrastructure/external_tools/tofu/template/mod.rs` - OpenTofu templates
- ✅ `src/infrastructure/external_tools/tofu/template/renderer/mod.rs` - Template rendering
- ✅ `src/infrastructure/external_tools/tofu/template/renderer/cloud_init.rs` - Cloud-init rendering
- ✅ `src/infrastructure/external_tools/tofu/template/wrappers/lxd/` - LXD template wrappers

**Level 3: Remote System Operations:**

- ✅ `src/infrastructure/remote_actions/mod.rs` - Remote operations root
- ✅ `src/infrastructure/remote_actions/cloud_init.rs` - Validate cloud-init completion
- ✅ `src/infrastructure/remote_actions/docker.rs` - Verify Docker installation
- ✅ `src/infrastructure/remote_actions/docker_compose.rs` - Validate Docker Compose

**Persistence Layer:**

- ✅ `src/infrastructure/persistence/mod.rs` - Persistence layer root
- ✅ `src/infrastructure/persistence/filesystem/mod.rs` - Filesystem persistence
- ✅ `src/infrastructure/persistence/filesystem/file_environment_repository.rs` - Environment file storage
- ✅ `src/infrastructure/persistence/filesystem/file_lock.rs` - File locking mechanism
- ✅ `src/infrastructure/persistence/filesystem/json_file_repository.rs` - Generic JSON file repository
- ✅ `src/infrastructure/persistence/repository_factory.rs` - Repository factory

**Trace System:**

- ✅ `src/infrastructure/trace/mod.rs` - Trace system root
- ✅ `src/infrastructure/trace/common.rs` - Common trace utilities
- ✅ `src/infrastructure/trace/provision.rs` - Provision command trace writer
- ✅ `src/infrastructure/trace/configure.rs` - Configure command trace writer

### Shared Layer

**Cross-Cutting Concerns:**

Generic utilities used across all layers:

- ✅ `src/shared/mod.rs` - Shared utilities root
- ✅ `src/shared/command/mod.rs` - Command execution utilities (used by all adapters)
- ✅ `src/shared/clock.rs` - Time abstraction for deterministic testing
- ✅ `src/shared/error/mod.rs` - Shared error types
- ✅ `src/shared/port_checker.rs` - Network port checking
- ✅ `src/shared/port_usage_checker.rs` - Port usage validation
- ✅ `src/shared/username.rs` - Username value object

Note: SSH and Docker adapters have been moved to `src/adapters/`

### Testing Infrastructure

**E2E Testing Framework:**

- ✅ `src/e2e/mod.rs` - E2E testing framework coordination
- ✅ `src/e2e/containers/mod.rs` - Container-based testing infrastructure
- ✅ `src/e2e/containers/actions/` - E2E test actions
- ✅ `src/e2e/containers/provisioned.rs` - Provisioned container management

**Configuration:**

- ✅ `src/config/mod.rs` - Application configuration management

## 🔄 Architecture Flow

The typical deployment flow follows this pattern:

1. **Commands** (Application Layer) receive user input and orchestrate the deployment
2. **Steps** (Application Layer) execute specific operations by coordinating:
   - **Domain Entities** - Environment state transitions
   - **Infrastructure Services** - External tool adapters, persistence, remote actions
3. **Infrastructure Layer** handles all external integrations:
   - External tool execution (OpenTofu, Ansible, LXD)
   - File system operations (templates, state persistence)
   - Remote SSH operations
   - Trace file generation

## 📊 Architecture Benefits

### Code Quality

- **DDD Principles**: Clear separation between domain logic, application use cases, and infrastructure
- **Reduced complexity**: Large operations broken into focused components
- **Better testability**: Each layer and component can be tested independently
- **Type Safety**: Type-state pattern prevents invalid state transitions at compile time

### Maintainability

- **Modular structure**: Changes in one layer don't affect others
- **Clear interfaces**: Well-defined boundaries between layers
- **Easy extension**: Adding new commands/steps/actions follows established patterns
- **Dependency Direction**: Domain remains independent of infrastructure details

### Production Readiness

- **State Management**: Type-safe environment state transitions with persistence
- **Error Context**: Structured error handling with trace files for debugging
- **Progress Reporting**: User-friendly feedback during long-running operations
- **File Locking**: Prevents concurrent access conflicts

## 📊 Module Statistics

- **Total Modules**: ~100+ Rust files
- **Architecture Layers**: 3 (Domain, Application, Infrastructure) + Shared
- **External Tool Integrations**: 3 (OpenTofu, Ansible, LXD)
- **Step Categories**: 7 (Infrastructure, System, Software, Validation, Connectivity, Application, Rendering)
- **State Types**: 13+ environment states with type-state pattern

## 💡 Key Design Principles

- **Domain-Driven Design**: Pure domain logic independent of infrastructure
- **Separation of Concerns**: Each module has a single, well-defined responsibility
- **Dependency Inversion**: Depend on abstractions, not concretions
- **Type Safety**: Leverage Rust's type system for correctness
- **Composability**: Steps combine to create complex deployment workflows
- **Observability**: Comprehensive logging and trace file generation
- **Testability**: E2E framework enables full deployment workflow testing
