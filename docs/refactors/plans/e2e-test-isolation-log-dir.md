# E2E Test Isolation - Complete Log Directory Support

## 📋 Overview

Add `.log_dir()` support to all E2E test ProcessRunner calls to achieve complete test isolation and prevent pollution of the production `data/` folder. Currently, tests use `.working_dir()` for environment data isolation but logs still go to the default `./data/logs/` location, violating the test isolation principle.

**Target Files:**

- `tests/e2e/create_command.rs` (5 ProcessRunner calls)
- `tests/e2e/destroy_command.rs` (7 ProcessRunner calls)
- `tests/e2e/list_command.rs` (6 ProcessRunner calls)
- `tests/e2e/purge_command.rs` (14 ProcessRunner calls)
- `tests/e2e/show_command.rs` (6 ProcessRunner calls)
- `tests/e2e/validate_command.rs` (4 ProcessRunner calls)

**Already Fixed:**

- ✅ `tests/e2e/render_command.rs` (12 ProcessRunner calls) - Fixed in commit d21754f6
- ✅ `tests/validate_ai_training_examples.rs` (2 ProcessRunner calls) - Fixed in commit d21754f6
- ✅ `src/testing/e2e/process_runner.rs` - All 14 command methods now support `--log-dir` argument

**Scope:**

- Add `.log_dir(temp_workspace.path().join("logs"))` to all ProcessRunner calls in remaining E2E test files
- Verify complete test isolation: `data/` folder must remain completely empty after running all tests
- Follow the pattern established in render_command.rs tests

## 📊 Progress Tracking

**Total Active Proposals**: 6
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 6
**In Progress**: 0
**Not Started**: 0

### Phase Summary

- **Phase 0 - Validate Command Tests (High Impact, Low Effort)**: ✅ 1/1 completed (100%) - Commit 1d576a5a
- **Phase 1 - Create Command Tests (High Impact, Low Effort)**: ✅ 1/1 completed (100%) - Commit e452efe6
- **Phase 2 - List Command Tests (High Impact, Low Effort)**: ✅ 1/1 completed (100%) - Commit 53b87a8d
- **Phase 3 - Show Command Tests (High Impact, Low Effort)**: ✅ 1/1 completed (100%) - Commit 5682ae05
- **Phase 4 - Destroy Command Tests (High Impact, Low Effort)**: ✅ 1/1 completed (100%) - Commit 82946459
- **Phase 5 - Purge Command Tests (High Impact, Low Effort)**: ✅ 1/1 completed (100%) - Commit 1c437d80

### Discarded Proposals

None

### Postponed Proposals

None

## 🎯 Key Problems Identified

### 1. Incomplete Test Isolation

Tests currently use `TempWorkspace` and `.working_dir()` for environment data isolation, but logs still write to production `./data/logs/`:

```rust
// Current pattern (incomplete isolation)
let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");
let result = ProcessRunner::new()
    .working_dir(temp_workspace.path())  // ✅ Environment data isolated
    .run_create_command("./environment.json")  // ❌ Logs go to ./data/logs
    .expect("Failed to run create command");
```

After running `cargo test`, the production `data/logs/` directory contains a 37KB+ log file, violating the principle that tests should not pollute production directories.

### 2. Independent Global Arguments

The `--log-dir` and `--working-dir` arguments are independent by design (documented in ADR and code). This means:

- Setting `--working-dir` does **not** automatically redirect logs
- Tests must explicitly pass both arguments for complete isolation
- ProcessRunner now supports `.log_dir()` builder method (added in commit d21754f6)

### 3. Inconsistent Test Hygiene

Some tests (render_command.rs, validate_ai_training_examples.rs) now follow proper isolation, while others don't:

- ✅ **Properly Isolated**: render_command.rs (12 calls), validate_ai_training_examples.rs (2 calls)
- ❌ **Still Polluting**: create_command.rs, destroy_command.rs, list_command.rs, purge_command.rs, show_command.rs, validate_command.rs

This creates technical debt and makes it unclear which tests follow best practices.

## 🚀 Refactoring Phases

---

## Phase 0: Validate Command Tests (Highest Priority)

**Rationale**: Start with the simplest file (4 calls) to establish momentum and verify the pattern works across different test scenarios.

### Proposal #0: Add log_dir to validate_command.rs Tests

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High (Prevents production folder pollution)  
**Effort**: 🔵 Low (4 ProcessRunner calls, simple pattern)  
**Priority**: P0  
**Depends On**: None  
**Target File**: `tests/e2e/validate_command.rs`

#### Problem

The validate command tests create 4 ProcessRunner instances without `.log_dir()`, causing logs to write to production `./data/logs/`:

```rust
// Line 58
let result = ProcessRunner::new()
    .working_dir(temp_workspace.path())
    .run_validate_command(&config_path)
    .expect("Failed to run validate command");
```

This pattern repeats 4 times in the file, polluting the production data folder.

#### Proposed Solution

Add `.log_dir(temp_workspace.path().join("logs"))` to all ProcessRunner calls:

```rust
let result = ProcessRunner::new()
    .working_dir(temp_workspace.path())
    .log_dir(temp_workspace.path().join("logs"))  // ✅ Complete isolation
    .run_validate_command(&config_path)
    .expect("Failed to run validate command");
```

#### Rationale

- Follows the established pattern from render_command.rs (commit d21754f6)
- ProcessRunner.run_validate_command() already supports `--log-dir` argument
- Each test uses TempWorkspace, so logs go to temp directory and are auto-cleaned
- Simple mechanical transformation with clear benefits

#### Benefits

- ✅ Prevents pollution of production `data/logs/` directory
- ✅ Complete test isolation (both environment data and logs)
- ✅ Consistent with other properly isolated tests
- ✅ Tests auto-clean logs with TempWorkspace cleanup

#### Implementation Checklist

- [x] Add `.log_dir()` to ProcessRunner call at line ~58
- [x] Add `.log_dir()` to ProcessRunner call at line ~97
- [x] Add `.log_dir()` to ProcessRunner call at line ~146
- [x] Add `.log_dir()` to ProcessRunner call at line ~201
- [x] Run `cargo test --test e2e_integration validate_command`
- [x] Verify all tests pass (4 tests expected)
- [x] Clean data folder: `rm -rf data/*`
- [x] Run tests again and verify `data/` folder remains empty
- [x] Commit changes: `fix: [#365] add log_dir to validate_command tests for complete isolation`

**Status**: ✅ Completed (Commit 1d576a5a)  
**Result**: All 4 tests pass, data/ folder remains empty

#### Testing Strategy

Before changes:

```bash
rm -rf data/* && cargo test --test e2e_integration validate_command && ls -la data/
# Expected: data/logs/ directory created with log file
```

After changes:

```bash
rm -rf data/* && cargo test --test e2e_integration validate_command && ls -la data/
# Expected: data/ folder completely empty (only . and ..)
```

#### Results (if completed)

- **Lines Removed**: 0
- **Lines Added**: 4 (one `.log_dir()` call per ProcessRunner)
- **Net Change**: +4 lines
- **Tests**: [Status]
- **Linters**: [Status]

---

## Phase 1: Create Command Tests

**Rationale**: Second simplest file (5 calls), establishes pattern for command tests that create environments.

### Proposal #1: Add log_dir to create_command.rs Tests

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low (5 ProcessRunner calls)  
**Priority**: P1  
**Depends On**: Proposal #0 (validate pattern works)  
**Target File**: `tests/e2e/create_command.rs`

#### Problem

5 ProcessRunner calls without `.log_dir()` at lines: ~69, ~104, ~131, ~163, ~175

#### Proposed Solution

Add `.log_dir(temp_workspace.path().join("logs"))` to all 5 ProcessRunner calls.

#### Implementation Checklist

- [x] Add `.log_dir()` to ProcessRunner call at line ~69
- [x] Add `.log_dir()` to ProcessRunner call at line ~104
- [x] Add `.log_dir()` to ProcessRunner call at line ~131
- [x] Add `.log_dir()` to ProcessRunner call at line ~163
- [x] Add `.log_dir()` to ProcessRunner call at line ~175
- [x] Run `cargo test --test e2e_integration create_command`
- [x] Verify all tests pass
- [x] Clean data folder and verify it remains empty after tests
- [x] Commit changes: `fix: [#365] add log_dir to create_command tests for complete isolation`

**Status**: ✅ Completed (Commit e452efe6)  
**Result**: All 4 tests pass, data/ folder remains empty

#### Testing Strategy

Verify `data/` folder remains empty after running create_command tests.

---

## Phase 2: List Command Tests

**Rationale**: Build on established pattern with slightly more complex scenarios (6 calls).

### Proposal #2: Add log_dir to list_command.rs Tests

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low (6 ProcessRunner calls)  
**Priority**: P2  
**Depends On**: Proposal #1  
**Target File**: `tests/e2e/list_command.rs`

#### Problem

6 ProcessRunner calls without `.log_dir()` at lines: ~53, ~91, ~107, ~142, ~159, ~176

#### Proposed Solution

Add `.log_dir(temp_workspace.path().join("logs"))` to all 6 ProcessRunner calls.

#### Implementation Checklist

- [x] Add `.log_dir()` to ProcessRunner call at line ~53
- [x] Add `.log_dir()` to ProcessRunner call at line ~91
- [x] Add `.log_dir()` to ProcessRunner call at line ~107
- [x] Add `.log_dir()` to ProcessRunner call at line ~142
- [x] Add `.log_dir()` to ProcessRunner call at line ~159
- [x] Add `.log_dir()` to ProcessRunner call at line ~176
- [x] Run `cargo test --test e2e_integration list_command`
- [x] Verify all tests pass
- [x] Clean data folder and verify it remains empty after tests
- [x] Commit changes: `fix: [#365] add log_dir to list_command tests for complete isolation`

**Status**: ✅ Completed (Commit 53b87a8d)  
**Result**: All 3 tests pass, data/ folder remains empty

#### Testing Strategy

Verify `data/` folder remains empty after running list_command tests.

---

## Phase 3: Show Command Tests

**Rationale**: Continue pattern with show command tests (6 calls).

### Proposal #3: Add log_dir to show_command.rs Tests

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low (6 ProcessRunner calls)  
**Priority**: P3  
**Depends On**: Proposal #2  
**Target File**: `tests/e2e/show_command.rs`

#### Problem

6 ProcessRunner calls without `.log_dir()` at lines: ~52, ~90, ~106, ~142, ~154, ~191, ~203

#### Proposed Solution

Add `.log_dir(temp_workspace.path().join("logs"))` to all 6 ProcessRunner calls.

#### Implementation Checklist

- [x] Add `.log_dir()` to ProcessRunner call at line ~52
- [x] Add `.log_dir()` to ProcessRunner call at line ~90
- [x] Add `.log_dir()` to ProcessRunner call at line ~106
- [x] Add `.log_dir()` to ProcessRunner call at line ~142
- [x] Add `.log_dir()` to ProcessRunner call at line ~154
- [x] Add `.log_dir()` to ProcessRunner call at line ~191
- [x] Add `.log_dir()` to ProcessRunner call at line ~203
- [x] Run `cargo test --test e2e_integration show_command`
- [x] Verify all tests pass
- [x] Clean data folder and verify it remains empty after tests
- [x] Commit changes: `fix: [#365] add log_dir to show_command tests for complete isolation`

**Status**: ✅ Completed (Commit 5682ae05)  
**Result**: All 4 tests pass, data/ folder remains empty

#### Testing Strategy

Verify `data/` folder remains empty after running show_command tests.

---

## Phase 4: Destroy Command Tests

**Rationale**: More complex scenarios with environment lifecycle (7 calls).

### Proposal #4: Add log_dir to destroy_command.rs Tests

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low (7 ProcessRunner calls)  
**Priority**: P4  
**Depends On**: Proposal #3  
**Target File**: `tests/e2e/destroy_command.rs`

#### Problem

7 ProcessRunner calls without `.log_dir()` at lines: ~67, ~83, ~117, ~133, ~161, ~194, ~211

#### Proposed Solution

Add `.log_dir(temp_workspace.path().join("logs"))` to all 7 ProcessRunner calls.

#### Implementation Checklist

- [x] Add `.log_dir()` to ProcessRunner call at line ~67
- [x] Add `.log_dir()` to ProcessRunner call at line ~83
- [x] Add `.log_dir()` to ProcessRunner call at line ~117
- [x] Add `.log_dir()` to ProcessRunner call at line ~133
- [x] Add `.log_dir()` to ProcessRunner call at line ~161
- [x] Add `.log_dir()` to ProcessRunner call at line ~194
- [x] Add `.log_dir()` to ProcessRunner call at line ~211
- [x] Run `cargo test --test e2e_integration destroy_command`
- [x] Verify all tests pass
- [x] Clean data folder and verify it remains empty after tests
- [x] Commit changes: `fix: [#365] add log_dir to destroy_command tests for complete isolation`

**Status**: ✅ Completed (Commit 82946459)  
**Result**: All 4 tests pass, data/ folder remains empty

#### Testing Strategy

Verify `data/` folder remains empty after running destroy_command tests.

---

## Phase 5: Purge Command Tests (Final Phase)

**Rationale**: Most complex file with most ProcessRunner calls (14 calls). Save for last to ensure pattern is solid.

### Proposal #5: Add log_dir to purge_command.rs Tests

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium (14 ProcessRunner calls - largest file)  
**Priority**: P5  
**Depends On**: Proposal #4  
**Target File**: `tests/e2e/purge_command.rs`

#### Problem

14 ProcessRunner calls without `.log_dir()` at lines: ~75, ~87, ~104, ~140, ~174, ~186, ~198, ~233, ~249, ~264, ~303, ~316, ~324, ~331, ~344

#### Proposed Solution

Add `.log_dir(temp_workspace.path().join("logs"))` to all 14 ProcessRunner calls.

#### Implementation Checklist

- [x] Add `.log_dir()` to ProcessRunner call at line ~75
- [x] Add `.log_dir()` to ProcessRunner call at line ~87
- [x] Add `.log_dir()` to ProcessRunner call at line ~104
- [x] Add `.log_dir()` to ProcessRunner call at line ~140
- [x] Add `.log_dir()` to ProcessRunner call at line ~174
- [x] Add `.log_dir()` to ProcessRunner call at line ~186
- [x] Add `.log_dir()` to ProcessRunner call at line ~198
- [x] Add `.log_dir()` to ProcessRunner call at line ~233
- [x] Add `.log_dir()` to ProcessRunner call at line ~249
- [x] Add `.log_dir()` to ProcessRunner call at line ~264
- [x] Add `.log_dir()` to ProcessRunner call at line ~303
- [x] Add `.log_dir()` to ProcessRunner call at line ~316
- [x] Add `.log_dir()` to ProcessRunner call at line ~324
- [x] Add `.log_dir()` to ProcessRunner call at line ~331
- [x] Add `.log_dir()` to ProcessRunner call at line ~344
- [x] Run `cargo test --test e2e_integration purge_command`
- [x] Verify all tests pass
- [x] Clean data folder and verify it remains empty after tests
- [x] Commit changes: `fix: [#365] add log_dir to purge_command tests for complete isolation`

**Status**: ✅ Completed (Commit 1c437d80)  
**Result**: All 5 tests pass, data/ folder remains empty  
**Note**: Found 15 ProcessRunner calls (not 14 as estimated)

#### Testing Strategy

Verify `data/` folder remains empty after running purge_command tests.

---

## 📈 Timeline

- **Start Date**: 2026-02-18
- **Target Completion**: 2026-02-18 (same day - low effort refactoring)
- **Actual Completion**: 2026-02-18

**Final Statistics**:

- Total ProcessRunner calls fixed: 45 (4 + 5 + 6 + 7 + 7 + 15 + 1 additional)
- Total test files modified: 6 (validate, create, list, show, destroy, purge commands)
- Total commits: 6 (one per phase)
- All tests pass: 408 passed; 0 failed; 95 ignored
- Production data/ folder: Completely empty after all tests

## 🔍 Review Process

### Approval Criteria

- [x] Technical feasibility validated (ProcessRunner already supports `--log-dir`)
- [x] Aligns with [Development Principles](../development-principles.md) (Testability, Observability)
- [x] Pattern proven in render_command.rs tests (commit d21754f6)
- [x] Implementation plan is clear and actionable

### Completion Criteria

- [ ] All 6 proposals implemented (44 ProcessRunner calls updated)
- [ ] Run `cargo test` and verify all tests pass
- [ ] Clean `data/` folder and run `cargo test` again
- [ ] Verify `data/` folder remains completely empty (only `.` and `..`)
- [ ] All linters passing (`cargo run --bin linter all`)
- [ ] Changes committed and pushed to branch 365-fix-render-working-dir
- [ ] PR #366 updated with final verification

## 📚 Related Documentation

- [Development Principles](../development-principles.md) - Testability principle
- [Testing Conventions](../contributing/testing/unit-testing/naming-conventions.md)
- [Issue #365](https://github.com/torrust/torrust-tracker-deployer/issues/365) - Original bug report
- [PR #366](https://github.com/torrust/torrust-tracker-deployer/pull/366) - Fix for render command
- [ADR: Environment Variable Independence](../decisions/environment-variable-prefix.md) - Explains why `--log-dir` and `--working-dir` are independent

## 💡 Notes

### Pattern to Follow

All ProcessRunner calls should follow this pattern:

```rust
let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

let result = ProcessRunner::new()
    .working_dir(temp_workspace.path())              // ✅ Isolate environment data
    .log_dir(temp_workspace.path().join("logs"))     // ✅ Isolate log files
    .run_COMMAND_command(...)                         // Any command
    .expect("Failed to run command");
```

### Verification Command

After implementing all phases, run this comprehensive verification:

```bash
# Clean data folder
rm -rf data/*

# Run ALL tests
cargo test

# Verify data folder is empty
ls -la data/
# Expected output:
# total 8
# drwxrwxr-x  2 user user 4096 Feb 18 XX:XX .
# drwxrwxr-x 22 user user 4096 Feb 18 XX:XX ..
```

### Discovery Notes

- Initial investigation found 56 total ProcessRunner calls across all E2E tests
- render_command.rs (12 calls) and validate_ai_training_examples.rs (2 calls) already fixed
- Remaining: 44 calls across 6 test files
- All ProcessRunner command methods now support `--log-dir` (implemented in commit d21754f6)
- Pattern proven to work - just needs mechanical application to remaining test files

### Future Improvements

After this refactoring:

- Consider adding a ProcessRunner builder method that automatically sets both `.working_dir()` and `.log_dir()` from a single TempWorkspace
- Document the pattern in testing conventions guide
- Add a lint rule or test to catch future ProcessRunner usages without `.log_dir()`

---

**Created**: 2026-02-18  
**Last Updated**: 2026-02-18  
**Status**: 📋 Planning
