# Output Sink Abstraction

**Issue**: [#140](https://github.com/torrust/torrust-tracker-deployer/issues/140)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**:

- Refactoring Plan: [docs/refactors/plans/user-output-architecture-improvements.md](../refactors/plans/user-output-architecture-improvements.md)
- Proposal #3 (Dependency): Use Message Trait for Extensibility ([#127](https://github.com/torrust/torrust-tracker-deployer/issues/127))

## Overview

This task introduces an `OutputSink` abstraction layer to enable writing output to multiple destinations beyond stdout/stderr. Currently, `UserOutput` is tightly coupled to the `Write` trait for console output. This proposal enables output to files, network endpoints, telemetry systems, or multiple destinations simultaneously through a composable sink pattern.

**Current State**: The codebase has implemented:

- `OutputMessage` trait with message types (Proposal #3, Issue #127)
- Type-safe channel routing with `StdoutWriter` and `StderrWriter` (Proposal #6, Issue #135)
- Direct writing to stdout/stderr through `Write` trait

The current architecture works well for console output but doesn't support alternative destinations or multi-destination output without modifying `UserOutput` internals.

## Goals

- [ ] Define `OutputSink` trait for output destinations
- [ ] Implement `StandardSink` for stdout/stderr (backward compatible)
- [ ] Implement `CompositeSink` for multiple destinations
- [ ] Enable extensibility for custom sinks (file, network, telemetry)
- [ ] Maintain backward compatibility with existing API
- [ ] Support testing with mock sinks

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/user_output.rs`
**Pattern**: Strategy pattern with composite pattern for multi-destination output

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Keep presentation logic in presentation layer (no domain concerns)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Sinks must be composable (composite pattern)
- [ ] Sinks must integrate with existing `OutputMessage` trait
- [ ] Must maintain backward compatibility with direct stdout/stderr usage
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Breaking existing API (must be backward compatible)
- ❌ Complex sink hierarchies (keep composition simple)
- ❌ Mixing concerns (sinks handle output, not formatting)
- ❌ Tight coupling between sink implementations

## Specifications

### OutputSink Trait Definition

Define the core trait that all output destinations implement:

````rust
/// Trait for output destinations
///
/// An output sink receives formatted messages and writes them to a destination.
/// Sinks handle the mechanics of where output goes, not how it's formatted.
///
/// # Design Philosophy
///
/// Sinks receive already-formatted messages (with theme applied) and route them
/// to appropriate destinations. They don't handle formatting or verbosity filtering -
/// those concerns are handled by message types and filters respectively.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::{OutputSink, OutputMessage};
///
/// struct FileSink {
///     file: File,
/// }
///
/// impl OutputSink for FileSink {
///     fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
///         writeln!(self.file, "{}", formatted).ok();
///     }
/// }
/// ```
pub trait OutputSink: Send + Sync {
    /// Write a formatted message to this sink
    ///
    /// # Arguments
    ///
    /// * `message` - The message object (for metadata like channel)
    /// * `formatted` - The already-formatted message text
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str);
}
````

### StandardSink Implementation

Standard sink that writes to stdout/stderr (backward compatible):

````rust
/// Standard sink writing to stdout/stderr
///
/// This is the default sink that maintains backward compatibility with the
/// existing console output behavior.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::StandardSink;
///
/// let sink = StandardSink::new(
///     Box::new(std::io::stdout()),
///     Box::new(std::io::stderr())
/// );
/// ```
pub struct StandardSink {
    stdout: StdoutWriter,
    stderr: StderrWriter,
}

impl StandardSink {
    /// Create a new standard sink with the given writers
    pub fn new(
        stdout: Box<dyn Write + Send + Sync>,
        stderr: Box<dyn Write + Send + Sync>
    ) -> Self {
        Self {
            stdout: StdoutWriter(stdout),
            stderr: StderrWriter(stderr),
        }
    }

    /// Create a standard sink using default stdout/stderr
    pub fn default_console() -> Self {
        Self::new(
            Box::new(std::io::stdout()),
            Box::new(std::io::stderr())
        )
    }
}

impl OutputSink for StandardSink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        match message.channel() {
            Channel::Stdout => {
                self.stdout.write_line(formatted).ok();
            }
            Channel::Stderr => {
                self.stderr.write_line(formatted).ok();
            }
        }
    }
}
````

### CompositeSink Implementation

Composite sink that writes to multiple destinations:

````rust
/// Composite sink that writes to multiple destinations
///
/// Enables fan-out of messages to multiple sinks simultaneously. Useful for
/// scenarios like writing to both console and log file, or sending to both
/// stderr and telemetry service.
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::user_output::{CompositeSink, StandardSink, FileSink};
///
/// let composite = CompositeSink::new(vec![
///     Box::new(StandardSink::default_console()),
///     Box::new(FileSink::new("output.log")),
/// ]);
/// ```
pub struct CompositeSink {
    sinks: Vec<Box<dyn OutputSink>>,
}

impl CompositeSink {
    /// Create a new composite sink with the given child sinks
    pub fn new(sinks: Vec<Box<dyn OutputSink>>) -> Self {
        Self { sinks }
    }

    /// Add a sink to the composite
    pub fn add_sink(&mut self, sink: Box<dyn OutputSink>) {
        self.sinks.push(sink);
    }
}

impl OutputSink for CompositeSink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        for sink in &mut self.sinks {
            sink.write_message(message, formatted);
        }
    }
}
````

### UserOutput Integration

Update `UserOutput` to use the sink abstraction:

```rust
pub struct UserOutput {
    theme: Theme,
    verbosity_filter: VerbosityFilter,
    sink: Box<dyn OutputSink>,
    formatter_override: Option<Box<dyn FormatterOverride>>,
}

impl UserOutput {
    /// Create a new `UserOutput` with default console sink
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self::with_sink(
            verbosity,
            Box::new(StandardSink::default_console())
        )
    }

    /// Create a `UserOutput` with a custom sink
    pub fn with_sink(
        verbosity: VerbosityLevel,
        sink: Box<dyn OutputSink>
    ) -> Self {
        Self {
            theme: Theme::default(),
            verbosity_filter: VerbosityFilter::new(verbosity),
            sink,
            formatter_override: None,
        }
    }

    /// Write a message through the sink
    pub fn write(&mut self, message: &dyn OutputMessage) {
        if !self.verbosity_filter.should_show(message.required_verbosity()) {
            return;
        }

        let mut formatted = message.format(&self.theme);

        // Apply optional formatter override
        if let Some(override_formatter) = &self.formatter_override {
            formatted = override_formatter.transform(&formatted, message);
        }

        // Write through sink
        self.sink.write_message(message, &formatted);
    }
}
```

### Example Custom Sinks

#### FileSink Example

```rust
/// Example: File sink that writes all output to a file
pub struct FileSink {
    file: std::fs::File,
}

impl FileSink {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }
}

impl OutputSink for FileSink {
    fn write_message(&mut self, _message: &dyn OutputMessage, formatted: &str) {
        use std::io::Write;
        writeln!(self.file, "{}", formatted).ok();
    }
}
```

#### TelemetrySink Example

```rust
/// Example: Telemetry sink for observability
pub struct TelemetrySink {
    // In real implementation, this would be a telemetry client
    endpoint: String,
}

impl TelemetrySink {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl OutputSink for TelemetrySink {
    fn write_message(&mut self, message: &dyn OutputMessage, formatted: &str) {
        // In real implementation, send to telemetry service
        tracing::debug!(
            endpoint = %self.endpoint,
            message_type = message.type_name(),
            channel = ?message.channel(),
            content = formatted,
            "Telemetry event"
        );
    }
}
```

### Testing Strategy

```rust
#[cfg(test)]
mod sink_tests {
    use super::*;

    #[test]
    fn it_should_write_to_standard_sink() {
        let mut sink = StandardSink::new(
            Box::new(Vec::new()),
            Box::new(Vec::new())
        );
        let message = ProgressMessage { text: "Test".to_string() };
        sink.write_message(&message, "⏳ Test\n");

        // Verify message was written (would need accessor methods)
    }

    #[test]
    fn it_should_write_to_all_sinks_in_composite() {
        let sink1 = Box::new(MockSink::new());
        let sink2 = Box::new(MockSink::new());
        let mut composite = CompositeSink::new(vec![sink1, sink2]);

        let message = ProgressMessage { text: "Test".to_string() };
        composite.write_message(&message, "⏳ Test\n");

        // Verify both sinks received the message
    }

    #[test]
    fn it_should_route_messages_by_channel() {
        let mut output = create_test_output_with_sink(
            VerbosityLevel::Normal,
            Box::new(StandardSink::default_console())
        );

        output.output().progress("Progress");
        output.output().result("Result");

        // Verify correct channel routing
    }

    // Mock sink for testing
    struct MockSink {
        messages: Vec<String>,
    }

    impl MockSink {
        fn new() -> Self {
            Self { messages: Vec::new() }
        }
    }

    impl OutputSink for MockSink {
        fn write_message(&mut self, _message: &dyn OutputMessage, formatted: &str) {
            self.messages.push(formatted.to_string());
        }
    }
}
```

## Implementation Plan

### Phase 1: Core Trait and StandardSink (2 hours)

- [ ] Define `OutputSink` trait with `write_message()` method
- [ ] Implement `StandardSink` with stdout/stderr routing
- [ ] Add `StandardSink::default_console()` convenience constructor
- [ ] Ensure backward compatibility with existing behavior
- [ ] Add rustdoc documentation with examples

### Phase 2: CompositeSink Implementation (1 hour)

- [ ] Implement `CompositeSink` for multi-destination output
- [ ] Add `new()` and `add_sink()` methods
- [ ] Test fan-out behavior to multiple sinks
- [ ] Add rustdoc documentation with use cases

### Phase 3: UserOutput Integration (1.5 hours)

- [ ] Update `UserOutput` to use `Box<dyn OutputSink>`
- [ ] Add `with_sink()` constructor for custom sinks
- [ ] Update `write()` method to use sink
- [ ] Verify all existing functionality still works
- [ ] Ensure backward compatibility with `new()` constructor

### Phase 4: Example Sinks and Documentation (1 hour)

- [ ] Implement `FileSink` as example
- [ ] Implement `TelemetrySink` as example (mock)
- [ ] Add comprehensive usage examples to rustdoc
- [ ] Document when to use different sink types

### Phase 5: Testing (1.5 hours)

- [ ] Add unit tests for `StandardSink`
- [ ] Add unit tests for `CompositeSink`
- [ ] Add integration tests with `UserOutput`
- [ ] Create `MockSink` for test infrastructure
- [ ] Verify all existing tests still pass

### Phase 6: Quality Assurance (1 hour)

- [ ] Run `./scripts/pre-commit.sh` and fix any issues
- [ ] Verify backward compatibility
- [ ] Check code coverage for new code
- [ ] Review documentation completeness

**Total Estimated Time**: 8 hours

## Acceptance Criteria

### Functional Requirements

- [ ] `OutputSink` trait defines contract for output destinations
- [ ] `StandardSink` maintains backward-compatible console output
- [ ] `CompositeSink` enables multi-destination output
- [ ] `UserOutput` works with any sink implementation
- [ ] Custom sinks can be implemented externally

### API Design Requirements

- [ ] Sink trait is simple and focused
- [ ] Composition is straightforward and intuitive
- [ ] Backward compatibility is maintained
- [ ] API follows Rust conventions and idioms

### Testing Requirements

- [ ] Unit tests cover sink implementations
- [ ] Integration tests verify `UserOutput` with different sinks
- [ ] Mock sink enables easy testing
- [ ] All existing tests continue to pass

### Documentation Requirements

- [ ] Trait has comprehensive rustdoc
- [ ] Each sink implementation is documented with examples
- [ ] Usage patterns are clearly explained
- [ ] Examples show custom sink implementation

### Quality Requirements (applies to every commit and PR)

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Code follows project conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Error handling follows project patterns (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Backward compatibility is verified

**Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

## Related Documentation

- [Development Principles](../development-principles.md) - Core principles including extensibility
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Testing Conventions](../contributing/testing/) - Testing best practices
- [Codebase Architecture](../codebase-architecture.md) - DDD layer guidance

## Notes

### Why Output Sink Abstraction?

- **Extensibility**: Support multiple output destinations without modifying core code
- **Composition**: Combine sinks for fan-out scenarios
- **Testability**: Mock sinks enable better testing
- **Observability**: Easy to add telemetry/monitoring sinks
- **Flexibility**: Users can implement custom sinks for their needs

### Design Decisions

- **Trait-Based**: Enables external implementations and testing
- **Composable**: Composite pattern for multi-destination support
- **Backward Compatible**: Standard sink maintains existing behavior
- **Simple Interface**: Single method keeps complexity low

### Use Cases

**Console Only (Default)**:

```rust
let mut output = UserOutput::new(VerbosityLevel::Normal);
// Uses StandardSink::default_console()
```

**Console + File**:

```rust
let composite = CompositeSink::new(vec![
    Box::new(StandardSink::default_console()),
    Box::new(FileSink::new("output.log")?),
]);
let mut output = UserOutput::with_sink(VerbosityLevel::Normal, Box::new(composite));
```

**Console + Telemetry**:

```rust
let composite = CompositeSink::new(vec![
    Box::new(StandardSink::default_console()),
    Box::new(TelemetrySink::new("https://telemetry.example.com".to_string())),
]);
let mut output = UserOutput::with_sink(VerbosityLevel::Normal, Box::new(composite));
```

### Future Enhancements

Potential future sink features (not in this proposal):

- Buffered sinks with configurable flush strategies
- Filtered sinks that only pass certain message types
- Async sinks for network destinations
- Sink decorators for adding timestamps, metadata
- Sink metrics for monitoring message flow

---

**Created**: November 4, 2025
**Status**: 📋 Not Started
**Priority**: P2 (Phase 2 - Polish & Extensions)
**Estimated Effort**: 8 hours
