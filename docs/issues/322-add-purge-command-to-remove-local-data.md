# Add Purge Command to Remove Local Environment Data

**Issue**: #322
**Related Roadmap Section**: 10. Improve usability (UX)
**Estimated Time**: 4-6 hours

## Summary

Add a `purge` command that removes local data for environments in any state, with proper confirmation to prevent accidental data loss.

## Architecture

- **DDD Layer**: Application (command handler), Presentation (CLI subcommand)
- **Module Path**: `src/application/command_handlers/purge/`
- **Related Modules**:
  - `src/presentation/cli/` (CLI command definition)
  - `src/application/steps/` (purge step implementation)
- **Constraints**:
  - Follow existing command handler pattern (see `destroy` command)
  - Use confirmation prompt in presentation layer
  - Implement atomic file operations for safety

## Context

When the `destroy` command runs, it:

1. Tears down the real infrastructure (VMs, networks, etc.)
2. Changes the environment state to "Destroyed"
3. **Preserves local data** in the `data/` folder

**Why local data is preserved:**

- If destruction fails, users can access environment data to debug
- Contains configuration, state, and generated artifacts
- Allows inspection of what was deployed

**Problem:**

- If destruction succeeds, the environment name is still "taken"
- User cannot create a new environment with the same name
- Forces manual cleanup (removing `data/{env-name}` folder)
- Users shouldn't need to know internal storage details

## Proposed Solution

Add a `purge` command with clear separation from `destroy`:

| Command   | What it does                                  | When to use                                     |
| --------- | --------------------------------------------- | ----------------------------------------------- |
| `destroy` | Destroys infrastructure, keeps local data     | Normal teardown                                 |
| `purge`   | Removes local data for destroyed environments | After destroy, to reuse name or free disk space |

### Command Usage

```bash
# After destroying an environment
torrust-tracker-deployer purge lxd-local-example
```

### Example Output

**Successful purge:**

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment 'lxd-local-example' is in Destroyed state (took 0ms)
⏳ [2/2] Purging local data...
⏳   ✓ Local data removed (took 5ms)
✅ Environment 'lxd-local-example' purged successfully
```

**Attempting to purge a running environment (with confirmation):**

```text
⚠️  WARNING: Environment 'lxd-local-example' is in 'Running' state
This will remove local data but NOT destroy infrastructure!
You may lose access to running resources.

Continue? [y/N]: n
❌ Purge cancelled by user
```

Or with `--force` flag:

```bash
torrust-tracker-deployer purge lxd-local-example --force
```

```text
⚠️  WARNING: Purging 'lxd-local-example' in 'Running' state
⏳ [1/2] Validating environment...
⏳   ✓ Environment 'lxd-local-example' found (took 0ms)
⏳ [2/2] Purging local data...
⏳   ✓ Local data removed (took 5ms)
✅ Environment 'lxd-local-example' purged successfully
```

**Hint after destroy command:**

```text
✅ Environment 'lxd-local-example' destroyed successfully

Infrastructure has been torn down. Local data preserved for debugging.
To fully remove: torrust-tracker-deployer purge lxd-local-example
```

## Design Decisions

### Decision 1: No `--clean` flag on destroy

**Rejected option:** `destroy --clean` to auto-purge on success

**Reason:** You don't know beforehand if destruction will fail. Using `--clean` could:

- Leave orphaned infrastructure resources without any local data to debug
- Lose valuable information if something goes wrong

**Conclusion:** Better to always require explicit `purge` after successful `destroy`.

### Decision 2: No auto-clean on success

**Rejected option:** Automatically purge data if `destroy` succeeds

**Reason:** System should behave consistently regardless of operation result. Either:

- `destroy` always preserves data, OR
- `destroy` always removes data

Mixing behaviors based on success/failure is confusing and unpredictable.

**Conclusion:** `destroy` always preserves data. `purge` is always explicit.

### Decision 3: Use "purge" not "clean"

**Rejected option:** `clean` command

**Reason:** "Clean" is ambiguous - users might confuse whether to run `destroy` or `clean`.

**Why "purge" is better:**

- Common in package managers (`apt purge` vs `apt remove`)
- Sounds more final/destructive than "clean"
- Clear semantic: "destroy" = infrastructure, "purge" = local data
- Unambiguous: you can't "purge" running infrastructure

### Decision 4: Allow purge in any state with confirmation

`purge` should work on environments in **any state**, but with safeguards:

- **Always requires confirmation** - User must explicitly confirm the destructive operation
- **`--force` flag available** - Skip confirmation for automation/scripts
- **Clear warnings** - Show current state and explain consequences

**Rationale:**

The initial goal was to allow users to reuse environment names after destruction. However, this led to a broader insight: users should be able to totally remove state from the app without knowing internal implementation details (where state is stored, what needs to be deleted).

Key considerations from design discussion:

1. **Real-world scenarios**: Infrastructure might be destroyed independently (manually, provider issues, etc.) and users need to clean deployer state
2. **User shouldn't know internals**: Users shouldn't need to manually delete `data/` and `build/` folders
3. **Any state can be valuable**: Even "Created" environments may contain secrets or configurations not stored elsewhere
4. **Destructiveness varies by state**: More dangerous in some cases (e.g., Running) than others (e.g., Destroyed), but we can't assume the user's intent
5. **Confirmation provides safety**: Instead of blocking operations by state, warn users and require explicit confirmation

This design aligns with the project's user-friendliness principles: the system should be helpful and clear about consequences, not restrictive.

**Example confirmation prompts:**

```text
# Destroyed state (normal case)
⚠️  About to purge environment 'my-env' (state: Destroyed)
This will remove all local data. Infrastructure already destroyed.
Continue? [y/N]:

# Running state (dangerous case)
⚠️  WARNING: Environment 'my-env' is in 'Running' state
This will remove local data but NOT destroy infrastructure!
You may lose access to running resources.
Continue? [y/N]:
```

## Implementation Notes

- Always prompt for confirmation (interactive mode)
- Support `--force` flag to skip confirmation (for automation)
- Remove `data/{env-name}/` directory
- Remove `build/{env-name}/` directory (generated artifacts)
- Provide clear warnings that vary by environment state
- For non-Destroyed states, warn about potential infrastructure orphaning
- Update `destroy` output to hint about `purge` command

## What Gets Purged

The `purge` command removes:

```text
data/{env-name}/          # Environment state and configuration
├── environment.json      # Serialized environment state
├── templates/            # Any copied templates
└── ...

build/{env-name}/         # Generated artifacts
├── ansible/              # Generated Ansible playbooks
├── docker-compose/       # Generated docker-compose files
├── tofu/                 # Generated OpenTofu files
└── ...
```

## Implementation Plan

### Phase 1: Presentation & Application Layer ✅ COMPLETED

- [x] Add `purge` command to CLI subcommands (Phase 1: Presentation stub - 6841d94b)
- [x] Create routing and controller in presentation layer
- [x] Create `PurgeCommandHandler` in `src/application/command_handlers/purge/` (Phase 2 - e053e57b)
- [x] Implement directory removal logic (removes `data/{env}/` and `build/{env}/`)
- [x] Add confirmation prompt in presentation layer (Phase 3 - 45513e99)
- [x] Support `--force` flag to skip confirmation

### Phase 2: E2E Testing ✅ COMPLETED

- [x] Add `purge` subcommand to CLI
- [x] Wire command handler in application container
- [x] Add E2E tests in `tests/e2e/purge_command.rs` (Phase 4 - 1aaf7573)
  - [x] Purge destroyed environment successfully
  - [x] Fail when purging nonexistent environment
  - [x] Purge with custom working directory
  - [x] Complete lifecycle (create → destroy → purge)
  - [x] Remove only specified environment data (isolation test)
- [x] Add `ProcessRunner::run_purge_command()` helper
- [x] Add assertion helpers for purge verification
- [x] All 5 E2E tests passing

### Phase 3: Unit Testing 🔄 IN PROGRESS

- [ ] Add unit tests for application layer (`PurgeCommandHandler`)
  - [ ] Test successful purge removes all artifacts
  - [ ] Test idempotency (already removed)
  - [ ] Test permission errors
  - [ ] Test repository deletion
- [ ] Add unit tests for presentation layer (`PurgeCommandController`)
  - [ ] Test environment name validation
  - [ ] Test force flag behavior
  - [ ] Test error handling and user output

### Phase 4: User Experience & Documentation 📝 TODO

- [ ] Update `destroy` command output with purge hint
  - [ ] Add message after successful destroy
  - [ ] Suggest: "Use 'purge' command to remove local data and reuse environment name"
- [ ] Create user documentation `docs/user-guide/commands/purge.md`
  - [ ] Command description and purpose
  - [ ] Usage examples (with and without --force)
  - [ ] When to use purge command
  - [ ] Warning about irreversibility
  - [ ] Common workflows (destroy → purge)
- [ ] Update `docs/user-guide/commands/README.md`
  - [ ] Add purge command to command list
  - [ ] Link to purge.md documentation
  - [ ] Update command overview section

Create comprehensive E2E tests modeled after `tests/e2e/destroy_command.rs`. The test file should be `tests/e2e/purge_command.rs`.

**Test Module Structure:**

```rust
//! End-to-End Black Box Tests for Purge Command
//!
//! This test suite provides true black-box testing of the purge command
//! by running the production application as an external process. These tests
//! verify that the purge command correctly removes local environment data
//! and handles the working directory parameter.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests purge in different states and working directories
//! - **Verification**: Validates data/build directories are removed
```

**Required Test Scenarios:**

1. **`it_should_purge_destroyed_environment_with_default_working_directory()`**
   - Create → Destroy → Purge (with `--force`)
   - Verify `data/{env-name}/` directory is removed
   - Verify `build/{env-name}/` directory is removed
   - Verify environment no longer exists in registry

2. ✅ **`it_should_fail_when_purging_nonexistent_environment()`**
   - Try to purge non-existent environment
   - Verify error message is clear

3. ✅ **`it_should_purge_with_custom_working_directory()`**
   - Create → Destroy → Purge using custom working directory
   - Tests that purge respects `--working-dir` parameter

4. ✅ **`it_should_complete_full_lifecycle_from_create_to_purge()`**
   - Create → Destroy → Purge (with `--force`) → Verify cleanup
   - Complete workflow test

5. ✅ **`it_should_remove_only_specified_environment_data()`**
   - Create two environments → Destroy both → Purge one
   - Verify only specified environment is removed
   - Verify other environment remains intact

**Test Infrastructure Added:**

- ✅ `ProcessRunner::run_purge_command()` - Executes purge with `--force` flag
- ✅ `EnvironmentStateAssertions::assert_environment_not_exists()` - Verifies registry removal
- ✅ `EnvironmentStateAssertions::assert_data_directory_not_exists()` - Verifies `data/` removal
- ✅ `EnvironmentStateAssertions::assert_build_directory_not_exists()` - Verifies `build/` removal

**Test Results:**

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

---

</details>

---

### Phase 5: Documentation ✨ NEXT STEPS)

<details>
<summary>Original test plan (expand to view)</summary>

Create comprehensive E2E tests modeled after `tests/e2e/destroy_command.rs`. The test file should be `tests/e2e/purge_command.rs`.

**Required Test Scenarios:**

1. **`it_should_purge_destroyed_environment_with_default_working_directory()`**
   - Create → Destroy → Purge (with `--force`) → Verify cleanup
   - Complete workflow test

**Test Helpers Needed:**

- Use existing `ProcessRunner` to run purge command
- Use existing `EnvironmentStateAssertions` for verification
- Add method to `ProcessRunner`: `run_purge_command(name: &str) -> Result<ProcessOutput>`
- Add method to `EnvironmentStateAssertions`: `assert_environment_removed(name: &str)`
- All purge commands in tests should use `--force` flag to avoid interactive prompts

**Key Verification Points:**

- Command exit codes (0 for success, non-zero for errors)
- Directory removal: both `data/{env}/` and `build/{env}/` are deleted
- Error messages are clear and actionable
- Working directory parameter is respected

### Phase 3: User Experience Enhancements (1 hour)

- [ ] Update `destroy` command output with purge hint
- [ ] Add helpful error messages
- [ ] Update user documentation
      **Core Functionality** ✅
- [x] `purge` command removes both `data/{env-name}/` and `build/{env-name}/` directories
- [x] Always prompts for confirmation in interactive mode
- [x] `--force` flag skips confirmation
- [x] Works with any environment state
- [x] Command properly removes environment from registry
- [x] Idempotent - succeeds if directories already removed

**User Experience** 🔄

- [x] Clear warning messages about irreversibility
- [x] Helpful error messages with troubleshooting
- [ ] `destroy` command output hints about `purge`
- [ ] User documentation in `docs/user-guide/commands/`

**Testing** ✅

- [x] E2E tests in `tests/e2e/purge_command.rs` verify:
  - [x] Purge destroyed environment (default working dir)
  - [x] Purge with custom working directory
  - [x] Error handling for non-existent environments
  - [x] Full lifecycle: create → destroy → purge
  - [x] Both `data/` and `build/` directories are removed
  - [x] Environment isolation (purge one, others remain)
- [ ] Unit tests cover application and presentation layers
- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Documentation** 📝

- [ ] Create `docs/user-guide/commands/purge.md`
- [ ] Update `docs/user-guide/commands/README.md` with purge command
- [ ] Update destroy command output to mention purge
  - [ ] Full lifecycle: create → destroy → purge
  - [ ] Both `data/` and `build/` directories are removed
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Documentation updated in `docs/user-guide/commands/`

## Resolved Design Questions

### Q1: Should `purge` work on "Created" state (never provisioned)?

**Answer**: Yes, allow purging in ANY state with proper confirmation.

**User's reasoning**:

> "If we do that we open the door for the rest of states, why do not do it also when we fail to destroy the infra? I think the intention of the command was to allow user really clean everything. We decided to use only the 'Destroyed' state because we wanted to reuse the name of the environment. However I think it would be useful to let the user always to totally remove state from the app without knowing the internal details of the implementation (the users should not know where the state is and what it needs to delete to remove all the information about an environment).
>
> We can extend the initial goal to any state. The only difference is the command can be more destructive in other cases.
>
> But as you mention what happens if a user destroys the infrastructure independently and wants to clean the state in the deployer?
>
> Maybe we should always allow the execution of the command but be more careful. Maybe we can ask for a confirmation in the presentation layer.
>
> In conclusion, we can allow it always and there can be reasons to need that in any state but make sure the user does not do it accidentally. Since it's a very destructive action we should ask for confirmation always. It's more dangerous in some cases than other but we do not know that. Even an environment that has just been created but not provisioned may contain a secret that is not stored elsewhere."

**Summary**: Users shouldn't need to know internal implementation details. Real-world scenarios exist where infrastructure is destroyed outside the deployer. Safety comes from confirmation, not restrictions.

### Q2: Should there be a `--force` flag to skip confirmation?

**Answer**: Yes, use Option C - always confirm unless `--force` is provided.

**Options considered**:

- Option A: Always require confirmation (interactive) - blocks automation
- Option B: Confirmation only for non-Destroyed states - inconsistent UX
- Option C: Always confirm + `--force` flag to skip - **CHOSEN**

**Reason**: Allows automation while protecting interactive users from accidents. Consistent behavior regardless of state.

### Q3: Should `list` command show "Destroyed" environments differently?

**Answer**: No (Option A) - keep current behavior, show all states equally.

**User's reasoning**: "The user can purge envs if they do not want to see them, and users are not likely to have many envs."

**Options considered**:

- Option A: Show all states equally (current behavior) - **CHOSEN**
- Option B: Visual indicator for purgeable environments - adds complexity
- Option C: Add `--include-destroyed` flag - unnecessary filtering

**Reason**: Simple and clear. Users can purge environments they don't want to see. No need to complicate the list command.

## Related Documentation

- [Codebase Architecture](../codebase-architecture.md) - Three-level pattern (Commands → Steps → Actions)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md) - Where to place new code
- [User Documentation](../user-guide/commands/) - Command documentation
- [Existing Destroy Command](../reference/command-outputs/lxd-local-example.md) - Pattern to follow
- [Destroy Command E2E Tests](../../tests/e2e/destroy_command.rs) - Test pattern to follow for purge E2E tests

## Reference

Current `destroy` output (see what needs to change):

- [lxd-local-example.md](../../reference/command-outputs/lxd-local-example.md)
