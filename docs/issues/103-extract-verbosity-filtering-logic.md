# Extract Verbosity Filtering Logic

**Issue**: [#103](https://github.com/torrust/torrust-tracker-deployer/issues/103)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**: [Refactoring Plan - Proposal #0](../refactors/plans/user-output-architecture-improvements.md#proposal-0-extract-verbosity-filtering-logic)

## Overview

Extract verbosity checking logic from `UserOutput` into a dedicated `VerbosityFilter` struct. This eliminates code duplication across all output methods and makes verbosity rules testable independently.

## Goals

- [ ] Create `VerbosityFilter` struct that encapsulates verbosity logic
- [ ] Replace inline verbosity checks with filter method calls
- [ ] Make verbosity rules independently testable
- [ ] Maintain all existing functionality and test coverage

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output.rs`
**Pattern**: Extract to struct pattern (internal refactoring within module)

### Module Structure Requirements

- [ ] Keep changes within existing `user_output.rs` module
- [ ] Follow module organization conventions (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))
- [ ] Maintain backward compatibility with existing API

### Architectural Constraints

- [ ] No breaking changes to public `UserOutput` API
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Follow testing conventions (see [docs/contributing/testing/](../docs/contributing/testing/))

### Anti-Patterns to Avoid

- ❌ Exposing `VerbosityFilter` struct as public (it's an internal implementation detail, not part of the public API)
- ❌ Making `verbosity_filter` field public (users should only interact through existing methods)
- ❌ Complex verbosity logic - keep methods simple and self-documenting
- ❌ Breaking existing tests or requiring extensive test rewrites

**Note**: `VerbosityLevel` enum remains public (it's already part of the public API). Users pass `VerbosityLevel` to `UserOutput` constructors, and `UserOutput` creates the `VerbosityFilter` internally.

## Problem

Verbosity checks are duplicated across all output methods, violating DRY principle:

```rust
pub fn progress(&mut self, message: &str) {
    if self.verbosity >= VerbosityLevel::Normal {
        writeln!(self.stderr_writer, "⏳ {message}").ok();
    }
}

pub fn success(&mut self, message: &str) {
    if self.verbosity >= VerbosityLevel::Normal {
        writeln!(self.stderr_writer, "✅ {message}").ok();
    }
}
```

This makes the code harder to maintain and test, and violates the Single Responsibility Principle.

## Specifications

### VerbosityFilter Struct

Create a private struct that encapsulates verbosity logic:

```rust
/// Determines what messages should be displayed based on verbosity level
struct VerbosityFilter {
    level: VerbosityLevel,
}

impl VerbosityFilter {
    fn new(level: VerbosityLevel) -> Self {
        Self { level }
    }

    /// Check if messages at the given level should be shown
    fn should_show(&self, required_level: VerbosityLevel) -> bool {
        self.level >= required_level
    }

    /// Progress messages require Normal level
    fn should_show_progress(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Errors are always shown
    fn should_show_errors(&self) -> bool {
        true
    }
}
```

### UserOutput Integration

Update `UserOutput` to use the filter as a **private field**:

```rust
pub struct UserOutput {
    verbosity_filter: VerbosityFilter,  // Private field - no `pub` keyword
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self::with_writers(
            verbosity,
            Box::new(std::io::stdout()),
            Box::new(std::io::stderr()),
        )
    }

    pub fn with_writers(
        verbosity: VerbosityLevel,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self {
            verbosity_filter: VerbosityFilter::new(verbosity),  // Create filter internally
            stdout_writer,
            stderr_writer,
        }
    }

    pub fn progress(&mut self, message: &str) {
        if self.verbosity_filter.should_show_progress() {
            writeln!(self.stderr_writer, "⏳ {message}").ok();
        }
    }
}
```

**Key Points**:

- **`VerbosityLevel` enum**: Remains public (already part of the API) - users pass this to constructors
- **`VerbosityFilter` struct**: Module-private (no `pub` keyword) - internal implementation detail
- **`verbosity_filter` field**: Private field in `UserOutput` - not accessible from outside
- **Public API unchanged**: Users continue to pass `VerbosityLevel::Normal`, `VerbosityLevel::Quiet`, etc. to constructors
- **Internal creation**: `UserOutput` creates the `VerbosityFilter` internally using `VerbosityFilter::new(verbosity)`

This design allows future CLI arguments like `--verbosity=quiet` to control output, while keeping the internal filtering mechanism private.

## Implementation Plan

### Phase 1: Create VerbosityFilter (30 minutes)

- [ ] Create `VerbosityFilter` struct with `level` field
- [ ] Implement `new()` constructor
- [ ] Implement `should_show()` base method
- [ ] Add convenience methods: `should_show_progress()`, `should_show_errors()`, etc.

### Phase 2: Add Unit Tests (45 minutes)

- [ ] Test `should_show_progress()` at different verbosity levels
- [ ] Test `should_show_errors()` always returns true
- [ ] Test general `should_show()` method
- [ ] Verify edge cases and boundary conditions

### Phase 3: Integrate with UserOutput (1 hour)

- [ ] Replace `verbosity: VerbosityLevel` field with `verbosity_filter: VerbosityFilter`
- [ ] Update `UserOutput::new()` constructor
- [ ] Update `UserOutput::with_writers()` constructor
- [ ] Update all output methods to use `verbosity_filter.should_show_X()`

### Phase 4: Verify & Clean Up (30 minutes)

- [ ] Run all existing tests and verify they pass
- [ ] Run linters: `cargo run --bin linter all`
- [ ] Review code for any remaining inline verbosity checks
- [ ] Update module documentation if needed

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `VerbosityFilter` struct created with all required methods
- [ ] All inline verbosity checks replaced with filter method calls
- [ ] Unit tests added for `VerbosityFilter` behavior
- [ ] All existing tests still pass without modification
- [ ] No breaking changes to public `UserOutput` API
- [ ] Module documentation updated if needed

## Related Documentation

- [Refactoring Plan](../refactors/plans/user-output-architecture-improvements.md) - Complete refactoring plan with all proposals
- [Development Principles](../development-principles.md) - Core principles including testability and maintainability
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Testing Conventions](../contributing/testing/) - Testing best practices

## Notes

### Rationale

- **DRY Principle**: Eliminates repeated verbosity checks across multiple methods
- **Testability**: Verbosity logic can now be tested independently from output formatting
- **Clarity**: Named methods like `should_show_progress()` are self-documenting
- **Extensibility**: Easy to add complex filtering rules in one place

### Benefits

- ✅ Removes code duplication across all output methods
- ✅ Makes verbosity rules testable independently
- ✅ Self-documenting code with named filter methods
- ✅ Single source of truth for verbosity logic
- ✅ Easy to extend with more complex filtering rules

### Estimated Time

**Total**: ~3 hours (conservative estimate with testing and verification)
