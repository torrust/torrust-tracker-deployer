# Fix Destroy Command: Accept Working Directory Parameter

**Epic Subissue**: 9 of 10
**Issue**: [#51](https://github.com/torrust/torrust-tracker-deployer/issues/51)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Create Environment Command
**Depends On**: [#50](https://github.com/torrust/torrust-tracker-deployer/issues/50) - Fix Destroy Command Created State Handling
**Related**: [Destroy Presentation Layer](../../src/presentation/commands/destroy.rs), [Command Dispatcher](../../src/presentation/commands/mod.rs)

## Overview

Fix a critical bug in the destroy command where it doesn't accept or use the `--working-dir` parameter, causing it to fail when environments are created with custom working directories. The destroy command is hardcoded to look for environments in the `data/` directory, ignoring the `--working-dir` CLI argument that the create command properly supports.

This bug prevents users from managing environment lifecycles when working with custom workspace locations, a feature explicitly supported by the `--working-dir` flag in the CLI.

**Dependencies**: This issue depends on [#50](https://github.com/torrust/torrust-tracker-deployer/issues/50) (Created State Handling) being completed first, as manual testing requires being able to destroy Created state environments successfully.

## Goals

- [ ] Fix destroy command to accept `working_dir` parameter
- [ ] Pass `working_dir` through the command execution chain
- [ ] Maintain backward compatibility with default behavior
- [ ] Add comprehensive tests for both default and custom working directories

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (CLI interface)
**Module Path**: `src/presentation/commands/destroy.rs` + `src/presentation/commands/mod.rs`
**Pattern**: CLI Command Pattern

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../../codebase-architecture.md))
- [ ] Presentation layer handles CLI argument passing to command handlers
- [ ] Maintain consistency with create command's working directory handling

### Architectural Constraints

- [ ] Working directory must flow from CLI through to repository factory
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Maintain backward compatibility with default working directory (`.`)

### Anti-Patterns to Avoid

- ❌ Hardcoding directory paths in command handlers
- ❌ Breaking changes to existing destroy behavior with default working directory
- ❌ Inconsistency between create and destroy command working directory handling

## Specifications

### Current Behavior (Incorrect)

The destroy command is hardcoded to look for environments in the `data/` directory:

```rust
// src/presentation/commands/destroy.rs (line 74)
pub fn handle(environment_name: &str) -> Result<(), DestroyError> {
    // ...

    // Create repository for loading environment state
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(std::path::PathBuf::from("data")); // ❌ Hardcoded

    // ...
}
```

The command dispatcher doesn't pass `working_dir` to the destroy handler:

```rust
// src/presentation/commands/mod.rs
pub fn execute(command: Commands, working_dir: &std::path::Path) -> Result<(), CommandError> {
    match command {
        Commands::Create { action } => {
            create::handle_create_command(action, working_dir)?; // ✅ Uses working_dir
            Ok(())
        }
        Commands::Destroy { environment } => {
            destroy::handle(&environment)?; // ❌ Doesn't receive working_dir
            Ok(())
        }
    }
}
```

**Impact**: Environments created with custom `--working-dir` cannot be found by the destroy command:

```bash
# Create environment in custom directory
./torrust-tracker-deployer --working-dir /tmp/workspace create environment --env-file config.json

# Destroy fails - looks in ./data/ instead of /tmp/workspace/data/
./torrust-tracker-deployer --working-dir /tmp/workspace destroy test-env
# Error: Environment not found
```

### Expected Behavior

The destroy command should accept and use the `working_dir` parameter just like the create command:

**Step 1**: Update `destroy::handle()` signature to accept `working_dir`:

```rust
// src/presentation/commands/destroy.rs
pub fn handle(environment_name: &str, working_dir: &std::path::Path) -> Result<(), DestroyError> {
    // Create user output with default stdout/stderr channels
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    // Display initial progress (to stderr)
    output.progress(&format!("Destroying environment '{environment_name}'..."));

    // Validate environment name
    let env_name = EnvironmentName::new(environment_name.to_string()).map_err(|source| {
        let error = DestroyError::InvalidEnvironmentName {
            name: environment_name.to_string(),
            source,
        };
        output.error(&error.to_string());
        error
    })?;

    // Create repository for loading environment state - use working_dir parameter
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(working_dir.to_path_buf()); // ✅ Uses working_dir

    // Create clock for timing information
    let clock = std::sync::Arc::new(crate::shared::SystemClock);

    // Create and execute destroy command handler
    output.progress("Tearing down infrastructure...");

    let command_handler = DestroyCommandHandler::new(repository, clock);

    // Execute destroy - the handler will load the environment and handle all states internally
    let _destroyed_env = command_handler.execute(&env_name).map_err(|source| {
        let error = DestroyError::DestroyOperationFailed {
            name: environment_name.to_string(),
            source,
        };
        output.error(&error.to_string());
        error
    })?;

    output.progress("Cleaning up resources...");
    output.success(&format!(
        "Environment '{environment_name}' destroyed successfully"
    ));

    Ok(())
}
```

**Step 2**: Update the command dispatcher to pass `working_dir`:

```rust
// src/presentation/commands/mod.rs
pub fn execute(command: Commands, working_dir: &std::path::Path) -> Result<(), CommandError> {
    match command {
        Commands::Create { action } => {
            create::handle_create_command(action, working_dir)?;
            Ok(())
        }
        Commands::Destroy { environment } => {
            destroy::handle(&environment, working_dir)?; // ✅ Pass working_dir
            Ok(())
        }
    }
}
```

### Consistency with Create Command

The fix ensures destroy behaves exactly like create:

```rust
// Create command (existing pattern)
create::handle_create_command(action, working_dir)?;

// Destroy command (after fix)
destroy::handle(&environment, working_dir)?;
```

Both commands now:

- Accept `working_dir` parameter from CLI
- Pass it to repository factory
- Support custom workspace locations
- Default to `.` (current directory) when not specified

## Implementation Plan

### Subtask 1: Update Presentation Layer (30 minutes)

- [ ] Modify `destroy::handle()` signature to accept `working_dir: &Path`
- [ ] Update repository creation to use the provided working directory
- [ ] Verify compilation succeeds

### Subtask 2: Update Command Dispatcher (30 minutes)

- [ ] Modify `commands::handle_command()` to pass `working_dir` to `destroy::handle()`
- [ ] Verify integration with CLI argument parsing
- [ ] Test with both default and custom working directories

### Subtask 3: Testing (2-3 hours)

- [ ] Add unit test: destroy with custom working directory
- [ ] Add integration test: create → destroy with default working directory
- [ ] Add integration test: create → destroy with custom working directory (temp dir)
- [ ] Add integration test: create → provision → destroy with custom working directory
- [ ] Update existing tests if needed
- [ ] Verify all pre-commit checks pass

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Bug Fix - Working Directory Support**:

- [ ] Destroy command accepts `working_dir` parameter in its signature
- [ ] `working_dir` is properly passed from CLI through command dispatcher to destroy handler
- [ ] Repository factory uses provided `working_dir` instead of hardcoded `"data"`
- [ ] Default behavior unchanged (works with environments in `./data/` when using default `--working-dir .`)
- [ ] Manual Test 1 - Default working directory:

  ```bash
  # Create test configuration
  cat > /tmp/test-default.json << 'EOF'
  {
    "environment": {
      "name": "test-default"
    },
    "ssh_credentials": {
      "private_key_path": "fixtures/testing_rsa",
      "public_key_path": "fixtures/testing_rsa.pub",
      "username": "torrust",
      "port": 22
    }
  }
  EOF

  # Create environment in default location (current directory)
  ./target/release/torrust-tracker-deployer create environment --env-file /tmp/test-default.json --log-output file-and-stderr

  # Verify environment created in ./data/test-default/
  [ -f ./data/test-default/environment.json ] && echo "✅ Environment created in default location"

  # Destroy should work without specifying working-dir (uses default: .)
  ./target/release/torrust-tracker-deployer destroy test-default --log-output file-and-stderr

  # Verify environment removed
  [ ! -d ./data/test-default ] && echo "✅ Environment destroyed successfully"

  # Cleanup
  rm /tmp/test-default.json
  ```

- [ ] Manual Test 2 - Custom working directory:

  ```bash
  # Create test configuration
  cat > /tmp/test-custom.json << 'EOF'
  {
    "environment": {
      "name": "test-custom"
    },
    "ssh_credentials": {
      "private_key_path": "fixtures/testing_rsa",
      "public_key_path": "fixtures/testing_rsa.pub",
      "username": "torrust",
      "port": 22
    }
  }
  EOF

  # Create environment in temporary directory
  TEMP_DIR=$(mktemp -d)
  echo "Using temp directory: $TEMP_DIR"

  ./target/release/torrust-tracker-deployer --working-dir "$TEMP_DIR" create environment --env-file /tmp/test-custom.json --log-output file-and-stderr

  # Verify environment created in temp directory
  [ -f "$TEMP_DIR/data/test-custom/environment.json" ] && echo "✅ Environment created in custom location"

  # Destroy should work with same working-dir
  ./target/release/torrust-tracker-deployer --working-dir "$TEMP_DIR" destroy test-custom --log-output file-and-stderr

  # Verify environment removed
  [ ! -d "$TEMP_DIR/data/test-custom" ] && echo "✅ Environment destroyed successfully"

  # Cleanup
  rm -rf "$TEMP_DIR"
  rm /tmp/test-custom.json
  ```

- [ ] Manual Test 3 - Full lifecycle with custom working directory:

  ```bash
  # This test verifies create → provision → destroy works with custom working directory
  # Note: This test requires LXD to be available and configured

  # Create test configuration
  cat > /tmp/test-lifecycle.json << 'EOF'
  {
    "environment": {
      "name": "test-lifecycle"
    },
    "ssh_credentials": {
      "private_key_path": "fixtures/testing_rsa",
      "public_key_path": "fixtures/testing_rsa.pub",
      "username": "torrust",
      "port": 22
    }
  }
  EOF

  # Create temporary workspace
  TEMP_DIR=$(mktemp -d)
  echo "Using temp directory: $TEMP_DIR"

  # Create environment
  ./target/release/torrust-tracker-deployer --working-dir "$TEMP_DIR" create environment --env-file /tmp/test-lifecycle.json

  # Provision environment (currently provision command may not support --working-dir, use default for now)
  # TODO: Update this test when provision command also supports --working-dir
  # For now, this test documents the expected behavior

  # Destroy should work with custom working-dir
  ./target/release/torrust-tracker-deployer --working-dir "$TEMP_DIR" destroy test-lifecycle --log-output file-and-stderr

  # Verify environment removed
  [ ! -d "$TEMP_DIR/data/test-lifecycle" ] && echo "✅ Environment destroyed successfully"

  # Cleanup
  rm -rf "$TEMP_DIR"
  rm /tmp/test-lifecycle.json
  ```

**Integration**:

- [ ] Destroy command works correctly with default working directory (backward compatibility)
- [ ] Destroy command works correctly with custom working directories
- [ ] Destroy command behavior matches create command's working directory handling
- [ ] Error messages are clear when environment is not found in the specified working directory

## Related Documentation

- [Destroy Presentation Layer](../../src/presentation/commands/destroy.rs)
- [Create Presentation Layer](../../src/presentation/commands/create/subcommand.rs) - Reference for working directory handling pattern
- [Command Dispatcher](../../src/presentation/commands/mod.rs)
- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [#50](https://github.com/torrust/torrust-tracker-deployer/issues/50) - Fix Destroy Command Created State Handling (prerequisite)

## Notes

### Discovery Context

This bug was discovered during manual testing of the create environment command (Issue #34):

1. Created a new environment using the create command with default `--working-dir .`
2. Environment was created at `./my-test/environment.json` (root-level data directory)
3. Attempted to destroy the environment
4. Destroy command failed with "Environment not found" because it was hardcoded to look in `data/my-test/`
5. After manually moving the environment to `data/my-test/`, the destroy command could find it
6. This revealed the destroy command doesn't accept or use the `--working-dir` parameter

### Why This Issue Depends on Issue #50

Manual testing for this issue requires creating environments and then destroying them. To test properly, we need:

1. **Issue #50 completed**: Ensures destroy works for Created state environments
2. **This issue**: Ensures destroy works with custom working directories

Without Issue #50 fixed, testing would fail because:

- Create environment (Created state)
- Destroy would fail due to Created state bug (not the working directory bug)
- We couldn't verify if the working directory fix works correctly

By completing Issue #50 first, we ensure that any failures in this issue are truly related to working directory handling, not state handling.

### Consistency Across Commands

After this fix, working directory handling will be consistent:

**Before**:

- ✅ `create` command: Uses `--working-dir` parameter
- ❌ `destroy` command: Ignores `--working-dir`, hardcoded to `data/`

**After**:

- ✅ `create` command: Uses `--working-dir` parameter
- ✅ `destroy` command: Uses `--working-dir` parameter

**Future work**: Other commands (`provision`, `configure`, etc.) should also be audited for consistent working directory support.

### Backward Compatibility

This fix maintains backward compatibility:

- **Default behavior**: When `--working-dir` is not specified, it defaults to `.` (current directory)
- **Existing workflows**: All existing commands that don't specify `--working-dir` continue to work exactly as before
- **Data location**: Environments are still found at `./data/{ENV_NAME}/` by default

The fix only changes behavior when users **explicitly** specify a custom `--working-dir`, which currently doesn't work for destroy.

### Implementation Simplicity

The fix is straightforward:

1. Add one parameter to function signature: `working_dir: &Path`
2. Change one line: `repository_factory.create(working_dir.to_path_buf())`
3. Update one call site: `destroy::handle(&environment, working_dir)?`

**Total lines changed**: ~3 lines of code

This is a minimal, low-risk change that significantly improves user experience when working with multiple workspaces.
