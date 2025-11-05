# Refactor user_output.rs Module to Folder Structure

**Issue**: [#149](https://github.com/torrust/torrust-tracker-deployer/issues/149)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**:

- [User Output Module](https://github.com/torrust/torrust-tracker-deployer/blob/main/src/presentation/user_output.rs)
- [Module Organization Guide](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/module-organization.md)

## Overview

The `src/presentation/user_output.rs` file has grown to **4,226 lines**, making it difficult to navigate and maintain. This task refactors the monolithic file into a well-organized folder module structure with focused, cohesive submodules that follow project conventions.

This refactoring improves:

- **Discoverability**: Clear separation makes it easier to find specific functionality
- **Maintainability**: Smaller, focused files are easier to understand and modify
- **Testability**: Tests can be organized alongside their related code
- **Collaboration**: Multiple developers can work on different aspects simultaneously
- **Code Review**: Smaller, focused changes are easier to review

## Goals

- [ ] Convert `user_output.rs` to `user_output/` folder module
- [ ] Separate concerns into logical submodules
- [ ] Maintain backward compatibility (no public API changes)
- [ ] Ensure all tests pass without modification
- [ ] Follow project module organization conventions
- [ ] Improve code discoverability and navigation

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output/`
**Pattern**: Folder module with focused submodules

### Module Structure Requirements

- [ ] Follow folder module conventions (see [Module Organization Guide](../contributing/module-organization.md))
- [ ] Use `mod.rs` as the public API entry point with re-exports
- [ ] Group related functionality in focused submodules
- [ ] Organize tests alongside their code (when appropriate)
- [ ] Maintain clear public/private boundaries

### Architectural Constraints

- [ ] **No public API changes** - All existing public types, traits, and functions must remain accessible
- [ ] **Backward compatibility** - Existing code using `UserOutput` must work without changes
- [ ] **Test compatibility** - All existing tests must pass without modification
- [ ] **Import paths** - Existing imports like `use crate::presentation::user_output::UserOutput;` must continue to work

### Anti-Patterns to Avoid

- ❌ Breaking public API surface
- ❌ Circular dependencies between submodules
- ❌ Leaking internal implementation details through public re-exports
- ❌ Creating submodules that are too granular (prefer cohesion)

## Specifications

### Current State Analysis

The `user_output.rs` file (4,226 lines) contains:

**Core Components** (~800 lines):

- `Theme` struct and implementations
- `VerbosityLevel` enum
- `Channel` enum
- `UserOutput` main struct

**Traits and Abstractions** (~300 lines):

- `OutputMessage` trait
- `FormatterOverride` trait
- `OutputSink` trait

**Message Types** (~600 lines):

- `ProgressMessage`
- `SuccessMessage`
- `WarningMessage`
- `ErrorMessage`
- `ResultMessage`
- `StepsMessage` and `StepsMessageBuilder`
- `InfoBlockMessage` and `InfoBlockMessageBuilder`

**Infrastructure** (~500 lines):

- `VerbosityFilter` private component
- Type-safe wrappers: `StdoutWriter`, `StderrWriter`
- `StandardSink`, `CompositeSink`, `FileSink`, `TelemetrySink`

**Testing Infrastructure** (~200 lines):

- `test_support` module with `TestWriter` and `TestUserOutput`

**Tests** (~1,800 lines):

- Comprehensive test suite covering all functionality

### Proposed Folder Structure

```text
src/presentation/user_output/
├── mod.rs                    # Public API, re-exports, module documentation
├── core.rs                   # UserOutput main struct and core impl
├── theme.rs                  # Theme struct and predefined themes
├── verbosity.rs              # VerbosityLevel enum and VerbosityFilter
├── channel.rs                # Channel enum
├── traits.rs                 # OutputMessage, FormatterOverride, OutputSink traits
├── messages/                 # Message type implementations
│   ├── mod.rs
│   ├── progress.rs           # ProgressMessage
│   ├── success.rs            # SuccessMessage
│   ├── warning.rs            # WarningMessage
│   ├── error.rs              # ErrorMessage
│   ├── result.rs             # ResultMessage
│   ├── steps.rs              # StepsMessage and builder
│   └── info_block.rs         # InfoBlockMessage and builder
├── sinks/                    # OutputSink implementations
│   ├── mod.rs
│   ├── standard.rs           # StandardSink
│   ├── composite.rs          # CompositeSink
│   ├── file.rs               # FileSink
│   ├── telemetry.rs          # TelemetrySink
│   └── writers.rs            # StdoutWriter, StderrWriter wrappers
├── formatters/               # FormatterOverride implementations
│   ├── mod.rs
│   └── json.rs               # JsonFormatter
└── test_support.rs           # TestWriter, TestUserOutput (kept as submodule)
```

### Module Responsibility Matrix

| Module            | Responsibility                       | Public Items                                                 | Lines (approx) |
| ----------------- | ------------------------------------ | ------------------------------------------------------------ | -------------- |
| `mod.rs`          | Public API and documentation         | Re-exports all public items                                  | ~150           |
| `core.rs`         | Main `UserOutput` struct and methods | `UserOutput`                                                 | ~400           |
| `theme.rs`        | Theme configuration                  | `Theme`                                                      | ~150           |
| `verbosity.rs`    | Verbosity levels and filtering       | `VerbosityLevel`, `VerbosityFilter` (private)                | ~200           |
| `channel.rs`      | Output channel routing               | `Channel`                                                    | ~50            |
| `traits.rs`       | Core abstractions                    | `OutputMessage`, `FormatterOverride`, `OutputSink`           | ~150           |
| `messages/*.rs`   | Message implementations              | All message types and builders                               | ~700           |
| `sinks/*.rs`      | Output sink implementations          | `StandardSink`, `CompositeSink`, `FileSink`, `TelemetrySink` | ~500           |
| `formatters/*.rs` | Formatter implementations            | `JsonFormatter`                                              | ~100           |
| `test_support.rs` | Testing utilities                    | `TestWriter`, `TestUserOutput`                               | ~200           |

### Key Design Decisions

#### 1. Folder Module Pattern

Use a folder with `mod.rs` as the entry point, following Rust conventions:

- `mod.rs` contains public API and re-exports
- Internal modules are private by default
- Public types/traits are explicitly re-exported
- Module-level documentation lives in `mod.rs`

#### 2. Message Organization

Group message types in a `messages/` subfolder:

- Each message type gets its own file
- Builders stay with their message types
- `messages/mod.rs` re-exports all message types

#### 3. Sink Organization

Group sink implementations in a `sinks/` subfolder:

- Each sink type gets its own file
- Type-safe writers (`StdoutWriter`, `StderrWriter`) in `writers.rs`
- `sinks/mod.rs` re-exports all sink types

#### 4. Test Organization

**Keep unit tests co-located with their modules** using `#[cfg(test)]`:

- Tests stay in the same file as the code they test (e.g., `theme.rs` contains `Theme` and its tests)
- Use `#[cfg(test)]` modules to group tests within each file
- Only create a separate `tests/` folder if integration tests are needed (tests that import from multiple modules)
- Current tests are all unit tests (only use types from within `user_output` module), so they should stay co-located

**Rationale**: Unit tests that only depend on types from a single module should be kept close to the code they test. This improves discoverability and makes it easier to maintain tests alongside implementation changes.

#### 5. Test Infrastructure

The `test_support.rs` module provides utilities for testing:

- `TestWriter` - Captures output for verification
- `TestUserOutput` - Test fixture with captured stdout/stderr

This module should remain accessible for other modules that need to test user output functionality.

#### 6. Backward Compatibility

Maintain existing import paths:

```rust
// These must continue to work
use crate::presentation::user_output::UserOutput;
use crate::presentation::user_output::{Theme, VerbosityLevel};
use crate::presentation::user_output::test_support::TestUserOutput;
```

## Implementation Plan

### Phase 1: Create Folder Structure and Core Modules (1-2 hours)

- [ ] Create `src/presentation/user_output/` directory
- [ ] Create `mod.rs` with module documentation and structure outline
- [ ] Extract `Theme` to `theme.rs` with tests in `#[cfg(test)]` module
- [ ] Extract `VerbosityLevel` and `VerbosityFilter` to `verbosity.rs` with tests in `#[cfg(test)]` module
- [ ] Extract `Channel` to `channel.rs` with tests in `#[cfg(test)]` module
- [ ] Update `mod.rs` with re-exports for backward compatibility
- [ ] Verify existing code still compiles and all tests pass

### Phase 2: Extract Traits and Core Logic (1-2 hours)

- [ ] Extract `OutputMessage`, `FormatterOverride`, `OutputSink` to `traits.rs`
- [ ] Extract main `UserOutput` struct and methods to `core.rs`
- [ ] Update `mod.rs` with re-exports
- [ ] Verify all functionality works

### Phase 3: Organize Message Types (2-3 hours)

- [ ] Create `messages/` subdirectory
- [ ] Extract each message type to its own file with tests in `#[cfg(test)]` modules:
  - [ ] `progress.rs` - `ProgressMessage` with tests
  - [ ] `success.rs` - `SuccessMessage` with tests
  - [ ] `warning.rs` - `WarningMessage` with tests
  - [ ] `error.rs` - `ErrorMessage` with tests
  - [ ] `result.rs` - `ResultMessage` with tests
  - [ ] `steps.rs` - `StepsMessage`, `StepsMessageBuilder`, and tests
  - [ ] `info_block.rs` - `InfoBlockMessage`, `InfoBlockMessageBuilder`, and tests
- [ ] Create `messages/mod.rs` with re-exports
- [ ] Update parent `mod.rs` with message re-exports
- [ ] Verify all message tests pass

### Phase 4: Organize Sink Implementations (1-2 hours)

- [ ] Create `sinks/` subdirectory
- [ ] Extract `StdoutWriter` and `StderrWriter` to `writers.rs` with tests in `#[cfg(test)]` module
- [ ] Extract sink implementations to individual files with tests:
  - [ ] `standard.rs` - `StandardSink` with tests
  - [ ] `composite.rs` - `CompositeSink` with tests
  - [ ] `file.rs` - `FileSink` with tests
  - [ ] `telemetry.rs` - `TelemetrySink` with tests
- [ ] Create `sinks/mod.rs` with re-exports
- [ ] Update parent `mod.rs` with sink re-exports
- [ ] Verify all sink tests pass

### Phase 5: Organize Formatters (30 minutes)

- [ ] Create `formatters/` subdirectory
- [ ] Extract `JsonFormatter` to `json.rs` with tests in `#[cfg(test)]` module
- [ ] Create `formatters/mod.rs` with re-exports
- [ ] Update parent `mod.rs` with formatter re-exports
- [ ] Verify formatter tests pass

### Phase 6: Organize Test Infrastructure (30 minutes)

- [ ] Extract `test_support` module to `test_support.rs`
- [ ] Verify `test_support` is accessible for testing in other modules
- [ ] Verify all co-located unit tests still pass
- [ ] Check that test utilities (TestWriter, TestUserOutput) work correctly

### Phase 7: Documentation and Cleanup (30 minutes)

- [ ] Update module-level documentation in `mod.rs`
- [ ] Add module documentation to each submodule
- [ ] Update imports in `mod.rs` for clarity
- [ ] Remove old `user_output.rs` file
- [ ] Verify documentation builds: `cargo doc --no-deps`

### Phase 8: Final Verification (30 minutes)

- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`
- [ ] Verify no public API changes
- [ ] Verify all tests pass
- [ ] Verify no unused dependencies: `cargo machete`
- [ ] Check for any remaining TODO comments
- [ ] Manual smoke test of basic functionality

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All tests pass without modification: `cargo test`
- [ ] Documentation builds successfully: `cargo doc --no-deps`
- [ ] No unused dependencies: `cargo machete`

**Structural Criteria**:

- [ ] `user_output.rs` is converted to `user_output/` folder
- [ ] `mod.rs` serves as the public API entry point
- [ ] All submodules are appropriately sized (< 500 lines each)
- [ ] Clear separation of concerns across submodules
- [ ] Test organization mirrors module structure

**Backward Compatibility**:

- [ ] All existing public types remain accessible
- [ ] All existing import paths continue to work
- [ ] No changes required in code using `UserOutput`
- [ ] All existing tests pass without modification
- [ ] Public API surface is identical

**Code Quality**:

- [ ] Each module has clear, focused responsibility
- [ ] Module documentation explains purpose and usage
- [ ] Follows project module organization conventions
- [ ] No circular dependencies between submodules
- [ ] Private implementation details not leaked

**Testing**:

- [ ] Tests organized by component
- [ ] Test coverage maintained (no reduction)
- [ ] Test infrastructure (`test_support`) easily accessible
- [ ] Integration tests verify cross-component functionality

## Related Documentation

- [Module Organization Guide](../contributing/module-organization.md) - Module organization conventions
- [User Output Epic #102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - Parent epic
- [Codebase Architecture](../codebase-architecture.md) - Overall architecture principles
- [Testing Conventions](../contributing/testing/) - Testing best practices

## Notes

### Rationale for Refactoring

The current 4,226-line file violates the Single Responsibility Principle and makes navigation difficult. Breaking it into focused modules:

1. **Improves Discoverability**: Clear file names make it easy to find specific functionality
2. **Reduces Cognitive Load**: Developers work with smaller, focused files
3. **Enables Parallel Development**: Multiple developers can work on different aspects
4. **Simplifies Testing**: Tests can be organized alongside related code
5. **Follows Best Practices**: Aligns with Rust community conventions

### Design Considerations

**Why Not More Granular?**

The proposed structure balances granularity with cohesion:

- **Messages**: Each message type is simple enough to stand alone
- **Sinks**: Each sink has distinct responsibility (console, file, composite, telemetry)
- **Core**: Keeps related `UserOutput` methods together

**Why Separate `messages/` and `sinks/`?**

These represent different architectural concerns:

- **Messages**: Define the "what" (content and formatting)
- **Sinks**: Define the "where" (output destinations)

#### Test Organization Strategy

Tests are organized in a dedicated `tests/` folder to:

- Keep test code separate from production code
- Mirror the module structure for easy navigation
- Group integration tests separately from unit tests

### Migration Safety

This refactoring is **zero-risk** for existing code:

- All public items remain in the same namespace via re-exports
- No API changes required
- Existing tests validate correctness
- Can be done incrementally and tested at each step

### Estimated Total Time

**8-10 hours** total:

- Core refactoring: 6-8 hours
- Testing and verification: 2 hours
- Documentation: 1 hour

Can be done in multiple commits/sessions without breaking existing functionality.
