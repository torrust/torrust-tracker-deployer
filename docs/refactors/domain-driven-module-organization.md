# Layer-Based Architecture Reorganization

**Date**: September 23, 2025  
**Status**: In Progress  
**Type**: Code Organization Refactoring  
**Decision**: Implement simplified layer-based architecture with single bounded context

## 🎯 Updated Strategy

**Decision Date**: September 23, 2025

We are implementing a **simplified layer-based architecture** approach focusing on DDD layers but with a **single bounded context** to start. This addresses the immediate naming confusion between command-related modules while establishing a foundation for future domain-driven organization.

## 📋 Rationale for Simplified Approach

1. **Immediate Problem**: Resolve naming confusion between `command.rs`, `command_wrappers`, and `commands`
2. **Layer Clarity**: Establish clear Infrastructure and Application layer separation
3. **Incremental Progress**: Take first step toward DDD without over-architecting
4. **Single Context**: Avoid premature bounded context separation until patterns emerge
5. **Foundation Building**: Create structure that can evolve into full DDD when needed

## 🎯 Original Objective

Reorganize the Rust module structure from the current technical-layer approach to a **domain-driven organization** that better reflects the deployment workflow and makes navigation more intuitive for new contributors.

## 📋 Current State Analysis

### Current Strengths

The project already demonstrates several best practices:

✅ **Clear Three-Level Architecture**: Well-defined pattern (Commands → Steps → Remote Actions)  
✅ **Comprehensive Module Documentation**: Every module has `//!` documentation  
✅ **Domain-Based Step Organization**: Steps organized by operational domain  
✅ **External Tool Integration**: Clean abstraction layers for third-party tools  
✅ **Template-Driven Configuration**: Organized template wrappers

### Current Challenges

❌ **Mixed Technical/Domain Organization**: Some modules organized by technical layers, others by domain  
❌ **Scattered Related Functionality**: Related operations spread across different top-level modules  
❌ **Navigation Complexity**: New contributors must understand both technical and domain organization  
❌ **Cross-Module Dependencies**: Many imports spanning different organizational approaches

### Current Module Statistics

- **Total Modules**: 86 Rust files
- **Architecture Levels**: 3 (Commands → Steps → Remote Actions)
- **External Tool Integrations**: 4 (OpenTofu, Ansible, LXD, SSH)
- **Step Categories**: 7 (Infrastructure, System, Software, Validation, Connectivity, Application, Rendering)

## 🏗️ Revised DDD Layer Organization

### Updated Strategy: Complete DDD Layer Separation

Based on refined DDD understanding, we're implementing a complete layer-based organization where **Ansible and OpenTofu are infrastructure concerns** (like web frameworks or databases in traditional DDD):

```text
src/
├── main.rs
├── lib.rs
├── bin/
│   ├── e2e_tests.rs
│   └── linter.rs
├── infrastructure/                 # Infrastructure Layer (DDD)
│   ├── mod.rs
│   ├── executor.rs                 # Current: command.rs - low-level command execution
│   ├── adapters/                   # Current: command_wrappers/ - external tool adapters
│   │   ├── mod.rs
│   │   ├── ansible.rs
│   │   ├── lxd/
│   │   ├── opentofu/
│   │   └── ssh/
│   ├── remote_actions/             # Current: remote_actions/ - repository-like implementations
│   │   ├── mod.rs
│   │   ├── cloud_init.rs
│   │   ├── docker.rs
│   │   └── docker_compose.rs
│   ├── ansible/                    # Current: ansible/ - Ansible implementation details
│   │   ├── mod.rs
│   │   └── template/
│   ├── tofu/                       # Current: tofu/ - OpenTofu implementation details
│   │   ├── mod.rs
│   │   └── template/
│   └── template/                   # Current: template/wrappers/ - template delivery mechanism
│       └── wrappers/
├── application/                    # Application Layer (DDD)
│   ├── mod.rs
│   ├── commands/                   # Current: commands/ - high-level application commands
│   │   ├── mod.rs
│   │   ├── configure.rs
│   │   ├── provision.rs
│   │   └── test.rs
│   └── steps/                      # Current: steps/ - workflow orchestration
│       ├── mod.rs
│       ├── application/
│       ├── connectivity/
│       ├── infrastructure/
│       ├── rendering/
│       ├── software/
│       ├── system/
│       └── validation/
├── domain/                         # Domain Layer (DDD)
│   ├── mod.rs
│   └── template/                   # Current: template/ (minus wrappers) - domain models
│       ├── mod.rs
│       ├── embedded.rs
│       ├── engine.rs
│       ├── file.rs
│       └── file_ops.rs
└── [other existing modules remain unchanged...]
    ├── config/
    ├── e2e/
    ├── container.rs
    └── logging.rs
```

### DDD Layer Rationale

#### Infrastructure Layer (`src/infrastructure/`)

**Contains technical delivery mechanisms and external system integration:**

- **`executor.rs`** (was `command.rs`): Low-level command execution utilities
- **`adapters/`** (was `command_wrappers/`): External tool integration adapters
- **`remote_actions/`**: Repository-like implementations for remote operations (SSH, Docker, cloud-init)
  - _Like repository implementations in traditional DDD_
  - _Current concrete implementations, interfaces can be extracted later_
- **`ansible/`**: Ansible implementation details (like a web framework or ORM)
  - _Technical delivery mechanism, not domain concept_
- **`tofu/`**: OpenTofu implementation details (like database technology)
  - _Infrastructure provisioning delivery mechanism_
- **`template/wrappers/`**: Template rendering delivery mechanism

#### Application Layer (`src/application/`)

**Contains use case orchestration and workflow coordination:**

- **`commands/`**: High-level application commands using Command pattern
- **`steps/`**: Workflow orchestration and business process steps
  - _Perfect fit for application services in DDD_
  - _Orchestrates infrastructure services to fulfill business use cases_

#### Domain Layer (`src/domain/`)

**Contains pure domain models and business logic:**

- **`template/`** (minus `wrappers/`): Core template domain models
  - _Template engine, file operations, embedded template management_
  - _Domain concepts independent of delivery mechanism_

### Benefits of This Revised Organization

1. **True DDD Separation**: Infrastructure concerns (Ansible, OpenTofu) properly separated from domain
2. **Clear Dependency Direction**: Domain ← Application ← Infrastructure
3. **Repository Pattern Foundation**: `remote_actions` positioned for interface extraction
4. **Delivery Mechanism Isolation**: Technical tools isolated from business logic
5. **Scalable Architecture**: Clean foundation for bounded context evolution

### Complete Module Mapping

| Current Location         | New Location                            | DDD Layer      | Rationale                                       |
| ------------------------ | --------------------------------------- | -------------- | ----------------------------------------------- |
| `src/command.rs`         | `src/infrastructure/executor.rs`        | Infrastructure | Low-level command execution utilities           |
| `src/command_wrappers/`  | `src/infrastructure/adapters/`          | Infrastructure | External tool integration adapters              |
| `src/remote_actions/`    | `src/infrastructure/remote_actions/`    | Infrastructure | Repository-like implementations (SSH, etc.)     |
| `src/ansible/`           | `src/infrastructure/ansible/`           | Infrastructure | Ansible delivery mechanism (like web framework) |
| `src/tofu/`              | `src/infrastructure/tofu/`              | Infrastructure | OpenTofu delivery mechanism (like database)     |
| `src/template/wrappers/` | `src/infrastructure/template/wrappers/` | Infrastructure | Template rendering delivery mechanism           |
| `src/commands/`          | `src/application/commands/`             | Application    | High-level application commands                 |
| `src/steps/`             | `src/application/steps/`                | Application    | Workflow orchestration (application services)   |
| `src/template/` (rest)   | `src/domain/template/`                  | Domain         | Core template domain models                     |

### Future Evolution Path

Once this complete DDD layer foundation is established, we can:

- **Extract interfaces** from `remote_actions` to create proper repository abstractions
- **Add bounded contexts** within domain layer as patterns emerge
- **Refactor cross-layer communication** to use dependency inversion
- **Implement domain events** for decoupled communication
- **Evolve into hexagonal architecture** with ports and adapters

## 📈 Benefits of Complete DDD Organization

### Architectural Benefits

1. **True Layer Separation**: Infrastructure concerns (Ansible, OpenTofu) properly isolated from business logic
2. **Clean Dependency Direction**: Domain ← Application ← Infrastructure (hexagonal architecture ready)
3. **Repository Pattern Foundation**: `remote_actions` positioned for interface extraction later
4. **Delivery Mechanism Isolation**: Technical tools (Ansible, OpenTofu) treated as delivery mechanisms
5. **Domain Purity**: Core template logic separated from technical implementation details

### Practical Benefits

1. **Clear Mental Model**: Ansible/OpenTofu are like databases or web frameworks - infrastructure concerns
2. **Easy Testing**: Domain logic can be tested without infrastructure dependencies
3. **Technology Independence**: Can swap Ansible for other config management tools
4. **Scalable Architecture**: Proper foundation for bounded contexts and microservices
5. **Maintainable Codebase**: Clear boundaries prevent architectural drift

## 🚧 Complete Implementation Plan

### Phase 1: Complete DDD Layer Organization

**Estimated Time**: 4-6 hours

- [ ] **1.1** Create layer directories

  - [ ] Create `src/infrastructure/` directory (already exists)
  - [ ] Create `src/infrastructure/remote_actions/` directory
  - [ ] Create `src/infrastructure/ansible/` directory
  - [ ] Create `src/infrastructure/tofu/` directory
  - [ ] Create `src/infrastructure/template/wrappers/` directory
  - [ ] Create `src/application/` directory (already exists)
  - [ ] Create `src/application/steps/` directory
  - [ ] Create `src/domain/` directory (already exists)
  - [ ] Create `src/domain/template/` directory

- [ ] **1.2** Move infrastructure layer files

  - [ ] Move `src/command.rs` → `src/infrastructure/executor.rs` (already done)
  - [ ] Move `src/command_wrappers/` → `src/infrastructure/adapters/` (already done)
  - [ ] Move `src/remote_actions/` → `src/infrastructure/remote_actions/`
  - [ ] Move `src/ansible/` → `src/infrastructure/ansible/`
  - [ ] Move `src/tofu/` → `src/infrastructure/tofu/`
  - [ ] Move `src/template/wrappers/` → `src/infrastructure/template/wrappers/`

- [ ] **1.3** Move application layer files

  - [ ] Move `src/commands/` → `src/application/commands/` (already done)
  - [ ] Move `src/steps/` → `src/application/steps/`

- [ ] **1.4** Move domain layer files

  - [ ] Move `src/template/` (minus `wrappers/`) → `src/domain/template/`
    - [ ] Move `src/template/embedded.rs` → `src/domain/template/embedded.rs`
    - [ ] Move `src/template/engine.rs` → `src/domain/template/engine.rs`
    - [ ] Move `src/template/file.rs` → `src/domain/template/file.rs`
    - [ ] Move `src/template/file_ops.rs` → `src/domain/template/file_ops.rs`
    - [ ] Move `src/template/mod.rs` → `src/domain/template/mod.rs` (updated)
    - [ ] Remove original `src/template/` directory

- [ ] **1.5** Update module files and re-exports

  - [ ] Update `src/infrastructure/mod.rs`
  - [ ] Update `src/application/mod.rs`
  - [ ] Update `src/domain/mod.rs`
  - [ ] Update `src/lib.rs` with new module structure

- [ ] **1.6** Update imports and references

  - [ ] Update all `use crate::remote_actions` → `use crate::infrastructure::remote_actions`
  - [ ] Update all `use crate::ansible` → `use crate::infrastructure::ansible`
  - [ ] Update all `use crate::tofu` → `use crate::infrastructure::tofu`
  - [ ] Update all `use crate::template::wrappers` → `use crate::infrastructure::template::wrappers`
  - [ ] Update all `use crate::steps` → `use crate::application::steps`
  - [ ] Update all `use crate::template` → `use crate::domain::template` (for non-wrapper imports)

- [ ] **1.7** Validation and testing

  - [ ] Run `cargo build` and fix compilation errors
  - [ ] Run `cargo test` and fix test issues
  - [ ] Run `cargo run --bin linter all` and fix linting issues
  - [ ] Run `cargo run --bin e2e-tests` for integration verification

### Future Phases

**Phase 2**: Interface Extraction

- Extract repository interfaces from `infrastructure/remote_actions`
- Implement dependency inversion between application and infrastructure layers
- Add configuration abstractions for external tools

**Phase 3**: Domain Evolution

- Add bounded contexts within domain layer as complexity grows
- Implement domain events for decoupled communication
- Evolve toward hexagonal architecture with ports and adapters

## 📊 Progress Tracking

### Phase 1 Progress: **Ready to Start**

| Step                           | Status         | Completion | Notes                    |
| ------------------------------ | -------------- | ---------- | ------------------------ |
| 1.1: Create layer directories  | ⏳ Not Started | 0%         | Partial: infra/app exist |
| 1.2: Move infrastructure files | ⏳ Not Started | 0%         | executor/adapters done   |
| 1.3: Move application files    | ⏳ Not Started | 0%         | commands done            |
| 1.4: Move domain files         | ⏳ Not Started | 0%         | Template separation      |
| 1.5: Update module files       | ⏳ Not Started | 0%         | Re-export updates        |
| 1.6: Update imports/references | ⏳ Not Started | 0%         | Comprehensive refactor   |
| 1.7: Validation and testing    | ⏳ Not Started | 0%         | Final validation         |

**Legend**: ⏳ Not Started | 🔄 In Progress | ✅ Complete

## 🔄 Current Status: **Plan Updated with DDD Insights**

**Key Insight**: Ansible and OpenTofu are delivery mechanisms (like web frameworks or databases), not domain concepts.

**Updated Strategy**: Complete DDD layer separation with proper infrastructure/application/domain boundaries.

**Next Steps**:

1. Commit this refined plan with DDD insights
2. Implement complete layer reorganization
3. Validate architectural boundaries are properly maintained
4. Plan interface extraction for repository pattern evolution
