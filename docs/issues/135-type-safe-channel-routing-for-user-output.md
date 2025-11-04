# Type-Safe Channel Routing for User Output

**Issue**: [#135](https://github.com/torrust/torrust-tracker-deployer/issues/135)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**: [Refactoring Plan - Proposal 6](../refactors/plans/user-output-architecture-improvements.md#proposal-6-type-safe-channel-routing)

## Overview

Add compile-time type safety for channel routing (stdout vs stderr) in the `UserOutput` module by introducing newtype wrappers for writers. Currently, channel routing is done with runtime pattern matching which, while functional, doesn't provide compile-time guarantees that messages go to the correct channel.

This refactoring introduces `StdoutWriter` and `StderrWriter` newtype wrappers that make channel routing explicit in the type system, preventing accidental channel confusion at compile time.

## Goals

- [ ] Add type-safe newtype wrappers for stdout and stderr writers
- [ ] Replace runtime channel matching with compile-time type safety
- [ ] Make channel routing explicit and self-documenting in code
- [ ] Maintain existing functionality and API compatibility
- [ ] Improve IDE support and code navigation for channel-specific operations

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output.rs`
**Pattern**: Type-safe wrappers using newtype pattern

### Module Structure Requirements

- [ ] Follow module organization conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Keep writer wrappers private as implementation details
- [ ] Maintain public API compatibility with existing code

### Architectural Constraints

- [ ] Zero-cost abstraction using newtype pattern
- [ ] No runtime overhead compared to current implementation
- [ ] Preserve existing error handling behavior
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Exposing writer wrappers in public API unnecessarily
- ❌ Adding runtime checks when compile-time safety is available
- ❌ Breaking existing test infrastructure

## Specifications

### Newtype Wrappers for Stdout and Stderr

Create type-safe wrappers using the newtype pattern:

```rust
/// Stdout writer wrapper for type safety
///
/// This newtype wrapper ensures that stdout-specific operations
/// can only be performed on stdout writers, preventing accidental
/// channel confusion at compile time.
struct StdoutWriter(Box<dyn Write + Send + Sync>);

impl StdoutWriter {
    /// Create a new stdout writer wrapper
    fn new(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self(writer)
    }

    /// Write a line to stdout
    ///
    /// Writes the given message followed by a newline to the stdout channel.
    /// Errors are silently ignored as output operations are best-effort.
    fn write_line(&mut self, message: &str) {
        writeln!(self.0, "{message}").ok();
    }
}

/// Stderr writer wrapper for type safety
///
/// This newtype wrapper ensures that stderr-specific operations
/// can only be performed on stderr writers, preventing accidental
/// channel confusion at compile time.
struct StderrWriter(Box<dyn Write + Send + Sync>);

impl StderrWriter {
    /// Create a new stderr writer wrapper
    fn new(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self(writer)
    }

    /// Write a line to stderr
    ///
    /// Writes the given message followed by a newline to the stderr channel.
    /// Errors are silently ignored as output operations are best-effort.
    fn write_line(&mut self, message: &str) {
        writeln!(self.0, "{message}").ok();
    }
}
```

### Updated UserOutput Structure

Replace raw `Box<dyn Write>` fields with typed wrappers:

```rust
pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    stdout: StdoutWriter,
    stderr: StderrWriter,
    formatter_override: Option<Box<dyn FormatterOverride>>,
}
```

### Type-Safe Writer Access

Add private helper methods that leverage type safety:

```rust
impl UserOutput {
    /// Write a message to stdout using the typed writer
    fn write_to_stdout(&mut self, formatted: &str) {
        self.stdout.write_line(formatted);
    }

    /// Write a message to stderr using the typed writer
    fn write_to_stderr(&mut self, formatted: &str) {
        self.stderr.write_line(formatted);
    }
}
```

### Updated Message Writing Logic

Replace runtime channel matching with compile-time dispatch:

```rust
impl UserOutput {
    /// Write a message to the appropriate channel
    pub fn write(&mut self, message: &dyn OutputMessage) {
        if !self.verbosity_filter.should_show(message.required_verbosity()) {
            return;
        }

        let formatted = if let Some(ref formatter) = self.formatter_override {
            formatter.transform(&message.format(&self.theme), message)
        } else {
            message.format(&self.theme)
        };

        // Type-safe dispatch based on channel
        match message.channel() {
            Channel::Stdout => self.write_to_stdout(&formatted),
            Channel::Stderr => self.write_to_stderr(&formatted),
        }
    }
}
```

### Constructor Updates

Update all constructors to use typed wrappers:

```rust
impl UserOutput {
    pub fn with_theme_and_writers(
        verbosity: VerbosityLevel,
        theme: Theme,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self {
            theme,
            verbosity_filter: VerbosityFilter::new(verbosity),
            stdout: StdoutWriter::new(stdout_writer),
            stderr: StderrWriter::new(stderr_writer),
            formatter_override: None,
        }
    }

    pub fn with_theme_writers_and_formatter(
        verbosity: VerbosityLevel,
        theme: Theme,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>,
        formatter_override: Option<Box<dyn FormatterOverride>>,
    ) -> Self {
        Self {
            theme,
            verbosity_filter: VerbosityFilter::new(verbosity),
            stdout: StdoutWriter::new(stdout_writer),
            stderr: StderrWriter::new(stderr_writer),
            formatter_override,
        }
    }
}
```

## Implementation Plan

### Phase 1: Create Newtype Wrappers (30 minutes)

- [ ] Task 1.1: Create `StdoutWriter` newtype struct with documentation
- [ ] Task 1.2: Create `StderrWriter` newtype struct with documentation
- [ ] Task 1.3: Implement `new()` constructor for both wrappers
- [ ] Task 1.4: Implement `write_line()` method for both wrappers
- [ ] Task 1.5: Add unit tests for wrapper creation and writing

### Phase 2: Update UserOutput Structure (45 minutes)

- [ ] Task 2.1: Update `UserOutput` struct fields to use typed wrappers
- [ ] Task 2.2: Add private helper methods `write_to_stdout()` and `write_to_stderr()`
- [ ] Task 2.3: Update all constructors to wrap writers in typed newtype
- [ ] Task 2.4: Update `write()` method to use typed writer helpers
- [ ] Task 2.5: Verify all existing public methods compile and work correctly

### Phase 3: Update Tests (30 minutes)

- [ ] Task 3.1: Update test infrastructure to work with newtype wrappers
- [ ] Task 3.2: Add tests for type-safe channel routing
- [ ] Task 3.3: Verify all existing tests pass without modification
- [ ] Task 3.4: Add test cases demonstrating compile-time safety benefits

### Phase 4: Documentation and Quality (30 minutes)

- [ ] Task 4.1: Update module documentation to mention type-safe routing
- [ ] Task 4.2: Add code examples showing type safety benefits
- [ ] Task 4.3: Run pre-commit checks and fix any issues
- [ ] Task 4.4: Verify documentation builds correctly

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All tests pass: `cargo test`
- [ ] Documentation builds: `cargo doc --no-deps`

**Task-Specific Criteria**:

- [ ] `StdoutWriter` and `StderrWriter` newtype wrappers are implemented
- [ ] Both wrappers have `new()` and `write_line()` methods
- [ ] `UserOutput` struct uses typed wrappers instead of raw `Box<dyn Write>`
- [ ] All constructors wrap raw writers in typed newtypes
- [ ] Private helper methods `write_to_stdout()` and `write_to_stderr()` exist
- [ ] The `write()` method uses typed helpers instead of direct writer access
- [ ] All existing tests pass without modification
- [ ] New tests demonstrate compile-time safety benefits
- [ ] Module documentation is updated to reflect type-safe routing
- [ ] Code examples show the benefits of the newtype pattern
- [ ] No performance regression (zero-cost abstraction)
- [ ] IDE autocomplete shows channel-specific methods

## Benefits

✅ **Compile-Time Safety**: Type system prevents accidental channel confusion
✅ **Self-Documenting Code**: Method names explicitly show which channel is used
✅ **Better IDE Support**: Autocomplete and navigation work better with explicit types
✅ **Prevents Channel Swaps**: Impossible to accidentally write to wrong channel
✅ **Zero-Cost Abstraction**: No runtime overhead with newtype pattern
✅ **Improved Maintainability**: Type errors caught at compile time instead of runtime

## Related Documentation

- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Error Handling Guide](../contributing/error-handling.md) - Error handling patterns
- [User Output Architecture Improvements Plan](../refactors/plans/user-output-architecture-improvements.md) - Complete refactoring plan
- [Development Principles](../development-principles.md) - Core development principles

## Notes

### Why Newtype Pattern?

The newtype pattern is a zero-cost abstraction in Rust - the wrapper types have the same memory layout and performance characteristics as the wrapped type, but provide compile-time type safety.

### Design Decision: Private Wrappers

The newtype wrappers (`StdoutWriter` and `StderrWriter`) are intentionally kept private as implementation details. The public API continues to accept `Box<dyn Write + Send + Sync>` for maximum flexibility, and the wrapping happens internally.

This allows:

- Existing test code to continue working without modification
- Users to inject any writer implementation without knowing about the wrappers
- Internal refactoring without breaking the public API

### Compatibility

This refactoring maintains full backward compatibility:

- All public methods have the same signatures
- All existing tests continue to pass
- No changes required in calling code
- The type safety improvements are internal implementation details

### Future Extensions

This type-safe foundation makes future improvements easier:

- Adding buffering control becomes trivial with typed wrappers
- Per-channel configuration (colors, formatting) is straightforward
- Mock writers for testing are easier to implement and use
