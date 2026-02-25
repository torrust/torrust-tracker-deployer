# Extract Shared Types Package

## 📋 Overview

Extract cross-cutting value objects and traits from `src/shared/` and `src/domain/` into a new workspace package (`packages/deployer-types/`). This package provides the foundational types that both the SDK package and the root crate depend on — enabling the SDK to be independently consumable without pulling the entire deployer as a transitive dependency.

This is the third step in a 4-part incremental plan:

1. [Presentation CLI/SDK Separation](presentation-cli-sdk-separation.md) — Separate CLI from SDK
2. [Extract SDK Workspace Package](extract-sdk-workspace-package.md) — Move SDK into `packages/sdk/` (**prerequisite**)
3. **This plan** — Extract shared types into `packages/deployer-types/`
4. [SDK DDD Layer Boundary Fixes](sdk-ddd-layer-boundary-fixes.md) — Fix remaining DDD violations

**Target Files:**

- `src/shared/` — value objects and traits currently in the cross-cutting layer
- `src/domain/environment/name.rs` — `EnvironmentName` / `EnvironmentNameError`
- `packages/deployer-types/` — new package location

**Scope:**

- Create `packages/deployer-types/` with shared value objects and traits
- Move types that SDK consumers encounter (directly or transitively) into the new package
- Both the root crate and the SDK package depend on `torrust-deployer-types`
- Replace the SDK's dependency on the root crate for these types with a dependency on the types package

**Out of Scope:**

- Moving application-layer command handler types (they stay in the root crate)
- Moving infrastructure-specific types (e.g., `CommandExecutor`, `CommandResult`)
- Removing the SDK's dependency on the root crate entirely (the SDK still needs application-layer handlers)

## 📊 Progress Tracking

**Total Active Proposals**: 2
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 2

### Phase Summary

- **Phase 0 - Create Package with SDK-Facing Types (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)
- **Phase 1 - Move Additional Shared Types (Medium Impact, Low Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

None.

### Postponed Proposals

None.

## 🎯 Key Problems Identified

### 1. SDK Consumers Encounter Types from Multiple Internal Layers

When using the SDK, consumers transitively encounter value objects from `shared/` (e.g., `DomainName`, `Username`, `Password`) and from `domain/` (e.g., `EnvironmentName`). These types are semantically cross-cutting — they're simple validated value objects with no business logic — but they live inside the monolithic root crate.

### 2. SDK Package Cannot Be Fully Independent

After Plan 2, the SDK package depends on the root crate to access types like `EnvironmentName`, `DomainName`, `Username`, etc. Extracting these into a lightweight types package lets the SDK depend on `torrust-deployer-types` instead, reducing coupling.

### 3. No Canonical Location for Cross-Cutting Types

The `shared/` module is documented as "cross-cutting concerns" but lives inside the root crate. Having a dedicated types package establishes an explicit, importable boundary for foundational types.

## 🚀 Refactoring Phases

---

## Phase 0: Create Package with SDK-Facing Types (High Impact, Medium Effort)

Move the value objects and traits that SDK consumers directly or transitively encounter into the new package.

### Proposal #0: Create `packages/deployer-types/` with SDK-Facing Shared Types

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P0
**Depends On**: [Extract SDK Workspace Package](extract-sdk-workspace-package.md) (Plan 2)
**Completed**: -
**Commit**: -

#### Problem

SDK consumers encounter the following types from the root crate's `shared/` and `domain/` modules:

**Direct SDK imports (used as method parameters or returned types):**

| Type                   | Current Location            | SDK Usage                     |
| ---------------------- | --------------------------- | ----------------------------- |
| `EnvironmentName`      | `domain::environment::name` | Parameter to most SDK methods |
| `EnvironmentNameError` | `domain::environment::name` | Returned in `SdkError`        |

**Transitive types (encountered via DTOs returned by SDK methods):**

| Type                         | Current Location                             | Appears In                                                    |
| ---------------------------- | -------------------------------------------- | ------------------------------------------------------------- |
| `DomainName`                 | `shared::domain_name`                        | `DnsWarning.domain`, `EnvironmentCreationConfig` sub-sections |
| `Username`                   | `shared::username`                           | `EnvironmentCreationConfig` (SSH, Grafana)                    |
| `Password` / `PlainPassword` | `shared::secrets::password`                  | `EnvironmentCreationConfig` (MySQL, Grafana)                  |
| `ApiToken` / `PlainApiToken` | `shared::secrets::api_token`                 | `EnvironmentCreationConfig` (tracker API)                     |
| `Email`                      | `shared::email`                              | `EnvironmentCreationConfig` (HTTPS/Let's Encrypt)             |
| `ServiceEndpoint`            | `shared::service_endpoint`                   | `EnvironmentCreationConfig` (Prometheus)                      |
| `ExposeSecret`               | `shared::secrets` (re-export from `secrecy`) | Trait for accessing secret values                             |

**Error infrastructure types (used by all SDK error types):**

| Type        | Current Location           | SDK Usage                                             |
| ----------- | -------------------------- | ----------------------------------------------------- |
| `Traceable` | `shared::error::traceable` | Trait implemented by all `*CommandHandlerError` types |
| `ErrorKind` | `shared::error::kind`      | Classification in `Traceable::kind()`                 |

**Internal SDK types (not visible to consumers but needed for compilation):**

| Type          | Current Location | SDK Usage                                           |
| ------------- | ---------------- | --------------------------------------------------- |
| `Clock`       | `shared::clock`  | Trait used internally by `Deployer`                 |
| `SystemClock` | `shared::clock`  | Default `Clock` implementation in `DeployerBuilder` |

All of these are generic, validated value objects or cross-cutting traits — none contain business logic. They belong in a shared types package.

#### Proposed Solution

Create `packages/deployer-types/` containing these types. The package has minimal external dependencies (only validation logic, `serde`, `secrecy`, `thiserror`).

**Package structure:**

```text
packages/deployer-types/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs                 ← Public API (re-exports all types)
    ├── clock.rs               ← Clock trait + SystemClock
    ├── domain_name.rs         ← DomainName + DomainNameError
    ├── email.rs               ← Email + EmailError
    ├── environment_name.rs    ← EnvironmentName + EnvironmentNameError
    ├── error/
    │   ├── mod.rs
    │   ├── kind.rs            ← ErrorKind
    │   └── traceable.rs       ← Traceable trait
    ├── secrets/
    │   ├── mod.rs
    │   ├── api_token.rs       ← ApiToken + PlainApiToken
    │   └── password.rs        ← Password + PlainPassword
    ├── service_endpoint.rs    ← ServiceEndpoint + InvalidServiceEndpointUrl
    └── username.rs            ← Username + UsernameError
```

**`packages/deployer-types/Cargo.toml`:**

```toml
[package]
name = "torrust-deployer-types"
version = "0.1.0"
edition = "2021"
description = "Shared value objects and traits for the Torrust Tracker Deployer"
license = "MIT"

[dependencies]
chrono = { version = "0.4", features = ["serde"] }
email_address = "0.2.9"
secrecy = { version = "0.10", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
url = { version = "2.0", features = ["serde"] }
```

**Dependency graph after this plan:**

```text
torrust-deployer-types                              ← new lightweight types package
    ↑                     ↑
torrust-tracker-deployer  torrust-tracker-deployer-sdk
    (root crate)              (SDK package)
```

Both the root crate and the SDK package import `torrust-deployer-types`. The SDK no longer needs the root crate just for value objects.

#### Rationale

- **Lightweight**: The types package has minimal dependencies (no tokio, no infrastructure)
- **Single source of truth**: Both crate and SDK import the same types — no wrapper duplication
- **EnvironmentName migration**: The user explicitly identified `EnvironmentName` as a cross-cutting type that should be shared — it's a validated string with no domain business logic
- **Follows workspace pattern**: Matches existing `packages/linting/` and `packages/dependency-installer/`

#### Benefits

- ✅ SDK depends on a lightweight types package instead of the full root crate for value objects
- ✅ `EnvironmentName` can be used across packages without circular dependencies
- ✅ Clear, importable boundary for cross-cutting types
- ✅ Types package can be independently versioned and published
- ✅ Reduces the root crate's surface area

#### Implementation Checklist

- [ ] Create `packages/deployer-types/` directory structure
- [ ] Create `packages/deployer-types/Cargo.toml`
- [ ] Create `packages/deployer-types/README.md`
- [ ] Move `src/shared/domain_name.rs` → `packages/deployer-types/src/domain_name.rs`
- [ ] Move `src/shared/email.rs` → `packages/deployer-types/src/email.rs`
- [ ] Move `src/shared/username.rs` → `packages/deployer-types/src/username.rs`
- [ ] Move `src/shared/clock.rs` → `packages/deployer-types/src/clock.rs`
- [ ] Move `src/shared/service_endpoint.rs` → `packages/deployer-types/src/service_endpoint.rs`
- [ ] Move `src/shared/secrets/` → `packages/deployer-types/src/secrets/`
- [ ] Move `src/shared/error/kind.rs` → `packages/deployer-types/src/error/kind.rs`
- [ ] Move `src/shared/error/traceable.rs` → `packages/deployer-types/src/error/traceable.rs`
- [ ] Move `src/domain/environment/name.rs` → `packages/deployer-types/src/environment_name.rs`
- [ ] Create `packages/deployer-types/src/lib.rs` with public re-exports
- [ ] Add `"packages/deployer-types"` to workspace members in root `Cargo.toml`
- [ ] Add `torrust-deployer-types = { path = "packages/deployer-types" }` to root `[dependencies]`
- [ ] Update `src/shared/mod.rs` to re-export from `torrust_deployer_types` (backward compat):

  ```rust
  pub use torrust_deployer_types::{DomainName, DomainNameError, Email, EmailError, ...};
  ```

- [ ] Update `src/domain/mod.rs` to re-export `EnvironmentName` from `torrust_deployer_types`
- [ ] Update SDK package (`packages/sdk/Cargo.toml`) to depend on `torrust-deployer-types`
- [ ] Update SDK imports to use `torrust_deployer_types::EnvironmentName` instead of going through the root crate
- [ ] Move unit tests for migrated types into `packages/deployer-types/`
- [ ] Verify `cargo build --workspace` succeeds
- [ ] Verify `cargo test --workspace` succeeds
- [ ] Run linter and fix any issues

#### Testing Strategy

- All unit tests for migrated types move to the new package
- The root crate's re-exports ensure all existing code continues to compile
- `cargo test --workspace` verifies everything works together
- SDK examples must compile and work identically

#### Types NOT Moved (Remain in `src/shared/`)

The following types are infrastructure-specific utilities, not value objects, and remain in the root crate's `shared/` module:

| Type              | Reason to Keep                            |
| ----------------- | ----------------------------------------- |
| `CommandExecutor` | Infrastructure tool (runs shell commands) |
| `CommandResult`   | Infrastructure tool result                |
| `CommandError`    | Infrastructure error type                 |

---

## Phase 1: Move Additional Shared Types (Medium Impact, Low Effort)

### Proposal #1: Migrate Additional Cross-Cutting Types on Demand

**Status**: ⏳ Not Started
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P1
**Depends On**: Proposal #0
**Completed**: -
**Commit**: -

#### Problem

As the codebase evolves, additional types in `shared/` or `domain/` may be identified as cross-cutting value objects that belong in the types package. This proposal establishes the guidelines for when and how to move them.

#### Proposed Solution

A type should be moved to `torrust-deployer-types` when it meets ALL of these criteria:

1. **Value object or trait** — no business logic, only validation and conversion
2. **Used by multiple packages** — needed by at least 2 workspace packages (e.g., SDK + root crate)
3. **No internal dependencies** — does not import from `application/`, `domain/` (except other types), or `infrastructure/`
4. **Stable API** — the type's public interface is unlikely to change frequently

**Current candidates for future migration:**

| Type                                 | Current Location        | Status                    |
| ------------------------------------ | ----------------------- | ------------------------- |
| `InstanceName` / `InstanceNameError` | `domain::instance_name` | Candidate if SDK needs it |
| `ProfileName` / `ProfileNameError`   | `domain::profile_name`  | Candidate if SDK needs it |
| `Provider` / `ProviderConfig`        | `domain::provider`      | Candidate if SDK needs it |

#### Rationale

Not every type needs to move immediately. The types package should grow organically as the SDK's needs expand.

#### Implementation Checklist

- [ ] Review new SDK methods for types that should be in the types package
- [ ] Apply the 4 criteria above to determine eligibility
- [ ] Move types following the same pattern as Proposal #0
- [ ] Update re-exports in `src/shared/mod.rs` or `src/domain/mod.rs`
- [ ] Verify all tests pass

---

## 📈 Timeline

- **Start Date**: TBD (after Plan 2 is complete)
- **Actual Completion**: TBD

## 🔍 Review Process

### Approval Criteria

- [ ] Package name confirmed (`torrust-deployer-types`)
- [ ] Type selection reviewed and approved
- [ ] External dependency list is minimal

### Completion Criteria

- [ ] `packages/deployer-types/` exists as a workspace package
- [ ] All identified types migrated with their unit tests
- [ ] Root crate re-exports maintain backward compatibility
- [ ] SDK package depends on `torrust-deployer-types` for value objects
- [ ] `cargo test --workspace` passes
- [ ] All linters pass

## 📚 Related Documentation

- [Codebase Architecture](../../codebase-architecture.md)
- [Extract SDK Workspace Package](extract-sdk-workspace-package.md) (prerequisite)
- [SDK DDD Layer Boundary Fixes](sdk-ddd-layer-boundary-fixes.md) (next step)

## 💡 Notes

- The package name `torrust-deployer-types` was chosen to be short and descriptive. Alternatives considered: `torrust-tracker-deployer-types` (too long), `deployer-types` (missing namespace prefix)
- `EnvironmentName` currently lives in `domain/` but is semantically a validated string — the same category as `DomainName`, `Username`, `Email`. Moving it to the types package is justified because it has no business logic or domain dependencies
- Re-export wrappers in `src/shared/mod.rs` and `src/domain/mod.rs` can be removed in a later cleanup once all internal consumers are updated to import from `torrust_deployer_types` directly
- The `ExposeSecret` trait is a re-export of `secrecy::ExposeSecret` — the types package re-exports it for convenience so SDK consumers don't need to depend on `secrecy` directly
- Consider whether `Traceable` and `ErrorKind` should move in Phase 0 or be deferred — they are used by all error types, not just SDK-facing ones. Including them in Phase 0 is recommended because SDK consumers need them for error handling

---

**Created**: 2026-02-24
**Last Updated**: 2026-02-24
**Status**: 📋 Planning
