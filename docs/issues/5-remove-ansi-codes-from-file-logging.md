# Remove ANSI Color Codes from File Logging

**Issue**: [#5](https://github.com/torrust/torrust-tracker-deployer/issues/5) (follow-up to #3)
**Parent Epic**: [#2](https://github.com/torrust/torrust-tracker-deployer/issues/2) - Scaffolding for main app
**Related**:

- Issue [#3](https://github.com/torrust/torrust-tracker-deployer/issues/3) - Setup logging for production CLI
- [User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md)
- [Development Principles](../development-principles.md)

## Overview

Remove ANSI color codes from log files when using `compact` and `pretty` formats, or implement independent format control for file vs stderr outputs. Currently, both `compact` and `pretty` formats write ANSI escape sequences to log files, making them harder to parse with standard text tools (grep, awk, editors).

This is a follow-up refinement to Issue #3, discovered during PR review. The JSON format correctly omits ANSI codes, but the text-based formats include them regardless of output destination.

## Problem Statement

### Current Behavior

When using `--log-format compact` or `--log-format pretty` with file logging:

```bash
# Logs contain ANSI escape codes in files
$ torrust-tracker-deployer --log-format compact --log-dir /tmp/logs
$ hexdump -C /tmp/logs/log.txt | head -5
00000000  1b 5b 32 6d 32 30 32 35  2d 31 30 2d 31 35 54 31  |.[2m2025-10-15T1|
00000010  35 3a 30 39 3a 33 31 2e  34 39 39 36 35 32 5a 1b  |5:09:31.499652Z.|
00000020  5b 30 6d 20 1b 5b 33 32  6d 20 49 4e 46 4f 1b 5b  |[0m .[32m INFO.[|
00000030  30 6d 20 1b 5b 32 6d 74  6f 72 72 75 73 74 5f 74  |0m .[2mtorrust_t|
```

ANSI codes present:

- `\x1b[2m` - dim text
- `\x1b[32m` - green color
- `\x1b[0m` - reset formatting

### Why This Is Problematic

1. **Parsing difficulty**: Tools like `grep`, `awk`, `sed` see escape sequences as part of the text
2. **File size**: ANSI codes add ~15-20% overhead to log file size
3. **Readability**: Opening in plain text editors shows escape codes instead of text
4. **Processing**: Log aggregation tools may not handle ANSI codes correctly
5. **Searching**: Searching for patterns becomes harder with embedded codes

### What Works Correctly

✅ **JSON format**: No ANSI codes, clean machine-readable output

```json
{
  "timestamp": "2025-10-15T15:09:48.417627Z",
  "level": "INFO",
  "fields": { "message": "Application started" }
}
```

## Goals

- [ ] Remove ANSI codes from log files when using `compact` format
- [ ] Remove ANSI codes from log files when using `pretty` format
- [ ] Keep ANSI codes for stderr output (development mode)
- [ ] Maintain JSON format as-is (already correct)
- [ ] Update documentation to reflect behavior
- [ ] Verify backwards compatibility with existing log files

## Specifications

### Option A: Independent Format Control (Recommended)

Allow users to specify different formats for file and stderr, giving them full control over what format they want in each destination:

```rust
#[derive(Parser)]
pub struct Cli {
    /// Format for file logging (default: compact without ANSI)
    #[arg(long, value_enum, default_value = "compact")]
    pub log_file_format: LogFormat,

    /// Format for stderr logging (default: pretty with ANSI)
    #[arg(long, value_enum, default_value = "pretty")]
    pub log_stderr_format: LogFormat,

    // ... other args
}
```

Example usage:

```bash
# JSON to file, pretty to stderr (common for development)
torrust-tracker-deployer \
    --log-file-format json \
    --log-stderr-format pretty \
    --log-output file-and-stderr

# Compact to file, compact to stderr (production with real-time monitoring)
torrust-tracker-deployer \
    --log-file-format compact \
    --log-stderr-format compact \
    --log-output file-and-stderr

# JSON to file only (production default)
torrust-tracker-deployer --log-file-format json --log-output file-only
```

**Benefits:**

- Users choose exactly what format they want for each destination
- No assumptions about user preferences (some may want compact in files, others JSON)
- ANSI codes automatically disabled for file formats (compact/pretty write without ANSI)
- ANSI codes automatically enabled for stderr formats (compact/pretty write with ANSI)
- Backwards compatible: can deprecate old `--log-format` in favor of new flags

### Option B: Auto-detect and Force No-ANSI for Files (Simpler but Less Flexible)

Automatically disable ANSI codes when writing to files, regardless of format chosen:

```rust
// Pseudocode - actual implementation in src/logging.rs
match output_mode {
    LogOutput::FileOnly => {
        // File layer: no ANSI codes (forced)
        let file_layer = fmt::layer()
            .with_ansi(false)  // Always disable ANSI for files
            .with_format(self.format)
            .with_writer(file);

        subscriber.with(file_layer)
    },
    LogOutput::FileAndStderr => {
        // File layer: no ANSI codes (forced)
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_format(self.format)
            .with_writer(file);

        // Stderr layer: with ANSI codes (forced)
        let stderr_layer = fmt::layer()
            .with_ansi(true)
            .with_format(self.format)
            .with_writer(stderr);

        subscriber.with(file_layer).with(stderr_layer)
    }
}
```

**Drawbacks:**

- Forces same format for both file and stderr
- Removes user choice (what if they want pretty in files with ANSI?)
- Less flexible for advanced use cases

### Recommendation

**Implement Option A** (independent format control):

- Gives users full control over logging behavior
- Most flexible solution
- Better aligns with principle of not making assumptions
- Can support diverse production scenarios (JSON for aggregation, compact for grep, pretty for debugging)

## Implementation Plan

### Phase 1: Update CLI Arguments (1-2 hours)

- [ ] Task 1.1: Add `--log-file-format` argument to `src/app.rs`
- [ ] Task 1.2: Add `--log-stderr-format` argument to `src/app.rs`
- [ ] Task 1.3: Deprecate old `--log-format` argument (or keep for backwards compat)
- [ ] Task 1.4: Update `LoggingBuilder` to accept separate file and stderr formats
- [ ] Task 1.5: Update help text to explain independent format control

### Phase 2: Modify Logging Layer (1-2 hours)

- [ ] Task 2.1: Update `src/logging.rs` to support two separate formats
- [ ] Task 2.2: Add `.with_ansi(false)` for file writers (all formats)
- [ ] Task 2.3: Add `.with_ansi(true)` for stderr writers (all formats)
- [ ] Task 2.4: Handle `FileOnly` mode with single format
- [ ] Task 2.5: Handle `FileAndStderr` mode with dual formats
- [ ] Task 2.6: Test all format combinations (compact, pretty, json × file-only, file-and-stderr)

### Phase 3: Update Tests (45 minutes)

- [ ] Task 3.1: Update `tests/logging_integration.rs` to verify no ANSI in files
- [ ] Task 3.2: Add test to verify ANSI codes present in stderr output
- [ ] Task 3.3: Test different format combinations (json file + pretty stderr, etc.)
- [ ] Task 3.4: Verify backwards compatibility if keeping old `--log-format`
- [ ] Task 3.5: Review E2E test binaries (`src/bin/e2e_*.rs`) - verify if they need updates or if defaults are acceptable

### Phase 4: Documentation Updates (45 minutes)

- [ ] Task 4.1: Update `docs/user-guide/logging.md` to explain independent format control
- [ ] Task 4.2: Add examples showing different format combinations
- [ ] Task 4.3: Add note about ANSI codes (files clean, stderr colored)
- [ ] Task 4.4: Update `README.md` with new argument examples
- [ ] Task 4.5: Update `docs/contributing/logging-guide.md` for contributors
- [ ] Task 4.6: Document migration path from old `--log-format` (if deprecated)

### Phase 5: Verification (30 minutes)

- [ ] Task 5.1: Manual testing with all format combinations
- [ ] Task 5.2: Verify file sizes reduced (no ANSI overhead in files)
- [ ] Task 5.3: Test grep/awk/sed work cleanly on log files
- [ ] Task 5.4: Verify stderr colors display correctly in terminal
- [ ] Task 5.5: Test backwards compatibility (if keeping old args)
- [ ] Task 5.6: Run full test suite (`./scripts/pre-commit.sh`)

## Technical Details

### Current Implementation Location

The logging configuration is in `src/logging.rs`, specifically in the `LoggingBuilder` implementation:

```rust
// Current code (approximate location)
pub fn init(self) {
    // ... setup code ...

    let file_layer = fmt::layer()
        .with_format(self.format)
        .with_writer(file);

    // Need to add: .with_ansi(false) here for files
}
```

### Key Files to Modify

1. **`src/app.rs`**: Add new CLI arguments (`--log-file-format`, `--log-stderr-format`)
2. **`src/logging.rs`**: Main implementation (add `.with_ansi()` calls, support dual formats)
3. **`src/bin/e2e_*.rs`**: E2E test binaries (verify logging setup, may need updates)
4. **`tests/logging_integration.rs`**: Add/update tests for ANSI detection
5. **`docs/user-guide/logging.md`**: Document clean file output behavior
6. **`docs/contributing/logging-guide.md`**: Update contributor guide

### Backward Compatibility

**Breaking changes** (requires major version bump or deprecation period):

- Old `--log-format` argument replaced with `--log-file-format` and `--log-stderr-format`
- **Migration path**: Keep `--log-format` for backwards compatibility, applies to both outputs if new args not specified

**Recommended approach** (backwards compatible):

```rust
// If new args not provided, fall back to old --log-format for both
let file_format = cli.log_file_format.unwrap_or(cli.log_format);
let stderr_format = cli.log_stderr_format.unwrap_or(cli.log_format);
```

**No data breaking changes**:

- Existing log files with ANSI codes remain valid
- New log files will be cleaner and easier to process
- JSON format unchanged
- E2E test binaries continue to work

## Acceptance Criteria

- [ ] Compact format files contain no ANSI escape codes
- [ ] Pretty format files contain no ANSI escape codes
- [ ] JSON format files remain unchanged (no ANSI codes)
- [ ] Stderr output shows colors when using `--log-output file-and-stderr`
- [ ] File-only mode produces clean, grep-friendly logs
- [ ] Log file sizes reduced (less ANSI overhead)
- [ ] Documentation updated to explain ANSI behavior
- [ ] Integration tests verify ANSI presence/absence
- [ ] All existing tests pass (`./scripts/pre-commit.sh`)
- [ ] Manual testing confirms grep/awk work cleanly on log files

## Example Outputs

### Before (Current - with ANSI codes in files)

```bash
$ torrust-tracker-deployer --log-format compact
$ cat data/logs/log.txt
^[[2m2025-10-15T15:09:31.499652Z^[[0m ^[[32m INFO^[[0m ^[[2mtorrust_tracker_deployer::app^[[0m...
# Hard to grep, includes escape sequences
```

### After (Fixed - no ANSI codes in files)

```bash
$ torrust-tracker-deployer --log-format compact
$ cat data/logs/log.txt
2025-10-15T15:09:31.499652Z  INFO torrust_tracker_deployer::app: Application started...
# Clean, grep-friendly output

$ grep "Application started" data/logs/log.txt
2025-10-15T15:09:31.499652Z  INFO torrust_tracker_deployer::app: Application started...
# Works perfectly!
```

### Stderr Still Has Colors (Development Mode)

```bash
$ torrust-tracker-deployer --log-format pretty --log-output file-and-stderr
  2025-10-15T15:09:31.499652Z  INFO torrust_tracker_deployer::app: Application started...
# Terminal shows colored output for real-time monitoring
# But file remains clean for post-mortem analysis
```

## Related Documentation

- [User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md) - Design principles
- [Development Principles](../development-principles.md) - Observability and user-friendliness
- [Logging User Guide](../user-guide/logging.md) - End-user documentation
- [Logging Contributor Guide](../contributing/logging-guide.md) - Developer documentation
- Issue #3 - Parent issue that introduced logging CLI
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/) - Upstream library reference

## Notes

### Why This Wasn't Caught Earlier

- The implementation in Issue #3 correctly followed standard Rust logging patterns
- ANSI codes in files is common behavior for many Rust logging libraries
- The focus was on getting logging infrastructure working (achieved successfully)
- This is a refinement, not a bug - JSON format works correctly for production

### Production Workaround (Until Fixed)

Users who need clean log files immediately can:

```bash
# Option 1: Use JSON format (already clean)
torrust-tracker-deployer --log-format json

# Option 2: Strip ANSI codes with sed (post-processing)
sed 's/\x1b\[[0-9;]*m//g' data/logs/log.txt > data/logs/log-clean.txt
```

### E2E Test Binaries Consideration

The E2E test binaries (`src/bin/e2e_provision_tests.rs`, `src/bin/e2e_config_tests.rs`, etc.) currently initialize logging directly:

```rust
// Example from e2e_provision_tests.rs
LoggingBuilder::new(std::path::Path::new("./data/logs"))
    .with_format(LogFormat::Compact)
    .with_output(LogOutput::FileAndStderr)
    .init();
```

**Action needed**: Review whether E2E tests should:

- **Option 1**: Keep current behavior (compact format for both file and stderr) - likely acceptable since they need file+stderr for CI visibility
- **Option 2**: Update to use new dual format API once available
- **Option 3**: Use JSON for files, pretty for stderr (better for debugging)

**Recommendation**: Start with Option 1 (no changes needed) unless testing reveals issues with ANSI codes in E2E log files affecting CI/CD tooling.

### Future Enhancements (Out of Scope)

- Automatic log rotation based on size
- Compression of old log files
- Integration with log aggregation services (CloudWatch, DataDog, etc.)
- Separate verbosity control for file vs stderr
- Custom format templates

## Estimated Effort

**Total**: 5-6 hours

- Phase 1: 1-2 hours (CLI arguments update)
- Phase 2: 1-2 hours (logging layer modifications)
- Phase 3: 45 minutes (tests)
- Phase 4: 45 minutes (documentation)
- Phase 5: 30 minutes (verification)
- Buffer: 1 hour

This is a moderate refactoring that adds new functionality while maintaining backwards compatibility.
