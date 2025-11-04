# Add Buffering Control

**Issue**: [#138](https://github.com/torrust/torrust-tracker-deployer/issues/138)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**:

- Refactoring Plan: [docs/refactors/plans/user-output-architecture-improvements.md](../refactors/plans/user-output-architecture-improvements.md)
- Proposal #6 (Dependency): Type-Safe Channel Routing ([#135](https://github.com/torrust/torrust-tracker-deployer/issues/135))

## Overview

This task adds explicit buffering control to the `UserOutput` module by providing a `flush()` method. While writes are typically line-buffered by default, explicit flush control ensures output appears immediately when needed and provides better testing capabilities.

**Current State**: The codebase has implemented proposals 0-6, including:

- `Theme` support (Proposal #2, Issue #124)
- `OutputMessage` trait with concrete message types (Proposal #3, Issue #127)
- Type-safe channel routing with `StdoutWriter` and `StderrWriter` newtypes (Proposal #6, Issue #135)

Output is written immediately through the typed writer wrappers, but there's no explicit flush control available to callers.

## Goals

- [ ] Add `flush()` method to `UserOutput` for explicit buffer control
- [ ] Document buffering behavior in module documentation
- [ ] Add tests demonstrating flush behavior
- [ ] Maintain backward compatibility with existing API
- [ ] Follow Rust standard patterns from `Write` trait

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output.rs`
**Pattern**: Standard Rust `Write` trait pattern for buffering control

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Keep presentation logic in presentation layer (no domain concerns)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Must work with existing typed writer wrappers (`StdoutWriter`, `StderrWriter`)
- [ ] Must be safe to call multiple times without side effects
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Testing covers successful flush and error scenarios

### Anti-Patterns to Avoid

- ❌ Automatic flushing after every write (defeats purpose of buffering)
- ❌ Silent error suppression (flush errors should be propagated or logged)
- ❌ Complex buffering strategies (keep it simple)
- ❌ Breaking existing behavior (flushing should be opt-in)

## Specifications

### Flush Method Implementation

Add an explicit `flush()` method that flushes both stdout and stderr writers:

````rust
impl UserOutput {
    /// Flush all pending output to stdout and stderr
    ///
    /// This is typically not needed as writes are line-buffered by default,
    /// but can be useful for ensuring output appears immediately.
    ///
    /// # Errors
    ///
    /// Returns an error if flushing either stream fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let mut output = UserOutput::new(VerbosityLevel::Normal);
    /// output.progress("Processing...");
    /// output.flush().expect("Failed to flush output");
    /// ```
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.0.flush()?;
        self.stderr.0.flush()?;
        Ok(())
    }
}
````

### Documentation Updates

Update module-level documentation to explain buffering behavior:

````rust
//! ## Buffering Behavior
//!
//! Output is line-buffered by default. Messages are typically flushed automatically
//! after each newline. For cases where immediate output is critical (e.g., before
//! long-running operations), call `flush()` explicitly:
//!
//! ```rust,ignore
//! output.progress("Starting long operation...");
//! output.flush()?; // Ensure message appears before operation starts
//! perform_long_operation();
//! ```
````

### Testing Strategy

Add tests to verify flush behavior:

```rust
#[cfg(test)]
mod buffering_tests {
    use super::*;

    #[test]
    fn it_should_flush_all_writers() {
        let mut output = create_test_output(VerbosityLevel::Normal);
        output.output().progress("Test");
        output.output().flush().expect("Flush should succeed");

        // Verify output is present (flushed)
        assert!(!output.stderr().is_empty());
    }

    #[test]
    fn it_should_be_safe_to_flush_multiple_times() {
        let mut output = create_test_output(VerbosityLevel::Normal);
        output.output().progress("Test");

        // Multiple flushes should be safe
        output.output().flush().expect("First flush should succeed");
        output.output().flush().expect("Second flush should succeed");
        output.output().flush().expect("Third flush should succeed");
    }

    #[test]
    fn it_should_flush_empty_buffers_safely() {
        let mut output = create_test_output(VerbosityLevel::Normal);

        // Flushing with no output should be safe
        output.output().flush().expect("Flushing empty buffers should succeed");
    }
}
```

## Implementation Plan

### Phase 1: Core Implementation (30 minutes)

- [ ] Add `flush()` method to `UserOutput`
- [ ] Implement flush for both `stdout` and `stderr` writers
- [ ] Add rustdoc documentation for the method
- [ ] Verify method signature matches Rust conventions

### Phase 2: Documentation (20 minutes)

- [ ] Update module-level documentation with buffering behavior section
- [ ] Add usage examples showing when to use `flush()`
- [ ] Document error handling for flush failures
- [ ] Update related documentation if needed

### Phase 3: Testing (30 minutes)

- [ ] Add unit tests for successful flush
- [ ] Add tests for multiple flush calls
- [ ] Add tests for empty buffer flushing
- [ ] Verify all existing tests still pass

### Phase 4: Quality Assurance (20 minutes)

- [ ] Run `./scripts/pre-commit.sh` and fix any issues
- [ ] Verify all linters pass
- [ ] Check code coverage for new code
- [ ] Review for adherence to project conventions

**Total Estimated Time**: 2 hours

## Acceptance Criteria

### Functional Requirements

- [ ] `UserOutput::flush()` method flushes both stdout and stderr
- [ ] Flush can be called multiple times safely
- [ ] Flush returns `std::io::Result<()>` for error handling
- [ ] Existing functionality is not affected by the addition

### Documentation Requirements

- [ ] Method has comprehensive rustdoc comments
- [ ] Module documentation includes buffering behavior section
- [ ] Usage examples demonstrate when to call `flush()`
- [ ] Error handling is documented

### Testing Requirements

- [ ] Unit tests cover successful flush scenarios
- [ ] Tests verify idempotency (multiple flushes are safe)
- [ ] Tests verify empty buffer handling
- [ ] All existing tests continue to pass

### Quality Requirements (applies to every commit and PR)

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Code follows project conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Error handling follows project patterns (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Changes are documented in code comments and rustdoc

**Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

## Related Documentation

- [Development Principles](../development-principles.md) - Core principles including testability
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Error Handling Guide](../contributing/error-handling.md) - Error handling patterns
- [User Output Research](../research/UX/console-app-output-patterns.md) - Background on output patterns

## Notes

### Why This Matters

- **Testing**: Explicit flush control makes test assertions more reliable
- **Immediate Feedback**: Ensures critical messages appear before long operations
- **Standard Pattern**: Follows Rust's `Write` trait conventions
- **Minimal Impact**: Low-risk enhancement with clear benefits

### Implementation Considerations

- Line-buffered output typically flushes after newlines automatically
- Explicit flush is mainly useful before long-running operations or in tests
- Error handling should propagate flush failures to callers
- No need for automatic flushing after every write (would defeat buffering purpose)

### Future Enhancements

- Consider adding `auto_flush` configuration option if needed
- Could add per-channel flush methods (`flush_stdout()`, `flush_stderr()`) if use case emerges
- May want to add flush timing metrics for observability (future work)

---

**Created**: November 4, 2025
**Status**: 📋 Not Started
**Priority**: P2 (Phase 2 - Polish & Extensions)
**Estimated Effort**: 2 hours
