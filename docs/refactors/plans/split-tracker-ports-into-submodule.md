# Split `tracker_ports.rs` into Submodule

## Overview

The file `src/testing/e2e/containers/tracker_ports.rs` (562 lines) has grown to contain 4 public types, 9 internal JSON deserialization structs, helper functions, a deprecated type alias, and tests — all mixed together in a single file. This refactoring splits it into a focused submodule folder under `containers/`, removes the deprecated `E2eEnvironmentInfo` alias, and renames the module to better reflect its actual purpose.

**Target Files:**

- `src/testing/e2e/containers/tracker_ports.rs` (split into submodule)
- `src/testing/e2e/containers/mod.rs` (update module declaration + re-exports)
- `src/testing/e2e/tasks/black_box/generate_config.rs` (replace deprecated alias usage)
- `src/bin/e2e_deployment_workflow_tests.rs` (update import path)

**Scope:**

- Split one 562-line file into a submodule folder with focused files
- Rename module `tracker_ports` → `tracker_container_setup`
- Remove deprecated type alias `E2eEnvironmentInfo` (replace all usages with `E2eConfigEnvironment`)
- Maintain identical public API (except alias removal)
- Zero behavior changes

## Progress Tracking

**Total Active Proposals**: 3
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 3

### Phase Summary

- **Phase 0 - Remove deprecated alias (High Impact, Low Effort)**: ⏳ 0/1 completed (0%)
- **Phase 1 - Rename and split module (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)
- **Phase 2 - Update all external imports (Medium Impact, Low Effort)**: ⏳ 0/1 completed (0%)

## Key Problems Identified

### 1. Oversized file mixing concerns

A single 562-line file contains 4 public types, 9 internal JSON structs, helpers, and tests. Finding a specific type requires scrolling or searching. Internal deserialization details sit alongside the public API.

### 2. Misleading module name

The module is called `tracker_ports` but it holds environment configuration, runtime state, container port mapping, and JSON format parsing — far more than "tracker ports."

### 3. Deprecated alias creating confusion

`E2eEnvironmentInfo` is a type alias for `E2eConfigEnvironment` kept for backward compatibility. It's used in 4 places in `generate_config.rs` and re-exported from `containers/mod.rs`. Having two names for the same type is confusing and should be cleaned up.

### 4. No separation between public API and internal details

The 9 JSON deserialization structs (`ConfigJson`, `EnvironmentJson`, `UserInputs`, etc.) are implementation details of `TrackerPorts::from_env_file()` but sit at the same module level as the public types.

## Dependency Analysis

**Key finding**: `tracker_ports.rs` is completely isolated from the rest of `containers/`. No file in the container lifecycle cluster (`provisioned.rs`, `config_builder.rs`, `image_builder.rs`, `errors.rs`, `executor.rs`, `timeout.rs`) imports anything from `tracker_ports.rs`.

### Internal dependencies within tracker_ports.rs

```text
E2eRuntimeEnvironment
  ├── E2eConfigEnvironment
  └── ContainerPorts (leaf — zero deps)

E2eConfigEnvironment
  ├── TrackerPorts
  └── extract_ssh_port_from_file() → format structs

TrackerPorts
  └── format structs (ConfigJson, EnvironmentJson, TrackerBinding)
```

### External consumers (only 2 files)

| File                                                 | Types used                                                        |
| ---------------------------------------------------- | ----------------------------------------------------------------- |
| `src/bin/e2e_deployment_workflow_tests.rs`           | `ContainerPorts`, `E2eConfigEnvironment`, `E2eRuntimeEnvironment` |
| `src/testing/e2e/tasks/black_box/generate_config.rs` | `E2eEnvironmentInfo` (deprecated alias), `TrackerPorts`           |

### Why a subfolder under `containers/` (not flat files)

Despite being isolated, these types are always consumed alongside container types in the E2E binary. `E2eRuntimeEnvironment` wraps `ContainerPorts` which is conceptually tied to Docker containers. A subfolder keeps the types grouped while separating public API from internal JSON parsing. Flat files in `containers/` would add 5+ files to an already 8-file directory.

## Refactoring Phases

---

## Phase 0: Remove Deprecated Alias (Quick Win)

This phase can be done independently and immediately. It has no structural risk and simplifies the subsequent phases.

### Proposal 0: Remove `E2eEnvironmentInfo` type alias

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: None

#### Problem

`E2eEnvironmentInfo` is a deprecated alias for `E2eConfigEnvironment`:

```rust
// src/testing/e2e/containers/tracker_ports.rs
/// @deprecated Use `E2eConfigEnvironment` instead
pub type E2eEnvironmentInfo = E2eConfigEnvironment;
```

It's used in 4 places in `generate_config.rs` and re-exported from `containers/mod.rs`. Two names for the same type causes confusion.

#### Proposed Solution

Replace all usages of `E2eEnvironmentInfo` with `E2eConfigEnvironment` and delete the alias.

#### Implementation Checklist

- [ ] **Step 0.1**: In `src/testing/e2e/tasks/black_box/generate_config.rs`:
  - Replace `use crate::testing::e2e::containers::E2eEnvironmentInfo` → `use crate::testing::e2e::containers::E2eConfigEnvironment`
  - Replace all 4 occurrences of `E2eEnvironmentInfo` → `E2eConfigEnvironment` in function signatures and doc comments
- [ ] **Step 0.2**: In `src/testing/e2e/containers/mod.rs`:
  - Change `pub use tracker_ports::{E2eEnvironmentInfo, TrackerPorts}` → `pub use tracker_ports::{E2eConfigEnvironment, TrackerPorts}`
- [ ] **Step 0.3**: In `src/testing/e2e/containers/tracker_ports.rs`:
  - Delete the line `pub type E2eEnvironmentInfo = E2eConfigEnvironment;` and its doc comment
- [ ] **Step 0.4**: Run pre-commit checks

  ```bash
  ./scripts/pre-commit.sh
  ```

#### Testing Strategy

`cargo build` + `cargo test` — the compiler will catch any missed references. No behavior changes.

---

## Phase 1: Rename and Split Module (Core Refactor)

This is the main structural change. It converts the single file into a submodule folder with focused files.

### Proposal 1: Convert `tracker_ports.rs` into `tracker_container_setup/` submodule

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P1
**Depends On**: Proposal 0

#### Problem

562 lines mixing 4 public types, 9 internal structs, helpers, and tests in a single file named `tracker_ports` that doesn't accurately describe its contents.

#### Proposed Solution

Create `src/testing/e2e/containers/tracker_container_setup/` with this structure:

```text
tracker_container_setup/
├── mod.rs                      # Public API: re-exports + module docs
├── config.rs                   # E2eConfigEnvironment + extract_ssh_port_from_file
├── runtime.rs                  # E2eRuntimeEnvironment
├── container_ports.rs          # ContainerPorts
├── tracker_ports.rs            # TrackerPorts + extract_port_from_bind_address
└── formats/                    # Internal JSON deserialization (private)
    ├── mod.rs                  # Re-exports + shared TrackerBinding
    ├── creation_config.rs      # New format structs (ConfigJson, etc.)
    └── environment_json.rs     # Old format structs (EnvironmentJson, etc.)
```

**Type placement rationale:**

| File                          | Type                                       | Why here                                                          |
| ----------------------------- | ------------------------------------------ | ----------------------------------------------------------------- |
| `config.rs`                   | `E2eConfigEnvironment`                     | Owns JSON generation and file-reading; depends on `TrackerPorts`  |
| `runtime.rs`                  | `E2eRuntimeEnvironment`                    | Separate concern: runtime state wrapping config + container ports |
| `container_ports.rs`          | `ContainerPorts`                           | Leaf type, zero deps, represents Docker-specific port mapping     |
| `tracker_ports.rs`            | `TrackerPorts`                             | Core port extraction logic; depends on `formats/`                 |
| `formats/creation_config.rs`  | `ConfigJson`, `SshCredentialsConfig`, etc. | New JSON format — isolated internal detail                        |
| `formats/environment_json.rs` | `EnvironmentJson`, `UserInputs`, etc.      | Old JSON format — isolated internal detail                        |
| `formats/mod.rs`              | `TrackerBinding`                           | Shared between both formats                                       |

**Implementation is split into small, verifiable steps:**

#### Implementation Checklist

Each step ends with a passing `./scripts/pre-commit.sh`. The old file and new submodule coexist briefly during transition.

- [ ] **Step 1.1**: Create `tracker_container_setup/` folder and `mod.rs` that re-exports everything from the old `tracker_ports` module (thin proxy):

  ```rust
  //! Tracker container setup for E2E testing
  //!
  //! Provides types for managing tracker container configurations and
  //! runtime state in E2E tests.

  // Temporarily re-export everything from the original module
  pub use super::tracker_ports::ContainerPorts;
  pub use super::tracker_ports::E2eConfigEnvironment;
  pub use super::tracker_ports::E2eRuntimeEnvironment;
  pub use super::tracker_ports::TrackerPorts;
  ```

  In `containers/mod.rs`: add `pub mod tracker_container_setup;` (keep `tracker_ports` too for now).
  Run `./scripts/pre-commit.sh`.

- [ ] **Step 1.2**: Create `formats/mod.rs`, `formats/creation_config.rs`, `formats/environment_json.rs`:
  - Move `TrackerBinding` to `formats/mod.rs`
  - Move `ConfigJson`, `SshCredentialsConfig`, `TrackerConfigCreation`, `TrackerCoreConfig`, `HttpApiConfigCreation` to `formats/creation_config.rs`
  - Move `EnvironmentJson`, `UserInputs`, `TrackerConfig`, `HttpApiConfig`, `default_ssh_port()`, `default_api_bind_address()` to `formats/environment_json.rs`
  - Add `pub(super)` visibility so they're accessible within the submodule but not outside
  - In the old `tracker_ports.rs`: replace inline structs with imports from the new `formats` module

    Run `./scripts/pre-commit.sh`.

- [ ] **Step 1.3**: Create `tracker_container_setup/container_ports.rs`:
  - Move `ContainerPorts` struct + impl + tests from old file
  - Update `mod.rs` to export from local file instead of re-exporting from old module

    Run `./scripts/pre-commit.sh`.

- [ ] **Step 1.4**: Create `tracker_container_setup/tracker_ports.rs`:
  - Move `TrackerPorts` struct + `Default` impl + all methods + `extract_port_from_bind_address()` + 5 tests

  - Update `mod.rs` to export from local file
    Run `./scripts/pre-commit.sh`.

- [ ] **Step 1.5**: Create `tracker_container_setup/config.rs`:
  - Move `E2eConfigEnvironment` struct + impl + `extract_ssh_port_from_file()`
  - Update `mod.rs` to export from local file
    Run `./scripts/pre-commit.sh`.

- [ ] **Step 1.6**: Create `tracker_container_setup/runtime.rs`:
  - Move `E2eRuntimeEnvironment` struct + impl
  - Update `mod.rs` to export from local file
    Run `./scripts/pre-commit.sh`.

- [ ] **Step 1.7**: Verify the old `tracker_ports.rs` is now empty (or has only the old module declaration):
  - Delete the old `tracker_ports.rs` file
  - Remove `pub mod tracker_ports;` from `containers/mod.rs`
  - Update `containers/mod.rs` re-exports to point to `tracker_container_setup`:

    ```rust
    pub use tracker_container_setup::{E2eConfigEnvironment, TrackerPorts};
    ```

    Run `./scripts/pre-commit.sh`.

#### Testing Strategy

Each step runs `./scripts/pre-commit.sh` which includes `cargo build`, `cargo test`, clippy, and formatting checks. The key insight is that during Steps 1.1-1.6, both the old module and the new submodule coexist, so nothing breaks. Step 1.7 is the only "swap" step.

---

## Phase 2: Update External Imports (Cleanup)

### Proposal 2: Update external import paths to use new module name

**Status**: ⏳ Not Started
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P2
**Depends On**: Proposal 1

#### Problem

After Phase 1, external callers still import via the re-exports in `containers/mod.rs`. One file (`e2e_deployment_workflow_tests.rs`) imports directly from the old module path `containers::tracker_ports::`. This needs to be updated to the new path.

#### Proposed Solution

Update import paths in external consumers to use `tracker_container_setup` directly where they already import specific types.

#### Implementation Checklist

- [ ] **Step 2.1**: In `src/bin/e2e_deployment_workflow_tests.rs`:
  - Change:

    ```rust
    use torrust_tracker_deployer_lib::testing::e2e::containers::tracker_ports::{
        ContainerPorts, E2eConfigEnvironment, E2eRuntimeEnvironment,
    };
    ```

  - To:

    ```rust
    use torrust_tracker_deployer_lib::testing::e2e::containers::tracker_container_setup::{
        ContainerPorts, E2eConfigEnvironment, E2eRuntimeEnvironment,
    };
    ```

    Run `./scripts/pre-commit.sh`.

- [ ] **Step 2.2**: Verify no other files reference the old `tracker_ports` module path:

  ```bash
  grep -r "tracker_ports" src/ tests/
  ```

  Run `./scripts/pre-commit.sh`.

#### Testing Strategy

Compiler will catch any missed imports. `grep` search confirms no stale references remain.

---

## Final Module Structure

After all phases are complete:

```text
src/testing/e2e/containers/
├── mod.rs                                  # (updated re-exports)
├── actions/
│   ├── mod.rs
│   ├── ssh_key_setup.rs
│   └── ssh_wait.rs
├── config_builder.rs
├── errors.rs
├── executor.rs
├── image_builder.rs
├── provisioned.rs
├── timeout.rs
└── tracker_container_setup/                # NEW (was tracker_ports.rs)
    ├── mod.rs                              # Public API re-exports (~20 lines)
    ├── config.rs                           # E2eConfigEnvironment (~140 lines)
    ├── runtime.rs                          # E2eRuntimeEnvironment (~60 lines)
    ├── container_ports.rs                  # ContainerPorts (~50 lines)
    ├── tracker_ports.rs                    # TrackerPorts (~160 lines)
    └── formats/                            # Private JSON deserialization
        ├── mod.rs                          # TrackerBinding + re-exports (~15 lines)
        ├── creation_config.rs              # New format structs (~35 lines)
        └── environment_json.rs             # Old format structs (~45 lines)
```

## Timeline

- **Start Date**: TBD
- **Estimated Duration**: ~1 hour (3 phases)

## Review Process

### Approval Criteria

- [ ] Plan reviewed
- [ ] Aligns with [Development Principles](../../development-principles.md)
- [ ] Implementation plan is clear and actionable

### Completion Criteria

- [ ] All 3 proposals implemented
- [ ] `./scripts/pre-commit.sh` passes
- [ ] No remaining references to `tracker_ports` module path
- [ ] No remaining `E2eEnvironmentInfo` alias
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

## Related Documentation

- [Development Principles](../../development-principles.md)
- [Module Organization](../../contributing/module-organization.md)
- [DDD Layer Placement](../../contributing/ddd-layer-placement.md)

---

**Created**: February 5, 2026
**Last Updated**: February 5, 2026
**Status**: 📋 Planning
