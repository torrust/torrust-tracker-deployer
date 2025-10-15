# Setup Logging for Production CLI

**Related Roadmap Item**: [1.1 Setup logging](../roadmap.md#1-add-scaffolding-for-main-app)

**Status**: 📝 Specification - Ready for implementation

---

## 📋 Overview

Implement production-ready logging configuration for the main `torrust-tracker-deployer` CLI binary. Currently, logging infrastructure exists and is used in E2E test binaries, but the main application entry point (`src/main.rs`) lacks logging initialization.

This issue focuses on integrating the existing logging system into the production CLI with proper configuration options, following the project's principles of observability, user-friendliness, and separation of concerns between user output and internal logging.

---

## 🎯 Goals

1. **Initialize logging in main CLI**: Set up logging when the application starts
2. **CLI configuration**: Allow users to configure log output, format, and directory via command-line arguments
3. **Separation of concerns**: Maintain clear distinction between user-facing output (stdout/stderr) and internal logging (log files)
4. **Production-ready defaults**: Use sensible defaults for production use (file-only logging, compact format)
5. **Developer-friendly**: Support verbose modes for development and troubleshooting

**Note**: User output verbosity (`-v`, `-vv`, `-vvv`) will be added in a future issue (Roadmap item 1.7).

---

## 📖 Background

### Current State

- **Logging infrastructure**: Fully implemented in `src/logging.rs` with:

  - `LoggingBuilder` for flexible configuration
  - Three format options: `Pretty`, `Json`, `Compact`
  - Two output modes: `FileOnly`, `FileAndStderr`
  - Environment-based filtering via `RUST_LOG`
  - Persistent logging to `./data/logs/log.txt`

- **Current usage**: Logging is initialized in E2E test binaries but **not in main CLI**:

  ```rust
  // src/bin/e2e_provision_tests.rs
  LoggingBuilder::new(std::path::Path::new("./data/logs"))
      .with_format(LogFormat::Compact)
      .with_output(LogOutput::FileAndStderr)
      .init();
  ```

- **Main binary**: Currently only prints static information, no logging initialization:

  ```rust
  // src/main.rs - No logging!
  fn main() {
      println!("🏗️  Torrust Tracker Deployer");
      // ...
  }
  ```

### Design Principles

This implementation must follow the project's architectural decisions:

1. **User Output vs Logging Separation** ([research/UX/user-output-vs-logging-separation.md](../research/UX/user-output-vs-logging-separation.md)):

   - User output: Human-friendly messages to stdout/stderr for progress and guidance
   - Internal logging: Structured logs to files for debugging and observability
   - **Never mix**: Logging is always persistent to files, independent of user verbosity

2. **Observability Principle** ([development-principles.md](../development-principles.md)):
   - If it happens, we can see it - even after it happens
   - All operations must be logged with sufficient detail for post-mortem analysis
   - Traceability through the three-level architecture (Commands → Steps → Actions)

---

## 🔧 Specifications

### 1. Application Structure

Create a new `src/app.rs` module to keep `main.rs` minimal and focused.

**File: `src/app.rs`**

```rust
use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use crate::logging::LoggingBuilder;

#[derive(Parser)]
#[command(name = "torrust-tracker-deployer")]
#[command(about = "Automated deployment infrastructure for Torrust Tracker")]
#[command(version)]
pub struct Cli {
    /// Logging format (default: compact)
    #[arg(long, value_enum, default_value = "compact", global = true)]
    pub log_format: LogFormat,

    /// Log output mode (default: file-only for production)
    #[arg(long, value_enum, default_value = "file-only", global = true)]
    pub log_output: LogOutput,

    /// Log directory (default: ./data/logs)
    #[arg(long, default_value = "./data/logs", global = true)]
    pub log_dir: PathBuf,
}

pub fn run() {
    let cli = Cli::parse();

    // Initialize logging FIRST
    LoggingBuilder::new(&cli.log_dir)
        .with_format(cli.log_format)
        .with_output(cli.log_output)
        .init();

    // Log startup event with configuration details
    info!(
        app = "torrust-tracker-deployer",
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %cli.log_dir.display(),
        log_format = ?cli.log_format,
        log_output = ?cli.log_output,
        "Application started"
    );

    // Display info to user (keep existing behavior for now)
    println!("🏗️  Torrust Tracker Deployer");
    println!("=========================");
    println!();
    println!("This repository provides automated deployment infrastructure for Torrust tracker projects.");
    println!("The infrastructure includes VM provisioning with OpenTofu and configuration");
    println!("management with Ansible playbooks.");
    println!();
    println!("📋 Getting Started:");
    println!("   Please follow the instructions in the README.md file to:");
    println!("   1. Set up the required dependencies (OpenTofu, Ansible, LXD)");
    println!("   2. Provision the deployment infrastructure");
    println!("   3. Deploy and configure the services");
    println!();
    println!("🧪 Running E2E Tests:");
    println!("   Use the e2e tests binaries to run end-to-end tests:");
    println!("   cargo e2e-provision && cargo e2e-config");
    println!();
    println!("📖 For detailed instructions, see: README.md");

    info!("Application finished");
}
```

**Note**: Subcommands (provision, configure, deploy, destroy) will be added in future issues when their functionality is implemented.

### 2. Main Entry Point

Keep `src/main.rs` minimal and focused:

**File: `src/main.rs`**

```rust
mod app;

fn main() {
    app::run();
}
```

This clean separation ensures:

- `main.rs` remains minimal and focused on program entry
- Application logic lives in `app.rs` module
- Easy to test application behavior independently of main entry point
- Follows Rust best practices for binary structure

**Note**: Commands will be added in separate issues (see roadmap items 1.2, 1.5, 1.6).

### 3. Default Behavior

**Production defaults** (when no arguments provided):

```bash
torrust-tracker-deployer
```

- Log format: `Compact` (space-efficient, readable)
- Log output: `FileOnly` (no stderr pollution)
- Log directory: `./data/logs/log.txt`
- Internal log level: Controlled by `RUST_LOG` (default: `info`)

**Development/troubleshooting**:

```bash
# Enable stderr output for real-time visibility
torrust-tracker-deployer --log-output file-and-stderr

# Use pretty format for readability
torrust-tracker-deployer --log-format pretty --log-output file-and-stderr

# Control internal log level via environment
RUST_LOG=debug torrust-tracker-deployer
```

### 4. Integration with Existing E2E Tests

**No changes required** for E2E test binaries. They should continue using their current logging setup:

```rust
// E2E tests continue to work as-is
LoggingBuilder::new(std::path::Path::new("./data/logs"))
    .with_format(LogFormat::Compact)
    .with_output(LogOutput::FileAndStderr)
    .init();
```

### 5. Documentation Requirements

Update the following documentation:

- **README.md**: Add section on logging configuration
- **docs/user-guide/logging.md**: Create user-facing documentation explaining logging options and common scenarios
- **docs/contributing/logging-guide.md**: Create or update contributor documentation on how to use logging in code
- **Help text**: Ensure `--help` provides clear guidance on logging options

---

## 📐 Implementation Plan

### Phase 1: Basic CLI Structure (Estimated: 1-2 hours)

**Goal**: Set up clap-based CLI with logging initialization

**Tasks**:

- [ ] **Task 1.1**: Add `clap` dependency to `Cargo.toml` with required features (if not already present)
- [ ] **Task 1.2**: Create new `src/app.rs` module
  - [ ] Add module declaration in `src/lib.rs` or `src/main.rs`
  - [ ] Create `Cli` struct with logging arguments
    - [ ] Add `--log-format` argument with compact as default
    - [ ] Add `--log-output` argument with file-only as default
    - [ ] Add `--log-dir` argument with `./data/logs` as default
  - [ ] Implement `pub fn run()` function with application logic
- [ ] **Task 1.3**: Update `src/main.rs` to call `app::run()`
  - [ ] Keep main function minimal (just call `app::run()`)
- [ ] **Task 1.4**: Initialize logging using `LoggingBuilder` at application startup (before any other logic)
- [ ] **Task 1.5**: Add startup logging event with configuration details
- [ ] **Task 1.6**: Add shutdown logging event
- [ ] **Task 1.7**: Keep existing user-facing output (the informational text about the project)

**Acceptance Criteria**:

- [ ] New `src/app.rs` module created with `run()` function
- [ ] `src/main.rs` is minimal (only calls `app::run()`)
- [ ] CLI accepts `--log-format`, `--log-output`, `--log-dir` arguments
- [ ] Logging is initialized before any application logic
- [ ] Default configuration works without any arguments
- [ ] Log files are created in the specified directory
- [ ] Application logs startup and shutdown events with context (version, configuration)
- [ ] Existing informational output remains unchanged
- [ ] Code follows project module organization conventions

**Example test commands**:

```bash
# Test default behavior
cargo run

# Test custom log directory
cargo run -- --log-dir /tmp/deployer-logs

# Test stderr output
cargo run -- --log-output file-and-stderr

# Test pretty format
cargo run -- --log-format pretty --log-output file-and-stderr

# Test with RUST_LOG for different log levels
RUST_LOG=debug cargo run
RUST_LOG=trace cargo run -- --log-output file-and-stderr
```

### Phase 2: Argument Validation (Estimated: 1 hour)

**Goal**: Ensure log directory is writable and handle errors gracefully

**Tasks**:

- [ ] **Task 2.1**: Add comprehensive help text for all logging-related arguments
  - [ ] `--log-format`: Explain each format option (pretty, json, compact) and when to use them
  - [ ] `--log-output`: Explain file-only vs file-and-stderr modes
  - [ ] `--log-dir`: Explain log directory path and default location
- [ ] **Task 2.2**: Document panic behavior if logging initialization fails
- [ ] **Task 2.3**: Test edge cases
  - [ ] Invalid log directory paths
  - [ ] Permission issues (read-only parent directories)
  - [ ] Non-existent parent directories (should be created automatically)
  - [ ] Relative vs absolute paths

**Acceptance Criteria**:

- [ ] Help text (`--help`) clearly explains each logging option
- [ ] Help text explains that logging is critical and failures will cause application exit
- [ ] Non-existent parent directories are created automatically (via `LoggingBuilder`)
- [ ] Edge cases are tested and documented behavior is clear

**Note**: Current `LoggingBuilder::init()` panics on failure. This is intentional as logging is critical for observability. The panic includes context about the failure. This behavior should be:

- Documented in the `--help` text
- Documented in user-facing guides
- Kept as-is (no need for `try_init()` method for this issue)

### Phase 3: Documentation (Estimated: 1-1.5 hours)

**Goal**: Document logging configuration for users and contributors

**Tasks**:

- [ ] **Task 3.1**: Update README.md with basic logging section
  - [ ] Add "Logging" section explaining default behavior
  - [ ] Link to user guide for detailed information
  - [ ] Mention RUST_LOG environment variable
- [ ] **Task 3.2**: Create `docs/user-guide/logging.md` for end users
  - [ ] Explain logging purpose and default behavior
  - [ ] Document all CLI logging options (`--log-format`, `--log-output`, `--log-dir`)
  - [ ] Provide examples for common scenarios (production, development, troubleshooting)
  - [ ] Explain how to control log levels with `RUST_LOG`
  - [ ] Document log file location and format
- [ ] **Task 3.3**: Create or update `docs/contributing/logging-guide.md` for contributors
  - [ ] Explain how to use tracing in code (info!, debug!, etc.)
  - [ ] Document structured logging conventions (key-value pairs)
  - [ ] Explain the three-level architecture logging pattern (Commands → Steps → Actions)
  - [ ] Provide code examples
- [ ] **Task 3.4**: Verify all links work correctly

**Acceptance Criteria**:

- [ ] README includes logging section with link to detailed user guide
- [ ] User guide (`docs/user-guide/logging.md`) explains all logging options for end users
- [ ] Contributor guide (`docs/contributing/logging-guide.md`) explains how to use logging in code
- [ ] Examples provided for common scenarios (production, development, troubleshooting)
- [ ] Documentation explains `RUST_LOG` environment variable usage
- [ ] All documentation links are valid

### Phase 4: Integration Testing (Estimated: 30-45 minutes)

**Goal**: Verify logging works correctly in all configurations

**Tasks**:

- [ ] **Task 4.1**: Test default configuration
  - [ ] Run without any arguments
  - [ ] Verify log file created at `./data/logs/log.txt`
  - [ ] Verify compact format is used
  - [ ] Verify file-only output (no stderr)
  - [ ] Verify startup and shutdown events are logged
- [ ] **Task 4.2**: Test format options
  - [ ] Test `--log-format pretty`
  - [ ] Test `--log-format json`
  - [ ] Test `--log-format compact`
  - [ ] Verify each produces correctly formatted output
- [ ] **Task 4.3**: Test output modes
  - [ ] Test `--log-output file-only` (verify no stderr output)
  - [ ] Test `--log-output file-and-stderr` (verify both destinations)
- [ ] **Task 4.4**: Test custom log directory
  - [ ] Test with relative path: `--log-dir ./custom-logs`
  - [ ] Test with absolute path: `--log-dir /tmp/deployer-logs`
  - [ ] Verify directories are created automatically
- [ ] **Task 4.5**: Test RUST_LOG environment variable
  - [ ] Test `RUST_LOG=info` (default)
  - [ ] Test `RUST_LOG=debug` (more verbose)
  - [ ] Test `RUST_LOG=trace` (most verbose)
  - [ ] Verify filtering works correctly
- [ ] **Task 4.6**: Test log file persistence
  - [ ] Run application multiple times
  - [ ] Verify log entries are appended (not overwritten)

**Acceptance Criteria**:

- [ ] All format options produce valid log output
- [ ] File-only mode writes to file only (no stderr)
- [ ] File-and-stderr mode writes to both destinations
- [ ] Custom log directories are created and used correctly
- [ ] RUST_LOG filtering works as expected (info, debug, trace levels)
- [ ] Log files persist across application runs (append mode)
- [ ] All tests documented and reproducible

---

## 🧪 Testing Strategy

### Manual Testing

1. **Default behavior**: Run without arguments, verify file-only logging
2. **Custom configurations**: Test each flag individually
3. **Combined flags**: Test multiple flags together
4. **Error scenarios**: Test with invalid paths, no permissions
5. **Environment variables**: Test `RUST_LOG` integration

### Automated Testing

Since this is CLI argument parsing and initialization, automated testing should focus on:

1. **Unit tests**: Test CLI argument parsing (if logic is extracted)
2. **Integration tests**: Existing logging tests in `tests/logging_integration.rs` already cover the logging infrastructure
3. **E2E tests**: Existing E2E test binaries verify end-to-end logging behavior

**No new test files required** - existing test coverage is sufficient.

---

## ✅ Acceptance Criteria Summary

### Functional Requirements

- [ ] Main CLI initializes logging at startup
- [ ] All logging CLI arguments work correctly (`--log-format`, `--log-output`, `--log-dir`)
- [ ] Default configuration uses production-safe settings (file-only, compact)
- [ ] Log files are created in the specified directory
- [ ] Application logs startup and shutdown events with context
- [ ] `RUST_LOG` environment variable is respected
- [ ] Help text clearly explains logging options

### Code Quality

- [ ] Code follows project conventions (module organization, error handling)
- [ ] No code duplication with E2E test binaries
- [ ] Logging infrastructure remains unchanged (no breaking changes)
- [ ] All linters pass (`cargo run --bin linter all`)
- [ ] Pre-commit checks pass (`./scripts/pre-commit.sh`)

### Documentation

- [ ] README.md includes logging configuration section
- [ ] Logging guide document created or updated
- [ ] Help text (`--help`) is clear and comprehensive
- [ ] Examples provided for common scenarios

### User Experience

- [ ] Clear separation between user output and internal logging
- [ ] Sensible defaults for production use
- [ ] Easy to enable verbose logging for troubleshooting
- [ ] Error messages are actionable if logging fails

---

## 🔗 Related Documentation

### Core Documentation

- [Development Principles](../development-principles.md) - Observability and traceability principles
- [User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md) - Architectural rationale
- [User Guide: Logging](../user-guide/logging.md) - End-user documentation on logging options (to be created)
- [Contributor Guide: Logging](../contributing/logging-guide.md) - How to use logging in code (to be created/updated)

### Technical References

- `src/logging.rs` - Logging infrastructure implementation
- `src/bin/e2e_provision_tests.rs` - Example of logging initialization in E2E tests
- `tests/logging_integration.rs` - Integration tests for logging system

### Related Issues

- Roadmap item [1.2: Create destroy command](../roadmap.md#1-add-scaffolding-for-main-app) - Will add destroy command (future work)
- Roadmap item [1.3: Refactor shared code between testing and production](../roadmap.md#1-add-scaffolding-for-main-app) - Potential future work
- Roadmap item [1.4: Improve command abstraction](../roadmap.md#1-add-scaffolding-for-main-app) - Will use this logging setup
- Roadmap item [1.5: Create create command](../roadmap.md#1-add-scaffolding-for-main-app) - Will add create command (future work)
- Roadmap item [1.6: Create deploy command](../roadmap.md#1-add-scaffolding-for-main-app) - Will add deploy command (future work)
- Roadmap item [1.7: Add levels of verbosity](../roadmap.md#1-add-scaffolding-for-main-app) - User output verbosity (separate from logging)

---

## 📝 Implementation Notes

### Dependencies

Ensure `Cargo.toml` includes:

```toml
[dependencies]
clap = { version = "4", features = ["derive", "color", "help", "usage", "error-context", "suggestions"] }
tracing = "0.1"
# ... other dependencies already present
```

### Future Enhancements (Out of Scope)

The following are **not** part of this issue but may be considered in future work:

- **Subcommands**: Will be added in separate issues (provision, configure, deploy, destroy)
- **User output verbosity system**: Roadmap item 1.7 - Implementing `-v`, `-vv`, `-vvv` flags for user-facing output
- **Structured logging events**: Defining common logging events as structured types
- **Log rotation**: Implementing automatic log file rotation for long-running deployments
- **Log aggregation**: Integration with external logging services (CloudWatch, DataDog, etc.)

These should be addressed in separate issues when needed.

---

## 🎯 Success Metrics

The implementation is successful when:

1. **Developers can run the main CLI** with proper logging initialization
2. **Production deployments** have persistent, structured logs for troubleshooting
3. **Debugging is easier** with configurable log formats and output modes
4. **Documentation is clear** on how to configure logging for different scenarios
5. **Zero breaking changes** to existing E2E test binaries and logging infrastructure

---

## 📅 Estimated Effort

**Total**: 4 - 5 hours

- Phase 1 (Basic CLI): 1-2 hours (7 subtasks)
- Phase 2 (Validation): 1 hour (3 subtasks)
- Phase 3 (Documentation): 1-1.5 hours (4 subtasks)
- Phase 4 (Testing): 0.5-0.75 hours (6 subtasks)
- Buffer for unexpected issues: 0.5 hours

---

## 🚀 Next Steps

1. **Review this document**: Get feedback from maintainers/reviewers
2. **Create GitHub issue**: Reference this document in the repository
3. **Start implementation**: Follow the phase-by-phase plan
4. **Iterate on feedback**: Address any review comments during development
5. **Merge and close**: Complete when all acceptance criteria are met
