# Consolidate External Tool Adapters in src/adapters/

## 📋 Overview

This refactoring consolidates all external tool adapters (SSH, Docker, Ansible, LXD, OpenTofu) into a new top-level `src/adapters/` module for improved discoverability, semantic clarity, and consistency. Currently, these adapters are scattered across `src/shared/` and `src/infrastructure/external_tools/`, creating confusion and making them hard to find.

**Decision Record**: [External Tool Adapters Organization](../../decisions/external-tool-adapters-organization.md)

**Target Files/Modules:**

- `src/shared/ssh/` → `src/adapters/ssh/`
- `src/shared/docker/` → `src/adapters/docker/`
- `src/infrastructure/external_tools/ansible/adapter/` → `src/adapters/ansible/`
- `src/infrastructure/external_tools/lxd/adapter/` → `src/adapters/lxd/`
- `src/infrastructure/external_tools/tofu/adapter/` → `src/adapters/tofu/`
- `src/infrastructure/external_tools/ansible/template/` → STAY (application-specific)
- `src/infrastructure/external_tools/tofu/template/` → STAY (application-specific)
- All files importing from moved adapter modules (20+ files for SSH, 3+ files for Docker, etc.)
- Integration tests: `tests/ssh_client_integration.rs`, `tests/ssh_client/`

**Scope:**

- Create new `src/adapters/` module with submodules for each external tool
- Move adapter implementations (thin CLI wrappers) from current locations
- Leave template rendering code in `src/infrastructure/external_tools/` (application-specific)
- Update all imports across the codebase
- Ensure all tests pass after migration
- Update documentation references

## 📊 Progress Tracking

**Total Active Proposals**: 4
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 4

### Phase Summary

- **Phase 0 - Foundation (High Impact, Low Effort)**: ⏳ 0/1 completed (0%)
- **Phase 1 - Adapters Migration (High Impact, Medium Effort)**: ⏳ 0/2 completed (0%)
- **Phase 2 - Cleanup (Medium Impact, Low Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

None

### Postponed Proposals

None

## 🎯 Key Problems Identified

### 1. Inconsistent Discoverability

External tool wrappers are split across two unrelated modules (`shared/` and `infrastructure/external_tools/`), making them hard to find and understand as a cohesive group.

### 2. Semantic Confusion

`src/shared/` mixes pure utilities (Clock, Username) with infrastructure adapters (SSH, Docker), violating the single responsibility principle at the module level.

### 3. Artificial Organization Split

The current split is based on _assumed reusability_ (where they might be used) rather than _nature of the code_ (what they actually are), leading to inconsistent organization.

### 4. Pattern Inconsistency

All adapters follow the same pattern (thin clients using CommandExecutor) but are organized differently, making it harder to apply uniform conventions.

## 🚀 Refactoring Phases

---

## Phase 0: Foundation (Highest Priority)

Create the new module structure and prepare for migration.

### Proposal #0: Create src/adapters/ Module Structure

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High (Foundation for all other work)  
**Effort**: 🔵 Low (Just directory and module setup)  
**Priority**: P0  
**Depends On**: None

#### Problem

Need to create the new `src/adapters/` module structure before moving any code.

#### Proposed Solution

1. Create `src/adapters/` directory
2. Create `src/adapters/mod.rs` with initial module declarations
3. Create `src/infrastructure/templates/` directory
4. Create `src/infrastructure/templates/mod.rs`
5. Verify structure compiles

#### Implementation Checklist

- [ ] Create `src/adapters/` directory
- [ ] Create `src/adapters/mod.rs` with module documentation
- [ ] Create placeholder submodule files:
  - [ ] `src/adapters/ansible.rs`
  - [ ] `src/adapters/docker.rs`
  - [ ] `src/adapters/lxd.rs`
  - [ ] `src/adapters/ssh.rs`
  - [ ] `src/adapters/tofu.rs`
- [ ] Add `pub mod adapters;` to `src/lib.rs`
- [ ] Run `cargo check` to verify compilation

---

## Phase 1: Adapters Migration

Move adapters and update all imports.

### Proposal #1: Move Docker and SSH Adapters from src/shared/

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High (Used in many places)  
**Effort**: 🔵🔵 Medium (Need to update ~23 imports)  
**Priority**: P1  
**Depends On**: Proposal #0

#### Problem

Docker and SSH adapters are in `src/shared/` but are infrastructure concerns, not pure utilities.

- SSH: ~20 imports across domain, application, infrastructure, testing
- Docker: 3 imports, all in testing infrastructure

#### Proposed Solution

1. Move `src/shared/ssh/` → `src/adapters/ssh/`
2. Move `src/shared/docker/` → `src/adapters/docker/`
3. Update `src/adapters/mod.rs` to re-export these modules
4. Update all imports from `crate::shared::ssh` → `crate::adapters::ssh`
5. Update all imports from `crate::shared::docker` → `crate::adapters::docker`
6. Keep `src/shared/command/` where it is (truly generic utility)

#### Implementation Checklist

- [ ] **SSH Migration**:
  - [ ] Move directory: `src/shared/ssh/` → `src/adapters/ssh/`
  - [ ] Update `src/adapters/mod.rs` to include `pub mod ssh;`
  - [ ] Update imports in domain layer (~4 files)
  - [ ] Update imports in application layer (~4 files)
  - [ ] Update imports in infrastructure layer (~8 files)
  - [ ] Update imports in testing layer (~4 files)
  - [ ] Update integration tests if needed
  - [ ] Run `cargo check` and fix any remaining import issues
- [ ] **Docker Migration**:
  - [ ] Move directory: `src/shared/docker/` → `src/adapters/docker/`
  - [ ] Update `src/adapters/mod.rs` to include `pub mod docker;`
  - [ ] Update imports in `src/testing/integration/ssh_server/` (~3 files)
  - [ ] Run `cargo check` and fix any remaining import issues
- [ ] **Cleanup src/shared/mod.rs**:
  - [ ] Remove `pub mod ssh;` and SSH re-exports
  - [ ] Remove `pub mod docker;` and Docker re-exports
  - [ ] Verify `command` module remains in shared
- [ ] Run full test suite: `cargo test`

### Proposal #2: Move Adapters from src/infrastructure/external_tools/

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High (Core infrastructure)  
**Effort**: 🔵🔵 Medium (Need to separate adapters from templates)  
**Priority**: P1  
**Depends On**: Proposal #0

#### Problem

Ansible, LXD, and OpenTofu adapters are in `src/infrastructure/external_tools/` mixed with application-specific template code.

#### Proposed Solution

1. Move adapter code only:
   - `src/infrastructure/external_tools/ansible/adapter/` → `src/adapters/ansible/`
   - `src/infrastructure/external_tools/lxd/adapter/` → `src/adapters/lxd/`
   - `src/infrastructure/external_tools/tofu/adapter/` → `src/adapters/tofu/`
2. Keep module structure (each has `client.rs`, `mod.rs`, etc.)
3. Update `src/adapters/mod.rs` to re-export these modules
4. Update all imports from `crate::infrastructure::external_tools::*::adapter` → `crate::adapters::*`

#### Implementation Checklist

- [ ] **Ansible Adapter**:
  - [ ] Move directory: `src/infrastructure/external_tools/ansible/adapter/` → `src/adapters/ansible/`
  - [ ] Update `src/adapters/mod.rs` to include `pub mod ansible;`
  - [ ] Update imports in application layer (~3 files)
  - [ ] Run `cargo check` and fix import issues
- [ ] **LXD Adapter**:
  - [ ] Move directory: `src/infrastructure/external_tools/lxd/adapter/` → `src/adapters/lxd/`
  - [ ] Update `src/adapters/mod.rs` to include `pub mod lxd;`
  - [ ] Update imports if any (search codebase)
  - [ ] Run `cargo check` and fix import issues
- [ ] **OpenTofu Adapter**:
  - [ ] Move directory: `src/infrastructure/external_tools/tofu/adapter/` → `src/adapters/tofu/`
  - [ ] Update `src/adapters/mod.rs` to include `pub mod tofu;`
  - [ ] Update imports in application layer (~6 files)
  - [ ] Run `cargo check` and fix import issues
- [ ] Run full test suite: `cargo test`

---

## Phase 2: Cleanup

Restructure remaining external_tools and finalize documentation.

### Proposal #3: Restructure src/infrastructure/external_tools/ and Update Documentation

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium (Clean final state)  
**Effort**: 🔵 Low (Just cleanup and docs)  
**Priority**: P2  
**Depends On**: Proposals #1, #2

#### Problem

After moving adapters, `src/infrastructure/external_tools/` will only contain application-specific template code. The structure should be cleaned up and documentation updated.

#### Proposed Solution

1. Clean up `src/infrastructure/external_tools/` structure:
   - Remove `adapter/` subdirectories (already moved)
   - Keep `template/` subdirectories (application-specific)
   - Update `mod.rs` files to reflect new structure
2. Update `src/infrastructure/external_tools/mod.rs`:
   - Remove adapter re-exports
   - Keep template module declarations
3. Update documentation:
   - Update any references in `docs/` to the new structure
   - Update module-level documentation in moved files
   - Update codebase architecture documentation
   - Clarify that `external_tools/` now contains only application-specific tool configuration

#### Implementation Checklist

- [ ] **Clean up external_tools Structure**:
  - [ ] Remove `src/infrastructure/external_tools/ansible/adapter/` (already moved)
  - [ ] Remove `src/infrastructure/external_tools/lxd/adapter/` (already moved)
  - [ ] Remove `src/infrastructure/external_tools/tofu/adapter/` (already moved)
  - [ ] Keep `src/infrastructure/external_tools/ansible/template/` (application-specific)
  - [ ] Keep `src/infrastructure/external_tools/tofu/template/` (application-specific)
  - [ ] Update `src/infrastructure/external_tools/ansible/mod.rs` to remove adapter re-exports
  - [ ] Update `src/infrastructure/external_tools/lxd/mod.rs` (may only have adapter - consider removing entire lxd/ if empty)
  - [ ] Update `src/infrastructure/external_tools/tofu/mod.rs` to remove adapter re-exports
  - [ ] Update `src/infrastructure/external_tools/mod.rs` to reflect new structure
  - [ ] Run `cargo check` to verify no broken references
- [ ] **Update Documentation**:
  - [ ] Update `docs/codebase-architecture.md` with new structure
  - [ ] Update `.github/copilot-instructions.md` if needed
  - [ ] Update module-level docs in `src/adapters/mod.rs`
  - [ ] Update module-level docs in `src/infrastructure/external_tools/mod.rs`
  - [ ] Check for any other doc references to old paths
- [ ] **Final Verification**:
  - [ ] Run full linter: `cargo run --bin linter all`
  - [ ] Run full test suite: `cargo test`
  - [ ] Run E2E tests: `cargo run --bin e2e-tests-full`
  - [ ] Verify git history is clean with `git status`

## 📅 Timeline

**Estimated Duration**: 2-3 hours

**Breakdown by Phase**:

- **Phase 0**: 15-30 minutes (setup and structure)
- **Phase 1**: 60-90 minutes (migration and import updates)
- **Phase 2**: 15-30 minutes (cleanup and documentation)

**Sprint Planning**: Can be completed in a single session. Should be done atomically to avoid breaking the build.

## ✅ Review Process

### Approval Criteria

1. All proposals implemented according to checklists
2. `cargo check` passes without errors
3. `cargo test` passes all tests
4. `cargo run --bin linter all` passes
5. E2E tests pass
6. Documentation updated
7. Git history is clean (no broken intermediate commits)

### Completion Criteria

- [ ] All code moved to new locations
- [ ] All imports updated
- [ ] All tests passing
- [ ] All linters passing
- [ ] Documentation updated
- [ ] Old directories removed
- [ ] Code reviewed and approved
- [ ] Changes committed and pushed

## 📚 Related Documentation

- [External Tool Adapters Organization ADR](../../decisions/external-tool-adapters-organization.md)
- [Module Organization Guide](../contributing/module-organization.md)
- [Development Principles](../development-principles.md)

## 🔗 Migration Commands

### Useful Git Commands

```bash
# Move files preserving history
git mv src/shared/ssh src/adapters/ssh

# Find all files importing from old location
grep -r "use crate::shared::ssh" src/
grep -r "use crate::infrastructure::external_tools" src/

# Search and replace imports (be careful!)
find src/ -type f -name "*.rs" -exec sed -i 's/crate::shared::ssh/crate::adapters::ssh/g' {} +
find src/ -type f -name "*.rs" -exec sed -i 's/crate::infrastructure::external_tools::\(.*\)::adapter/crate::adapters::\1/g' {} +

# Verify no broken imports
cargo check 2>&1 | grep "unresolved import"
```

### Safety Checks

```bash
# Before each phase
cargo check
cargo test
git status

# After each phase
cargo check
cargo test
cargo run --bin linter all
git add -A
git commit -m "refactor: [description]"
```

## 🎯 Success Metrics

- **All adapters in one location**: `src/adapters/`
- **Clear separation**: Templates in `src/infrastructure/templates/`
- **No broken imports**: `cargo check` passes
- **All tests passing**: 100% test success rate
- **Improved discoverability**: Easier for new contributors to find adapter code
- **Better semantics**: Clear distinction between utilities, adapters, and application infrastructure
