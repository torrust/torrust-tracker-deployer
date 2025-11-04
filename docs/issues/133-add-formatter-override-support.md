# Add Formatter Override Support for Alternative Output Formats

**Issue**: [#133](https://github.com/torrust/torrust-tracker-deployer/issues/133)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**:

- [Refactoring Plan - Proposal 4](../refactors/plans/user-output-architecture-improvements.md#proposal-4-add-alternative-formatters-optional-enhancement)
- [User Output Module](../../src/presentation/user_output.rs)

## Overview

Add optional `FormatterOverride` trait support to enable alternative output formats (JSON, structured logs, colored output) without modifying individual message types. This enhancement preserves the trait-based message design (Proposal 3) while allowing format transformation for machine-readable outputs.

**Note**: This is an **optional enhancement** that builds on the existing trait-based design. The current implementation using `Theme` and `OutputMessage` trait already provides good separation and flexibility. This proposal is only needed if we require format overrides beyond what themes provide (e.g., JSON output, ANSI color codes, structured logging).

## Goals

- [ ] Define `FormatterOverride` trait for post-processing message output
- [ ] Implement `JsonFormatter` as example override
- [ ] Add optional formatter override field to `UserOutput`
- [ ] Update `write()` method to apply override when present
- [ ] Maintain backward compatibility with existing code
- [ ] Document when to use override vs. custom message types or themes

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output.rs`
**Pattern**: Trait-based Extension (Strategy Pattern)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Keep presentation layer focused on user interface concerns
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))

### Architectural Constraints

- [ ] Must not break existing `OutputMessage` trait design
- [ ] Must preserve theme-based formatting as primary mechanism
- [ ] Override should be optional (`Option<Box<dyn FormatterOverride>>`)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Testing strategy aligns with layer responsibilities

### Anti-Patterns to Avoid

- ❌ Don't make formatter override mandatory (keep it optional)
- ❌ Don't bypass theme formatting entirely (override should transform, not replace)
- ❌ Don't add business logic to formatters (they're presentation-only)
- ❌ Don't couple formatters to specific message types (keep generic)

## Specifications

### FormatterOverride Trait

Define a trait for optional post-processing of formatted messages:

````rust
/// Optional trait for post-processing message output
///
/// This allows transforming the standard message format without
/// modifying individual message types. Use sparingly - prefer
/// extending the message trait or using themes for most cases.
///
/// # When to Use
///
/// - **Machine-readable formats**: JSON, XML, structured logs
/// - **Additional decoration**: ANSI colors, markup codes
/// - **Output wrapping**: Adding metadata, timestamps, process info
///
/// # When NOT to Use
///
/// - **Symbol changes**: Use `Theme` instead
/// - **New message types**: Implement `OutputMessage` trait instead
/// - **Channel routing changes**: Define in message type's `channel()` method
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::{FormatterOverride, OutputMessage};
///
/// struct JsonFormatter;
///
/// impl FormatterOverride for JsonFormatter {
///     fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String {
///         // Transform to JSON representation
///         format!(r#"{{"content": "{}"}}"#, formatted.trim())
///     }
/// }
/// ```
pub trait FormatterOverride: Send + Sync {
    /// Transform formatted message output
    ///
    /// This method receives the already-formatted message (with theme applied)
    /// and the original message object for context. It should return the
    /// transformed output.
    ///
    /// # Arguments
    ///
    /// * `formatted` - The message already formatted with theme
    /// * `message` - The original message object (for metadata/context)
    ///
    /// # Returns
    ///
    /// The transformed message string
    fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String;
}
````

### JsonFormatter Implementation

Provide a concrete example formatter for JSON output:

````rust
/// JSON formatter for machine-readable output
///
/// Transforms messages into JSON objects with metadata including:
/// - Message type (for programmatic filtering)
/// - Channel (stdout/stderr)
/// - Content (the formatted message)
/// - Timestamp (ISO 8601 format)
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::{JsonFormatter, UserOutput, VerbosityLevel};
///
/// let formatter = JsonFormatter;
/// let mut output = UserOutput::with_formatter_override(
///     VerbosityLevel::Normal,
///     Box::new(formatter)
/// );
///
/// output.progress("Starting process");
/// // Output: {"type":"ProgressMessage","channel":"Stderr","content":"⏳ Starting process","timestamp":"2025-11-04T12:34:56Z"}
/// ```
pub struct JsonFormatter;

impl FormatterOverride for JsonFormatter {
    fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String {
        use std::any::type_name_of_val;

        // Extract type name (e.g., "torrust_tracker_deployer_lib::presentation::user_output::ProgressMessage")
        let full_type_name = type_name_of_val(message);

        // Get just the struct name (e.g., "ProgressMessage")
        let type_name = full_type_name
            .split("::")
            .last()
            .unwrap_or(full_type_name);

        serde_json::json!({
            "type": type_name,
            "channel": format!("{:?}", message.channel()),
            "content": formatted.trim(), // Remove trailing newlines for cleaner JSON
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }).to_string()
    }
}
````

### UserOutput Integration

Add formatter override field and update methods:

````rust
pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
    formatter_override: Option<Box<dyn FormatterOverride>>, // 👈 NEW FIELD
}

impl UserOutput {
    // Existing constructors remain unchanged for backward compatibility

    /// Create `UserOutput` with an optional formatter override
    ///
    /// This allows applying custom formatting (e.g., JSON, colored output)
    /// on top of the theme-based formatting.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::{
    ///     UserOutput, VerbosityLevel, JsonFormatter
    /// };
    ///
    /// let mut output = UserOutput::with_formatter_override(
    ///     VerbosityLevel::Normal,
    ///     Box::new(JsonFormatter),
    /// );
    ///
    /// output.progress("Processing");
    /// // Output: {"type":"ProgressMessage","channel":"Stderr","content":"⏳ Processing","timestamp":"..."}
    /// ```
    #[must_use]
    pub fn with_formatter_override(
        verbosity: VerbosityLevel,
        formatter_override: Box<dyn FormatterOverride>,
    ) -> Self {
        Self {
            theme: Theme::default(),
            verbosity_filter: VerbosityFilter::new(verbosity),
            stdout_writer: Box::new(std::io::stdout()),
            stderr_writer: Box::new(std::io::stderr()),
            formatter_override: Some(formatter_override),
        }
    }

    /// Create `UserOutput` with theme and optional formatter override
    ///
    /// Combines theme selection with optional formatter override.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::user_output::{
    ///     UserOutput, VerbosityLevel, Theme, JsonFormatter
    /// };
    ///
    /// let mut output = UserOutput::with_theme_and_formatter(
    ///     VerbosityLevel::Normal,
    ///     Theme::plain(),
    ///     Some(Box::new(JsonFormatter)),
    /// );
    /// ```
    #[must_use]
    pub fn with_theme_and_formatter(
        verbosity: VerbosityLevel,
        theme: Theme,
        formatter_override: Option<Box<dyn FormatterOverride>>,
    ) -> Self {
        Self {
            theme,
            verbosity_filter: VerbosityFilter::new(verbosity),
            stdout_writer: Box::new(std::io::stdout()),
            stderr_writer: Box::new(std::io::stderr()),
            formatter_override,
        }
    }

    /// Update write() method to apply formatter override
    pub fn write(&mut self, message: &dyn OutputMessage) {
        if !self
            .verbosity_filter
            .should_show(message.required_verbosity())
        {
            return;
        }

        let mut formatted = message.format(&self.theme);

        // Apply optional format override
        if let Some(override_formatter) = &self.formatter_override {
            formatted = override_formatter.transform(&formatted, message);
        }

        let writer = match message.channel() {
            Channel::Stdout => &mut self.stdout_writer,
            Channel::Stderr => &mut self.stderr_writer,
        };

        write!(writer, "{formatted}").ok();
    }
}
````

**Note**: All existing constructors (`new()`, `with_theme()`, `with_writers()`, etc.) remain unchanged and will set `formatter_override: None` for backward compatibility.

## Implementation Plan

### Phase 1: Core Trait and Integration (2-3 hours)

- [ ] Define `FormatterOverride` trait with comprehensive documentation
- [ ] Add `formatter_override: Option<Box<dyn FormatterOverride>>` field to `UserOutput`
- [ ] Update `write()` method to apply override when present
- [ ] Add new constructors: `with_formatter_override()` and `with_theme_and_formatter()`
- [ ] Verify backward compatibility - all existing constructors work unchanged
- [ ] Update module-level documentation to explain override usage

### Phase 2: Example Implementation (1-2 hours)

- [ ] Add `serde_json` and `chrono` dependencies to `Cargo.toml` (if not already present)
- [ ] Implement `JsonFormatter` struct with `FormatterOverride` trait
- [ ] Add helper function to extract clean type name from `type_name_of_val`
- [ ] Document JSON output format and use cases

### Phase 3: Testing (2-3 hours)

- [ ] Add unit tests for `FormatterOverride` trait behavior
- [ ] Test `JsonFormatter` output structure and content
- [ ] Test formatter override with different message types
- [ ] Test that override works correctly with different themes
- [ ] Test that `None` override behaves identically to current implementation
- [ ] Verify all existing tests still pass

### Phase 4: Documentation (1 hour)

- [ ] Document when to use formatter override vs. themes vs. custom message types
- [ ] Add examples to module documentation
- [ ] Update rustdoc with formatter override patterns
- [ ] Document JSON output format specification

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

- [ ] `FormatterOverride` trait is properly documented with use cases and anti-patterns
- [ ] `JsonFormatter` produces valid JSON with all required fields
- [ ] `write()` method correctly applies override when present
- [ ] All existing constructors remain unchanged and working
- [ ] New constructors follow naming conventions and patterns
- [ ] Backward compatibility: code without formatter override behaves identically
- [ ] Unit tests cover:
  - [ ] Formatter override application
  - [ ] JSON formatter output structure
  - [ ] Different message types with override
  - [ ] Override with different themes
  - [ ] `None` override (no transformation)
- [ ] Documentation clearly explains:
  - [ ] When to use formatter override
  - [ ] When NOT to use it (prefer themes or message types)
  - [ ] JSON output format specification
  - [ ] Example usage patterns

**Code Quality**:

- [ ] Error handling follows project conventions ([docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Module organization follows conventions ([docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Tests follow testing conventions ([docs/contributing/testing/](../contributing/testing/))
- [ ] Code is well-documented with rustdoc
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt

## Related Documentation

- [User Output Architecture Improvements Plan](../refactors/plans/user-output-architecture-improvements.md)
- [Development Principles](../development-principles.md) - Testability and maintainability guidelines
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Testing Conventions](../contributing/testing/) - Testing best practices
- [Error Handling Guide](../contributing/error-handling.md) - Error handling patterns

## Notes

### Design Rationale

**Why Optional?**: The formatter override is optional because:

- Theme-based formatting handles 90% of use cases
- Most users don't need machine-readable formats
- Adding complexity only when needed (YAGNI principle)
- Maintains backward compatibility

**Why Transform, Not Replace?**: The override transforms already-formatted messages rather than replacing the format entirely because:

- Preserves theme-based formatting as primary mechanism
- Allows override to add metadata/wrapping without reimplementing formatting logic
- Keeps message types simple and focused
- Supports composition (theme → message format → override transform)

**Why Not Just Use Themes?**: Themes control symbols/appearance, while formatter overrides handle:

- Structural transformations (text → JSON)
- Metadata addition (timestamps, types)
- Encoding changes (plain text → ANSI codes)

### Dependencies

This implementation requires:

- `serde_json` - For JSON serialization in `JsonFormatter`
- `chrono` - For RFC3339 timestamps in JSON output

Both should already be in the project dependencies. Verify with:

```bash
grep -E "serde_json|chrono" Cargo.toml
```

### Future Extensions

Possible future formatters (not in this proposal):

- `AnsiColorFormatter` - Adds terminal color codes
- `StructuredLogFormatter` - Formats as structured log entries
- `MarkdownFormatter` - Wraps output in markdown code blocks
- `XmlFormatter` - XML output for legacy systems

### Testing Notes

When testing JSON output:

- Timestamps will vary - use pattern matching, not exact comparison
- Type names include full module path - extract just the struct name
- Content may have trailing newlines from message formatting - trim them
- JSON should be valid and parseable with `serde_json::from_str()`
