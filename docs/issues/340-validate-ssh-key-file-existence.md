# Validate Command Should Not Check SSH Key File Existence

**Issue**: #340
**Parent Epic**: N/A
**Related**: [Validate Command Documentation](../user-guide/commands/validate.md)

## Overview

The `validate` command currently checks whether SSH key files exist on the filesystem. This is incorrect - the validate command should only perform **syntactic validation** (checking that paths are absolute), not **runtime validation** (checking that files exist).

**Current Behavior**:

- ✅ Correctly rejects relative paths: `fixtures/testing_rsa` → Error: "SSH private key path must be absolute"
- ❌ **Bug**: Rejects absolute paths if files don't exist: `/home/user/.ssh/id_rsa` → Error: "SSH private key file not found"

**Expected Behavior**: The validate command should:

- ✅ Reject relative paths like `fixtures/testing_rsa` (already working)
- ✅ Accept absolute paths like `/home/user/.ssh/id_rsa` **without checking if files exist**
- ⚠️ File existence validation should only happen in `create environment` command

**Rationale**: The validate command performs syntactic validation only - it checks configuration structure and constraints in isolation. File existence is a runtime validation that depends on the current system state and should only be checked when actually creating the environment.

## Goals

- [ ] Remove file existence check from validate command
- [ ] Keep absolute path validation (already working correctly)
- [ ] Ensure file existence check only happens in `create environment` command
- [ ] Add unit tests confirming validation passes for non-existent absolute paths
- [ ] Update E2E tests to verify both behaviors

## 🏗️ Architecture Requirements

**DDD Layer**: Application
**Module Path**: `src/application/command_handlers/validate/` or `src/application/command_handlers/create/config/ssh_credentials_config.rs`
**Pattern**: DTO validation / Command handler

### Module Structure Requirements

- [ ] Follow DDD layer separation (validation may happen in DTO → Domain conversion)
- [ ] Respect dependency flow rules (Application → Domain)
- [ ] Use appropriate error types (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))

### Architectural Constraints

- [ ] Validation logic should be testable in isolation
- [ ] Error messages must be actionable and user-friendly (see [docs/development-principles.md](../docs/development-principles.md))
- [ ] No file I/O during validate command (syntactic validation only)

### Anti-Patterns to Avoid

- ❌ Checking file existence in validate command (should only be in create environment)
- ❌ Generic error messages without actionable guidance
- ❌ Mixing syntactic and runtime validation concerns

## Specifications

### Current Validation Behavior

The validate command currently performs TWO checks:

1. ✅ **Path must be absolute** - Working correctly, should be kept
2. ❌ **File must exist** - Bug: This should only happen in create environment command

**Test Results:**

```bash
# Test 1: Relative paths (correctly rejected)
$ cargo run -- validate --env-file envs/bug-validation-invalid-relative-paths.json
❌ Error: SSH private key path must be absolute: "fixtures/testing_rsa"

# Test 2: Absolute paths to non-existent files (incorrectly rejected - this is the bug)
$ cargo run -- validate --env-file envs/bug-validation-valid-absolute-paths.json
❌ Error: SSH private key file not found: /home/user/.ssh/id_rsa
```

### Required Fix

**Remove file existence check from validate command** while keeping the absolute path validation.

**After fix, expected behavior:**

```bash
# Test 1: Relative paths (should still be rejected)
$ cargo run -- validate --env-file envs/bug-validation-invalid-relative-paths.json
❌ Error: SSH private key path must be absolute: "fixtures/testing_rsa"

# Test 2: Absolute paths to non-existent files (should now pass)
$ cargo run -- validate --env-file envs/bug-validation-valid-absolute-paths.json
✅ Configuration file 'envs/bug-validation-valid-absolute-paths.json' is valid
```

### Test Configurations

Two minimal test configurations are provided in `envs/`:

**Valid (absolute paths)**: [`envs/bug-validation-valid-absolute-paths.json`](../../envs/bug-validation-valid-absolute-paths.json)

```json
{
  "environment": {
    "name": "bug-test-valid"
  },
  "ssh_credentials": {
    "private_key_path": "/home/user/.ssh/id_rsa",
    "public_key_path": "/home/user/.ssh/id_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-bug-test"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [],
    "http_trackers": [],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "health_check_api": {
      "bind_address": "127.0.0.1:1313"
    }
  }
}
```

**Invalid (relative paths)**: [`envs/bug-validation-invalid-relative-paths.json`](../../envs/bug-validation-invalid-relative-paths.json)

```json
{
  "environment": {
    "name": "bug-test-invalid"
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-bug-test"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [],
    "http_trackers": [],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "health_check_api": {
      "bind_address": "127.0.0.1:1313"
    }
  }
}
```

### How to Reproduce

```bash
# Test 1: Relative paths (correctly rejected)
cargo run --bin torrust-tracker-deployer -- validate --env-file envs/bug-validation-invalid-relative-paths.json
# Expected: ❌ Error: SSH private key path must be absolute: "fixtures/testing_rsa"
# Actual: ❌ Error: SSH private key path must be absolute: "fixtures/testing_rsa" ✓

# Test 2: Absolute paths to non-existent files (BUG - incorrectly rejected)
cargo run --bin torrust-tracker-deployer -- validate --env-file envs/bug-validation-valid-absolute-paths.json
# Expected: ✅ Configuration file is valid
# Actual: ❌ Error: SSH private key file not found: /home/user/.ssh/id_rsa (BUG)
```

### Implementation Location

The file existence check is likely in one of these locations:

1. `src/adapters/ssh/credentials.rs` - SshCredentials construction
2. `src/application/command_handlers/validate/handler.rs` - Validate command handler
3. `src/application/command_handlers/create/config/ssh_credentials_config.rs` - TryFrom conversion

**Strategy**: Add a parameter or flag to skip file existence check when called from validate command, but still check when called from create environment command.

## Implementation Plan

### Phase 1: Locate File Existence Check (15 minutes)

- [ ] Search codebase for file existence validation logic
- [ ] Identify where `SshCredentials` checks if files exist
- [ ] Determine if check is in domain layer, application layer, or adapter layer
- [ ] Document current flow: validate command → ... → file existence check

### Phase 2: Refactor to Skip File Check in Validate (45 minutes)

- [ ] Add parameter/flag to skip file existence check (e.g., `validate_existence: bool`)
- [ ] Update `SshCredentials::new()` or conversion method to accept flag
- [ ] Validate command passes `validate_existence: false`
- [ ] Create environment command passes `validate_existence: true`
- [ ] Ensure absolute path validation still runs in both cases

### Phase 3: Unit Tests (30 minutes)

- [ ] Add test: `it_should_accept_absolute_paths_without_checking_existence`
- [ ] Add test: `it_should_still_reject_relative_paths`
- [ ] Add test: `it_should_check_file_existence_when_flag_is_true` (for create command)
- [ ] Verify tests follow naming conventions from [docs/contributing/testing/unit-testing.md](../docs/contributing/testing/unit-testing.md)

### Phase 4: E2E Tests (30 minutes)

- [ ] Test using `bug-validation-valid-absolute-paths.json` (should now pass)
- [ ] Test using `bug-validation-invalid-relative-paths.json` (should still fail)
- [ ] Verify create environment command still checks file existence
- [ ] Update `tests/e2e/validate_command.rs` with regression tests

### Phase 5: Documentation (15 minutes)

- [ ] Update [docs/user-guide/commands/validate.md](../user-guide/commands/validate.md) to clarify validation scope
- [ ] Document that validate checks path format only, not file existence
- [ ] Add example showing this distinction
- [ ] Update error message examples if needed

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] ✅ Validate command accepts absolute SSH key paths without checking file existence
- [ ] ❌ Validate command still rejects relative SSH key paths with clear error
- [ ] ✅ Create environment command still checks if SSH key files exist
- [ ] Unit tests cover both validate (no file check) and create (with file check) scenarios
- [ ] E2E tests verify `bug-validation-valid-absolute-paths.json` now passes validation
- [ ] E2E tests verify `bug-validation-invalid-relative-paths.json` still fails validation
- [ ] Documentation clarifies validation scope (syntactic vs runtime checks)

## Related Documentation

- [Validate Command Documentation](../user-guide/commands/validate.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Unit Testing Conventions](../contributing/testing/unit-testing.md)
- [Config Validation Feature Questions](../features/config-validation-command/questions.md)

## Notes

### What Was Discovered

Testing revealed that the validate command already correctly validates that SSH paths must be absolute. The bug is that it **also checks file existence**, which it shouldn't do.

**Test Results**:

- `bug-validation-invalid-relative-paths.json` → Correctly fails: "SSH private key path must be absolute"
- `bug-validation-valid-absolute-paths.json` → Incorrectly fails: "SSH private key file not found"

The second case is the bug - it should pass validation because the path is absolute, regardless of whether the file exists.

### Why Not Check File Existence in Validate?

The validate command performs **syntactic validation** only - it checks the configuration structure and field constraints in isolation. File existence is a **runtime validation** that depends on the current system state and should only be checked when actually creating the environment.

This separation aligns with the three levels of validation documented in [docs/features/config-validation-command/questions.md](../features/config-validation-command/questions.md):

1. **Syntactic** ✅ - JSON structure, types, required fields, absolute paths (path format)
2. **Config-intrinsic semantics** ✅ - Cross-field rules (e.g., Grafana requires Prometheus)
3. **State-dependent semantics** ❌ - Depends on current app/system state (file existence, environment name conflicts)

The validate command implements levels 1 and 2, while level 3 is deferred to the create environment command.

### Real-World Use Case

A common workflow is:

1. User writes config on Machine A with SSH keys at `/home/alice/.ssh/id_rsa`
2. User validates config on Machine A (should pass)
3. User commits config to git repository
4. CI/CD pipeline validates config on Machine B where `/home/alice/.ssh/id_rsa` doesn't exist (should still pass syntactic validation)
5. Deployment runs on Machine C where the actual SSH keys exist (file existence checked here)

The current behavior breaks step 4 - CI/CD validation incorrectly fails even though the config syntax is correct.
