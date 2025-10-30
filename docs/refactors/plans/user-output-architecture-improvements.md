# User Output Architecture Improvements

## 📋 Overview

This refactoring improves the `UserOutput` module to enhance clarity, testability, maintainability, and sustainability. The current implementation is well-documented but mixes concerns (formatting, routing, verbosity control) and lacks extensibility for different output styles and destinations.

**Target Files:**

- `src/presentation/user_output.rs`

**Scope:**

- Separate formatting concerns from output routing
- Extract and simplify verbosity filtering logic
- Introduce theme/configuration support for different output styles
- Simplify test infrastructure
- Reduce code duplication in tests
- Improve extensibility for future output types and destinations

**Alignment with Project Principles:**

- **Clarity**: Separate concerns make code easier to understand
- **Testability**: Independent components enable isolated testing
- **Maintainability**: Reduced duplication and clear abstractions
- **Sustainability**: Easy to extend with new formats and destinations

## 📊 Progress Tracking

**Total Active Proposals**: 10
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 10

### Phase Summary

- **Phase 0 - Quick Wins (High Impact, Low Effort)**: ⏳ 0/3 completed (0%)
- **Phase 1 - Strategic Improvements (High Impact, Medium Effort)**: ⏳ 0/2 completed (0%)
- **Phase 2 - Polish & Extensions (Medium Impact, Low-Medium Effort)**: ⏳ 0/5 completed (0%)

### Discarded Proposals

None at this time.

### Postponed Proposals

None - all 10 proposals are active and will be implemented.

## 🎯 Key Problems Identified

### 1. Mixed Concerns

The `UserOutput` struct combines multiple responsibilities:

- Verbosity filtering
- Output formatting (emoji symbols)
- Channel routing (stdout vs stderr)
- Writer management

This makes it harder to test each concern independently and reduces flexibility.

### 2. Scattered Formatting Logic

Emoji symbols and formatting are hardcoded across multiple methods:

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

This makes it impossible to:

- Support plain text mode for CI/CD
- Change symbols without modifying every method
- Test formatting independently

### 3. Repeated Verbosity Checks

Every output method repeats the same verbosity check pattern:

```rust
if self.verbosity >= VerbosityLevel::Normal {
    // output logic
}
```

This violates DRY principle and makes changes error-prone.

### 4. Complex Test Infrastructure

Test helper uses `Arc<Mutex<Vec<u8>>>` and custom `SharedWriter`, which is more complex than necessary:

```rust
struct SharedWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    // ...
}
```

### 5. Test Code Duplication

Many similar tests that differ only in the method called and expected symbol. This creates maintenance burden when adding new output types.

### 6. Limited Extensibility

Hard to support:

- Different output styles (plain text, colored, emoji)
- Different environments (CI/CD, interactive terminals)
- User preferences or accessibility needs
- New output message types without adding methods

## 🚀 Refactoring Phases

---

## Phase 0: Quick Wins (Highest Priority)

These proposals provide immediate benefits with minimal effort and risk. They can be implemented independently and provide foundation for later improvements.

### Proposal #0: Extract Verbosity Filtering Logic

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: None

#### Problem

Verbosity checks are duplicated across all output methods, making the code harder to maintain and test:

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

#### Proposed Solution

Create a `VerbosityFilter` struct that encapsulates verbosity logic:

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

pub struct UserOutput {
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    pub fn progress(&mut self, message: &str) {
        if self.verbosity_filter.should_show_progress() {
            writeln!(self.stderr_writer, "⏳ {message}").ok();
        }
    }
}
```

#### Rationale

- **DRY Principle**: Eliminates repeated verbosity checks
- **Testability**: Can test verbosity logic independently
- **Clarity**: Named methods like `should_show_progress()` are self-documenting
- **Extensibility**: Easy to add complex filtering rules in one place

#### Benefits

- ✅ Removes code duplication across all output methods
- ✅ Makes verbosity rules testable independently
- ✅ Self-documenting code with named filter methods
- ✅ Single source of truth for verbosity logic
- ✅ Easy to extend with more complex filtering rules

#### Implementation Checklist

- [ ] Create `VerbosityFilter` struct with `should_show()` method
- [ ] Add convenience methods: `should_show_progress()`, `should_show_errors()`, etc.
- [ ] Add unit tests for `VerbosityFilter` behavior
- [ ] Replace `verbosity: VerbosityLevel` field with `verbosity_filter: VerbosityFilter` in `UserOutput`
- [ ] Update all output methods to use `verbosity_filter.should_show_X()` instead of direct checks
- [ ] Update `UserOutput::new()` and `UserOutput::with_writers()` constructors
- [ ] Verify all existing tests still pass
- [ ] Run linter and fix any issues
- [ ] Update module documentation if needed

#### Testing Strategy

```rust
#[cfg(test)]
mod verbosity_filter_tests {
    use super::*;

    #[test]
    fn it_should_show_progress_at_normal_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(filter.should_show_progress());
    }

    #[test]
    fn it_should_not_show_progress_at_quiet_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        assert!(!filter.should_show_progress());
    }

    #[test]
    fn it_should_always_show_errors() {
        let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        assert!(filter.should_show_errors());
    }
}
```

---

### Proposal #1: Simplify Test Infrastructure

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: None

#### Problem

Current test helper uses complex `Arc<Mutex<Vec<u8>>>` with custom `SharedWriter`:

```rust
fn create_test_user_output(
    verbosity: VerbosityLevel,
) -> (
    UserOutput,
    std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
) {
    let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
    let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

    let stdout_writer = Box::new(SharedWriter(Arc::clone(&stdout_buffer)));
    let stderr_writer = Box::new(SharedWriter(Arc::clone(&stderr_buffer)));

    let output = UserOutput::with_writers(verbosity, stdout_writer, stderr_writer);

    (output, stdout_buffer, stderr_buffer)
}

struct SharedWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}
```

This is overly complex for the use case and harder for contributors to understand.

#### Proposed Solution

Create a simpler `TestWriter` that accumulates output:

```rust
#[cfg(test)]
mod test_support {
    use super::*;
    use std::io::Write;

    /// Simple writer for testing that captures all output
    pub struct TestWriter {
        buffer: Vec<u8>,
    }

    impl TestWriter {
        pub fn new() -> Self {
            Self { buffer: Vec::new() }
        }

        pub fn as_string(&self) -> String {
            String::from_utf8_lossy(&self.buffer).to_string()
        }

        pub fn clear(&mut self) {
            self.buffer.clear();
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Helper to create UserOutput with test writers
    pub fn create_test_user_output(
        verbosity: VerbosityLevel,
    ) -> (UserOutput, TestWriter, TestWriter) {
        let stdout = TestWriter::new();
        let stderr = TestWriter::new();

        // Note: We need to share the writers, so we'll use Rc<RefCell<>>
        use std::rc::Rc;
        use std::cell::RefCell;

        let stdout_rc = Rc::new(RefCell::new(stdout));
        let stderr_rc = Rc::new(RefCell::new(stderr));

        let output = UserOutput::with_writers(
            verbosity,
            Box::new(TestWriterAdapter(Rc::clone(&stdout_rc))),
            Box::new(TestWriterAdapter(Rc::clone(&stderr_rc))),
        );

        // Extract the writers for assertions
        let stdout_final = Rc::try_unwrap(stdout_rc).unwrap().into_inner();
        let stderr_final = Rc::try_unwrap(stderr_rc).unwrap().into_inner();

        (output, stdout_final, stderr_final)
    }

    struct TestWriterAdapter(Rc<RefCell<TestWriter>>);

    impl Write for TestWriterAdapter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.borrow_mut().flush()
        }
    }
}
```

Actually, even simpler - just use separate buffers:

```rust
#[cfg(test)]
mod test_support {
    use super::*;

    /// Helper to create UserOutput with in-memory buffers for testing
    pub fn create_test_output(verbosity: VerbosityLevel) -> TestUserOutput {
        TestUserOutput::new(verbosity)
    }

    /// Test wrapper that provides access to captured output
    pub struct TestUserOutput {
        output: UserOutput,
        stdout_buffer: Vec<u8>,
        stderr_buffer: Vec<u8>,
    }

    impl TestUserOutput {
        fn new(verbosity: VerbosityLevel) -> Self {
            let stdout_buffer = Vec::new();
            let stderr_buffer = Vec::new();

            let output = UserOutput::with_writers(
                verbosity,
                Box::new(std::io::Cursor::new(&mut stdout_buffer)),
                Box::new(std::io::Cursor::new(&mut stderr_buffer)),
            );

            Self {
                output,
                stdout_buffer,
                stderr_buffer,
            }
        }

        pub fn stdout(&self) -> String {
            String::from_utf8_lossy(&self.stdout_buffer).to_string()
        }

        pub fn stderr(&self) -> String {
            String::from_utf8_lossy(&self.stderr_buffer).to_string()
        }

        pub fn output(&mut self) -> &mut UserOutput {
            &mut self.output
        }
    }
}
```

#### Rationale

The simpler approach:

- Uses standard library types (`Vec<u8>`, `Cursor`)
- Easier for contributors to understand
- No need for `Arc`, `Mutex`, or custom `Write` implementations
- More maintainable test code

#### Benefits

- ✅ Simpler test setup code
- ✅ Easier for contributors to understand
- ✅ No need for complex synchronization primitives
- ✅ More maintainable test infrastructure
- ✅ Reduced cognitive load when writing tests

#### Implementation Checklist

- [ ] Create simplified `TestUserOutput` helper struct
- [ ] Implement helper methods: `stdout()`, `stderr()`, `output()`
- [ ] Update all existing tests to use new helper
- [ ] Remove old `SharedWriter` and `create_test_user_output()`
- [ ] Verify all tests pass with new infrastructure
- [ ] Run linter and fix any issues
- [ ] Update test documentation/examples

#### Testing Strategy

Verify the new test infrastructure works by running all existing tests. No new tests needed - this is internal test infrastructure improvement.

---

### Proposal #2: Add Theme/Configuration Support

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: None

#### Problem

Emoji symbols are hardcoded in each method, making it impossible to:

- Support plain text mode for CI/CD environments
- Change symbols globally
- Support user preferences or accessibility needs

```rust
pub fn progress(&mut self, message: &str) {
    writeln!(self.stderr_writer, "⏳ {message}").ok();
}
```

#### Proposed Solution

Extract symbols to a `Theme` struct:

```rust
/// Output theme controlling symbols and formatting
#[derive(Debug, Clone)]
pub struct Theme {
    progress_symbol: String,
    success_symbol: String,
    warning_symbol: String,
    error_symbol: String,
}

impl Theme {
    /// Theme with emoji symbols (default)
    pub fn emoji() -> Self {
        Self {
            progress_symbol: "⏳".to_string(),
            success_symbol: "✅".to_string(),
            warning_symbol: "⚠️".to_string(),
            error_symbol: "❌".to_string(),
        }
    }

    /// Plain text theme for CI/CD environments
    pub fn plain() -> Self {
        Self {
            progress_symbol: "[INFO]".to_string(),
            success_symbol: "[OK]".to_string(),
            warning_symbol: "[WARN]".to_string(),
            error_symbol: "[ERROR]".to_string(),
        }
    }

    /// ASCII-only symbols
    pub fn ascii() -> Self {
        Self {
            progress_symbol: "=>".to_string(),
            success_symbol: "✓".to_string(),
            warning_symbol: "!".to_string(),
            error_symbol: "✗".to_string(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::emoji()
    }
}

pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self::with_theme(verbosity, Theme::default())
    }

    pub fn with_theme(verbosity: VerbosityLevel, theme: Theme) -> Self {
        Self::with_theme_and_writers(
            verbosity,
            theme,
            Box::new(std::io::stdout()),
            Box::new(std::io::stderr()),
        )
    }

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

    pub fn progress(&mut self, message: &str) {
        if self.verbosity_filter.should_show_progress() {
            writeln!(self.stderr_writer, "{} {message}", self.theme.progress_symbol).ok();
        }
    }
}
```

#### Rationale

- **Flexibility**: Easy to switch themes based on environment
- **Accessibility**: Plain text mode for screen readers or CI/CD
- **Maintainability**: Symbols defined in one place
- **Extensibility**: Easy to add new themes (colored, custom, etc.)

#### Benefits

- ✅ Support plain text mode for CI/CD environments
- ✅ All symbols defined in one place
- ✅ Easy to add new themes without changing output methods
- ✅ Better accessibility support
- ✅ User preferences can be supported
- ✅ A/B testing of different UX approaches

#### Implementation Checklist

- [ ] Create `Theme` struct with symbol fields
- [ ] Implement `Theme::emoji()`, `Theme::plain()`, `Theme::ascii()`
- [ ] Implement `Default` for `Theme` (uses emoji)
- [ ] Add unit tests for theme creation
- [ ] Add `theme: Theme` field to `UserOutput`
- [ ] Update constructors: `new()`, `with_theme()`, `with_theme_and_writers()`
- [ ] Update all output methods to use `self.theme.X_symbol`
- [ ] Update test infrastructure to support theme injection
- [ ] Add tests for different themes
- [ ] Verify all existing tests still pass
- [ ] Run linter and fix any issues
- [ ] Update module documentation with theme examples
- [ ] Update user guide if needed

#### Testing Strategy

```rust
#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn it_should_use_emoji_symbols_by_default() {
        let theme = Theme::default();
        assert_eq!(theme.progress_symbol, "⏳");
        assert_eq!(theme.success_symbol, "✅");
    }

    #[test]
    fn it_should_support_plain_text_theme() {
        let theme = Theme::plain();
        assert_eq!(theme.progress_symbol, "[INFO]");
        assert_eq!(theme.error_symbol, "[ERROR]");
    }

    #[test]
    fn it_should_output_with_plain_theme() {
        let mut output = create_test_output_with_theme(
            VerbosityLevel::Normal,
            Theme::plain()
        );

        output.output().progress("Testing");
        assert_eq!(output.stderr(), "[INFO] Testing\n");
    }
}
```

---

## Phase 1: Strategic Improvements

These proposals provide significant architectural improvements that enhance long-term maintainability and extensibility.

### Proposal #3: Use Message Trait for Extensibility

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal #2 (Theme Support)

#### Problem

Adding new output types requires adding new methods to `UserOutput`. This violates the Open/Closed Principle - the struct should be closed for modification but open for extension.

Using an enum with pattern matching would still require modifying the formatter when adding new message types, defeating the purpose of the Open/Closed Principle.

#### Proposed Solution

Define an `OutputMessage` trait where each message type encapsulates its own behavior:

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

/// Result message (goes to stdout)
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

pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    /// Write a message to the appropriate channel
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

    // Convenience methods that create messages
    pub fn progress(&mut self, text: &str) {
        self.write(&ProgressMessage { text: text.to_string() });
    }

    pub fn success(&mut self, text: &str) {
        self.write(&SuccessMessage { text: text.to_string() });
    }

    pub fn error(&mut self, text: &str) {
        self.write(&ErrorMessage { text: text.to_string() });
    }
}
```

#### Rationale

- **True Open/Closed Principle**: Adding new message types requires NO modification to existing code
- **Encapsulation**: Each message knows how to format itself
- **Single Responsibility**: Each message type has one clear purpose
- **Extensibility**: Users can define custom message types in their own modules

#### Benefits

- ✅ **True extensibility**: Add new message types without touching `UserOutput` or any formatter
- ✅ **No giant match statements**: Each message handles its own formatting
- ✅ **Better encapsulation**: Message behavior lives with message definition
- ✅ **External extensibility**: Other crates can define custom message types
- ✅ **Cleaner codebase**: Less boilerplate, clearer separation of concerns

#### Implementation Checklist

- [ ] Define `OutputMessage` trait with three methods
- [ ] Define `Channel` enum
- [ ] Implement message types: `ProgressMessage`, `SuccessMessage`, `ErrorMessage`, `ResultMessage`
- [ ] Implement complex message types: `StepsMessage`, `InfoBlockMessage`, `WarningMessage`, `DataMessage`
- [ ] Update `UserOutput::write()` to accept `&dyn OutputMessage`
- [ ] Keep existing convenience methods as thin wrappers
- [ ] Add unit tests for each message type's formatting
- [ ] Add unit tests for verbosity and channel routing
- [ ] Verify all existing tests still pass
- [ ] Run linter and fix any issues
- [ ] Update module documentation with trait-based examples

#### Testing Strategy

```rust
#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn it_should_format_progress_with_theme() {
        let theme = Theme::emoji();
        let message = ProgressMessage { text: "Loading".to_string() };
        assert_eq!(message.format(&theme), "⏳ Loading");
    }

    #[test]
    fn it_should_route_results_to_stdout() {
        let message = ResultMessage { text: "Done".to_string() };
        assert_eq!(message.channel(), Channel::Stdout);
    }

    #[test]
    fn it_should_route_progress_to_stderr() {
        let message = ProgressMessage { text: "Loading".to_string() };
        assert_eq!(message.channel(), Channel::Stderr);
    }

    #[test]
    fn it_should_always_show_errors() {
        let message = ErrorMessage { text: "Failed".to_string() };
        assert_eq!(message.required_verbosity(), VerbosityLevel::Quiet);
    }

    #[test]
    fn it_should_format_steps_with_numbering() {
        let theme = Theme::emoji();
        let message = StepsMessage {
            title: "Next steps:".to_string(),
            items: vec!["First".to_string(), "Second".to_string()],
        };
        let formatted = message.format(&theme);
        assert_eq!(formatted, "Next steps:\n1. First\n2. Second\n");
    }
}
```

---

### Proposal #4: Add Alternative Formatters (Optional Enhancement)

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵🔵 Medium  
**Priority**: P2 (Moved to Phase 2)  
**Depends On**: Proposal #3 (Message Trait)

#### Problem

While each message can format itself (from Proposal #3), we may want to support alternative formatting strategies without modifying every message type. For example:

- JSON output for machine-readable logs
- Colored terminal output with ANSI codes
- Structured logging format
- Plain text without any symbols (beyond Theme support)

**Note**: This proposal is **optional** and can be postponed. The trait-based design from Proposal #3 already achieves good separation. This is only needed if we want format overrides without modifying message types.

#### Proposed Solution

Add an optional `FormatterOverride` trait that can wrap and transform message output:

```rust
/// Optional trait for post-processing message output
///
/// This allows transforming the standard message format without
/// modifying individual message types. Use sparingly - prefer
/// extending the message trait for most cases.
pub trait FormatterOverride {
    fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String;
}

/// JSON formatter override (example)
pub struct JsonFormatter;

impl FormatterOverride for JsonFormatter {
    fn transform(&self, formatted: &str, message: &dyn OutputMessage) -> String {
        serde_json::json!({
            "type": std::any::type_name_of_val(message),
            "channel": format!("{:?}", message.channel()),
            "content": formatted,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }).to_string()
    }
}

pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
    formatter_override: Option<Box<dyn FormatterOverride>>,
}

impl UserOutput {
    pub fn write(&mut self, message: &dyn OutputMessage) {
        if !self.verbosity_filter.should_show(message.required_verbosity()) {
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

        writeln!(writer, "{}", formatted).ok();
    }
}
```

#### Rationale

- **Optional Enhancement**: Not required for basic functionality
- **Separation Preserved**: Doesn't break the trait-based message design
- **Use Case**: Primarily for machine-readable formats (JSON, XML)
- **Theme is Still Primary**: Normal formatting uses Theme + message's own format()

#### Benefits

- ✅ Support for alternative output formats (JSON, structured logs)
- ✅ Doesn't require modifying message types
- ✅ Optional - only used when needed
- ✅ Can be added later without breaking changes

#### Implementation Checklist

- [ ] Define `FormatterOverride` trait
- [ ] Add `formatter_override: Option<Box<dyn FormatterOverride>>` to `UserOutput`
- [ ] Implement `JsonFormatter` as example
- [ ] Update `write()` to apply override if present
- [ ] Add unit tests for formatter overrides
- [ ] Document when to use override vs. custom message types
- [ ] Verify all existing tests still pass
- [ ] Run linter and fix any issues

#### Testing Strategy

```rust
#[cfg(test)]
mod formatter_override_tests {
    use super::*;

    #[test]
    fn it_should_apply_json_formatter() {
        let json_formatter = JsonFormatter;
        let message = ProgressMessage { text: "Test".to_string() };
        let theme = Theme::emoji();

        let standard = message.format(&theme);
        let json = json_formatter.transform(&standard, &message);

        assert!(json.contains("\"content\":\"⏳ Test\""));
        assert!(json.contains("\"type\":"));
    }
}
```

---

### Proposal #5: Parameterized Test Cases

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal #1 (Simplified Test Infrastructure)

#### Problem

Many tests are nearly identical, differing only in the method called and expected output:

```rust
#[test]
fn it_should_write_progress_messages_to_stderr() {
    let (mut output, stdout_buf, stderr_buf) = create_test_user_output(VerbosityLevel::Normal);
    output.progress("Testing progress message");
    let stderr_content = String::from_utf8(stderr_buf.lock().unwrap().clone()).unwrap();
    assert_eq!(stderr_content, "⏳ Testing progress message\n");
}

#[test]
fn it_should_write_success_messages_to_stderr() {
    let (mut output, stdout_buf, stderr_buf) = create_test_user_output(VerbosityLevel::Normal);
    output.success("Testing success message");
    let stderr_content = String::from_utf8(stderr_buf.lock().unwrap().clone()).unwrap();
    assert_eq!(stderr_content, "✅ Testing success message\n");
}
```

#### Proposed Solution

Use `rstest` for parameterized tests:

```rust
use rstest::rstest;

#[rstest]
#[case("progress", "⏳", VerbosityLevel::Normal, "stderr")]
#[case("success", "✅", VerbosityLevel::Normal, "stderr")]
#[case("warning", "⚠️", VerbosityLevel::Normal, "stderr")]
#[case("error", "❌", VerbosityLevel::Quiet, "stderr")]
#[case("result", "", VerbosityLevel::Normal, "stdout")]
fn it_should_write_to_correct_channel(
    #[case] method: &str,
    #[case] symbol: &str,
    #[case] min_verbosity: VerbosityLevel,
    #[case] channel: &str,
) {
    let mut output = create_test_output(min_verbosity);
    let message = "Test message";

    // Call the appropriate method
    match method {
        "progress" => output.output().progress(message),
        "success" => output.output().success(message),
        "warning" => output.output().warn(message),
        "error" => output.output().error(message),
        "result" => output.output().result(message),
        _ => panic!("Unknown method: {}", method),
    }

    // Check correct channel
    let expected = if symbol.is_empty() {
        format!("{}\n", message)
    } else {
        format!("{} {}\n", symbol, message)
    };

    match channel {
        "stdout" => {
            assert_eq!(output.stdout(), expected);
            assert_eq!(output.stderr(), "");
        }
        "stderr" => {
            assert_eq!(output.stderr(), expected);
            assert_eq!(output.stdout(), "");
        }
        _ => panic!("Unknown channel: {}", channel),
    }
}

#[rstest]
#[case(VerbosityLevel::Quiet, false)]
#[case(VerbosityLevel::Normal, true)]
#[case(VerbosityLevel::Verbose, true)]
fn it_should_respect_verbosity_for_progress(
    #[case] verbosity: VerbosityLevel,
    #[case] should_show: bool,
) {
    let mut output = create_test_output(verbosity);
    output.output().progress("Test");

    if should_show {
        assert!(!output.stderr().is_empty());
    } else {
        assert_eq!(output.stderr(), "");
    }
}
```

Add dependency to `Cargo.toml`:

```toml
[dev-dependencies]
rstest = "0.18"
```

#### Rationale

- **Less Duplication**: Single test implementation covers multiple cases
- **Easier to Extend**: Adding new cases is trivial
- **Clear Specification**: Test cases act as behavior documentation
- **Maintainability**: Changes affect one test instead of many

#### Benefits

- ✅ Significantly reduces test code duplication
- ✅ Easier to add new test cases
- ✅ Clear behavior matrix specification
- ✅ More maintainable test suite
- ✅ Better test coverage with less code

#### Implementation Checklist

- [ ] Add `rstest` to dev dependencies
- [ ] Create parameterized tests for channel routing
- [ ] Create parameterized tests for verbosity levels
- [ ] Create parameterized tests for formatting
- [ ] Remove duplicate test methods
- [ ] Verify test coverage is maintained or improved
- [ ] Run all tests and ensure they pass
- [ ] Run linter and fix any issues
- [ ] Update test documentation if needed

#### Testing Strategy

Run existing test suite to ensure coverage is maintained. The parameterized tests should cover all existing test cases with less code.

---

## Phase 2: Polish & Extensions

These proposals add additional robustness, type safety, and flexibility with minimal complexity.

### Proposal #6: Type-Safe Channel Routing

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: None

#### Problem

Channel routing (stdout vs stderr) is done with runtime checks. While this works, there's no compile-time guarantee that messages go to the correct channel.

```rust
let writer = match message.channel() {
    Channel::Stdout => &mut self.stdout_writer,
    Channel::Stderr => &mut self.stderr_writer,
};
```

#### Proposed Solution

Use newtype pattern for compile-time channel safety:

```rust
/// Stdout writer wrapper for type safety
struct StdoutWriter(Box<dyn Write + Send + Sync>);

/// Stderr writer wrapper for type safety
struct StderrWriter(Box<dyn Write + Send + Sync>);

impl StdoutWriter {
    fn write_line(&mut self, message: &str) -> std::io::Result<()> {
        writeln!(self.0, "{}", message)
    }
}

impl StderrWriter {
    fn write_line(&mut self, message: &str) -> std::io::Result<()> {
        writeln!(self.0, "{}", message)
    }
}

pub struct UserOutput {
    formatter: Box<dyn MessageFormatter>,
    verbosity_filter: VerbosityFilter,
    stdout: StdoutWriter,
    stderr: StderrWriter,
}

impl UserOutput {
    fn write_to_stdout(&mut self, formatted: &str) {
        self.stdout.write_line(formatted).ok();
    }

    fn write_to_stderr(&mut self, formatted: &str) {
        self.stderr.write_line(formatted).ok();
    }
}
```

#### Rationale

- **Compile-Time Safety**: Type system prevents accidental channel confusion
- **Self-Documenting**: Method names make channel routing explicit
- **Minimal Overhead**: Zero-cost abstraction with newtype pattern

#### Benefits

- ✅ Compile-time verification of channel routing
- ✅ More explicit and self-documenting code
- ✅ Prevents accidental channel swaps
- ✅ Better IDE support and autocomplete

#### Implementation Checklist

- [ ] Create `StdoutWriter` and `StderrWriter` newtype wrappers
- [ ] Implement `write_line()` methods
- [ ] Update `UserOutput` fields to use typed wrappers
- [ ] Create convenience methods: `write_to_stdout()`, `write_to_stderr()`
- [ ] Update all output methods to use typed writers
- [ ] Update test infrastructure
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

```rust
#[cfg(test)]
mod channel_safety_tests {
    use super::*;

    #[test]
    fn it_should_write_to_stdout_through_typed_writer() {
        let mut output = create_test_output(VerbosityLevel::Normal);
        output.output().result("Test result");
        assert!(!output.stdout().is_empty());
        assert!(output.stderr().is_empty());
    }

    #[test]
    fn it_should_write_to_stderr_through_typed_writer() {
        let mut output = create_test_output(VerbosityLevel::Normal);
        output.output().progress("Test progress");
        assert!(output.stdout().is_empty());
        assert!(!output.stderr().is_empty());
    }
}
```

---

### Proposal #7: Add Buffering Control

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: Proposal #6 (Type-Safe Channel Routing)

#### Problem

Output is written immediately without explicit buffering control. For high-volume output or specific use cases, we may want more control over when output is flushed.

#### Proposed Solution

Add explicit `flush()` method and document buffering behavior:

```rust
impl UserOutput {
    /// Flush all pending output to stdout and stderr
    ///
    /// This is typically not needed as writes are line-buffered by default,
    /// but can be useful for ensuring output appears immediately.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.0.flush()?;
        self.stderr.0.flush()?;
        Ok(())
    }
}
```

#### Rationale

- **Explicit Control**: Users can force output when needed
- **Testing**: Useful for test scenarios
- **Compatibility**: Standard pattern from `Write` trait

#### Benefits

- ✅ Explicit flush control when needed
- ✅ Better testing capabilities
- ✅ Standard Rust pattern
- ✅ Minimal code addition

#### Implementation Checklist

- [ ] Add `flush()` method to `UserOutput`
- [ ] Add documentation about buffering behavior
- [ ] Add test for flush behavior
- [ ] Update module documentation
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

```rust
#[cfg(test)]
mod buffering_tests {
    use super::*;

    #[test]
    fn it_should_flush_all_writers() {
        let mut output = create_test_output(VerbosityLevel::Normal);
        output.output().progress("Test");
        output.output().flush().expect("Flush should succeed");
        // Verify output is flushed (implementation-specific)
    }
}
```

---

### Proposal #8: Builder Pattern for Complex Messages

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposal #3 (Message Trait)

#### Problem

Complex messages like `StepsMessage` and `InfoBlockMessage` take multiple parameters. Adding optional features (indentation, bullets, colors) would make constructors unwieldy.

#### Proposed Solution

Add optional builder pattern for complex message types:

```rust
/// Builder for multi-line step instructions
pub struct StepsMessageBuilder {
    title: String,
    items: Vec<String>,
    // Future: indentation, bullet style, etc.
}

impl StepsMessageBuilder {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    pub fn add(mut self, step: impl Into<String>) -> Self {
        self.items.push(step.into());
        self
    }

    pub fn build(self) -> StepsMessage {
        StepsMessage {
            title: self.title,
            items: self.items,
        }
    }
}

impl StepsMessage {
    /// Convenience constructor for simple cases
    pub fn new(title: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }

    /// Builder for complex cases
    pub fn builder(title: impl Into<String>) -> StepsMessageBuilder {
        StepsMessageBuilder::new(title)
    }
}

// Usage examples:

// Simple case - direct construction
let msg = StepsMessage::new("Next steps:", vec![
    "Edit config".to_string(),
    "Run tests".to_string(),
]);
output.write(&msg);

// Complex case - builder pattern
let msg = StepsMessage::builder("Next steps:")
    .add("Edit configuration")
    .add("Review settings")
    .add("Deploy changes")
    .build();
output.write(&msg);
```

#### Rationale

- **Flexibility**: Easy to add optional parameters
- **Ergonomics**: Fluent API feels natural
- **Extensibility**: Can add features without breaking existing code
- **Backward Compatible**: Keep simple methods

#### Benefits

- ✅ More flexible API for complex messages
- ✅ Easy to extend with new options
- ✅ Fluent, readable syntax
- ✅ Backward compatible with simple methods

#### Implementation Checklist

- [ ] Create `StepsBuilder` struct
- [ ] Create `InfoBlockBuilder` struct
- [ ] Implement builder methods
- [ ] Add `build()` methods
- [ ] Keep existing simple methods for compatibility
- [ ] Add builder tests
- [ ] Add documentation with examples
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

```rust
#[cfg(test)]
mod builder_tests {
    use super::*;

    #[test]
    fn it_should_build_steps_with_fluent_api() {
        let message = StepsMessage::builder("Title")
            .add("Step 1")
            .add("Step 2")
            .build();

        assert_eq!(message.title, "Title");
        assert_eq!(message.items, vec!["Step 1", "Step 2"]);
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
}
```

---

### Proposal #9: Output Sink Abstraction

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: Proposal #3 (Message Trait)

#### Problem

Currently tied to `Write` trait for stdout/stderr. If we want to output to multiple destinations (file + stderr, network, telemetry), we'd need to modify `UserOutput` internals.

#### Proposed Solution

Create an `OutputSink` trait as an abstraction layer:

```rust
/// Trait for output destinations
trait OutputSink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str);
}

/// Standard sink writing to stdout/stderr
struct StandardSink {
    stdout: StdoutWriter,
    stderr: StderrWriter,
}

impl OutputSink for StandardSink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        match message.channel() {
            Channel::Stdout => self.stdout.write_line(formatted).ok(),
            Channel::Stderr => self.stderr.write_line(formatted).ok(),
        };
    }
}

/// Composite sink that writes to multiple destinations
struct CompositeSink {
    sinks: Vec<Box<dyn OutputSink>>,
}

impl OutputSink for CompositeSink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        for sink in &mut self.sinks {
            sink.write_message(message, formatted);
        }
    }
}

/// Telemetry sink (example)
struct TelemetrySink {
    client: TelemetryClient,
}

impl OutputSink for TelemetrySink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        // Send to telemetry service
        self.client.log_event(formatted, message.channel());
    }
}

pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    sink: Box<dyn OutputSink>,
}
```

#### Rationale

- **Extensibility**: Easy to add new output destinations
- **Composition**: Can combine multiple sinks
- **Testability**: Can mock sinks for testing
- **Observability**: Can add telemetry sink

#### Benefits

- ✅ Support for multiple output destinations
- ✅ Can add telemetry/observability easily
- ✅ Composite pattern for fan-out
- ✅ Better testability with mock sinks

#### Implementation Checklist

- [ ] Create `OutputSink` trait
- [ ] Implement `StandardSink`
- [ ] Implement `CompositeSink`
- [ ] Update `UserOutput` to use `Box<dyn OutputSink>`
- [ ] Update constructors
- [ ] Add sink-specific tests
- [ ] Update documentation
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

```rust
#[cfg(test)]
mod sink_tests {
    use super::*;

    #[test]
    fn it_should_write_to_standard_sink() {
        let mut sink = StandardSink::new(/* ... */);
        let message = ProgressMessage { text: "Test".to_string() };
        sink.write_message(&message, "⏳ Test");
        // Verify output appeared
    }

    #[test]
    fn it_should_write_to_composite_sink() {
        let sink1 = Box::new(StandardSink::new(/* ... */));
        let sink2 = Box::new(MockSink::new());
        let mut composite = CompositeSink::new(vec![sink1, sink2]);

        let message = ProgressMessage { text: "Test".to_string() };
        composite.write_message(&message, "⏳ Test");

        // Verify both sinks received the message
    }
}
```

---

## 📈 Timeline

- **Start Date**: October 30, 2025
- **Estimated Duration**: 3-4 weeks (with parallel development possible)
- **Target Completion**: End of November 2025

### Suggested Sprint Planning

#### Week 1 - Quick Wins (Phase 0)

- Days 1-2: Proposal #0 (Verbosity Filter)
- Days 3-4: Proposal #1 (Simplify Tests)
- Day 5: Proposal #2 (Theme Support)

#### Week 2 - Strategic Improvements (Phase 1)

- Days 1-3: Proposal #3 (Message Trait for Extensibility)
- Days 4-5: Proposal #5 (Parameterized Tests)

#### Week 3 - Polish & Extensions (Phase 2)

- Day 1: Proposal #6 (Type-Safe Channels) + Proposal #7 (Buffering)
- Day 2: Proposal #8 (Builder Pattern)
- Day 3: Proposal #9 (Output Sink Abstraction)
- Days 4-5: Proposal #4 (Alternative Formatters - optional, can be skipped)

#### Week 4 - Integration & Documentation

- Days 1-2: Integration testing across all phases
- Days 3-4: Documentation updates and examples
- Day 5: Final review and merge

## 🔍 Review Process

### Approval Criteria

- [x] All proposals reviewed by senior software crafter (analysis completed)
- [ ] Technical feasibility validated by maintainers
- [ ] Aligns with [Development Principles](../development-principles.md)
- [ ] Implementation plan is clear and actionable
- [ ] Timeline is realistic

### Completion Criteria

- [ ] All active proposals implemented (10 proposals)
- [ ] All tests passing
- [ ] All linters passing
- [ ] Module documentation updated
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../development-principles.md) - Core principles including testability and maintainability
- [Contributing Guidelines](../contributing/README.md) - General contribution process
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Testing Conventions](../contributing/testing/) - Testing best practices

## 💡 Notes

### Proposal Organization

All 10 proposals from the original analysis are included in this refactoring plan, organized into three phases:

**Phase 0 - Quick Wins (3 proposals):**

- Proposal #0: Extract Verbosity Filtering Logic
- Proposal #1: Simplify Test Infrastructure
- Proposal #2: Add Theme/Configuration Support

**Phase 1 - Strategic Improvements (2 proposals):**

- Proposal #3: Use Message Trait for Extensibility
- Proposal #5: Parameterized Test Cases

**Phase 2 - Polish & Extensions (5 proposals):**

- Proposal #4: Add Alternative Formatters (optional enhancement)
- Proposal #6: Type-Safe Channel Routing
- Proposal #7: Add Buffering Control
- Proposal #8: Builder Pattern for Complex Messages
- Proposal #9: Output Sink Abstraction

This comprehensive approach ensures we build a robust, extensible, and maintainable output system from the ground up. Each phase builds on the previous one, with minimal interdependencies allowing for parallel development where possible.

### Design Decisions

- **All proposals active**: After review, all 10 proposals are deemed valuable and straightforward to implement. None require postponement.

- **Three-phase approach**: Organized by impact and logical dependencies. Phase 0 provides foundation, Phase 1 adds architectural improvements, Phase 2 adds polish and extensibility.

- **Keeping convenience methods**: Even with builders and message types, simple convenience methods like `progress()` remain for ergonomic common cases.

- **Sequential numbering**: Proposals numbered #0-#9 for clear tracking during implementation.

### Future Considerations

- **Colored output**: Consider adding ANSI color support as a future theme
- **JSON output**: For machine-readable output (structured logging)
- **Progress bars**: For long-running operations
- **Conditional theme selection**: Auto-detect CI environment and use plain theme

### Testing Philosophy

This refactoring maintains comprehensive test coverage while reducing duplication. The goal is to make tests easier to write and maintain without sacrificing quality.

---

**Created**: October 30, 2025  
**Last Updated**: October 30, 2025  
**Status**: 📋 Planning
