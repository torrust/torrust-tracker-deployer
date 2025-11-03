# Add Theme/Configuration Support

**Issue**: [#124](https://github.com/torrust/torrust-tracker-deployer/issues/124)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**: [Refactoring Plan - Proposal #2](../refactors/plans/user-output-architecture-improvements.md#proposal-2-add-themeconfiguration-support)

## Overview

Extract emoji symbols from output methods into a configurable `Theme` struct. This enables support for different output styles (emoji, plain text, ASCII-only) based on environment or user preferences, making the application more accessible and CI/CD-friendly.

**Current Problem**: Emoji symbols are hardcoded in each output method, making it impossible to:

- Support plain text mode for CI/CD environments
- Change symbols globally
- Support user preferences or accessibility needs

**Proposed Solution**: Create a `Theme` struct that encapsulates all output symbols and formatting preferences, with predefined themes for common use cases.

## Goals

- [ ] Extract symbols to a configurable Theme struct
- [ ] Support multiple predefined themes (emoji, plain, ASCII)
- [ ] Enable runtime theme selection
- [ ] Maintain backward compatibility with default emoji theme
- [ ] Improve accessibility for different environments

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output.rs`
**Pattern**: Configuration/Theme Pattern

### Module Structure Requirements

- [ ] Create `Theme` struct within `user_output.rs` module
- [ ] Implement predefined theme constructors (`emoji()`, `plain()`, `ascii()`)
- [ ] Implement `Default` trait for `Theme` (emoji theme)
- [ ] Update `UserOutput` to use `Theme` for all symbol formatting
- [ ] Follow module organization conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] **No breaking changes**: Existing code using default constructor continues to work
- [ ] **Backward compatibility**: Default theme uses emoji (current behavior)
- [ ] **Extensibility**: Easy to add new themes without modifying output methods
- [ ] **Type safety**: Theme fields are strongly typed
- [ ] **Immutability**: Theme is set at construction, not modified during runtime

### Anti-Patterns to Avoid

- ❌ **Global mutable theme** - Don't use static mutable for theme configuration
- ❌ **Runtime theme switching** - Theme is set at construction, not changed afterward
- ❌ **Hardcoded symbols in methods** - All symbols must come from `Theme`

## Specifications

### Theme Struct Design

````rust
/// Output theme controlling symbols and formatting
///
/// A theme defines the visual appearance of user-facing messages through
/// configurable symbols. Themes enable consistent styling across all output
/// and support different environments (terminals, CI/CD, accessibility needs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    /// Symbol for progress messages (default: "⏳")
    progress_symbol: String,
    /// Symbol for success messages (default: "✅")
    success_symbol: String,
    /// Symbol for warning messages (default: "⚠️")
    warning_symbol: String,
    /// Symbol for error messages (default: "❌")
    error_symbol: String,
    /// Symbol for info messages (default: "ℹ️")
    info_symbol: String,
}

impl Theme {
    /// Emoji theme with Unicode symbols (default)
    ///
    /// Best for interactive terminals with good Unicode support.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let theme = Theme::emoji();
    /// assert_eq!(theme.progress_symbol(), "⏳");
    /// ```
    pub fn emoji() -> Self {
        Self {
            progress_symbol: "⏳".to_string(),
            success_symbol: "✅".to_string(),
            warning_symbol: "⚠️".to_string(),
            error_symbol: "❌".to_string(),
            info_symbol: "ℹ️".to_string(),
        }
    }

    /// Plain text theme for CI/CD environments
    ///
    /// Uses text labels like [INFO], [OK], [WARN], [ERROR] that work
    /// in any environment without Unicode support.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let theme = Theme::plain();
    /// assert_eq!(theme.progress_symbol(), "[INFO]");
    /// assert_eq!(theme.success_symbol(), "[OK]");
    /// ```
    pub fn plain() -> Self {
        Self {
            progress_symbol: "[INFO]".to_string(),
            success_symbol: "[OK]".to_string(),
            warning_symbol: "[WARN]".to_string(),
            error_symbol: "[ERROR]".to_string(),
            info_symbol: "[INFO]".to_string(),
        }
    }

    /// ASCII-only theme using basic characters
    ///
    /// Uses simple ASCII characters that work on any terminal.
    /// Good for environments with limited character set support.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let theme = Theme::ascii();
    /// assert_eq!(theme.success_symbol(), "[+]");
    /// ```
    pub fn ascii() -> Self {
        Self {
            progress_symbol: "=>".to_string(),
            success_symbol: "[+]".to_string(),
            warning_symbol: "[!]".to_string(),
            error_symbol: "[x]".to_string(),
            info_symbol: "[i]".to_string(),
        }
    }

    // Accessor methods for each symbol
    pub fn progress_symbol(&self) -> &str {
        &self.progress_symbol
    }

    pub fn success_symbol(&self) -> &str {
        &self.success_symbol
    }

    pub fn warning_symbol(&self) -> &str {
        &self.warning_symbol
    }

    pub fn error_symbol(&self) -> &str {
        &self.error_symbol
    }

    pub fn info_symbol(&self) -> &str {
        &self.info_symbol
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::emoji()
    }
}
````

### UserOutput Integration

```rust
pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    /// Create UserOutput with default theme (emoji)
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self::with_theme(verbosity, Theme::default())
    }

    /// Create UserOutput with a specific theme
    pub fn with_theme(verbosity: VerbosityLevel, theme: Theme) -> Self {
        Self::with_theme_and_writers(
            verbosity,
            theme,
            Box::new(std::io::stdout()),
            Box::new(std::io::stderr()),
        )
    }

    /// Create UserOutput with theme and custom writers (for testing)
    pub fn with_theme_and_writers(
        verbosity: VerbosityLevel,
        theme: Theme,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>,
    ) -> Self {
        Self {
            theme,
            verbosity_filter: VerbosityFilter::new(verbosity),
            stdout_writer,
            stderr_writer,
        }
    }

    // Update output methods to use theme
    pub fn progress(&mut self, message: &str) {
        if self.verbosity_filter.should_show_progress() {
            writeln!(
                self.stderr_writer,
                "{} {message}",
                self.theme.progress_symbol()
            )
            .ok();
        }
    }

    pub fn success(&mut self, message: &str) {
        if self.verbosity_filter.should_show_normal() {
            writeln!(
                self.stderr_writer,
                "{} {message}",
                self.theme.success_symbol()
            )
            .ok();
        }
    }

    pub fn error(&mut self, message: &str) {
        writeln!(
            self.stderr_writer,
            "{} {message}",
            self.theme.error_symbol()
        )
        .ok();
    }

    pub fn warning(&mut self, message: &str) {
        if self.verbosity_filter.should_show_normal() {
            writeln!(
                self.stderr_writer,
                "{} {message}",
                self.theme.warning_symbol()
            )
            .ok();
        }
    }

    pub fn info(&mut self, message: &str) {
        if self.verbosity_filter.should_show_normal() {
            writeln!(
                self.stderr_writer,
                "{} {message}",
                self.theme.info_symbol()
            )
            .ok();
        }
    }
}
```

### Usage Examples

```rust
// Default emoji theme (backward compatible)
let mut output = UserOutput::new(VerbosityLevel::Normal);
output.success("Deployment complete");
// Prints: ✅ Deployment complete

// Plain text theme for CI/CD
let mut output = UserOutput::with_theme(
    VerbosityLevel::Normal,
    Theme::plain()
);
output.success("Deployment complete");
// Prints: [OK] Deployment complete

// ASCII theme for limited terminals
let mut output = UserOutput::with_theme(
    VerbosityLevel::Normal,
    Theme::ascii()
);
output.success("Deployment complete");
// Prints: [+] Deployment complete
```

## Implementation Plan

### Phase 1: Create Theme Struct (1-2 hours)

- [ ] Create `Theme` struct with symbol fields
- [ ] Implement `Theme::emoji()` with current emoji symbols
- [ ] Implement `Theme::plain()` with text labels
- [ ] Implement `Theme::ascii()` with ASCII characters
- [ ] Implement accessor methods for all symbols
- [ ] Implement `Default` trait returning emoji theme
- [ ] Add comprehensive documentation with examples

### Phase 2: Add Unit Tests for Theme (1 hour)

- [ ] Test `Theme::emoji()` creates correct symbols
- [ ] Test `Theme::plain()` creates text labels
- [ ] Test `Theme::ascii()` creates ASCII characters
- [ ] Test `Default::default()` returns emoji theme
- [ ] Test accessor methods return correct values
- [ ] Test theme cloning and equality

### Phase 3: Integrate Theme into UserOutput (2-3 hours)

- [ ] Add `theme: Theme` field to `UserOutput` struct
- [ ] Update `UserOutput::new()` to use default theme
- [ ] Create `UserOutput::with_theme()` constructor
- [ ] Update `UserOutput::with_theme_and_writers()` to accept theme parameter
- [ ] Update all output methods to use `self.theme.X_symbol()`
- [ ] Remove hardcoded symbols from all methods
- [ ] Verify no hardcoded emoji symbols remain in methods

### Phase 4: Update Tests (2-3 hours)

- [ ] Update test infrastructure to support theme injection
- [ ] Create `TestUserOutput::with_theme()` helper method
- [ ] Add tests for each theme variant
- [ ] Test that default theme uses emoji
- [ ] Test that plain theme uses text labels
- [ ] Test that ASCII theme uses ASCII characters
- [ ] Verify all existing tests pass without modification

### Phase 5: Documentation and Verification (1 hour)

- [ ] Add module documentation explaining theme system
- [ ] Add usage examples in documentation
- [ ] Document each predefined theme and its use case
- [ ] Run full test suite: `cargo test`
- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`
- [ ] Update user guide if needed

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Theme Implementation Checks**:

- [ ] `Theme` struct exists with all symbol fields
- [ ] `Theme::emoji()` returns emoji symbols
- [ ] `Theme::plain()` returns text labels
- [ ] `Theme::ascii()` returns ASCII characters
- [ ] `Default` trait implementation returns emoji theme
- [ ] All accessor methods implemented

**Integration Checks**:

- [ ] `UserOutput` has `theme: Theme` field
- [ ] `UserOutput::new()` uses default emoji theme (backward compatible)
- [ ] `UserOutput::with_theme()` constructor exists
- [ ] `UserOutput::with_theme_and_writers()` accepts theme parameter
- [ ] All output methods use `self.theme.X_symbol()` instead of hardcoded symbols
- [ ] No hardcoded emoji or symbol strings remain in output methods

**Testing Checks**:

- [ ] Theme unit tests cover all predefined themes
- [ ] Tests verify correct symbol usage for each theme
- [ ] Test infrastructure supports theme injection
- [ ] All existing tests pass without modification
- [ ] New tests added for theme functionality

**Backward Compatibility Checks**:

- [ ] Default `UserOutput::new()` behavior unchanged (uses emoji)
- [ ] No breaking changes to public API
- [ ] Existing code continues to work without modifications

**Documentation Checks**:

- [ ] Theme struct and methods fully documented
- [ ] Usage examples provided for each theme
- [ ] Module documentation explains theme system
- [ ] Each theme variant documented with use case

## Related Documentation

- [Development Principles](../development-principles.md) - User friendliness and accessibility
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Refactoring Plan](../refactors/plans/user-output-architecture-improvements.md) - Complete refactoring plan

## Notes

### Future Enhancements

This proposal enables future enhancements like:

- **Environment detection**: Auto-select plain theme in CI/CD environments
- **Configuration file support**: Allow users to define custom themes
- **Color support**: Extend themes with ANSI color codes
- **Localization**: Support different symbol sets for different locales

### Design Decisions

- **Immutable themes**: Theme is set at construction and not changed during runtime
- **String-based symbols**: Using `String` instead of `&'static str` allows future custom themes
- **Three predefined themes**: Cover the most common use cases (interactive, CI/CD, limited terminals)
- **Default to emoji**: Maintains current behavior for backward compatibility
