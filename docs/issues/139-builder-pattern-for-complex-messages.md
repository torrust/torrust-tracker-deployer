# Builder Pattern for Complex Messages

**Issue**: [#139](https://github.com/torrust/torrust-tracker-deployer/issues/139)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**:

- Refactoring Plan: [docs/refactors/plans/user-output-architecture-improvements.md](../refactors/plans/user-output-architecture-improvements.md)
- Proposal #3 (Dependency): Use Message Trait for Extensibility ([#127](https://github.com/torrust/torrust-tracker-deployer/issues/127))

## Overview

This task adds optional builder patterns for complex message types in the `UserOutput` module. Currently, complex messages like `StepsMessage` and `InfoBlockMessage` take multiple parameters. This proposal introduces builders that provide a fluent API for constructing complex messages while maintaining backward compatibility with simple direct construction.

**Current State**: The codebase has implemented the `OutputMessage` trait (Proposal #3, Issue #127) with various message types including:

- Simple messages: `ProgressMessage`, `SuccessMessage`, `ErrorMessage`, `WarningMessage`
- Complex messages: `StepsMessage`, `InfoBlockMessage`, `DataMessage`

Complex messages currently use direct struct construction, which works but doesn't scale well as optional features (indentation, bullets, colors) are added.

## Goals

- [ ] Create builder pattern for `StepsMessage` with fluent API
- [ ] Create builder pattern for `InfoBlockMessage` with fluent API
- [ ] Maintain backward compatibility with simple constructors
- [ ] Enable future extensibility with optional parameters
- [ ] Provide ergonomic API for common use cases
- [ ] Document when to use builders vs simple constructors

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output.rs`
**Pattern**: Builder pattern with fluent API

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Keep presentation logic in presentation layer (no domain concerns)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Builders must be optional - keep simple constructors
- [ ] Builders must integrate with existing `OutputMessage` trait
- [ ] Must not break existing code using direct construction
- [ ] Testing strategy covers both builder and direct construction paths

### Anti-Patterns to Avoid

- ❌ Making builders mandatory (breaks existing code)
- ❌ Complex builder hierarchies (keep it simple)
- ❌ Builders for simple message types (overkill)
- ❌ Mutating builders (prefer consuming builder pattern)

## Specifications

### StepsMessageBuilder

Create a builder for `StepsMessage` with a fluent API:

````rust
/// Builder for multi-line step instructions
///
/// Provides a fluent API for constructing step messages with optional
/// customization. Use this for complex cases; simple cases can use
/// `StepsMessage::new()` directly.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
///
/// let message = StepsMessage::builder("Next steps:")
///     .add("Edit configuration")
///     .add("Review settings")
///     .add("Deploy changes")
///     .build();
/// ```
pub struct StepsMessageBuilder {
    title: String,
    items: Vec<String>,
    // Future: indentation, bullet style, etc.
}

impl StepsMessageBuilder {
    /// Create a new builder with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    /// Add a step to the list (consuming self)
    pub fn add(mut self, step: impl Into<String>) -> Self {
        self.items.push(step.into());
        self
    }

    /// Build the final `StepsMessage`
    pub fn build(self) -> StepsMessage {
        StepsMessage {
            title: self.title,
            items: self.items,
        }
    }
}

impl StepsMessage {
    /// Convenience constructor for simple cases
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
    ///
    /// let msg = StepsMessage::new("Next steps:", vec![
    ///     "Edit config".to_string(),
    ///     "Run tests".to_string(),
    /// ]);
    /// ```
    pub fn new(title: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }

    /// Builder for complex cases
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::StepsMessage;
    ///
    /// let msg = StepsMessage::builder("Next steps:")
    ///     .add("Edit configuration")
    ///     .add("Review settings")
    ///     .build();
    /// ```
    pub fn builder(title: impl Into<String>) -> StepsMessageBuilder {
        StepsMessageBuilder::new(title)
    }
}
````

### InfoBlockMessageBuilder

Create a builder for `InfoBlockMessage` with a fluent API:

````rust
/// Builder for informational block messages
///
/// Provides a fluent API for constructing info blocks with multiple lines.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::InfoBlockMessage;
///
/// let message = InfoBlockMessage::builder("Environment Details")
///     .add_line("Name: production")
///     .add_line("Status: running")
///     .add_line("Uptime: 24 hours")
///     .build();
/// ```
pub struct InfoBlockMessageBuilder {
    title: String,
    lines: Vec<String>,
}

impl InfoBlockMessageBuilder {
    /// Create a new builder with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
        }
    }

    /// Add a line to the info block (consuming self)
    pub fn add_line(mut self, line: impl Into<String>) -> Self {
        self.lines.push(line.into());
        self
    }

    /// Build the final `InfoBlockMessage`
    pub fn build(self) -> InfoBlockMessage {
        InfoBlockMessage {
            title: self.title,
            lines: self.lines,
        }
    }
}

impl InfoBlockMessage {
    /// Convenience constructor for simple cases
    pub fn new(title: impl Into<String>, lines: Vec<String>) -> Self {
        Self {
            title: title.into(),
            lines,
        }
    }

    /// Builder for complex cases
    pub fn builder(title: impl Into<String>) -> InfoBlockMessageBuilder {
        InfoBlockMessageBuilder::new(title)
    }
}
````

### Testing Strategy

Add comprehensive tests for both builder and direct construction:

```rust
#[cfg(test)]
mod builder_tests {
    use super::*;

    #[test]
    fn it_should_build_steps_with_fluent_api() {
        let message = StepsMessage::builder("Title")
            .add("Step 1")
            .add("Step 2")
            .add("Step 3")
            .build();

        assert_eq!(message.title, "Title");
        assert_eq!(message.items, vec!["Step 1", "Step 2", "Step 3"]);
    }

    #[test]
    fn it_should_create_simple_steps_directly() {
        let message = StepsMessage::new("Title", vec![
            "Step 1".to_string(),
            "Step 2".to_string(),
        ]);

        assert_eq!(message.title, "Title");
        assert_eq!(message.items, vec!["Step 1", "Step 2"]);
    }

    #[test]
    fn it_should_build_empty_steps() {
        let message = StepsMessage::builder("Title").build();

        assert_eq!(message.title, "Title");
        assert!(message.items.is_empty());
    }

    #[test]
    fn it_should_build_info_block_with_fluent_api() {
        let message = InfoBlockMessage::builder("Environment")
            .add_line("Name: production")
            .add_line("Status: active")
            .build();

        assert_eq!(message.title, "Environment");
        assert_eq!(message.lines, vec!["Name: production", "Status: active"]);
    }

    #[test]
    fn it_should_format_builder_messages_correctly() {
        let theme = Theme::emoji();
        let message = StepsMessage::builder("Next steps:")
            .add("Configure")
            .add("Deploy")
            .build();

        let formatted = message.format(&theme);
        assert!(formatted.contains("Next steps:"));
        assert!(formatted.contains("1. Configure"));
        assert!(formatted.contains("2. Deploy"));
    }
}
```

## Implementation Plan

### Phase 1: StepsMessageBuilder Implementation (1 hour)

- [ ] Create `StepsMessageBuilder` struct
- [ ] Implement `new()`, `add()`, and `build()` methods
- [ ] Add `StepsMessage::builder()` convenience method
- [ ] Keep existing `StepsMessage::new()` constructor
- [ ] Add rustdoc documentation with examples

### Phase 2: InfoBlockMessageBuilder Implementation (1 hour)

- [ ] Create `InfoBlockMessageBuilder` struct
- [ ] Implement `new()`, `add_line()`, and `build()` methods
- [ ] Add `InfoBlockMessage::builder()` convenience method
- [ ] Keep existing `InfoBlockMessage::new()` constructor
- [ ] Add rustdoc documentation with examples

### Phase 3: Testing (1.5 hours)

- [ ] Add tests for fluent API usage
- [ ] Add tests for direct construction (ensure backward compatibility)
- [ ] Add tests for empty builders
- [ ] Add integration tests showing formatting with builders
- [ ] Verify all existing tests still pass

### Phase 4: Documentation and Quality (30 minutes)

- [ ] Update module documentation with builder examples
- [ ] Document when to use builders vs simple constructors
- [ ] Add usage examples in rustdoc
- [ ] Run `./scripts/pre-commit.sh` and fix any issues

**Total Estimated Time**: 4 hours

## Acceptance Criteria

### Functional Requirements

- [ ] `StepsMessageBuilder` provides fluent API for step messages
- [ ] `InfoBlockMessageBuilder` provides fluent API for info blocks
- [ ] Simple constructors remain available and unchanged
- [ ] Builders integrate seamlessly with `OutputMessage` trait
- [ ] Builder pattern is consuming (not mutating)

### API Design Requirements

- [ ] Builder API is ergonomic and intuitive
- [ ] Method names are clear and follow Rust conventions
- [ ] `builder()` and `new()` methods coexist for different use cases
- [ ] Examples demonstrate both simple and builder patterns

### Testing Requirements

- [ ] Unit tests cover fluent API construction
- [ ] Tests verify backward compatibility with simple constructors
- [ ] Tests cover edge cases (empty builders, single item, many items)
- [ ] Integration tests show builders work with `UserOutput`

### Documentation Requirements

- [ ] Each builder has comprehensive rustdoc
- [ ] Examples show when to use builders vs simple constructors
- [ ] Module documentation includes builder pattern guidance
- [ ] Code comments explain design decisions

### Quality Requirements (applies to every commit and PR)

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Code follows project conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Builder pattern follows Rust idioms
- [ ] No clippy warnings for builder implementation

**Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

## Related Documentation

- [Development Principles](../development-principles.md) - Core principles including maintainability
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Testing Conventions](../contributing/testing/) - Testing best practices
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Builder pattern guidance

## Notes

### Why Builder Pattern?

- **Future-Proof**: Easy to add optional parameters without breaking changes
- **Ergonomic**: Fluent API feels natural and reads well
- **Flexible**: Supports both simple and complex use cases
- **Maintainable**: Adding features doesn't require changing every call site

### Design Decisions

- **Consuming Builder**: Use consuming pattern (`self`, not `&mut self`) for method chaining
- **Keep Simple Constructors**: Don't force builders on simple cases
- **Optional Enhancement**: Builders are convenience, not requirement
- **No Validation in Builder**: Validation happens in `build()` or message construction

### Future Enhancements

Potential future builder features (not in this proposal):

- Indentation control for nested steps
- Custom bullet styles for step lists
- Color/emphasis options for specific lines
- Conditional inclusion of steps
- Section grouping for large step lists

### Usage Guidance

**Use Simple Constructor When:**

- You have all data upfront in a vector
- The message is straightforward with no customization
- Code is simple and readable without builder

**Use Builder When:**

- Adding items dynamically
- Want fluent, self-documenting code
- May add optional parameters in the future
- Building message in multiple steps

---

**Created**: November 4, 2025
**Status**: 📋 Not Started
**Priority**: P2 (Phase 2 - Polish & Extensions)
**Estimated Effort**: 4 hours
