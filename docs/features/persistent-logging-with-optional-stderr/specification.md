# Persistent Logging with Optional Stderr Redirection - Specification

## 📋 Overview

This feature adds persistent file-based logging to prepare the application for production use while maintaining current E2E test visibility through optional stderr output.

### Context

Currently, the application uses `tracing` for logging with output directed to stderr. This works well for E2E tests where developers need immediate visibility into what's happening. However, as we prepare to introduce user-facing CLI output, we need to:

1. **Separate concerns**: Internal logs (for debugging) vs. user output (for progress/guidance)
2. **Persist logs**: Production deployments need permanent log files for troubleshooting
3. **Maintain test visibility**: E2E tests should continue showing logs during development

This feature is a **preparatory refactor** that enables the future separation of user output and internal logging.

### Problem Statement

**Current state**:

- All logs go to stderr only (no persistent storage)
- E2E tests rely on stderr visibility
- No log files exist for post-mortem analysis
- Main application has minimal functionality (just shows a message)

**Problems**:

1. No persistent logs for production troubleshooting
2. Cannot separate user output from internal logs
3. No historical record of deployments or errors

**Future state (this feature enables)**:

- Production CLI shows user-friendly output (progress, errors, guidance)
- Internal logs persist in files for debugging
- E2E tests can optionally see logs during development
- Clear separation between user-facing and internal concerns

## 🎯 Goals

### Primary Goals

- **Always persist logs to files**: Write logs to `./data/logs/log.txt` regardless of other outputs
- **Optional stderr output**: Add flag to enable stderr output for E2E tests and development
- **Maintain E2E test visibility**: E2E test binaries continue showing logs on stderr
- **Production-ready**: Main application logs to file only (no stderr noise)
- **Zero breaking changes**: Existing logging calls (`info!`, `debug!`, etc.) work unchanged

### Secondary Goals (Nice-to-Have)

- Automatic directory creation for log files
- Clear error messages if log file cannot be created
- Documentation for log file management
- Helper scripts for viewing logs

### Non-Goals

What this feature explicitly does NOT aim to do:

- User-facing output (comes later with CLI implementation)
- Log rotation or size management (future enhancement)
- Multiple log files or complex log hierarchies
- Configuration files for logging settings
- Log compression or archival
- Remote logging or log shipping
- Different log levels per output target

## 💡 Proposed Solution

### Approach

Use `tracing-subscriber` layering to write logs to both file (always) and stderr (optionally):

1. **File layer (always active)**: Writes all logs to `./data/logs/log.txt`
2. **Stderr layer (optional)**: Writes logs to stderr when flag enabled

The logging initialization functions will accept a `LogOutput` enum to control output targets:

```rust
pub enum LogOutput {
    FileOnly,        // Production: logs to file only
    FileAndStderr,   // Development/E2E: logs to both file and stderr
}
```

- **E2E test binaries**: Use `LogOutput::FileAndStderr` (maintain current stderr visibility)
- **Main application**: Use `LogOutput::FileOnly` (file only, no stderr noise)

### Design Overview

```text
┌─────────────────────────────────────────────────────────────┐
│ Application Code                                            │
│   - Uses tracing macros: info!(), debug!(), warn!(), etc.   │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
         ┌────────────────────────────┐
         │ Tracing Subscriber         │
         │  (configured at init)      │
         └────────┬──────────┬────────┘
                  │          │
         ┌────────▼───┐   ┌──▼─────────┐
         │ File Layer │   │ Stderr     │
         │  (always)  │   │ Layer      │
         │            │   │ (optional) │
         └────────┬───┘   └──┬─────────┘
                  │          │
                  ▼          ▼
         ./data/logs/    stderr
         log.txt

E2E Tests:     File + Stderr (both layers active)
Main App:      File only     (file layer only)
```

### Key Design Decisions

Based on answered questions in `questions.md`:

1. **Always write to file**: Even if stderr is enabled, file output always happens
2. **Enum-based API**: Use `LogOutput` enum for clear intent (`FileOnly` vs `FileAndStderr`) - Answer to Q5
3. **Fixed log path**: `./data/logs/log.txt` is fixed (not configurable initially) - Answer to Q3
4. **Append behavior**: Append to existing file rather than overwrite - Answer to Q4
5. **Automatic directory creation**: Create `./data/logs/` if it doesn't exist - Answer to Q9
6. **Same format for both**: Keep format consistent between file and stderr - Answer to Q8
7. **Fail fast on file errors**: Panic if log file cannot be created (logging is critical) - Answer to Q7
8. **No management tools initially**: No helper scripts for now, users can manually manage logs - Answer to Q19
9. **Same log levels**: File and stderr use the same log level filtering - Answer to Q21

### Alternatives Considered

#### Option 1: Environment Variable Control

- **Pros**: No code changes needed, configurable at runtime
- **Cons**: Less explicit, harder to test, environment pollution
- **Decision**: Not chosen - prefer explicit flag in code for clarity

#### Option 2: Separate Functions for Each Mode

```rust
logging::init()              // File only
logging::init_with_stderr()  // File + stderr
```

- **Pros**: Clear intent, no flag confusion
- **Cons**: Function proliferation, doesn't scale with more options
- **Decision**: Not chosen - enum is more explicit and type-safe

#### Option 3: Builder Pattern

```rust
logging::init()
    .with_file("./data/logs/log.txt")
    .with_stderr(LogOutput::FileAndStderr)
    .build()?;
```

- **Pros**: Extensible, clear configuration
- **Cons**: Overkill for current needs, more complex
- **Decision**: Not chosen - keep it simple for now, enum is sufficient

## 🔧 Implementation Details

### Architecture Changes

No major architectural changes. This feature enhances the existing `src/logging.rs` module.

### Component Design

#### Component 1: Logging Initialization

**Purpose**: Initialize tracing with file output and optional stderr output

**Interface**:

````rust
/// Output target for logging
pub enum LogOutput {
    /// Write logs to file only (production mode)
    FileOnly,
    /// Write logs to both file and stderr (development/testing mode)
    FileAndStderr,
}

/// Initialize logging with specified output target and format
///
/// # Arguments
///
/// * `output` - Where to write logs (file only or file + stderr)
/// * `format` - The log format to use
///
/// # Panics
///
/// Panics if log file cannot be created or log directory cannot be created.
/// This is intentional as logging is critical for observability.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deploy::logging::{LogOutput, LogFormat};
///
/// // E2E tests - enable stderr visibility
/// torrust_tracker_deploy::logging::init(LogOutput::FileAndStderr, LogFormat::Compact);
///
/// // Production - file only
/// torrust_tracker_deploy::logging::init(LogOutput::FileOnly, LogFormat::Compact);
/// ```
pub fn init(output: LogOutput, format: LogFormat) {
    // Implementation
}

/// Initialize logging with compact format
///
/// # Arguments
///
/// * `output` - Where to write logs (file only or file + stderr)
pub fn init_compact(output: LogOutput) {
    init(output, LogFormat::Compact);
}

/// Initialize logging with JSON format
///
/// # Arguments
///
/// * `output` - Where to write logs (file only or file + stderr)
pub fn init_json(output: LogOutput) {
    init(output, LogFormat::Json);
}
````

**Dependencies**:

- `tracing-subscriber` - For subscriber configuration
- `tracing-appender` - For file output
- `std::fs` - For directory creation

#### Component 2: Log File Management

**Purpose**: Create log directory and file handle

**Interface**:

```rust
/// Get or create the log file appender
///
/// Creates the log directory if it doesn't exist
fn create_log_file_appender() -> impl tracing_subscriber::fmt::MakeWriter {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};

    // Create directory if it doesn't exist
    std::fs::create_dir_all("./data/logs")
        .expect("Failed to create log directory: ./data/logs");

    // Create rolling file appender (no rotation, just append)
    tracing_appender::rolling::never("./data/logs", "log.txt")
}
```

**Dependencies**: `std::fs`, `tracing_appender`

### Data Model

No new data structures needed. Uses existing `tracing` types.

### API Changes

**Modified functions** in `src/logging.rs`:

```rust
// Before: Only stderr output
pub fn init() { /* ... */ }
pub fn init_compact() { /* ... */ }
pub fn init_json() { /* ... */ }
pub fn init_with_format(format: LogFormat) { /* ... */ }

// After: File output + optional stderr (enum-based API)
pub enum LogOutput {
    FileOnly,
    FileAndStderr,
}

pub fn init(output: LogOutput, format: LogFormat) { /* ... */ }
pub fn init_compact(output: LogOutput) { /* ... */ }
pub fn init_json(output: LogOutput) { /* ... */ }
```

**Breaking changes**: None. Existing functions remain available with file-only behavior.

### Configuration

No configuration files. Controlled by function parameter at initialization.

**Log file location** (fixed for now):

- Path: `./data/logs/log.txt`
- Relative to current working directory
- Future: Could make configurable via environment variable

## 📊 Impact Analysis

### Files to Modify

| File Path                            | Changes Required                                    | Effort |
| ------------------------------------ | --------------------------------------------------- | ------ |
| `src/logging.rs`                     | Add file appender, modify init functions (enum API) | Medium |
| `src/bin/e2e_tests_full.rs`          | Call `init_compact(LogOutput::FileAndStderr)`       | Low    |
| `src/bin/e2e_config_tests.rs`        | Call `init_compact(LogOutput::FileAndStderr)`       | Low    |
| `src/bin/e2e_provision_tests.rs`     | Call `init_compact(LogOutput::FileAndStderr)`       | Low    |
| `src/main.rs`                        | Call `init_compact(LogOutput::FileOnly)`            | Low    |
| `Cargo.toml`                         | Add `tracing-appender` dependency                   | Low    |
| `.gitignore`                         | Add `data/logs/` to gitignore                       | Low    |
| `docs/contributing/logging-guide.md` | Document new behavior                               | Medium |

### Breaking Changes

**None**. This is a backward-compatible addition:

- Existing logging calls (`info!`, `debug!`, etc.) work unchanged
- Existing init functions work with file-only behavior by default
- New `*_with_stderr()` variants provide opt-in stderr output

### Performance Impact

**Minor positive impact**:

- File I/O is buffered, minimal overhead
- Reduced stderr output in production (less terminal I/O)
- No changes to log volume or frequency

**Potential concern**: File system I/O, but `tracing-appender` uses buffered writes.

### Security Considerations

**File permissions**:

- Log files created with default permissions (user read/write)
- Ensure `./data/logs/` is not web-accessible in deployment
- Consider sensitive data in logs (passwords, tokens, etc.)

**Disk space**:

- Logs will grow over time without rotation
- Future enhancement: implement rotation or size limits

## 🗓️ Implementation Plan

### Phase 1: Core Implementation

- [ ] Add `tracing-appender` dependency to `Cargo.toml`
- [ ] Define `LogOutput` enum with `FileOnly` and `FileAndStderr` variants
- [ ] Implement `create_log_file_appender()` function
- [ ] Modify `init()` to accept `LogOutput` parameter and use file appender
- [ ] Implement conditional stderr layer based on `LogOutput` enum
- [ ] Update `init_compact()` and `init_json()` to use new `init()` signature
- [ ] Write unit tests for initialization functions
- [ ] Test file creation and directory creation

**Estimated Duration**: 2-3 hours

### Phase 2: Update Callers

- [ ] Update `src/main.rs` to call `init_compact(LogOutput::FileOnly)`
- [ ] Update E2E test binaries to call `init_compact(LogOutput::FileAndStderr)`
- [ ] Verify all existing tests still pass
- [ ] Manual testing: run main app and verify file logging

**Estimated Duration**: 1 hour

### Phase 3: Documentation and Cleanup

- [ ] Add `data/logs/` to `.gitignore`
- [ ] Update module documentation in `src/logging.rs`
- [ ] Update `docs/contributing/logging-guide.md`
- [ ] Add examples of log file location
- [ ] Document how to view logs during development

**Estimated Duration**: 1-2 hours

### Phase 4: Future Enhancements (Not in Initial Scope)

- [ ] Consider adding environment variable override for log path
- [ ] Consider adding helper scripts for log management (if needed)
- [ ] Update pre-commit script if needed

**Estimated Duration**: (Deferred - not part of initial implementation)

**Total Estimated Duration**: 4-6 hours (core implementation)

## ✅ Definition of Done

### Functional Requirements

- [ ] **FR1**: Logs are always written to `./data/logs/log.txt`
- [ ] **FR2**: E2E test binaries show logs on stderr (maintain current behavior)
- [ ] **FR3**: Main application logs to file only (no stderr output)
- [ ] **FR4**: Log directory `./data/logs/` is created automatically if missing
- [ ] **FR5**: Log file appends to existing content (doesn't truncate)
- [ ] **FR6**: Initialization panics with clear message if log file cannot be created

### Technical Requirements

- [ ] Code follows project conventions and style guidelines
- [ ] All linters pass (clippy, rustfmt, markdownlint, etc.)
- [ ] No compiler warnings
- [ ] No breaking changes to existing logging calls
- [ ] Error messages are clear and actionable
- [ ] Module documentation is updated and accurate

### Testing Requirements

**Unit Tests** (in `src/logging.rs` test module):

- [ ] **T1**: Unit test: Verify `LogOutput` enum can be constructed
- [ ] **T2**: Unit test: Verify initialization functions accept `LogOutput` parameter
- [ ] **T3**: Unit test: Verify directory path calculation is correct

**Manual E2E Tests** (per Q14 - no automated integration tests initially):

- [ ] **T4**: Run `cargo run --bin e2e-tests-full` - verify stderr output visible
- [ ] **T5**: Run `cargo run` (main app) - verify no stderr output, check file exists
- [ ] **T6**: Verify log file content at `./data/logs/log.txt` is correct and readable
- [ ] **T7**: Verify log file is created in append mode (run twice, check file size increases)
- [ ] **T8**: Verify directory is auto-created if `./data/logs/` doesn't exist

### Documentation Requirements

- [ ] **D1**: Update `src/logging.rs` module documentation
- [ ] **D2**: Update `docs/contributing/logging-guide.md` with:
  - Log file location
  - How to view logs
  - When stderr is enabled vs. disabled
- [ ] **D3**: Add `data/logs/` to `.gitignore` with comment
- [ ] **D4**: Update feature README with implementation status

### Code Review Checklist

- [ ] Implementation matches specification
- [ ] Error handling is appropriate (panic on critical failures)
- [ ] Code is readable and well-commented
- [ ] No unnecessary complexity
- [ ] Tests cover important scenarios
- [ ] Documentation is clear and complete

## 🔄 Migration Path

**No migration required**. This is a transparent enhancement:

1. **Existing code**: Works unchanged (gets file logging automatically)
2. **E2E tests**: Explicitly enable stderr to maintain visibility
3. **Production code**: Uses file-only logging by default

**Rollback**: If issues arise, remove the file appender and revert to stderr-only (previous behavior).

## 📚 Testing Strategy

Per **Q14 Answer**: Use **unit tests + manual E2E testing** initially. Automated integration tests for file I/O are deferred.

### Unit Tests

Test logging initialization API:

```rust
#[test]
fn it_should_accept_file_only_output() {
    // Verify LogOutput::FileOnly can be passed to init functions
    // No panic, initialization succeeds
}

#[test]
fn it_should_accept_file_and_stderr_output() {
    // Verify LogOutput::FileAndStderr can be passed to init functions
    // No panic, initialization succeeds
}

#[test]
fn it_should_calculate_log_directory_path() {
    // Verify log directory path is "./data/logs/"
}
```

### Manual E2E Tests

**Test Procedure** (documented in Testing Requirements section above):

1. **Run E2E tests** - verify stderr output is visible:

   ```bash
   cargo run --bin e2e-tests-full
   # Expect: Logs visible in terminal
   ```

2. **Run main app** - verify file-only logging:

   ```bash
   cargo run
   # Expect: No stderr output, file at ./data/logs/log.txt
   ```

3. **Inspect log file**:

   ```bash
   cat ./data/logs/log.txt
   # Verify: Contains expected log entries
   ```

4. **Verify append mode**:

   ```bash
   cargo run  # First run
   wc -l ./data/logs/log.txt  # Note line count
   cargo run  # Second run
   wc -l ./data/logs/log.txt  # Line count should increase
   ```

5. **Verify auto-creation**:

   ```bash
   rm -rf ./data/logs
   cargo run
   # Expect: Directory and file created automatically
   ```

All should pass with stderr visibility maintained.

### Manual Testing

1. **Main application**:

   ```bash
   cargo run
   # Verify: No logs on stderr
   # Verify: Logs appear in ./data/logs/log.txt
   ```

2. **E2E tests**:

   ```bash
   cargo run --bin e2e-tests-full
   # Verify: Logs appear on stderr (current behavior)
   # Verify: Logs also written to file
   ```

3. **Log file content**:

   ```bash
   cat ./data/logs/log.txt
   # Verify: Proper format, timestamps, levels
   ```

## 🚨 Risks and Mitigation

### Risk 1: File System Permissions

**Risk**: Application cannot write to `./data/logs/` due to permissions

**Impact**: Application fails to start (panic)

**Mitigation**:

- Use `expect()` with clear error message explaining the issue
- Document required permissions in deployment guide
- Consider alternative log location as future enhancement

### Risk 2: Disk Space Exhaustion

**Risk**: Log files grow without bound and fill disk

**Impact**: Application or system becomes unstable

**Mitigation**:

- Document log management in deployment guide
- Provide helper script to clear old logs
- Future: Implement automatic log rotation

### Risk 3: Performance Degradation

**Risk**: File I/O slows down application

**Impact**: Slower execution, especially during intensive logging

**Mitigation**:

- `tracing-appender` uses buffered writes (minimal impact)
- Test with realistic log volumes
- Monitor performance in E2E tests

### Risk 4: Breaking E2E Test Expectations

**Risk**: E2E tests fail because stderr behavior changed

**Impact**: CI/CD failures, debugging difficulty

**Mitigation**:

- Explicitly enable stderr in E2E test binaries
- Test thoroughly before merging
- Update tests if expectations change

## 📖 Related Documentation

- [User Output vs Internal Logging Separation](../../research/UX/user-output-vs-logging-separation.md) - Why we separate these concerns
- [Development Principles](../../development-principles.md) - Observability and traceability
- [Logging Guide](../../contributing/logging-guide.md) - How to use logging in the codebase
- [Error Handling Guide](../../contributing/error-handling.md) - Error handling principles

## 🎓 Lessons Learned (Post-Implementation)

> To be filled after implementation completes

**What went well**:

- [To be documented]

**What was challenging**:

- [To be documented]

**What we'd do differently**:

- [To be documented]

**Future improvements**:

- [To be documented]
