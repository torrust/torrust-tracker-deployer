# Move Configuration Module from Domain to Create Command Handler

## 📋 Overview

This refactoring moves the `src/domain/config/` module to `src/application/command_handlers/create/config/` to align with DDD principles and improve code organization. The config module contains Data Transfer Objects (DTOs) for the create command, not domain entities, and should be located with its primary consumer.

**Target Files:**

- `src/domain/config/mod.rs`
- `src/domain/config/environment_config.rs`
- `src/domain/config/ssh_credentials_config.rs`
- `src/domain/config/errors.rs`
- `src/domain/mod.rs` (re-exports)
- `src/application/command_handlers/create/handler.rs`
- `src/application/command_handlers/create/errors.rs`
- `src/application/command_handlers/create/tests/builders.rs`
- `src/application/command_handlers/create/tests/integration.rs`
- `src/presentation/commands/create/config_loader.rs`
- `src/presentation/commands/create/subcommand.rs`
- `src/presentation/commands/create/errors.rs`
- `src/testing/e2e/tasks/run_create_command.rs`

**Scope:**

- Move config module from domain layer to application layer (create command handler)
- Update all import statements across the codebase
- Update documentation and comments to reflect new location
- Verify all tests pass after the move
- Ensure no breaking changes to public API

## 📊 Progress Tracking

**Total Active Proposals**: 1
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 1

### Phase Summary

- **Phase 0 - Module Relocation (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

None.

### Postponed Proposals

None.

## 🎯 Key Problems Identified

### 1. DDD Layer Violation

The `src/domain/config/` module violates Domain-Driven Design principles by containing infrastructure concerns, serialization logic, and I/O operations in the domain layer.

**Evidence:**

- Uses `serde::Deserialize` for JSON parsing (data transfer concern)
- Contains `generate_template_file()` with `tokio::fs` I/O operations (infrastructure concern)
- Uses raw string primitives instead of domain value objects
- Acts as DTO/adapter between external formats and domain types

### 2. Misleading Module Location

The module is located in the domain layer but explicitly states in its documentation that it "sits at the boundary between external configuration sources and the internal domain model" - which is the definition of an application-layer DTO.

### 3. Low Cohesion

The config module is exclusively used by the `create` command handler but is separated from it by layer boundaries. This creates unnecessary distance between tightly-coupled code.

**Usage Analysis:**

- **Primary consumer**: `src/application/command_handlers/create/` (handler, errors, tests)
- **Secondary consumer**: `src/presentation/commands/create/` (loads and validates config)
- **Not used by**: Any other command handler or domain entity

## 🚀 Refactoring Phases

---

## Phase 0: Module Relocation (Highest Priority)

This phase moves the config module to its logical location with the create command handler, improving cohesion and aligning with DDD principles.

### Proposal #0: Move Config Module to Create Command Handler

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P0  
**Depends On**: None

#### Problem

The config module is misplaced in the domain layer, violating DDD principles:

```rust
// Current structure - WRONG
src/
  domain/
    config/                    # ❌ Should not be in domain
      mod.rs
      environment_config.rs    # Contains serde, I/O operations
      ssh_credentials_config.rs
      errors.rs

// Import path
use crate::domain::config::EnvironmentCreationConfig;
```

**Issues with current location:**

1. **Serialization concerns in domain**: Uses `serde::Deserialize` for JSON parsing
2. **Infrastructure operations in domain**: `generate_template_file()` performs file I/O
3. **String-based primitives**: Uses `String` instead of domain value objects (`EnvironmentName`, `Username`)
4. **DTO pattern in domain**: Converts external format to domain types via `to_environment_params()`
5. **Low cohesion**: Separated from its only consumer (create command handler)

#### Proposed Solution

Move the config module to be a submodule of the create command handler:

```rust
// New structure - CORRECT
src/
  application/
    command_handlers/
      create/
        config/              # ✅ With its consumer
          mod.rs
          environment_config.rs
          ssh_credentials_config.rs
          errors.rs
        handler.rs
        errors.rs
        mod.rs
        tests/

// New import path
use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
```

#### Rationale

**Why application layer (not presentation)?**

- Application command handlers are the primary consumers
- DTOs typically live in application layer in DDD
- Represents use-case input, not domain modeling
- Presentation layer can still import from application layer

**Why nested under create command (not generic `dto/` folder)?**

- Config is exclusively used by create command (verified by grep analysis)
- Follows "high cohesion" principle - keep related things together
- Avoids premature abstraction (YAGNI - You Aren't Gonna Need It)
- If other commands need similar config, refactor when actually needed

**Why not keep in domain?**

Domain layer should contain:

- ✅ Pure business logic: Rules about valid environments
- ✅ Entities: `Environment<S>` with type-state pattern
- ✅ Value Objects: `EnvironmentName`, `Username`
- ✅ Domain Services: Pure business operations

Domain layer should NOT contain:

- ❌ Data transfer: JSON serialization/deserialization
- ❌ Infrastructure operations: File system I/O
- ❌ String-based primitives: Pre-validation data
- ❌ Format conversion: External format → domain types

#### Benefits

- ✅ **Aligns with DDD principles**: Removes DTOs from domain layer
- ✅ **Higher cohesion**: Config lives with its consumer
- ✅ **Clearer boundaries**: Domain remains pure business logic
- ✅ **Better discoverability**: All create command code in one place
- ✅ **No premature abstraction**: No generic `dto/` folder for single use
- ✅ **Easier maintenance**: Changes to create command and its config are co-located

#### Implementation Checklist

- [ ] **Step 1: Create new directory structure**
  - [ ] Create `src/application/command_handlers/create/config/` directory
- [ ] **Step 2: Move files**
  - [ ] Move `src/domain/config/mod.rs` → `src/application/command_handlers/create/config/mod.rs`
  - [ ] Move `src/domain/config/environment_config.rs` → `src/application/command_handlers/create/config/environment_config.rs`
  - [ ] Move `src/domain/config/ssh_credentials_config.rs` → `src/application/command_handlers/create/config/ssh_credentials_config.rs`
  - [ ] Move `src/domain/config/errors.rs` → `src/application/command_handlers/create/config/errors.rs`
- [ ] **Step 3: Update module declarations**
  - [ ] Remove `pub mod config;` from `src/domain/mod.rs`
  - [ ] Remove config re-exports from `src/domain/mod.rs`
  - [ ] Add `pub mod config;` to `src/application/command_handlers/create/mod.rs`
  - [ ] Update config re-exports in `src/application/command_handlers/create/mod.rs`
- [ ] **Step 4: Update imports in application layer**
  - [ ] Update `src/application/command_handlers/create/handler.rs`
  - [ ] Update `src/application/command_handlers/create/errors.rs`
  - [ ] Update `src/application/command_handlers/create/mod.rs` (documentation examples)
  - [ ] Update `src/application/command_handlers/create/tests/builders.rs`
  - [ ] Update `src/application/command_handlers/create/tests/integration.rs`
- [ ] **Step 5: Update imports in presentation layer**
  - [ ] Update `src/presentation/commands/create/config_loader.rs`
  - [ ] Update `src/presentation/commands/create/subcommand.rs`
  - [ ] Update `src/presentation/commands/create/errors.rs`
- [ ] **Step 6: Update imports in testing utilities**
  - [ ] Update `src/testing/e2e/tasks/run_create_command.rs`
- [ ] **Step 7: Update documentation**
  - [ ] Update module-level documentation in `config/mod.rs`
  - [ ] Update doc examples in `config/environment_config.rs`
  - [ ] Update doc examples in `config/ssh_credentials_config.rs`
  - [ ] Update import paths in all docstring examples
- [ ] **Step 8: Clean up old domain/config directory**
  - [ ] Delete `src/domain/config/` directory
- [ ] **Step 9: Verification**
  - [ ] Run `cargo build` - verify compilation succeeds
  - [ ] Run `cargo test` - verify all tests pass
  - [ ] Run `cargo run --bin linter all` - verify linters pass
  - [ ] Run `cargo doc --no-deps` - verify documentation builds
  - [ ] Manually verify import paths are correct
  - [ ] Check for any remaining references to `domain::config`

#### Testing Strategy

**Compilation Verification:**

```bash
# Ensure code compiles after each file move
cargo build
```

**Test Verification:**

```bash
# Run full test suite
cargo test

# Run specific create command tests
cargo test --package torrust-tracker-deployer-lib create
```

**Linting Verification:**

```bash
# Run all linters
cargo run --bin linter all
```

**Documentation Verification:**

```bash
# Verify doc examples compile
cargo doc --no-deps --bins --examples --workspace --all-features
```

**Import Path Verification:**

```bash
# Search for any remaining old import paths
rg "use.*domain::config" src/
rg "crate::domain::config" src/

# Should return no results after refactoring
```

#### Results (if completed)

- **Files Moved**: 4
- **Import Statements Updated**: ~15
- **Tests**: [Status]
- **Linters**: [Status]
- **Documentation**: [Status]

---

## 📈 Timeline

- **Start Date**: [To be determined]
- **Estimated Duration**: 1-2 hours
- **Actual Completion**: [Date when completed]

## 🔍 Review Process

### Approval Criteria

- [ ] Plan reviewed by project maintainers
- [ ] Technical feasibility validated
- [ ] Aligns with [Development Principles](../../development-principles.md)
- [ ] Aligns with [DDD Architecture](../../codebase-architecture.md)
- [ ] Implementation plan is clear and actionable
- [ ] No breaking changes to public API

### Completion Criteria

- [ ] All files moved to new location
- [ ] All import statements updated
- [ ] All tests passing (`cargo test`)
- [ ] All linters passing (`cargo run --bin linter all`)
- [ ] Documentation builds successfully (`cargo doc --no-deps`)
- [ ] No references to old `domain::config` path remain
- [ ] Module documentation updated to reflect new location
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../../development-principles.md) - DDD principles and layer boundaries
- [Codebase Architecture](../../codebase-architecture.md) - DDD layer structure and dependency rules
- [Contributing Guidelines](../../contributing/README.md) - General contribution guidelines
- [Module Organization](../../contributing/module-organization.md) - Module structure conventions
- [Error Handling Guide](../../contributing/error-handling.md) - Error type organization

## 💡 Notes

### Design Decisions

**Why not create a generic `application/dto/` folder?**

The YAGNI (You Aren't Gonna Need It) principle suggests we should not create abstractions until we actually need them. Currently:

- Only the `create` command uses configuration DTOs
- No other commands have similar needs (yet)
- Creating a generic `dto/` folder would be premature abstraction

If other commands develop similar needs in the future, we can:

1. Identify common patterns across multiple commands
2. Extract shared DTO patterns to `application/dto/`
3. Keep command-specific DTOs in their respective command modules

**Why is presentation layer allowed to import from application layer?**

In DDD, the dependency rule is:

```text
Presentation → Application → Domain ← Infrastructure
```

This means:

- ✅ Presentation can import from Application (follows dependency flow)
- ✅ Application can import from Domain
- ✅ Infrastructure can import from Domain
- ❌ Domain cannot import from Application or Infrastructure
- ❌ Application cannot import from Presentation

The presentation layer's `ConfigLoader` loading DTOs from the application layer is perfectly valid DDD.

### Alternative Considered: Presentation Layer

We considered placing the config in `src/presentation/dto/config/` because:

- Presentation layer loads the config from JSON files
- Contains serialization concerns (serde)
- Includes file I/O for template generation

However, application layer was chosen because:

- Application command handlers are the primary consumers
- DTOs represent use-case input, not delivery mechanism details
- In DDD, DTOs typically belong to application layer
- Presentation → Application dependency is valid

If the config becomes too infrastructure-heavy (lots of Figment-specific code, complex file handling), we could reconsider moving to presentation layer in a future refactoring.

### Testing Impact

This refactoring only changes import paths, not behavior. Therefore:

- No new tests are needed
- Existing tests verify correctness
- All tests must continue to pass
- No test logic changes required

---

**Created**: 2025-10-28  
**Last Updated**: 2025-10-28  
**Status**: 📋 Planning
