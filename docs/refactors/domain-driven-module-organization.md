# Domain-Driven Module Organization Refactor

**Date**: September 17, 2025  
**Status**: Future Proposal - Deferred  
**Type**: Code Organization Refactoring  
**Decision**: Wait for system evolution before implementation

## 🚫 Implementation Decision

**Decision Date**: September 17, 2025

After careful consideration, we have decided to **defer this refactoring** for the following reasons:

1. **System Still Evolving**: The codebase is not yet large enough to justify the effort
2. **More Commands Coming**: Need to implement additional commands to better understand patterns
3. **Wait and See Approach**: Let the system grow organically to validate the proposed structure
4. **Risk vs Benefit**: Current organization is working well, refactoring effort may not be justified yet
5. **Future Reevaluation**: Will revisit this proposal when we have more commands and functionality

This document remains as a **reference proposal** for future consideration when the system has grown significantly.

## 🔄 Reevaluation Criteria

This proposal should be reconsidered when:

- [ ] We have implemented 5+ additional commands
- [ ] The codebase reaches 150+ Rust files
- [ ] New contributor feedback indicates navigation difficulties
- [ ] Cross-module dependencies become problematic
- [ ] Domain boundaries become clearer through usage patterns

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

## 🏗️ Proposed Organization

### Design Principles

1. **Domain-Driven Structure**: Organize by deployment domains rather than technical patterns
2. **Workflow Alignment**: Structure mirrors actual deployment workflow
3. **Bounded Contexts**: Clear ownership and responsibility per domain
4. **Discoverability**: Intuitive navigation for new contributors
5. **Minimal Cross-Dependencies**: Related functionality co-located
6. **Single Responsibility**: Each module has one clear purpose

### New Structure

```text
src/
├── main.rs
├── lib.rs
├── bin/
│   ├── e2e_tests.rs
│   └── linter.rs
├── core/                           # Core application infrastructure
│   ├── mod.rs
│   ├── config/
│   ├── logging.rs
│   ├── container.rs
│   └── command.rs
├── domains/                        # Domain-driven organization
│   ├── mod.rs
│   ├── infrastructure/             # Everything related to infrastructure
│   │   ├── mod.rs
│   │   ├── provisioning/           # Current: commands/provision + steps/infrastructure
│   │   │   ├── mod.rs
│   │   │   ├── commands.rs
│   │   │   ├── initialize.rs
│   │   │   ├── plan.rs
│   │   │   ├── apply.rs
│   │   │   └── instance_info.rs
│   │   ├── templates/              # Current: steps/rendering + tofu/template
│   │   │   ├── mod.rs
│   │   │   ├── opentofu.rs
│   │   │   └── cloud_init.rs
│   │   └── validation.rs           # Current: steps/validation (infra parts)
│   ├── configuration/              # Everything related to system configuration
│   │   ├── mod.rs
│   │   ├── management/             # Current: commands/configure + parts of steps
│   │   │   ├── mod.rs
│   │   │   ├── commands.rs
│   │   │   └── orchestration.rs
│   │   ├── templates/              # Current: ansible/template + template/wrappers/ansible
│   │   │   ├── mod.rs
│   │   │   ├── inventory.rs
│   │   │   └── playbooks.rs
│   │   ├── connectivity/           # Current: steps/connectivity
│   │   │   ├── mod.rs
│   │   │   └── ssh_wait.rs
│   │   └── system/                 # Current: steps/system
│   │       ├── mod.rs
│   │       └── cloud_init.rs
│   ├── software/                   # Everything related to software management
│   │   ├── mod.rs
│   │   ├── installation/           # Current: steps/software
│   │   │   ├── mod.rs
│   │   │   ├── docker.rs
│   │   │   └── docker_compose.rs
│   │   └── validation/             # Current: steps/validation (software parts)
│   │       ├── mod.rs
│   │       ├── docker.rs
│   │       └── docker_compose.rs
│   ├── application/                # Future application deployment domain
│   │   ├── mod.rs
│   │   ├── deployment/             # Current: steps/application (future)
│   │   └── lifecycle/
│   └── testing/                    # Everything related to testing and validation
│       ├── mod.rs
│       ├── commands/               # Current: commands/test
│       ├── e2e/                    # Current: e2e/
│       └── validation/             # Cross-cutting validation concerns
├── integrations/                   # External tool integrations
│   ├── mod.rs
│   ├── ansible/                    # Current: command_wrappers/ansible + ansible/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   └── templates.rs
│   ├── opentofu/                   # Current: command_wrappers/opentofu + tofu/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   └── json_parser.rs
│   ├── lxd/                        # Current: command_wrappers/lxd
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   ├── instance/
│   │   └── json_parser.rs
│   └── ssh/                        # Current: command_wrappers/ssh
│       ├── mod.rs
│       ├── client.rs
│       ├── connection.rs
│       └── credentials.rs
├── remote_operations/              # Current: remote_actions (more descriptive name)
│   ├── mod.rs
│   ├── cloud_init.rs
│   ├── docker.rs
│   └── docker_compose.rs
└── shared/                         # Shared utilities and templates
    ├── mod.rs
    ├── templates/                  # Current: template/ (core engine)
    │   ├── mod.rs
    │   ├── engine.rs
    │   ├── file.rs
    │   ├── file_ops.rs
    │   └── embedded.rs
    └── command_execution/          # Current: command.rs utilities
        └── mod.rs
```

### Module Mapping

| Current Location             | New Location                                                           | Rationale                              |
| ---------------------------- | ---------------------------------------------------------------------- | -------------------------------------- |
| `src/commands/provision.rs`  | `src/domains/infrastructure/provisioning/commands.rs`                  | Infrastructure domain ownership        |
| `src/commands/configure.rs`  | `src/domains/configuration/management/commands.rs`                     | Configuration domain ownership         |
| `src/commands/test.rs`       | `src/domains/testing/commands/`                                        | Testing domain ownership               |
| `src/steps/infrastructure/*` | `src/domains/infrastructure/provisioning/`                             | Co-locate with infrastructure commands |
| `src/steps/software/*`       | `src/domains/software/installation/`                                   | Software domain ownership              |
| `src/steps/validation/*`     | Split between respective domains                                       | Domain-specific validation             |
| `src/command_wrappers/*`     | `src/integrations/`                                                    | Clear external tool separation         |
| `src/ansible/*`              | `src/integrations/ansible/` + `src/domains/configuration/templates/`   | Split client vs templates              |
| `src/tofu/*`                 | `src/integrations/opentofu/` + `src/domains/infrastructure/templates/` | Split client vs templates              |
| `src/template/*`             | `src/shared/templates/`                                                | Shared utility                         |
| `src/remote_actions/*`       | `src/remote_operations/`                                               | More descriptive name                  |

## 📈 Benefits

### For New Contributors

1. **Intuitive Navigation**: Working on infrastructure? Look in `domains/infrastructure/`
2. **Clear Boundaries**: Each domain has obvious ownership and scope
3. **Workflow Understanding**: Structure matches deployment process
4. **Reduced Context Switching**: Related functionality co-located

### For Maintainers

1. **Reduced Cross-Dependencies**: Domain boundaries reduce scattered imports
2. **Clearer Architecture**: Separation of business logic from external integrations
3. **Easier Testing**: Domain-specific test organization
4. **Future Scalability**: Easy to add new domains without restructuring

### For the Codebase

1. **Better Cohesion**: Related functionality grouped together
2. **Looser Coupling**: Clear interfaces between domains
3. **Improved Discoverability**: Less time searching for relevant code
4. **Domain Expertise**: Contributors can focus on specific domains

## 🚧 Implementation Plan

### Phase 1: Foundation Setup

**Estimated Time**: 2-3 hours

- [ ] **1.1** Create new directory structure

  - [ ] Create `src/core/` directory
  - [ ] Create `src/domains/` with subdirectories
  - [ ] Create `src/integrations/` with subdirectories
  - [ ] Create `src/remote_operations/` directory
  - [ ] Create `src/shared/` directory

- [ ] **1.2** Move core infrastructure files

  - [ ] Move `src/config/` → `src/core/config/`
  - [ ] Move `src/logging.rs` → `src/core/logging.rs`
  - [ ] Move `src/container.rs` → `src/core/container.rs`
  - [ ] Move `src/command.rs` → `src/core/command.rs`
  - [ ] Update `src/core/mod.rs` with re-exports

- [ ] **1.3** Move shared utilities
  - [ ] Move `src/template/` → `src/shared/templates/`
  - [ ] Move command execution utilities → `src/shared/command_execution/`
  - [ ] Update `src/shared/mod.rs` with re-exports

### Phase 2: External Integrations

**Estimated Time**: 3-4 hours

- [ ] **2.1** Move external tool wrappers

  - [ ] Move `src/command_wrappers/ansible.rs` → `src/integrations/ansible/client.rs`
  - [ ] Move `src/command_wrappers/opentofu/` → `src/integrations/opentofu/`
  - [ ] Move `src/command_wrappers/lxd/` → `src/integrations/lxd/`
  - [ ] Move `src/command_wrappers/ssh/` → `src/integrations/ssh/`
  - [ ] Update `src/integrations/mod.rs` with re-exports

- [ ] **2.2** Consolidate template integrations
  - [ ] Move `src/ansible/template/` → `src/integrations/ansible/templates.rs`
  - [ ] Move `src/tofu/template/` → `src/integrations/opentofu/templates.rs`
  - [ ] Merge template functionality with clients

### Phase 3: Domain Organization

**Estimated Time**: 4-5 hours

- [ ] **3.1** Infrastructure domain

  - [ ] Move `src/commands/provision.rs` → `src/domains/infrastructure/provisioning/commands.rs`
  - [ ] Move `src/steps/infrastructure/` → `src/domains/infrastructure/provisioning/`
  - [ ] Move rendering steps → `src/domains/infrastructure/templates/`
  - [ ] Extract infrastructure validation → `src/domains/infrastructure/validation.rs`
  - [ ] Update `src/domains/infrastructure/mod.rs`

- [ ] **3.2** Configuration domain

  - [ ] Move `src/commands/configure.rs` → `src/domains/configuration/management/commands.rs`
  - [ ] Move `src/steps/connectivity/` → `src/domains/configuration/connectivity/`
  - [ ] Move `src/steps/system/` → `src/domains/configuration/system/`
  - [ ] Move Ansible templates → `src/domains/configuration/templates/`
  - [ ] Update `src/domains/configuration/mod.rs`

- [ ] **3.3** Software domain
  - [ ] Move `src/steps/software/` → `src/domains/software/installation/`
  - [ ] Extract software validation → `src/domains/software/validation/`
  - [ ] Update `src/domains/software/mod.rs`

### Phase 4: Testing Domain

**Estimated Time**: 2-3 hours

- [ ] **4.1** Testing consolidation
  - [ ] Move `src/commands/test.rs` → `src/domains/testing/commands/`
  - [ ] Move `src/e2e/` → `src/domains/testing/e2e/`
  - [ ] Organize cross-cutting validation → `src/domains/testing/validation/`
  - [ ] Update `src/domains/testing/mod.rs`

### Phase 5: Remote Operations & Cleanup

**Estimated Time**: 2-3 hours

- [ ] **5.1** Remote operations

  - [ ] Move `src/remote_actions/` → `src/remote_operations/`
  - [ ] Update module name references
  - [ ] Update `src/remote_operations/mod.rs`

- [ ] **5.2** Update imports and references
  - [ ] Update all `use crate::` statements throughout codebase
  - [ ] Update `src/lib.rs` with new module structure
  - [ ] Update documentation references

### Phase 6: Validation & Testing

**Estimated Time**: 2-3 hours

- [ ] **6.1** Compilation and testing

  - [ ] Run `cargo build` and fix compilation errors
  - [ ] Run `cargo test` and fix test issues
  - [ ] Run `cargo run --bin linter all` and fix linting issues
  - [ ] Run `cargo run --bin e2e-tests` for integration verification

- [ ] **6.2** Documentation updates
  - [ ] Update `docs/codebase-architecture.md` with new structure
  - [ ] Update module documentation (`//!` comments) with new organization
  - [ ] Update README.md if needed

### Phase 7: Final Verification

**Estimated Time**: 1-2 hours

- [ ] **7.1** Complete testing

  - [ ] Verify all functionality works as before
  - [ ] Check that all pre-commit checks pass
  - [ ] Validate E2E tests pass completely

- [ ] **7.2** Documentation finalization
  - [ ] Mark refactoring as complete
  - [ ] Document any lessons learned or additional improvements

## 📊 Progress Tracking

### Overall Progress: **DEFERRED** - Not Implementing

| Phase                                | Status      | Completion | Notes                        |
| ------------------------------------ | ----------- | ---------- | ---------------------------- |
| Phase 1: Foundation Setup            | ❌ Deferred | N/A        | Waiting for system evolution |
| Phase 2: External Integrations       | ❌ Deferred | N/A        | Waiting for system evolution |
| Phase 3: Domain Organization         | ❌ Deferred | N/A        | Waiting for system evolution |
| Phase 4: Testing Domain              | ❌ Deferred | N/A        | Waiting for system evolution |
| Phase 5: Remote Operations & Cleanup | ❌ Deferred | N/A        | Waiting for system evolution |
| Phase 6: Validation & Testing        | ❌ Deferred | N/A        | Waiting for system evolution |
| Phase 7: Final Verification          | ❌ Deferred | N/A        | Waiting for system evolution |

**Legend**: ⏳ Not Started | 🔄 In Progress | ✅ Complete | ❌ Deferred

## 🔄 Current Status: **DEFERRED**

**Decision**: This refactoring has been **postponed** until the system grows and evolves further.

**Rationale**:

1. The current codebase size (86 files) does not yet justify the refactoring effort
2. Need to implement more commands to better understand emerging patterns
3. Current organization is working well for the existing functionality
4. Will revisit when we have clear evidence that the reorganization provides significant value

## 📝 Historical Context

- **Proposal Created**: September 17, 2025
- **Analysis Completed**: Full current state analysis and proposed structure documented
- **Implementation Plan**: Complete 7-phase plan with 15-20 hour estimate
- **Decision**: Defer implementation pending system evolution

This document serves as a **reference architecture proposal** for future consideration when the codebase has grown significantly and patterns have emerged more clearly.

---

**Status**: Proposal archived for future consideration.
