# Rename App Commands to Command Handlers

**Issue**: [#22](https://github.com/torrust/torrust-tracker-deployer/issues/22)
**Parent Epic**: #10 - UI Layer Destroy Command
**Related**: [Epic #9: App Layer Destroy Command](https://github.com/torrust/torrust-tracker-deployer/issues/9)

## Overview

Refactor terminology to distinguish between UI commands (Clap subcommands) and Application Layer commands (DDD command handlers). This provides clear separation as we introduce UI-level commands.

As we add Clap subcommands to the CLI, we need clear terminology:

- **UI Command**: Clap subcommand (e.g., `destroy`, `provision`) - user-facing CLI interface
- **Command Handler**: DDD Application Layer command (e.g., `DestroyCommand`, `ProvisionCommand`) - business logic

## Goals

- [ ] Rename command structures and modules for clarity
- [ ] Update documentation to use "command handler" terminology
- [ ] Ensure consistent naming across the entire codebase
- [ ] Maintain all existing functionality without breaking changes

## Specifications

### Current Structure (to be renamed)

```text
src/application/commands/
├── destroy.rs          // Contains DestroyCommand
├── provision.rs        // Contains ProvisionCommand
└── configure.rs        // Contains ConfigureCommand
```

### Target Structure (after renaming)

```text
src/application/command_handlers/
├── destroy.rs          // Contains DestroyCommandHandler
├── provision.rs        // Contains ProvisionCommandHandler
└── configure.rs        // Contains ConfigureCommandHandler
```

### Renaming Mapping

| Current Name       | New Name                  | File Location                                   |
| ------------------ | ------------------------- | ----------------------------------------------- |
| `DestroyCommand`   | `DestroyCommandHandler`   | `src/application/command_handlers/destroy.rs`   |
| `ProvisionCommand` | `ProvisionCommandHandler` | `src/application/command_handlers/provision.rs` |
| `ConfigureCommand` | `ConfigureCommandHandler` | `src/application/command_handlers/configure.rs` |

### Module Structure Changes

```rust
// Before: src/application/commands/mod.rs
pub mod destroy;
pub mod provision;
pub mod configure;

pub use destroy::DestroyCommand;
pub use provision::ProvisionCommand;
pub use configure::ConfigureCommand;

// After: src/application/command_handlers/mod.rs
pub mod destroy;
pub mod provision;
pub mod configure;

pub use destroy::DestroyCommandHandler;
pub use provision::ProvisionCommandHandler;
pub use configure::ConfigureCommandHandler;
```

### Import Updates Required

All imports across the codebase need updating:

```rust
// Before
use crate::application::commands::destroy::DestroyCommand;

// After
use crate::application::command_handlers::destroy::DestroyCommandHandler;
```

### Documentation Updates

Update the following documentation files:

- `docs/codebase-architecture.md` - Update DDD layer references
- `docs/development-principles.md` - Update terminology references
- Code comments and rustdoc comments
- ADR documents that reference the command pattern

## Implementation Plan

### Subtask 1: Rename Files and Directories (30 minutes)

- [ ] Rename `src/application/commands/` to `src/application/command_handlers/`
- [ ] Update `mod.rs` files to reflect new structure
- [ ] Update `src/application/mod.rs` to import from `command_handlers`

### Subtask 2: Rename Structs and Types (45 minutes)

- [ ] Rename `DestroyCommand` to `DestroyCommandHandler` in `destroy.rs`
- [ ] Rename `ProvisionCommand` to `ProvisionCommandHandler` in `provision.rs`
- [ ] Rename `ConfigureCommand` to `ConfigureCommandHandler` in `configure.rs`
- [ ] Update all method signatures and implementations
- [ ] Update trait implementations if any

### Subtask 3: Update Imports and References (45 minutes)

- [ ] Find all imports using `grep -r "application::commands" src/`
- [ ] Update imports to use `application::command_handlers`
- [ ] Find all struct references using `grep -r "DestroyCommand\|ProvisionCommand\|ConfigureCommand" src/`
- [ ] Update struct instantiations and method calls
- [ ] Update test files and test imports

### Subtask 4: Update Documentation (30 minutes)

- [ ] Update `docs/codebase-architecture.md` with new terminology
- [ ] Update inline documentation and rustdoc comments
- [ ] Update any ADR documents referencing commands
- [ ] Update code examples in documentation

### Subtask 5: Verify and Test (30 minutes)

- [ ] Run `cargo build` to ensure compilation
- [ ] Run `cargo test` to ensure all tests pass
- [ ] Run `cargo run --bin linter all` to ensure code quality
- [ ] Verify no references to old naming remain using `grep -r "DestroyCommand\b" src/`

## Acceptance Criteria

- [ ] All command structures renamed to `*CommandHandler` pattern
- [ ] Module path updated from `commands` to `command_handlers`
- [ ] All imports updated throughout the codebase
- [ ] Documentation reflects new terminology consistently
- [ ] All tests pass without modifications to test logic
- [ ] No compilation errors or warnings
- [ ] Linting passes successfully
- [ ] No references to old naming pattern remain in codebase
- [ ] Functionality remains unchanged (no behavioral changes)

## Related Documentation

- [Codebase Architecture](../codebase-architecture.md) - DDD layer organization
- [Development Principles](../development-principles.md) - Code quality standards
- [Epic #10](./10-epic-ui-layer-destroy-command.md) - Parent epic context
- [Module Organization](../contributing/module-organization.md) - Code organization conventions

## Notes

**Estimated Time**: 2.5-3 hours

**Why This Change**:

- Clarifies distinction between UI layer (Clap subcommands) and Application layer (business logic)
- Follows DDD naming conventions where "Command" typically refers to the intent/message, while "CommandHandler" processes it
- Prepares codebase for clean UI layer implementation in subsequent issues

**Search Commands for Verification**:

```bash
# Find remaining old imports
grep -r "application::commands" src/

# Find remaining old struct references
grep -r "DestroyCommand\b\|ProvisionCommand\b\|ConfigureCommand\b" src/

# Verify new naming is used
grep -r "CommandHandler" src/
```
