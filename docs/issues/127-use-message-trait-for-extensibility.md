# Use Message Trait for Extensibility

**Issue**: [#127](https://github.com/torrust/torrust-tracker-deployer/issues/127)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**:

- Refactoring Plan: [docs/refactors/plans/user-output-architecture-improvements.md](../refactors/plans/user-output-architecture-improvements.md)
- Proposal #2 (Dependency): Add Theme/Configuration Support ([#124](https://github.com/torrust/torrust-tracker-deployer/issues/124))

## Overview

This task implements a trait-based message system for the `UserOutput` module to achieve true extensibility following the Open/Closed Principle. Currently, adding new output types requires modifying the `UserOutput` struct with new methods. This refactoring allows new message types to be added without modifying existing code by having each message type encapsulate its own formatting, verbosity requirements, and channel routing logic.

**Current State**: The codebase already has `Theme` support (Proposal #2, Issue #124) and `VerbosityFilter` (Proposal #0, Issue #103) implemented. The `UserOutput` struct currently uses:

- `theme: Theme` - for accessing symbols via `self.theme.progress_symbol()`, etc.
- `verbosity_filter: VerbosityFilter` - for checking visibility via `self.verbosity_filter.should_show_progress()`, etc.
- Direct formatting in each method (e.g., `format!("{} {message}", self.theme.progress_symbol())`)

This proposal builds on these foundations to extract message types into trait implementations.

## Goals

- [ ] Define `OutputMessage` trait for extensible message types
- [ ] Create concrete message type implementations (Progress, Success, Error, Result, Steps)
- [ ] Refactor `UserOutput` to use trait-based message dispatch
- [ ] Add convenience methods for common message types
- [ ] Achieve zero modifications to `UserOutput` when adding new message types
- [ ] Maintain backward compatibility with existing API
- [ ] Ensure all tests pass with new architecture

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output.rs`
**Pattern**: Trait-based polymorphism with concrete message types

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Keep presentation logic in presentation layer (no domain concerns)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Message types must be self-contained (formatting + verbosity + routing)
- [ ] No business logic in message implementations (presentation only)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Testing strategy covers trait implementations independently

### Anti-Patterns to Avoid

- ❌ Enum-based messages with centralized pattern matching (violates Open/Closed Principle)
- ❌ Mixing business logic with message formatting
- ❌ Tightly coupling message types to specific themes or writers
- ❌ Message types depending on each other

## Specifications

### OutputMessage Trait Definition

Define the core trait that all message types implement:

```rust
/// Trait for output messages
///
/// Each message type implements this trait to define its own:
/// - Formatting logic (how it appears to users)
/// - Verbosity requirements (when it should be shown)
/// - Channel routing (stdout vs stderr)
pub trait OutputMessage {
    /// Format this message using the given theme
    fn format(&self, theme: &Theme) -> String;

    /// Get the minimum verbosity level required to show this message
    fn required_verbosity(&self) -> VerbosityLevel;

    /// Get the output channel for this message
    fn channel(&self) -> Channel;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Stdout,
    Stderr,
}
```

### Concrete Message Type Implementations

Implement concrete message types that encapsulate their own behavior:

#### ProgressMessage

```rust
/// Progress message (e.g., "Loading data...")
pub struct ProgressMessage {
    pub text: String,
}

impl OutputMessage for ProgressMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}", theme.progress_symbol, self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }
}
```

#### SuccessMessage

```rust
/// Success message (e.g., "Operation completed")
pub struct SuccessMessage {
    pub text: String,
}

impl OutputMessage for SuccessMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}", theme.success_symbol, self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }
}
```

#### ErrorMessage

```rust
/// Error message (always shown)
pub struct ErrorMessage {
    pub text: String,
}

impl OutputMessage for ErrorMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}", theme.error_symbol, self.text)
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Quiet  // Always shown
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }
}
```

#### ResultMessage

```rust
/// Result message (goes to stdout, no symbol)
pub struct ResultMessage {
    pub text: String,
}

impl OutputMessage for ResultMessage {
    fn format(&self, theme: &Theme) -> String {
        self.text.clone()
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Quiet
    }

    fn channel(&self) -> Channel {
        Channel::Stdout
    }
}
```

#### StepsMessage

```rust
/// Multi-line steps message
pub struct StepsMessage {
    pub title: String,
    pub items: Vec<String>,
}

impl OutputMessage for StepsMessage {
    fn format(&self, _theme: &Theme) -> String {
        let mut output = format!("{}\n", self.title);
        for (idx, step) in self.items.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", idx + 1, step));
        }
        output
    }

    fn required_verbosity(&self) -> VerbosityLevel {
        VerbosityLevel::Normal
    }

    fn channel(&self) -> Channel {
        Channel::Stderr
    }
}
```

### UserOutput Refactoring

Refactor `UserOutput` to use trait-based dispatch:

````rust
// Note: UserOutput already has these fields (from Phase 0 implementations):
// - theme: Theme (Issue #124)
// - verbosity_filter: VerbosityFilter (Issue #103)
// - stdout_writer: Box<dyn Write + Send + Sync>
// - stderr_writer: Box<dyn Write + Send + Sync>

pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    /// Write a message to the appropriate channel (NEW METHOD)
    pub fn write(&mut self, message: &dyn OutputMessage) {
        if !self.verbosity_filter.should_show(message.required_verbosity()) {
            return;
        }

        let formatted = message.format(&self.theme);
        let writer = match message.channel() {
            Channel::Stdout => &mut self.stdout_writer,
            Channel::Stderr => &mut self.stderr_writer,
        };

        writeln!(writer, "{}", formatted).ok();
    }

    // Existing convenience methods - refactor to use new write() method
    // (maintains backward compatibility with current API)
    pub fn progress(&mut self, text: &str) {
        self.write(&ProgressMessage { text: text.to_string() });
    }

    pub fn success(&mut self, text: &str) {
        self.write(&SuccessMessage { text: text.to_string() });
    }

    pub fn error(&mut self, text: &str) {
        self.write(&ErrorMessage { text: text.to_string() });
    }

    pub fn result(&mut self, text: &str) {
        self.write(&ResultMessage { text: text.to_string() });
    }

    // Note: steps() currently takes &[&str], update signature to accept owned Vec
    pub fn steps(&mut self, title: &str, items: Vec<String>) {
        self.write(&StepsMessage {
            title: title.to_string(),
            items
        });
    }
}
```## Implementation Plan

### Phase 1: Define Core Trait and Types (estimated: 2 hours)

- [ ] Define `OutputMessage` trait with methods: `format()`, `required_verbosity()`, `channel()`
- [ ] Define `Channel` enum (Stdout, Stderr)
- [ ] Add comprehensive documentation for trait and types
- [ ] Write unit tests for trait definition

### Phase 2: Implement Concrete Message Types (estimated: 3 hours)

- [ ] Implement `ProgressMessage` with trait implementation
- [ ] Implement `SuccessMessage` with trait implementation
- [ ] Implement `ErrorMessage` with trait implementation
- [ ] Implement `ResultMessage` with trait implementation
- [ ] Implement `StepsMessage` with trait implementation
- [ ] Implement `WarningMessage` with trait implementation (if exists in current code)
- [ ] Write unit tests for each message type's trait methods

### Phase 3: Refactor UserOutput (estimated: 2.5 hours)

- [ ] Add `write(&mut self, message: &dyn OutputMessage)` method to `UserOutput`
- [ ] Update `progress()` to use `self.write(&ProgressMessage { ... })`
- [ ] Update `success()` to use `self.write(&SuccessMessage { ... })`
- [ ] Update `error()` to use `self.write(&ErrorMessage { ... })`
- [ ] Update `warn()` to use `self.write(&WarningMessage { ... })`
- [ ] Update `result()` to use `self.write(&ResultMessage { ... })`
- [ ] Update `steps()` method signature and use `self.write(&StepsMessage { ... })`
- [ ] Remove direct formatting code from convenience methods
- [ ] Ensure backward compatibility with existing API (same method signatures except steps)

### Phase 4: Testing and Validation (estimated: 2 hours)

- [ ] Run existing unit tests and verify they pass
- [ ] Add new tests for trait-based dispatch
- [ ] Add tests showing extensibility (custom message type example)
- [ ] Verify verbosity filtering works correctly
- [ ] Verify channel routing works correctly
- [ ] Run integration tests
- [ ] Run E2E tests

### Phase 5: Documentation and Cleanup (estimated: 1 hour)

- [ ] Update rustdoc comments for new architecture
- [ ] Add example of creating custom message type
- [ ] Document benefits of trait-based approach
- [ ] Clean up any unused code
- [ ] Verify code follows module organization conventions

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
  - [ ] No unused dependencies (`cargo machete`)
  - [ ] All linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
  - [ ] All unit tests pass (`cargo test`)
  - [ ] Documentation builds successfully (`cargo doc`)
  - [ ] All E2E tests pass (config, provision, full suite)

**Task-Specific Criteria**:

- [ ] `OutputMessage` trait is defined with all required methods
- [ ] All existing message types implement the trait
- [ ] `UserOutput` has a generic `write()` method that accepts trait objects
- [ ] All existing convenience methods (`progress()`, `success()`, etc.) still work
- [ ] Verbosity filtering works correctly for all message types
- [ ] Channel routing (stdout vs stderr) works correctly for all message types
- [ ] Adding a new message type requires ZERO changes to `UserOutput` struct
- [ ] All existing tests pass without modification (backward compatibility)
- [ ] New tests demonstrate extensibility with custom message type example
- [ ] Code follows Open/Closed Principle (open for extension, closed for modification)

**Architecture Validation**:

- [ ] No business logic in message implementations
- [ ] Message types are self-contained and independent
- [ ] Clear separation of concerns (formatting, verbosity, routing)
- [ ] Follows DDD presentation layer patterns

## Related Documentation

- [User Output Refactoring Plan](../refactors/plans/user-output-architecture-improvements.md) - Complete refactoring context
- [Codebase Architecture](../codebase-architecture.md) - DDD layer guidelines
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Error Handling Guide](../contributing/error-handling.md) - Error handling patterns
- [Testing Conventions](../contributing/testing/) - Testing best practices

## Notes

### Rationale for Trait-Based Approach

**Why Trait vs Enum?**

We chose a trait-based approach over an enum-based approach because:

1. **Open/Closed Principle**: With traits, new message types can be added without modifying `UserOutput`. With enums, you must modify the formatter's pattern matching.

2. **Encapsulation**: Each message type owns its formatting logic. With enums, formatting is centralized and separated from data.

3. **Extensibility**: Users can define custom message types in their own modules. With enums, only predefined variants are allowed.

4. **Single Responsibility**: Each message type has one job. With enums, the formatter must know about all message types.

### Dependencies

**✅ RESOLVED**: All dependencies are now complete:

- ✅ Proposal #0 (Verbosity Filtering) - Issue #103 - `VerbosityFilter` struct is implemented
- ✅ Proposal #2 (Theme Support) - Issue #124 - `Theme` struct with emoji/plain/ascii is implemented

This proposal can now proceed without waiting for other work.

### Benefits

- ✅ **True Open/Closed Principle**: Add new message types without touching existing code
- ✅ **Encapsulation**: Each message knows how to format itself
- ✅ **Single Responsibility**: Each message type has one clear purpose
- ✅ **Testability**: Message types can be tested independently
- ✅ **Extensibility**: Custom message types can be defined outside the module
- ✅ **Maintainability**: Clear abstractions reduce coupling

### Future Enhancements

After this implementation, future developers can:

- Add new message types (e.g., `DebugMessage`, `InfoMessage`) without modifying `UserOutput`
- Create application-specific message types in their own modules
- Compose complex messages from simpler ones
- Add message type variations without breaking existing code
````
