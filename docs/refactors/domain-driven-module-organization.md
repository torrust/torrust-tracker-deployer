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

## 🏗️ New Simplified Structure

### Phase 1: Layer-Based Organization (Current Implementation)

We start with a clean layer-based architecture using DDD layers but within a single bounded context:

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
│   └── adapters/                   # Current: command_wrappers/ - external tool adapters
│       ├── mod.rs
│       ├── ansible.rs
│       ├── lxd/
│       ├── opentofu/
│       └── ssh/
├── application/                    # Application Layer (DDD)
│   ├── mod.rs
│   └── commands/                   # Current: commands/ - high-level application commands
│       ├── mod.rs
│       ├── configure.rs
│       ├── provision.rs
│       └── test.rs
├── domain/                         # Domain Layer (kept minimal for now)
│   └── mod.rs
└── [other existing modules remain unchanged...]
    ├── steps/
    ├── remote_actions/
    ├── template/
    ├── config/
    ├── e2e/
    ├── ansible/
    └── tofu/
```

### Benefits of This Approach

1. **Immediate Naming Clarity**: No more confusion between command execution, tool adapters, and application commands
2. **Layer Separation**: Clear Infrastructure vs Application layer boundaries
3. **DDD Foundation**: Establishes architectural layers for future DDD evolution
4. **Minimal Disruption**: Most modules remain unchanged, only reorganizing the problematic areas
5. **Incremental Evolution**: Can add domain organization later without major restructuring

### Module Mapping for Phase 1

| Current Location        | New Location                     | Rationale                          |
| ----------------------- | -------------------------------- | ---------------------------------- |
| `src/command.rs`        | `src/infrastructure/executor.rs` | Low-level infrastructure concern   |
| `src/command_wrappers/` | `src/infrastructure/adapters/`   | External tool integration adapters |
| `src/commands/`         | `src/application/commands/`      | High-level application commands    |

### Future Evolution Path (Phase 2+)

Once this layer foundation is established and patterns become clearer, we can evolve toward full DDD:

- Move domain-specific logic from `steps/` into `domain/` modules
- Organize domain modules by bounded contexts (infrastructure, configuration, software, etc.)
- Refactor remaining modules to align with domain boundaries
- Maintain clear separation between layers

## 📈 Benefits of Simplified Approach

### Immediate Benefits

1. **Naming Clarity**: Resolves confusion between command execution, tool adapters, and application commands
2. **Layer Separation**: Clear Infrastructure vs Application boundaries following DDD principles
3. **Reduced Complexity**: Focuses on high-impact reorganization without over-architecting
4. **Easy Navigation**: Contributors know where to find infrastructure vs application logic
5. **Minimal Disruption**: Most existing code remains unchanged

### Foundation for Future Growth

1. **DDD Ready**: Establishes layer foundation for domain-driven evolution
2. **Scalable Structure**: Easy to add domain modules when patterns become clear
3. **Clean Architecture**: Proper separation of concerns across architectural layers
4. **Incremental Approach**: Can evolve into full bounded contexts gradually

## 🚧 Implementation Plan

### Phase 1: Layer-Based Architecture Foundation

**Estimated Time**: 2-3 hours

- [ ] **1.1** Create layer directories

  - [ ] Create `src/infrastructure/` directory
  - [ ] Create `src/infrastructure/adapters/` directory
  - [ ] Create `src/application/` directory
  - [ ] Create `src/application/commands/` directory
  - [ ] Create `src/domain/` directory (minimal, for future use)

- [ ] **1.2** Move infrastructure layer files

  - [ ] Move `src/command.rs` → `src/infrastructure/executor.rs`
  - [ ] Move `src/command_wrappers/` → `src/infrastructure/adapters/`
  - [ ] Update `src/infrastructure/mod.rs` with re-exports

- [ ] **1.3** Move application layer files

  - [ ] Move `src/commands/` → `src/application/commands/`
  - [ ] Update `src/application/mod.rs` with re-exports

- [ ] **1.4** Update imports and references

  - [ ] Update all `use crate::command` → `use crate::infrastructure::executor`
  - [ ] Update all `use crate::command_wrappers` → `use crate::infrastructure::adapters`
  - [ ] Update all `use crate::commands` → `use crate::application::commands`
  - [ ] Update `src/lib.rs` with new module structure

- [ ] **1.5** Validation and testing

  - [ ] Run `cargo build` and fix compilation errors
  - [ ] Run `cargo test` and fix test issues
  - [ ] Run `cargo run --bin linter all` and fix linting issues
  - [ ] Run `cargo run --bin e2e-tests` for integration verification

### Future Phases (Deferred)

**Phase 2+**: Domain organization within layers, bounded contexts, and full DDD structure - to be planned when patterns become clearer and system grows.

## 📊 Progress Tracking

### Phase 1 Progress: **In Progress**

| Step                           | Status         | Completion | Notes               |
| ------------------------------ | -------------- | ---------- | ------------------- |
| 1.1: Create layer directories  | ⏳ Not Started | 0%         | Ready to start      |
| 1.2: Move infrastructure files | ⏳ Not Started | 0%         | Depends on 1.1      |
| 1.3: Move application files    | ⏳ Not Started | 0%         | Depends on 1.1      |
| 1.4: Update imports/references | ⏳ Not Started | 0%         | Depends on 1.2, 1.3 |
| 1.5: Validation and testing    | ⏳ Not Started | 0%         | Final validation    |

**Legend**: ⏳ Not Started | 🔄 In Progress | ✅ Complete

## 🔄 Current Status: **Starting Implementation**

**Decision**: Beginning with simplified layer-based architecture approach.

**Next Steps**:

1. Commit this updated plan
2. Implement Phase 1 reorganization
3. Validate all functionality remains intact
4. Plan future domain-driven evolution based on emerging patterns
