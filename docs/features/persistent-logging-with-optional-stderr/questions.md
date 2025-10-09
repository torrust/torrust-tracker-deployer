# Clarifying Questions for Persistent Logging with Optional Stderr Redirection

This document contains questions to clarify requirements, scope, and priorities before implementation begins.

---

## 🔍 Scope and Requirements

### 1. **Core Functionality**

**Question**: What is the minimum viable functionality for this feature?

**Your Answer**:

- Always write logs to `./data/logs/log.txt`
- Add a flag to `logging::init()` functions to optionally enable stderr output
- E2E test binaries should enable stderr output
- Main application should NOT enable stderr output (file only)

### 2. **Out of Scope**

**Question**: What is explicitly NOT included in this feature?

**Your Answer**:

- User-facing output (that comes later when implementing CLI)
- Log rotation or size management
- Multiple log files or log levels per file
- Configuration file for logging settings
- Log file compression
- Remote logging or log shipping
- Split logs per environment (e.g., dev vs prod)

### 3. **Log File Location**

**Question**: Should the log file location be configurable, or is `./data/logs/log.txt` fixed?

**Your Answer**: Not, it should be fixed for now.

**Options**:

- Fixed path: `./data/logs/log.txt`

Later enhancements could include:

- Configurable via environment variable
- Configurable via CLI flag
- Different paths for different environments

### 4. **Log File Behavior**

**Question**: How should the log file behave on application restart?

**Your Answer**: Append to existing log file

**Options**:

- Append to existing log file
- Truncate/overwrite on each run
- Create new file with timestamp (e.g., `log-2025-10-09-16-30-00.txt`)

NOTE: The user can always delete or rotate logs manually for now. Maybe when the applications start we can show to the user the log file location and size.

### 5. **Stderr Flag Control**

**Question**: How should the stderr flag be named and controlled in the logging initialization?

**Your Answer**: Option C is fine.

**Proposed options**:

```rust
// Option A: Simple boolean flag
logging::init(enable_stderr: bool)

// Option B: Builder pattern
logging::init().with_stderr(true).build()

// Option C: Enum for output targets
logging::init(LogOutput::FileAndStderr)
// vs
logging::init(LogOutput::FileOnly)

// Option D: Separate functions
logging::init()           // File only
logging::init_with_stderr() // File + stderr
```

## 🎯 Technical Approach

### 6. **Logging Library Implementation**

**Question**: Should we use `tracing-subscriber` layers for this, or another approach?

**Your Answer**: Use the recommended one.

**Technical details**:

- Current code uses `tracing-subscriber` with `FmtSubscriber`
- Recommended: Use `tracing_subscriber::layer::Layered` with:
  - `fmt::Layer` for file output (always)
  - Optional `fmt::Layer` for stderr (when flag enabled)

### 7. **Error Handling**

**Question**: How should we handle log file creation failures?

**Your Answer**: Panic.

**Options**:

- Panic (fail fast - logging is critical)
- Return error and let caller decide
- Fall back to stderr only
- Silently fail (not recommended)

NOTE: We need the data folder for other things, so if we can't create the log file, we should panic.

### 8. **Log Format Consistency**

**Question**: Should the log format be the same for file and stderr?

**Your Answer**: Identical format for both for now.

**Options**:

- Identical format for both
- Different formats (e.g., JSON for file, human-readable for stderr)
- Configurable format per output

### 9. **Directory Creation**

**Question**: Should `./data/logs/` directory be created automatically if it doesn't exist?

**Your Answer**: Automatically create (recommended for convenience)

**Options**:

- Automatically create (recommended for convenience)
- Require manual creation (fail if missing)
- Create with specific permissions

## 📊 Priority and Timeline

### 10. **Priority Level**

**Question**: What is the priority of this feature? (High | Medium | Low)

**Your Answer**: **High**

**Rationale**: This is a preparatory refactor that enables future work on user-facing CLI output. It's urgent as we want to start implementing the CLI.

### 11. **Timeline Expectations**

**Question**: Is there a target date or sprint for completion?

**Your Answer**: Should be completed before starting CLI implementation

**Considerations**:

- Should be completed before starting CLI implementation
- Relatively small scope (1-2 days of work)
- Low risk of breaking existing functionality

### 12. **Dependencies**

**Question**: Does this feature depend on other work being completed first?

**Your Answer**: **No dependencies**

This is a standalone refactor that doesn't require other features to be completed first.

## ✅ Success Criteria

### 13. **Definition of Done**

**Question**: How do we know this feature is complete and working correctly?

**Your Answer**: You proposed criteria look good.

**Proposed criteria**:

- [ ] Logs are written to `./data/logs/log.txt` in all scenarios
- [ ] E2E test binaries show logs on stderr (current behavior maintained)
- [ ] Main application does NOT show logs on stderr (file only)
- [ ] Log file is created automatically if it doesn't exist
- [ ] Directory `./data/logs/` is created automatically if missing
- [ ] All existing tests still pass
- [ ] Log format is consistent and readable
- [ ] Error handling for log file failures is appropriate

### 14. **Testing Requirements**

**Question**: What level of testing is expected? (Unit | Integration | E2E)

**Your Answer**: For automated tests, unit tests are sufficient. Manual testing for E2E.

**Proposed testing**:

- **Unit tests**: Test logging initialization with different flags
- **Integration tests**: Verify logs are written to file correctly
- **E2E tests**: Confirm E2E tests still show logs on stderr
- **Manual testing**: Run main app and verify file logging only

### 15. **Documentation Requirements**

**Question**: What documentation needs to be updated or created?

**Your Answer**: [To be filled]

**Proposed documentation**:

- Update `src/logging.rs` module documentation
- Update logging guide in `docs/contributing/logging-guide.md`
- Add section about log file location and management
- Document the stderr flag behavior

## ⚠️ Risk Assessment

### 16. **Known Risks**

**Question**: What are the potential risks or challenges with this feature?

**Your Answer**: THe risks you mentioned are good.

**Potential risks**:

- File system permissions issues (can't write to `./data/logs/`)
- Disk space exhaustion if logs grow too large
- Performance impact of file I/O
- Breaking existing E2E test expectations

### 17. **Backward Compatibility**

**Question**: Does this feature need to maintain backward compatibility? With what?

**Your Answer**: **Yes - E2E tests must continue working**

**Considerations**:

- E2E tests currently rely on stderr visibility
- E2E tests should automatically enable stderr output
- No breaking changes to existing logging calls (`info!`, `debug!`, etc.)
- `logging::init()` function signature may change, but only called in a few places

### 18. **Migration Path**

**Question**: Do we need a migration path from current behavior?

**Your Answer**: I don't think so. The change is internal and should not affect users.

**Current state**:

- `logging::init()` - Simple initialization
- `logging::init_compact()` - Compact format
- `logging::init_json()` - JSON format
- `logging::init_with_format()` - Custom format

**Proposed change**:

- Add optional `enable_stderr` parameter to all init functions
- Or create separate `*_with_stderr()` variants
- Update E2E test binaries to enable stderr
- Main app uses file-only logging

The optional parameter with a enum is fine.

## 💡 Additional Questions

### 19. **Log File Management**

**Question**: Should we provide tools or guidance for managing log files (viewing, clearing, rotating)?

**Your Answer**: Not now. The file is not expected to grow too large in this initial phase. And users will likely not need complex management yet. End users will use the deployer for a single run and then discard it.

**Considerations**:

- Add script to clear logs: `./scripts/clear-logs.sh`
- Add script to tail logs: `./scripts/tail-logs.sh`
- Document manual log rotation
- Future: Implement automatic rotation

### 20. **Testing During Development**

**Question**: How should developers view logs during development workflows?

**Your Answer**: They can use `tail -f ./data/logs/log.txt` or change the flag when initializing logging in their dev builds.

**Options**:

- Always enable stderr in debug builds
- Provide environment variable to enable stderr
- Use separate binary for development with stderr enabled
- Document using `tail -f ./data/logs/log.txt`

### 21. **Log Levels and Filtering**

**Question**: Should the log level filtering differ between file and stderr?

**Your Answer**: Same level for both (simpler)

**Options**:

- Same level for both (simpler)
- Different levels (e.g., DEBUG to file, INFO to stderr)
- Configurable per output

## 📝 Notes

**Key Design Principle**: This feature should be a transparent refactor that maintains current E2E test visibility while adding persistent logging for production use.

**Future Considerations**: When implementing the CLI user output layer, we'll have:

- File: Internal structured logs (tracing)
- Stderr: User-friendly progress and error messages
- Stdout: Command output and results

This feature prepares the infrastructure for that separation.
