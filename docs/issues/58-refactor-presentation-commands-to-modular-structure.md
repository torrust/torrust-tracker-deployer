# Refactor Presentation Commands to Modular Directory Structure

**Issue**: #58
**Type**: Refactoring
**Related**: Follows pattern established in `CreateCommand` (see `src/presentation/commands/create/`)

## Overview

Refactor the existing single-file presentation command (`destroy`) to follow the new modular directory structure pattern established in `CreateCommand`. This improves code organization, maintainability, and consistency by separating concerns into dedicated files.

Currently, the `destroy` command uses a single-file pattern:

- `destroy.rs` (290 lines) - Mixing subcommand logic, error types, and handler orchestration

The `CreateCommand` demonstrates a better structure with separate files for subcommand logic, configuration loading, errors, and tests.

## Goals

- [ ] Migrate `DestroyCommand` to modular structure
- [ ] Improve code maintainability and readability
- [ ] Make test organization clearer and more scalable
- [ ] Establish consistent patterns across all presentation commands
- [ ] Enable easier addition of configuration loading if needed in the future

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation Layer
**Module Path**: `src/presentation/commands/destroy/`
**Pattern**: CLI Subcommand with modular organization

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Maintain existing presentation layer responsibilities
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))
- [ ] Preserve existing public API surface (no breaking changes to CLI interface)

### Architectural Constraints

- [ ] No changes to command business logic (refactoring only)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Testing strategy aligns with layer responsibilities (see [docs/contributing/testing.md](../docs/contributing/testing.md))
- [ ] All existing tests must continue to pass
- [ ] Preserve user output and progress reporting features

**Note**: Breaking changes to import paths are acceptable since this library is not yet publicly used. Simplifying the module structure takes priority over backward compatibility.

### Anti-Patterns to Avoid

- ❌ Changing business logic during refactoring
- ❌ Mixing refactoring with new features
- ❌ Creating incomplete module structures
- ❌ Moving presentation logic into application layer

## Specifications

### Current Structure (Single-File Pattern)

```text
src/presentation/commands/
├── create/              # Modular pattern (371 lines subcommand + separate files)
│   ├── mod.rs
│   ├── subcommand.rs
│   ├── config_loader.rs
│   ├── errors.rs
│   └── tests/
├── destroy.rs           # Single file (290 lines)
└── mod.rs
```

**Problems**:

- Mixed concerns: subcommand logic, errors, all in one file
- Inconsistent with `create/` command structure
- Harder to add features like configuration file support
- Test organization unclear

### Target Structure (Modular Pattern)

Based on the `create/` command pattern:

```text
src/presentation/commands/
├── create/
│   ├── mod.rs
│   ├── subcommand.rs
│   ├── config_loader.rs
│   ├── errors.rs
│   └── tests/
├── destroy/
│   ├── mod.rs           # Module documentation & public API
│   ├── subcommand.rs    # Main command handler
│   ├── errors.rs        # Error types with .help()
│   └── tests/           # Test organization
│       ├── mod.rs
│       ├── integration.rs
│       └── fixtures.rs (if needed)
└── mod.rs
```

### File Responsibilities

#### `mod.rs` (Module Documentation & Public API)

- Module-level documentation
- Re-exports public types (`pub use`)
- Declares submodules
- Serves as clean entry point

**Example** (based on `create/mod.rs`):

```rust
//! Destroy Command Presentation Module
//!
//! This module implements the CLI presentation layer for the destroy command,
//! handling argument processing and user interaction.

pub mod errors;
pub mod subcommand;

#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use errors::DestroySubcommandError;
pub use subcommand::handle_destroy_command;
```

#### `subcommand.rs` (Main Command Handler)

- Subcommand orchestration logic
- Environment validation
- Repository initialization
- Command handler invocation
- User output and progress reporting

**Size**: 150-200 lines typical

#### `errors.rs` (Error Types)

- Error enum definitions using `thiserror`
- `.help()` methods for actionable error messages
- Error documentation
- Presentation-layer specific error handling

**Size**: 80-100 lines typical

#### `tests/` (Test Organization)

- `mod.rs`: Test module entry, re-exports
- `integration.rs`: Integration test cases
- `fixtures.rs`: Test fixtures (if needed)

### Migration Strategy

1. **Create Directory Structure**

   ```bash
   mkdir -p src/presentation/commands/destroy/tests
   ```

2. **Split Files**

   - Extract errors → `errors.rs`
   - Move subcommand handler → `subcommand.rs`
   - Create `mod.rs` with documentation
   - Organize tests → `tests/`

3. **Update Imports Throughout Codebase**

   - Update parent `mod.rs` to use new module structure
   - Fix ALL imports in CLI/main that use the destroy command
   - Update any test imports
   - **Breaking changes acceptable** - simplify paths where possible

4. **Verify**

   - Run `cargo test` - all tests must pass
   - Run `cargo clippy` - no new warnings
   - Run `./scripts/pre-commit.sh` - all checks pass

## Implementation Plan

**Important**: This is a single-command refactoring. Run pre-commit checks and commit when complete.

### Subtask 1: Migrate DestroyCommand (2-3 hours)

- [ ] Create `src/presentation/commands/destroy/` directory
- [ ] Create `destroy/mod.rs` with module documentation and re-exports
- [ ] Create `destroy/subcommand.rs` - move handler logic from `destroy.rs`
- [ ] Create `destroy/errors.rs` - extract error types from `destroy.rs`
- [ ] Create `destroy/tests/mod.rs` - test module entry point
- [ ] Create `destroy/tests/integration.rs` - move existing tests (if any)
- [ ] Update `src/presentation/commands/mod.rs` to use new structure
- [ ] **Update ALL imports across codebase** (CLI, main, other modules)
- [ ] Delete old `destroy.rs` file
- [ ] Run `cargo test` to verify all tests pass
- [ ] Run `cargo clippy` to check for issues
- [ ] Run `./scripts/pre-commit.sh` to verify quality
- [ ] **Commit this refactoring**

### Subtask 2: Final Verification and Documentation (30 minutes)

- [ ] Verify destroy command structure matches create/ pattern
- [ ] Update any relevant documentation mentioning command structure
- [ ] Ensure all imports across the codebase are correct
- [ ] Final pre-commit verification: `./scripts/pre-commit.sh`
- [ ] **Final commit with any documentation updates**

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Refactoring-Specific Criteria**:

- [ ] Destroy command uses modular directory structure
- [ ] Directory structure matches `create/` pattern exactly
- [ ] All existing tests continue to pass
- [ ] No changes to business logic (pure refactoring)
- [ ] File sizes are manageable (< 250 lines per file)
- [ ] Clear separation of concerns (subcommand, errors, tests)
- [ ] Module documentation is comprehensive
- [ ] Error types include `.help()` methods
- [ ] Test organization is clear and scalable
- [ ] All imports updated correctly throughout entire codebase

**Import Updates**:

- [ ] All imports in CLI/main updated
- [ ] All imports in tests updated
- [ ] Parent `mod.rs` correctly re-exports types for convenience

**Structure Verification**:

- [ ] Destroy command has `mod.rs`, `subcommand.rs`, `errors.rs`
- [ ] Has `tests/` subdirectory with `mod.rs`, `integration.rs`
- [ ] Parent `mod.rs` correctly imports and re-exports destroy command
- [ ] No orphaned files or unused code

**Commit Strategy**:

- [ ] Refactoring committed independently
- [ ] Pre-commit checks pass before commit
- [ ] Clear, descriptive commit message following conventional commits format
- [ ] Documentation commit separate if needed

## Related Documentation

- [Module Organization Guide](../contributing/module-organization.md) - File organization conventions
- [Error Handling Guide](../contributing/error-handling.md) - Error type patterns and `.help()` methods
- [Testing Conventions](../contributing/testing.md) - Test organization and naming
- [Codebase Architecture](../codebase-architecture.md) - DDD layer structure
- Reference Implementation: `src/presentation/commands/create/` - Example of target structure

## Benefits

### Maintainability ✅

- **Smaller files**: Easier to navigate (150-200 lines vs 290 lines)
- **Single responsibility**: Each file has one clear purpose
- **Easier reviews**: Changes to errors don't require reviewing subcommand logic

### Scalability ✅

- **Room to grow**: Can add config_loader or validators without bloating main file
- **Test organization**: Can add more test modules easily
- **Better IDE support**: Smaller files improve code navigation

### Consistency ✅

- **Uniform patterns**: All presentation commands follow same structure
- **Predictable locations**: Developers know where to find subcommand vs errors vs tests
- **Easier onboarding**: New contributors understand structure quickly

### Testing ✅

- **Dedicated test organization**: Clear separation of test types
- **Integration tests isolated**: Easier to maintain and extend
- **Better test discoverability**: Tests organized by purpose

## Notes

### Why Start with Destroy (Not Other Commands)

We're migrating `destroy` first because:

1. It's the only other presentation command besides `create`
2. Single command makes this a straightforward refactoring
3. Once complete, all presentation commands will be consistent
4. Other commands (provision, configure) don't exist in presentation layer yet

### Future Commands

When adding new presentation commands (provision, configure, etc.):

- Start with modular structure from the beginning
- Follow the `create/` and `destroy/` patterns
- Include `subcommand.rs`, `errors.rs`, `tests/` from the start

### Breaking Changes: Acceptable

**Important**: Since this library is not yet publicly used, breaking changes to import paths are acceptable and even encouraged if they simplify the refactoring:

- ✅ Import paths will change (e.g., `use crate::presentation::commands::destroy::handle_destroy_command`)
- ✅ Can simplify re-exports if it improves clarity
- ✅ Focus on best final structure, not backward compatibility
- ⚠️ Business logic must remain unchanged (pure refactoring)
- ⚠️ All existing tests must still pass (behavior unchanged)

This freedom allows us to:

- Create the cleanest possible module structure
- Avoid complex re-export chains just for compatibility
- Optimize for maintainability over backward compatibility

### Preserving Git History

Consider using `git mv` where possible to preserve file history:

```bash
# When creating new files, use git mv for better history tracking
git mv src/presentation/commands/destroy.rs \
       src/presentation/commands/destroy/subcommand.rs
```

However, since we're splitting files, manual creation may be clearer.

### Related to Command Handlers Refactoring

This refactoring is complementary to issue #56 (Command Handlers refactoring):

- Issue #56: Application layer command handlers
- This issue: Presentation layer commands
- Both establish modular patterns for their respective layers
- Can be implemented independently or in parallel
