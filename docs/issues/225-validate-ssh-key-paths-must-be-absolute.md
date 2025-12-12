# Validate SSH Key Paths Must Be Absolute

**Issue**: #225
**Parent Epic**: N/A - Standalone improvement task
**Related**: [docs/contributing/roadmap-issues.md](../contributing/roadmap-issues.md)
**Time Estimate**: 2-3 hours

---

## Overview

The `create environment` command currently accepts relative paths for SSH key files (`private_key_path` and `public_key_path`) in the environment JSON configuration. This causes delayed failures when relative paths are used - the command succeeds during environment creation but fails later during the `configure` command when SSH keys cannot be found from a different working directory. This task implements early validation to reject relative paths immediately with a clear error message, following the "fail fast" principle.

## Goals

- [ ] Reject relative SSH key paths during environment creation
- [ ] Provide clear, actionable error messages explaining why absolute paths are required
- [ ] Update environment template generation to clarify absolute path requirement
- [ ] Add comprehensive unit tests for path validation
- [ ] Update documentation to specify absolute path requirement

## 🏗️ Architecture Requirements

**DDD Layer**: Application (Configuration validation)
**Module Path**: `src/application/command_handlers/create/config/ssh_credentials_config.rs`
**Pattern**: Configuration Value Object with validation

### Module Structure Requirements

- [ ] Validation logic stays in `SshCredentialsConfig::to_ssh_credentials()` method
- [ ] New error variant added to `CreateConfigError` enum
- [ ] Error implements `.help()` with actionable troubleshooting guidance

### Architectural Constraints

```rust
// The validation should happen during configuration conversion to domain types
// in the to_ssh_credentials() method where other SSH validations occur

impl SshCredentialsConfig {
    pub fn to_ssh_credentials(self) -> Result<SshCredentials, CreateConfigError> {
        // 1. Validate paths are absolute (NEW)
        // 2. Convert string paths to PathBuf
        // 3. Validate files exist
        // 4. Convert username to domain type
        // 5. Create domain credentials object
    }
}
```

**Related Files**:

- `src/application/command_handlers/create/config/ssh_credentials_config.rs` - Add validation
- `src/application/command_handlers/create/config/errors.rs` - Add error variant
- `src/presentation/controllers/create/tests/template.rs` - Update template generation
- `envs/environment-template-*.json` - Update template comments

## Specifications

### Current Behavior (Problem)

```json
{
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  }
}
```

**What happens**:

1. `create environment` command accepts the configuration ✅
2. Environment is created successfully ✅
3. Later, `configure` command fails ❌ when trying to use SSH keys from a different working directory

**Root cause**: Path resolution is context-dependent (relative to current working directory)

### Desired Behavior (Solution)

```json
{
  "ssh_credentials": {
    "private_key_path": "/absolute/path/to/testing_rsa",
    "public_key_path": "/absolute/path/to/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  }
}
```

**What should happen**:

1. `create environment` command validates paths are absolute
2. If relative path detected, reject immediately with clear error:

```text
❌ Create environment command failed: SSH key paths must be absolute

Configuration validation failed: SSH private key path must be absolute

Found relative path: fixtures/testing_rsa

SSH key paths must be absolute to ensure they work correctly across
different working directories and command invocations.

How to fix:
1. Convert relative path to absolute path:
   - Current: "fixtures/testing_rsa"
   - Correct: "/home/user/project/fixtures/testing_rsa"

2. Get absolute path from command line:
   realpath fixtures/testing_rsa

3. Update your configuration file with the absolute path

4. Alternative: Use environment variables or home directory expansion
   - ~/.ssh/id_rsa expands to /home/user/.ssh/id_rsa
   - $HOME/.ssh/id_rsa expands based on HOME environment variable

Related documentation:
- Environment configuration guide: docs/user-guide/configuration.md
- SSH key setup: docs/user-guide/ssh-keys.md
```

### Validation Logic

```rust
// In src/application/command_handlers/create/config/ssh_credentials_config.rs

impl SshCredentialsConfig {
    pub fn to_ssh_credentials(self) -> Result<SshCredentials, CreateConfigError> {
        // Convert string username to domain Username type
        let username = Username::new(&self.username)?;

        // Convert string paths to PathBuf
        let private_key_path = PathBuf::from(&self.private_key_path);
        let public_key_path = PathBuf::from(&self.public_key_path);

        // NEW: Validate paths are absolute
        if !private_key_path.is_absolute() {
            return Err(CreateConfigError::RelativePrivateKeyPath {
                path: private_key_path,
            });
        }

        if !public_key_path.is_absolute() {
            return Err(CreateConfigError::RelativePublicKeyPath {
                path: public_key_path,
            });
        }

        // Validate SSH key files exist
        if !private_key_path.exists() {
            return Err(CreateConfigError::PrivateKeyNotFound {
                path: private_key_path,
            });
        }

        if !public_key_path.exists() {
            return Err(CreateConfigError::PublicKeyNotFound {
                path: public_key_path,
            });
        }

        // Create domain credentials object
        Ok(SshCredentials::new(
            private_key_path,
            public_key_path,
            username,
        ))
    }
}
```

### Error Type Additions

```rust
// In src/application/command_handlers/create/config/errors.rs

#[derive(Debug, Error)]
pub enum CreateConfigError {
    // ... existing variants ...

    /// SSH private key path must be absolute
    #[error("SSH private key path must be absolute: {path:?}")]
    RelativePrivateKeyPath { path: PathBuf },

    /// SSH public key path must be absolute
    #[error("SSH public key path must be absolute: {path:?}")]
    RelativePublicKeyPath { path: PathBuf },
}

impl CreateConfigError {
    pub fn help(&self) -> String {
        match self {
            Self::RelativePrivateKeyPath { path } => format!(
                "SSH private key path must be absolute\n\n\
                Found relative path: {}\n\n\
                SSH key paths must be absolute to ensure they work correctly across\n\
                different working directories and command invocations.\n\n\
                How to fix:\n\
                1. Convert relative path to absolute path using: realpath {}\n\
                2. Update configuration file with the absolute path\n\
                3. Alternatively, use ~ for home directory (e.g., ~/.ssh/id_rsa)",
                path.display(),
                path.display()
            ),
            Self::RelativePublicKeyPath { path } => format!(
                "SSH public key path must be absolute\n\n\
                Found relative path: {}\n\n\
                SSH key paths must be absolute to ensure they work correctly across\n\
                different working directories and command invocations.\n\n\
                How to fix:\n\
                1. Convert relative path to absolute path using: realpath {}\n\
                2. Update configuration file with the absolute path\n\
                3. Alternatively, use ~ for home directory (e.g., ~/.ssh/id_rsa.pub)",
                path.display(),
                path.display()
            ),
            // ... existing variants ...
        }
    }
}
```

### Template Updates

Update the generated template files to clarify the absolute path requirement:

```json
{
  "ssh_credentials": {
    "private_key_path": "REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH",
    "public_key_path": "REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH",
    "username": "torrust",
    "port": 22
  }
}
```

The placeholder names already indicate "ABSOLUTE_PATH", but we should also update the template generation guidance message.

## Implementation Plan

### Phase 1: Add Validation Logic (30 min)

- [ ] Add `is_absolute()` check in `SshCredentialsConfig::to_ssh_credentials()` before file existence check
- [ ] Check private key path is absolute
- [ ] Check public key path is absolute
- [ ] Return appropriate error if relative path detected

**Acceptance**: Validation rejects relative paths with appropriate error

### Phase 2: Add Error Variants (30 min)

- [ ] Add `RelativePrivateKeyPath` variant to `CreateConfigError`
- [ ] Add `RelativePublicKeyPath` variant to `CreateConfigError`
- [ ] Implement `.help()` methods with actionable guidance
- [ ] Include `realpath` command suggestion in help text
- [ ] Include alternative solutions (~ expansion, environment variables)

**Acceptance**: Errors provide clear, actionable guidance for users

### Phase 3: Update Tests (45 min)

- [ ] Add unit test for relative private key path rejection
- [ ] Add unit test for relative public key path rejection
- [ ] Add unit test for absolute path acceptance
- [ ] Add unit test for home directory (~) expansion if supported
- [ ] Verify existing tests still pass
- [ ] Follow unit test naming conventions: `it_should_{behavior}_when_{condition}` (see [docs/contributing/testing/unit-testing.md](../contributing/testing/unit-testing.md))

**Example test names**:

```rust
#[test]
fn it_should_reject_config_when_private_key_path_is_relative() { /* ... */ }

#[test]
fn it_should_reject_config_when_public_key_path_is_relative() { /* ... */ }

#[test]
fn it_should_accept_config_when_ssh_key_paths_are_absolute() { /* ... */ }

#[test]
fn it_should_return_clear_error_message_when_relative_path_detected() { /* ... */ }
```

**Acceptance**: All test scenarios covered and passing

### Phase 4: Update Documentation (30 min)

- [ ] Update `create template` command output message to emphasize absolute paths
- [ ] Update any user guide documentation mentioning SSH key configuration
- [ ] Add troubleshooting entry for "SSH key not found" errors

**Acceptance**: Documentation clearly specifies absolute path requirement

### Phase 5: Final Verification (15 min)

- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`
- [ ] Manually test with relative path (should fail)
- [ ] Manually test with absolute path (should succeed)
- [ ] Verify error message clarity and actionability

**Acceptance**: All quality checks pass

## Acceptance Criteria

**Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

### Functional Requirements

- [ ] Relative SSH private key paths are rejected with clear error during `create environment`
- [ ] Relative SSH public key paths are rejected with clear error during `create environment`
- [ ] Absolute SSH key paths continue to work without issues
- [ ] Error messages include specific fix instructions (realpath command, alternatives)
- [ ] Validation happens before file existence check (fail fast on relative paths)

### Code Quality

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All unit tests pass (including new path validation tests)
- [ ] Error types follow project error handling conventions ([docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Code follows project architecture patterns ([docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Module organization follows project conventions ([docs/contributing/module-organization.md](../contributing/module-organization.md))

### User Experience

- [ ] Error messages are clear and actionable (not just "invalid path")
- [ ] Error messages include the problematic relative path value
- [ ] Error messages explain why absolute paths are required
- [ ] Error messages provide specific command to fix (realpath)
- [ ] Error messages suggest alternatives (~ expansion)

### Documentation

- [ ] Template generation clarifies absolute path requirement
- [ ] Any affected user guides updated
- [ ] Troubleshooting guide includes this error scenario

## Related Documentation

- [Configuration Error Handling](../contributing/error-handling.md) - Error message conventions
- [DDD Layer Placement](../contributing/ddd-layer-placement.md) - Where validation belongs
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Development Principles](../development-principles.md) - Observability and user friendliness
- [Unit Testing Conventions](../contributing/testing/unit-testing.md) - Test naming and structure

## Notes

### Why Absolute Paths?

1. **Working Directory Independence**: Commands may be run from different directories
2. **State Persistence**: Environment state stores paths that must remain valid
3. **Multi-command Workflows**: `create` → `provision` → `configure` sequence
4. **Clear Intent**: Absolute paths eliminate ambiguity

### Implementation Considerations

- The validation should occur in `to_ssh_credentials()` where other SSH validations happen
- Check `is_absolute()` **before** file existence to fail fast on incorrect path format
- Home directory expansion (`~`) may require special handling if supported
- Consider adding a helper function if path validation logic becomes complex

### Test Strategy

- Unit tests for validation logic in `ssh_credentials_config.rs`
- Existing E2E tests should continue to work (they use absolute paths)
- Manual testing with relative paths to verify user experience
- Consider adding E2E test for error case if valuable

### Future Enhancements (Out of Scope)

- Auto-conversion of relative to absolute paths (could hide user intent)
- Home directory (`~`) expansion support
- Environment variable expansion in paths
- Path validation for other configuration fields
