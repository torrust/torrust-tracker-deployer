# Refactor Command Handlers to Modular Directory Structure

**Issue**: #56
**Type**: Refactoring
**Related**: Follows pattern established in `CreateCommandHandler` (see `src/application/command_handlers/create/`)

## Overview

Refactor the existing single-file command handlers (`provision`, `configure`, `destroy`) to follow the new modular directory structure pattern established in `CreateCommandHandler`. This improves code organization, maintainability, and scalability by separating concerns into dedicated files.

Currently, three command handlers use a single-file pattern:

- `provision.rs` (752 lines) - Too large, mixing handler logic, errors, and tests
- `destroy.rs` (631 lines) - Getting unwieldy
- `configure.rs` (370 lines) - Manageable but would benefit from separation

The `CreateCommandHandler` demonstrates a better structure with separate files for handler logic, errors, and tests.

## Goals

- [ ] Migrate `ProvisionCommandHandler` to modular structure
- [ ] Migrate `ConfigureCommandHandler` to modular structure
- [ ] Migrate `DestroyCommandHandler` to modular structure
- [ ] Improve code maintainability and readability
- [ ] Make test organization clearer and more scalable
- [ ] Establish consistent patterns across all command handlers

## 🏗️ Architecture Requirements

**DDD Layer**: Application Layer
**Module Path**: `src/application/command_handlers/{provision|configure|destroy}/`
**Pattern**: Command Handler with modular organization

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Maintain existing dependency injection patterns
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))
- [ ] Preserve existing public API surface (no breaking changes)

### Architectural Constraints

- [ ] No changes to command handler business logic (refactoring only)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Testing strategy aligns with layer responsibilities (see [docs/contributing/testing.md](../docs/contributing/testing.md))
- [ ] All existing tests must continue to pass
- [ ] Preserve traceability and observability features

**Note**: Breaking changes to import paths are acceptable since this library is not yet publicly used. Simplifying the module structure takes priority over backward compatibility.

### Anti-Patterns to Avoid

- ❌ Changing business logic during refactoring
- ❌ Mixing refactoring with new features
- ❌ Creating incomplete module structures
- ❌ Half-migrating modules (finish one completely before moving to next)

## Specifications

### Current Structure (Single-File Pattern)

```text
src/application/command_handlers/
├── configure.rs       (370 lines)
├── destroy.rs         (631 lines)
├── provision.rs       (752 lines)
└── test.rs
```

**Problems**:

- Large files difficult to navigate (provision.rs: 752 lines)
- Mixed concerns: handler logic, errors, tests all in one file
- Poor scalability: adding features bloats single file
- Harder code reviews: changes to errors require reviewing entire handler
- Test organization unclear

### Target Structure (Modular Pattern)

Based on the `create/` command handler pattern:

```text
src/application/command_handlers/
├── configure/
│   ├── mod.rs          # Module documentation & public API
│   ├── handler.rs      # Command handler implementation
│   ├── errors.rs       # Error types with .help()
│   └── tests/
│       ├── mod.rs      # Test module entry
│       ├── builders.rs # Test builders and fixtures
│       └── integration.rs # Integration tests
├── destroy/
│   ├── mod.rs
│   ├── handler.rs
│   ├── errors.rs
│   └── tests/
│       ├── mod.rs
│       ├── builders.rs
│       └── integration.rs
├── provision/
│   ├── mod.rs
│   ├── handler.rs
│   ├── errors.rs
│   └── tests/
│       ├── mod.rs
│       ├── builders.rs
│       └── integration.rs
└── test.rs
```

### File Responsibilities

#### `mod.rs` (Module Documentation & Public API)

- Module-level documentation
- Re-exports public types (`pub use`)
- Declares submodules
- Serves as clean entry point

**Example** (based on `create/mod.rs`):

```rust
//! [Command Name] Command Module
//!
//! This module implements the delivery-agnostic `[Command]CommandHandler`
//! for orchestrating [command purpose] business logic.
//!
//! ## Architecture
//!
//! The `[Command]CommandHandler` implements the Command Pattern...

pub mod errors;
pub mod handler;

#[cfg(test)]
pub mod tests;

// Re-export public API
pub use errors::{[Command]CommandHandlerError};
pub use handler::{[Command]CommandHandler};
```

#### `handler.rs` (Command Handler Implementation)

- Command handler struct definition
- Business logic orchestration
- Dependency injection via constructor
- Public `execute()` method
- No error definitions (moved to `errors.rs`)

**Size**: 200-300 lines typical

#### `errors.rs` (Error Types)

- Error enum definitions using `thiserror`
- `Traceable` trait implementation
- `.help()` methods for actionable error messages
- Error documentation

**Size**: 150-200 lines typical

#### `tests/` (Test Organization)

- `mod.rs`: Test module entry, re-exports
- `builders.rs`: Test builders and fixtures
- `integration.rs`: Integration test cases

### Migration Strategy (Per Command Handler)

For each command handler (`provision`, `configure`, `destroy`):

1. **Create Directory Structure**

   ```bash
   mkdir -p src/application/command_handlers/{command}/tests
   ```

2. **Split Files**

   - Extract errors → `errors.rs`
   - Move handler → `handler.rs`
   - Create `mod.rs` with documentation
   - Organize tests → `tests/`

3. **Update Imports Throughout Codebase**

   - Update parent `mod.rs` to use new module structure
   - Fix ALL imports in other modules that use the command handler
   - Update CLI/presentation layer imports
   - Update any test imports
   - **Breaking changes acceptable** - simplify paths where possible

4. **Verify**

   - Run `cargo test` - all tests must pass
   - Run `cargo clippy` - no new warnings
   - Run `./scripts/pre-commit.sh` - all checks pass

## Implementation Plan

**Important**: Migrate and commit each command handler independently. Run pre-commit checks after completing each one before moving to the next.

### Subtask 1: Migrate ProvisionCommandHandler (3-4 hours)

- [ ] Create `src/application/command_handlers/provision/` directory
- [ ] Create `provision/mod.rs` with module documentation and re-exports
- [ ] Create `provision/handler.rs` - move handler implementation from `provision.rs`
- [ ] Create `provision/errors.rs` - extract error types from `provision.rs`
- [ ] Create `provision/tests/mod.rs` - test module entry point
- [ ] Create `provision/tests/builders.rs` - extract or create test builders
- [ ] Create `provision/tests/integration.rs` - move existing tests
- [ ] Update `src/application/command_handlers/mod.rs` to use new structure
- [ ] **Update ALL imports across codebase** (CLI, presentation, other modules)
- [ ] Delete old `provision.rs` file
- [ ] Run `cargo test` to verify all tests pass
- [ ] Run `cargo clippy` to check for issues
- [ ] Run `./scripts/pre-commit.sh` to verify quality
- [ ] **Commit this refactoring independently before moving to next handler**

### Subtask 2: Migrate ConfigureCommandHandler (2-3 hours)

- [ ] Create `src/application/command_handlers/configure/` directory
- [ ] Create `configure/mod.rs` with module documentation and re-exports
- [ ] Create `configure/handler.rs` - move handler implementation from `configure.rs`
- [ ] Create `configure/errors.rs` - extract error types from `configure.rs`
- [ ] Create `configure/tests/mod.rs` - test module entry point
- [ ] Create `configure/tests/builders.rs` - extract or create test builders
- [ ] Create `configure/tests/integration.rs` - move existing tests
- [ ] Update `src/application/command_handlers/mod.rs` to use new structure
- [ ] **Update ALL imports across codebase** (CLI, presentation, other modules)
- [ ] Delete old `configure.rs` file
- [ ] Run `cargo test` to verify all tests pass
- [ ] Run `cargo clippy` to check for issues
- [ ] Run `./scripts/pre-commit.sh` to verify quality
- [ ] **Commit this refactoring independently before moving to next handler**

### Subtask 3: Migrate DestroyCommandHandler (2-3 hours)

- [ ] Create `src/application/command_handlers/destroy/` directory
- [ ] Create `destroy/mod.rs` with module documentation and re-exports
- [ ] Create `destroy/handler.rs` - move handler implementation from `destroy.rs`
- [ ] Create `destroy/errors.rs` - extract error types from `destroy.rs`
- [ ] Create `destroy/tests/mod.rs` - test module entry point
- [ ] Create `destroy/tests/builders.rs` - extract or create test builders
- [ ] Create `destroy/tests/integration.rs` - move existing tests
- [ ] Update `src/application/command_handlers/mod.rs` to use new structure
- [ ] **Update ALL imports across codebase** (CLI, presentation, other modules)
- [ ] Delete old `destroy.rs` file
- [ ] Run `cargo test` to verify all tests pass
- [ ] Run `cargo clippy` to check for issues
- [ ] Run `./scripts/pre-commit.sh` to verify quality
- [ ] **Commit this refactoring independently**

### Subtask 4: Final Verification and Documentation (1 hour)

- [ ] Run full test suite: `cargo test`
- [ ] Run E2E tests: `cargo run --bin e2e-tests-full`
- [ ] Verify all command handlers follow consistent structure
- [ ] Update any relevant documentation mentioning command handler structure
- [ ] Ensure all imports across the codebase are correct
- [ ] Final pre-commit verification: `./scripts/pre-commit.sh`
- [ ] **Final commit with any documentation updates**

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Refactoring-Specific Criteria**:

- [ ] All three command handlers use modular directory structure
- [ ] Directory structure matches `create/` pattern exactly
- [ ] All existing tests continue to pass
- [ ] No changes to business logic (pure refactoring)
- [ ] File sizes reduced to manageable levels (< 300 lines per file)
- [ ] Clear separation of concerns (handler, errors, tests)
- [ ] Module documentation is comprehensive
- [ ] Error types include `.help()` methods
- [ ] Test organization is clear and scalable
- [ ] All imports updated correctly throughout entire codebase
- [ ] No clippy warnings introduced
- [ ] Code follows module organization conventions

**Import Updates**:

- [ ] All imports in CLI/presentation layer updated
- [ ] All imports in tests updated
- [ ] All imports in other application modules updated
- [ ] Parent `mod.rs` correctly re-exports types for convenience

**Structure Verification**:

- [ ] Each command handler has `mod.rs`, `handler.rs`, `errors.rs`
- [ ] Each has `tests/` subdirectory with `mod.rs`, `builders.rs`, `integration.rs`
- [ ] Parent `mod.rs` correctly imports and re-exports all handlers
- [ ] No orphaned files or unused code

**Commit Strategy**:

- [ ] Each command handler refactoring committed independently
- [ ] Pre-commit checks pass before each commit
- [ ] Clear, descriptive commit messages following conventional commits format
- [ ] Final documentation commit separate from refactoring commits

## Related Documentation

- [Module Organization Guide](../contributing/module-organization.md) - File organization conventions
- [Error Handling Guide](../contributing/error-handling.md) - Error type patterns and `.help()` methods
- [Testing Conventions](../contributing/testing.md) - Test organization and naming
- [Codebase Architecture](../codebase-architecture.md) - DDD layer structure
- Reference Implementation: `src/application/command_handlers/create/` - Example of target structure

## Benefits

### Maintainability ✅

- **Smaller files**: Easier to navigate (200-300 lines vs 752 lines)
- **Single responsibility**: Each file has one clear purpose
- **Easier reviews**: Changes to errors don't require reviewing handler logic

### Scalability ✅

- **Room to grow**: Can add validators, mappers without bloating main file
- **Test organization**: Can add more test modules easily
- **Better IDE support**: Smaller files improve code navigation

### Consistency ✅

- **Uniform patterns**: All command handlers follow same structure
- **Predictable locations**: Developers know where to find handler vs errors vs tests
- **Easier onboarding**: New contributors understand structure quickly

### Testing ✅

- **Dedicated test builders**: Clear separation of test fixtures
- **Integration tests isolated**: Easier to maintain and extend
- **Better test discoverability**: Tests organized by purpose

## Notes

### Migration Order Rationale

Starting with `provision` (largest, most complex) first because:

1. It demonstrates the most value (752 → ~600 total lines split appropriately)
2. Establishes patterns for subsequent migrations
3. Most complex case first helps validate the approach

### Preserving Git History

Consider using `git mv` where possible to preserve file history:

```bash
# When creating new files, use git mv for better history tracking
git mv src/application/command_handlers/provision.rs \
       src/application/command_handlers/provision/handler.rs
```

However, since we're splitting files, manual creation may be clearer.

### Breaking Changes: Acceptable

**Important**: Since this library is not yet publicly used, breaking changes to import paths are acceptable and even encouraged if they simplify the refactoring:

- ✅ Import paths will change (e.g., `use crate::application::command_handlers::provision::ProvisionCommandHandler`)
- ✅ Can simplify re-exports if it improves clarity
- ✅ Focus on best final structure, not backward compatibility
- ⚠️ Business logic must remain unchanged (pure refactoring)
- ⚠️ All existing tests must still pass (behavior unchanged)

This freedom allows us to:

- Create the cleanest possible module structure
- Avoid complex re-export chains just for compatibility
- Optimize for maintainability over backward compatibility

### Future Work

After this refactoring, consider:

- Adding more comprehensive test coverage where gaps exist
- Extracting common patterns into shared utilities
- Creating ADR documenting this as the standard command handler pattern
