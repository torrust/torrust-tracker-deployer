# Fix Destroy Command: Handle Created State Gracefully

**Epic Subissue**: 8 of 10
**Issue**: [#50](https://github.com/torrust/torrust-tracker-deployer/issues/50)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Create Environment Command
**Related**: [Destroy Command Handler](../../src/application/command_handlers/destroy.rs), [Destroy Presentation Layer](../../src/presentation/commands/destroy.rs), [#51](https://github.com/torrust/torrust-tracker-deployer/issues/51) - Fix Working Directory Support

## Overview

Fix a critical bug in the destroy command where it fails when attempting to destroy environments in the `Created` state. The command unconditionally tries to destroy OpenTofu infrastructure that was never provisioned, causing the operation to fail with "No such file or directory" errors.

This bug prevents users from cleaning up newly created environments that were never provisioned, forcing manual cleanup or confusing error messages.

**Note**: This issue focuses only on the Created state handling bug. A separate issue (Subissue 9) will address the working directory parameter support. For manual testing in this issue, use the **default working directory** (environments in `./data/`) to avoid the working directory bug.

## Goals

- [ ] Fix destroy command to handle `Created` state environments gracefully
- [ ] Skip infrastructure destruction for environments that were never provisioned
- [ ] Maintain backward compatibility with provisioned environment destruction
- [ ] Add comprehensive tests for both Created and Provisioned state scenarios

## 🏗️ Architecture Requirements

**DDD Layer**: Application (command logic)
**Module Path**: `src/application/command_handlers/destroy.rs`
**Pattern**: Command Handler Pattern

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../../codebase-architecture.md))
- [ ] Application layer handles state-aware logic
- [ ] Use existing error handling patterns with `.help()` methods

### Architectural Constraints

- [ ] Destroy command handler must be state-aware
- [ ] Infrastructure destruction should only happen when infrastructure exists
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Maintain idempotency of destroy operation

### Anti-Patterns to Avoid

- ❌ Assuming infrastructure always exists for all states
- ❌ Breaking changes to existing destroy behavior for provisioned environments
- ❌ Silent failures (always log when skipping infrastructure destruction)

## Specifications

### Current Behavior (Incorrect)

When destroying an environment in the `Created` state (never provisioned), the destroy command unconditionally attempts to run OpenTofu destroy operations:

```rust
// src/application/command_handlers/destroy.rs (execute_destruction_with_tracking)
fn execute_destruction_with_tracking(...) -> Result<...> {
    // Step 1: Destroy infrastructure via OpenTofu
    Self::destroy_infrastructure(opentofu_client)  // ❌ Fails for Created state
        .map_err(|e| (e, DestroyStep::DestroyInfrastructure))?;

    // Step 2: Clean up state files
    Self::cleanup_state_files(environment)
        .map_err(|e| (e, DestroyStep::CleanupStateFiles))?;

    Ok(())
}
```

This causes errors like:

```text
Failed to start command 'tofu destroy -var-file=variables.tfvars -auto-approve':
No such file or directory (os error 2)
```

The environment transitions to `DestroyFailed` state with this error context:

```json
{
  "DestroyFailed": {
    "context": { ... },
    "state": {
      "context": {
        "failed_step": "DestroyInfrastructure",
        "error_kind": "CommandExecution",
        "error_summary": "Command execution failed: Failed to start command 'tofu destroy ...",
        ...
      }
    }
  }
}
```

### Expected Behavior

The destroy command should be state-aware and skip infrastructure destruction for environments that never had infrastructure provisioned:

```rust
fn execute_destruction_with_tracking(
    environment: &Environment<Destroying>,
    opentofu_client: &Arc<OpenTofuClient>,
) -> Result<(), (DestroyCommandHandlerError, DestroyStep)> {
    // Only attempt infrastructure destruction if infrastructure was provisioned
    // States that had infrastructure: Provisioned, Configured, Released, Running
    // States that never had infrastructure: Created
    if should_destroy_infrastructure(environment) {
        info!(
            environment = %environment.name(),
            "Destroying provisioned infrastructure"
        );
        Self::destroy_infrastructure(opentofu_client)
            .map_err(|e| (e, DestroyStep::DestroyInfrastructure))?;
    } else {
        info!(
            environment = %environment.name(),
            "Skipping infrastructure destruction (environment was never provisioned)"
        );
    }

    // Always clean up state files
    Self::cleanup_state_files(environment)
        .map_err(|e| (e, DestroyStep::CleanupStateFiles))?;

    Ok(())
}

fn should_destroy_infrastructure(environment: &Environment<Destroying>) -> bool {
    // Check if the environment ever reached a provisioned state by checking
    // if the OpenTofu build directory exists with state files
    let tofu_build_dir = environment.tofu_build_dir();
    tofu_build_dir.exists()
}
```

### State Transition Context

The environment state machine shows these possible states before `Destroying`:

- `Created` → `Destroying` (never provisioned - **skip infrastructure destroy**)
- `Provisioning` → `Destroying` (provisioning in progress - **attempt infrastructure destroy**)
- `Provisioned` → `Destroying` (provisioned - **attempt infrastructure destroy**)
- `Configuring` → `Destroying` (configuring - **attempt infrastructure destroy**)
- `Configured` → `Destroying` (configured - **attempt infrastructure destroy**)
- And other states...

**Key Insight**: Only the `Created` state means no infrastructure was ever created. All other states should attempt infrastructure destruction.

### Implementation Strategy

The fix uses a simple and reliable heuristic: **check if the OpenTofu build directory exists** before attempting infrastructure destruction.

**Why This Works**:

- OpenTofu creates the build directory during provisioning
- If the directory doesn't exist, no infrastructure was provisioned
- This works reliably across all states and edge cases
- Handles manual directory deletion gracefully
- Maintains idempotency (can run destroy multiple times)

```rust
// In src/application/command_handlers/destroy.rs

fn should_destroy_infrastructure(environment: &Environment<Destroying>) -> bool {
    let tofu_build_dir = environment.tofu_build_dir();
    tofu_build_dir.exists()
}
```

**Alternative Approaches Considered and Rejected**:

1. **State Tracking Flag**: Adding a `was_provisioned` boolean to environment context

   - ❌ Requires changing environment state structure
   - ❌ More complex to implement and maintain
   - ❌ Harder to handle edge cases

2. **Checking Specific State Names**: Only attempt destroy for known provisioned states
   - ❌ Brittle - breaks when new states are added
   - ❌ Harder to maintain
   - ❌ Doesn't handle edge cases (manual state changes)

The directory existence check is simpler, more reliable, and handles edge cases better.

## Implementation Plan

### Subtask 1: Add Infrastructure Existence Check (1 hour)

- [ ] Add `should_destroy_infrastructure()` helper function to `DestroyCommandHandler`
- [ ] Implement infrastructure existence check using `environment.tofu_build_dir().exists()`
- [ ] Add comprehensive logging for both cases (destroying vs skipping)

### Subtask 2: Update Destruction Logic (1 hour)

- [ ] Update `execute_destruction_with_tracking()` to conditionally destroy infrastructure
- [ ] Ensure state file cleanup always happens regardless of infrastructure state
- [ ] Verify error handling and state transitions remain correct

### Subtask 3: Testing (2-3 hours)

- [ ] Add unit test: destroy environment in Created state (no infrastructure)
- [ ] Add unit test: destroy environment in Provisioned state (has infrastructure)
- [ ] Add integration test: create → destroy (Created state lifecycle)
- [ ] Add integration test: create → provision → destroy (full lifecycle - regression test)
- [ ] Update E2E tests if needed
- [ ] Verify all pre-commit checks pass

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Bug Fix - Created State Handling**:

- [ ] Destroy command succeeds for environments in `Created` state
- [ ] Infrastructure destruction is skipped for `Created` state environments
- [ ] State files are cleaned up regardless of infrastructure state
- [ ] Appropriate log messages indicate why infrastructure destruction was skipped
- [ ] Manual Test 1 - Destroy Created state environment (default working directory):

  ```bash
  # Note: Use default working directory to avoid Issue #51 (working directory bug)
  # Create test configuration
  cat > /tmp/test-config.json << 'EOF'
  {
    "environment": {
      "name": "test-created"
    },
    "ssh_credentials": {
      "private_key_path": "fixtures/testing_rsa",
      "public_key_path": "fixtures/testing_rsa.pub",
      "username": "torrust",
      "port": 22
    }
  }
  EOF

  # Create environment in default location (Created state, no provisioning)
  ./target/release/torrust-tracker-deployer create environment --env-file /tmp/test-config.json --log-output file-and-stderr

  # Verify environment in Created state
  cat ./data/test-created/environment.json | jq -r 'keys[0]'
  # Expected output: "Created"

  # Verify no OpenTofu build directory exists
  [ ! -d ./build/test-created/tofu ] && echo "✅ No infrastructure provisioned"

  # Destroy should succeed without trying to destroy non-existent infrastructure
  ./target/release/torrust-tracker-deployer destroy test-created --log-output file-and-stderr
  # Expected: Success with log message "Skipping infrastructure destruction (environment was never provisioned)"

  # Verify environment removed
  [ ! -d ./data/test-created ] && echo "✅ Environment destroyed successfully"

  # Cleanup
  rm /tmp/test-config.json
  ```

- [ ] Manual Test 2 - Destroy Provisioned state environment (regression test):

  ```bash
  # This test ensures the fix doesn't break normal destroy for provisioned environments
  # Note: This test requires LXD to be available and configured

  # Create test configuration
  cat > /tmp/test-provisioned.json << 'EOF'
  {
    "environment": {
      "name": "test-provisioned"
    },
    "ssh_credentials": {
      "private_key_path": "fixtures/testing_rsa",
      "public_key_path": "fixtures/testing_rsa.pub",
      "username": "torrust",
      "port": 22
    }
  }
  EOF

  # Create and provision environment
  ./target/release/torrust-tracker-deployer create environment --env-file /tmp/test-provisioned.json
  ./target/release/torrust-tracker-deployer provision test-provisioned --log-output file-and-stderr

  # Verify provisioned state
  cat ./data/test-provisioned/environment.json | jq -r 'keys[0]'
  # Expected output: "Provisioned" or later state

  # Verify OpenTofu build directory exists
  [ -d ./build/test-provisioned/tofu ] && echo "✅ Infrastructure provisioned"

  # Destroy should attempt infrastructure destruction
  ./target/release/torrust-tracker-deployer destroy test-provisioned --log-output file-and-stderr
  # Expected: Log message "Destroying provisioned infrastructure", then infrastructure destruction, then cleanup

  # Verify environment removed
  [ ! -d ./data/test-provisioned ] && echo "✅ Environment destroyed successfully"

  # Cleanup
  rm /tmp/test-provisioned.json
  ```

**Integration**:

- [ ] Destroy command works correctly in complete environment lifecycle (create → provision → destroy)
- [ ] Destroy command is idempotent (running twice doesn't fail)
- [ ] Error messages are clear and actionable when problems occur
- [ ] Log messages clearly indicate whether infrastructure destruction was attempted or skipped

## Related Documentation

- [Destroy Command Handler](../../src/application/command_handlers/destroy.rs)
- [Environment State Machine](../../src/domain/environment/mod.rs)
- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- Subissue 9: Fix Destroy Command Working Directory Support (to be created)

## Notes

### Discovery Context

This bug was discovered during manual testing of the create environment command (Issue #34):

1. Created a new environment using the create command (environment in `Created` state)
2. Attempted to destroy the environment immediately (without provisioning)
3. Destroy command failed with error: `Failed to start command 'tofu destroy -var-file=variables.tfvars -auto-approve': No such file or directory (os error 2)`
4. The command tried to run OpenTofu destroy even though no infrastructure was ever provisioned
5. Environment transitioned to `DestroyFailed` state instead of being properly destroyed

### Relationship to Working Directory Issue

**Important**: This issue focuses **only** on the Created state handling. There is a separate bug where the destroy command doesn't accept the `--working-dir` parameter (Subissue 9, to be created).

**For manual testing in this issue**: Always use the **default working directory** (environments created in `./data/`) to avoid the working directory bug. The working directory issue will be addressed separately.

### Why Directory Existence Check Works

The implementation uses directory existence as a reliable indicator:

```rust
fn should_destroy_infrastructure(environment: &Environment<Destroying>) -> bool {
    environment.tofu_build_dir().exists()
}
```

**This approach**:

- ✅ Works for all states correctly
- ✅ Handles edge cases (manual directory deletion, etc.)
- ✅ Requires minimal code changes
- ✅ Doesn't require modifying the environment state structure
- ✅ Maintains idempotency (multiple destroy attempts work)
- ✅ Simple to understand and maintain

### Backward Compatibility

This fix maintains backward compatibility:

- Environments that were provisioned (have infrastructure) → Infrastructure destruction **attempted** (current behavior)
- Environments in Created state (no infrastructure) → Infrastructure destruction **skipped** (new behavior, fixes current failure)

No existing workflows are affected by this change. The fix only changes behavior for environments that currently fail to destroy.

### State File Cleanup

Regardless of whether infrastructure destruction is attempted or skipped, the state file cleanup **always happens**:

```rust
// Always clean up state files
Self::cleanup_state_files(environment)
    .map_err(|e| (e, DestroyStep::CleanupStateFiles))?;
```

This ensures:

- Environment directories are removed (`./data/{ENV_NAME}/`)
- Build directories are removed (`./build/{ENV_NAME}/`)
- No orphaned state files remain
- Clean environment removal in all cases
