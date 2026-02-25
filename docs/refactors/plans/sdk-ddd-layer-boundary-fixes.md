# SDK DDD Layer Boundary Fixes

## 📋 Overview

Eliminate remaining DDD layer boundary violations in the SDK after the structural refactoring (Plans 1–3) is complete. Two categories of genuine DDD violations remain:

1. **Domain types leaking via application errors**: Re-exported `*CommandHandlerError` enums contain `#[from]` domain types (`RepositoryError`, `StateTypeError`, `ReleaseStep`)
2. **Presentation → Infrastructure coupling**: `DeployerBuilder` directly instantiates `RepositoryFactory` (infrastructure type)

This is the fourth and final step in a 4-part incremental plan:

1. [Presentation CLI/SDK Separation](presentation-cli-sdk-separation.md) — Separate CLI from SDK
2. [Extract SDK Workspace Package](extract-sdk-workspace-package.md) — Move SDK into `packages/sdk/`
3. [Extract Shared Types Package](extract-shared-types-package.md) — Create a shared types package
4. **This plan** — Fix remaining DDD violations in the SDK

**Note on superseded proposals**: The original plan included two additional proposals that have been superseded by Plans 1–3:

- **CLI/SDK Separation** → moved to [Plan 1](presentation-cli-sdk-separation.md)
- **Replace `DomainName` with `String` in `DnsWarning`** → superseded by [Plan 3](extract-shared-types-package.md). Once `DomainName` lives in the `torrust-deployer-types` package, it is a shared cross-cutting type accessible to SDK consumers without coupling to the root crate. No conversion to `String` is needed.

**Target Files:**

- `src/presentation/sdk/builder.rs` (or `packages/sdk/src/builder.rs` after Plan 2) — infrastructure coupling
- `src/presentation/sdk/deployer.rs` (or `packages/sdk/src/deployer.rs` after Plan 2) — holds infrastructure types as fields
- `src/application/command_handlers/*/errors.rs` — error enums with domain types
- `src/application/errors.rs` — new application-layer wrapper types

**Scope:**

- Introduce application-layer wrapper errors for `RepositoryError`, `StateTypeError`, and `ReleaseStep`
- Move infrastructure wiring out of the SDK builder into the bootstrap layer
- Preserve full backward compatibility for existing CLI controllers

**Out of Scope:**

- Changing the internal application → domain dependency (that direction is correct per DDD)
- Adding new SDK methods or modifying return types
- Refactoring error types that are NOT re-exported from the SDK (`RegisterCommandHandlerError`, `RenderCommandHandlerError`)

## 📊 Progress Tracking

**Total Active Proposals**: 2
**Total Postponed**: 0
**Total Discarded**: 1
**Completed**: 0
**In Progress**: 0
**Not Started**: 2

### Phase Summary

- **Phase 0 - Core Fix: Error Wrappers (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)
- **Phase 1 - Infrastructure Decoupling (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

- **Replace `DomainName` with `String` in `DnsWarning`**: Superseded by [Extract Shared Types Package](extract-shared-types-package.md). Once `DomainName` is in the `torrust-deployer-types` package, SDK consumers can use it directly without coupling to the root crate.

### Postponed Proposals

None.

## 🎯 Key Problems Identified

### 1. Presentation Layer Directly Instantiates Infrastructure Types (SEVERE)

`DeployerBuilder::build()` creates `RepositoryFactory` (from `crate::infrastructure`) and `SystemClock` (from `crate::shared`). The `Deployer` struct stores `Arc<RepositoryFactory>` as a field. This means the presentation layer has a compile-time dependency on the infrastructure layer, violating the Dependency Rule.

```rust
// src/presentation/sdk/builder.rs — CURRENT (violates DDD)
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::SystemClock;

pub fn build(self) -> Result<Deployer, DeployerBuildError> {
    let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
    let clock = Arc::new(SystemClock);
    // ...
}
```

The existing CLI bootstrap (`src/bootstrap/container.rs`) already does this wiring correctly — it lives in the `bootstrap` layer, which is explicitly allowed to cross-cut all layers for dependency injection.

**Note**: `SystemClock` is from the `shared` layer (cross-cutting), not infrastructure. After Plan 3, it will be in the `torrust-deployer-types` package. The real violation is `RepositoryFactory` from the infrastructure layer.

### 2. Domain Types Leak Through Application Error Enums (MODERATE)

The SDK re-exports 11 application `*CommandHandlerError` enums. Nine of them contain variants with domain-layer types as payloads:

| Domain Type       | Pattern                                                           | Appears In                                                                       |
| ----------------- | ----------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `RepositoryError` | `#[from] crate::domain::environment::repository::RepositoryError` | Create, Show, Provision, Configure, Destroy, Purge, Release, Run, Test (9 enums) |
| `StateTypeError`  | `#[from] crate::domain::environment::state::StateTypeError`       | Provision, Configure, Destroy, Release, Run, Test (6 enums)                      |
| `ReleaseStep`     | Field type in struct variants                                     | Release (4 variants)                                                             |

An SDK consumer who pattern-matches `ProvisionCommandHandlerError::StateTransition(e)` must have `StateTypeError` in scope — a domain type. This contradicts the ADR principle: "The public SDK surface contains only presentation-layer and application-layer types."

---

## 🚀 Refactoring Phases

---

## Phase 0: Core Fix — Application-Layer Error Wrappers (High Impact, Medium Effort)

This phase eliminates domain types from the public surface of all 9 affected application error enums.

### Proposal #0: Introduce Application-Layer Wrapper Types for Domain Errors

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P0
**Depends On**: [Extract SDK Workspace Package](extract-sdk-workspace-package.md) (Plan 2)
**Completed**: -
**Commit**: -

#### Problem

Nine `*CommandHandlerError` enums use `#[from]` on domain types (`RepositoryError`, `StateTypeError`) and embed domain types as fields (`ReleaseStep`). These enums are re-exported from the SDK, making the domain types part of the SDK's public API.

```rust
// CURRENT — domain types leak through application error
// src/application/command_handlers/provision/errors.rs
use crate::domain::environment::state::StateTypeError;

pub enum ProvisionCommandHandlerError {
    // ...
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),
    StateTransition(#[from] StateTypeError),
}
```

SDK consumers who pattern-match these variants must import domain types. Worse, if `RepositoryError` or `StateTypeError` changes internally, it becomes a breaking change in the SDK surface.

#### Proposed Solution

Create application-layer wrapper error types that mirror the domain errors but live in the application layer. The wrappers hold the original domain error as a `#[source]` but expose only `String`/primitive fields for public consumption.

**Step 1**: Create application-layer error wrappers in a shared module (for example, `src/application/errors.rs` or `src/application/error_wrappers.rs`):

```rust
// src/application/errors.rs — NEW
use thiserror::Error;

/// Application-layer wrapper for domain `RepositoryError`.
///
/// Shields the public API from the internal domain error type while
/// preserving the error message and source chain for debugging.
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Environment not found")]
    NotFound,

    #[error("Conflict: another process is accessing this environment")]
    Conflict,

    #[error("Repository error: {0}")]
    Internal(#[source] anyhow::Error),
}

/// Application-layer wrapper for domain `StateTypeError`.
///
/// Provides the same information (expected vs actual state) using
/// plain strings instead of the domain enum.
#[derive(Debug, Error)]
#[error("Expected state '{expected}', but found '{actual}'")]
pub struct InvalidStateError {
    pub expected: String,
    pub actual: String,
}

/// Application-layer representation of the release workflow step.
///
/// Mirrors `domain::environment::state::ReleaseStep` for error reporting
/// without coupling the application layer's public error surface to the
/// domain enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseWorkflowStep {
    CreateTrackerStorage,
    InitTrackerDatabase,
    RenderTrackerTemplates,
    DeployTrackerConfigToRemote,
    // ... (one variant per domain ReleaseStep variant)
}
```

**Step 2**: Convert the domain types in each affected `*CommandHandlerError`:

```rust
// PROPOSED — application error uses application wrapper types
// src/application/command_handlers/provision/errors.rs
use crate::application::errors::{InvalidStateError, PersistenceError};

pub enum ProvisionCommandHandlerError {
    // ...
    StatePersistence(#[from] PersistenceError),
    StateTransition(#[from] InvalidStateError),
}
```

**Step 3**: Add `From` conversions from the domain types to the wrappers, so command handler code that currently uses `?` on domain errors continues to work:

```rust
// In src/application/errors.rs
impl From<crate::domain::environment::repository::RepositoryError> for PersistenceError {
    fn from(e: crate::domain::environment::repository::RepositoryError) -> Self {
        match e {
            RepositoryError::NotFound => Self::NotFound,
            RepositoryError::Conflict => Self::Conflict,
            RepositoryError::Internal(source) => Self::Internal(source),
        }
    }
}

impl From<crate::domain::environment::state::StateTypeError> for InvalidStateError {
    fn from(e: crate::domain::environment::state::StateTypeError) -> Self {
        match e {
            StateTypeError::UnexpectedState { expected, actual } => Self {
                expected: expected.to_string(),
                actual,
            },
        }
    }
}
```

#### Rationale

- The `From` impls ensure that existing handler code (`repository.save(&env)?`) compiles without change — the `?` operator chains `RepositoryError → PersistenceError → *CommandHandlerError`
- The wrappers are pure application-layer types — safe to re-export through the SDK
- The domain errors remain the source chain (via `#[source]`) for debugging, but are not in the type signature

#### Benefits

- ✅ No domain types in the SDK's public compile-time surface
- ✅ Internal domain changes to `RepositoryError`/`StateTypeError` are no longer breaking changes for SDK consumers
- ✅ Existing handler code compiles with minimal changes (swap import, `#[from]` auto-converts via the `From` chain)
- ✅ CLI controllers are unaffected — they use the same application error types

#### Implementation Checklist

- [ ] Create `src/application/errors.rs` with `PersistenceError`, `InvalidStateError`, and `ReleaseWorkflowStep`
- [ ] Implement `From<RepositoryError>` for `PersistenceError`
- [ ] Implement `From<StateTypeError>` for `InvalidStateError`
- [ ] Implement `From<ReleaseStep>` for `ReleaseWorkflowStep`
- [ ] Update `ProvisionCommandHandlerError` to use `PersistenceError` and `InvalidStateError`
- [ ] Update `ConfigureCommandHandlerError` to use `PersistenceError` and `InvalidStateError`
- [ ] Update `DestroyCommandHandlerError` to use `PersistenceError` and `InvalidStateError`
- [ ] Update `ReleaseCommandHandlerError` to use `PersistenceError`, `InvalidStateError`, and `ReleaseWorkflowStep`
- [ ] Update `RunCommandHandlerError` to use `PersistenceError` and `InvalidStateError`
- [ ] Update `TestCommandHandlerError` to use `PersistenceError` and `InvalidStateError`
- [ ] Update `CreateCommandHandlerError` to use `PersistenceError`
- [ ] Update `ShowCommandHandlerError` to use `PersistenceError`
- [ ] Update `PurgeCommandHandlerError` to use `PersistenceError`
- [ ] Update CLI controller error handling if it pattern-matches on the old domain types
- [ ] Re-export `PersistenceError`, `InvalidStateError`, and `ReleaseWorkflowStep` from the SDK's public API
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

- All existing unit tests in each `*CommandHandlerError` module must continue to compile and pass
- The `From` conversions should have unit tests proving round-trip fidelity (domain error → wrapper → display string)
- SDK examples (`examples/sdk/error_handling.rs`) should compile without importing any `crate::domain` type
- `cargo test` full suite must pass

---

## Phase 1: Infrastructure Decoupling (High Impact, Medium Effort)

### Proposal #1: Move Infrastructure Wiring from `DeployerBuilder` to Bootstrap Layer

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P1
**Depends On**: [Extract SDK Workspace Package](extract-sdk-workspace-package.md) (Plan 2). Can be done in parallel with Proposal #0.
**Completed**: -
**Commit**: -

#### Problem

`DeployerBuilder::build()` directly instantiates:

- `RepositoryFactory` from `crate::infrastructure::persistence::repository_factory`
- `SystemClock` from `crate::shared::SystemClock`

The `Deployer` struct stores `Arc<RepositoryFactory>` as a field.

This violates the DDD Dependency Rule: the Presentation layer (where the SDK lives) depends on the Infrastructure layer at compile time. The existing CLI already solves this correctly — `bootstrap::Container` performs the wiring and injects trait objects.

```rust
// CURRENT — presentation layer imports infrastructure
// src/presentation/sdk/builder.rs
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::SystemClock;

let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
let clock = Arc::new(SystemClock);
```

```rust
// CURRENT — Deployer holds infrastructure type
// src/presentation/sdk/deployer.rs
pub struct Deployer {
    repository_factory: Arc<RepositoryFactory>,  // ← infrastructure type
    // ...
}
```

#### Proposed Solution

**Step 1**: Introduce an application-layer trait that abstracts what `RepositoryFactory` does:

```rust
// src/application/traits.rs — NEW or extended
pub trait RepositoryProvider: Send + Sync {
    fn create(&self, data_dir: PathBuf) -> Arc<dyn EnvironmentRepository + Send + Sync>;
}
```

**Step 2**: Implement this trait for `RepositoryFactory` in the infrastructure layer:

```rust
// src/infrastructure/persistence/repository_factory.rs
impl RepositoryProvider for RepositoryFactory {
    fn create(&self, data_dir: PathBuf) -> Arc<dyn EnvironmentRepository + Send + Sync> {
        // existing implementation
    }
}
```

**Step 3**: Replace `Arc<RepositoryFactory>` in `Deployer` with `Arc<dyn RepositoryProvider>`:

```rust
// PROPOSED — Deployer uses application-layer trait only
pub struct Deployer {
    repository_factory: Arc<dyn RepositoryProvider>,  // ← application trait
    clock: Arc<dyn Clock>,                            // ← already a trait
    // ...
}
```

**Step 4**: Create a bootstrap helper that `DeployerBuilder` delegates to:

```rust
// src/bootstrap/sdk.rs — NEW
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::SystemClock;

pub fn default_repository_provider(lock_timeout: Duration) -> Arc<dyn RepositoryProvider> {
    Arc::new(RepositoryFactory::new(lock_timeout))
}

pub fn default_clock() -> Arc<dyn Clock> {
    Arc::new(SystemClock)
}
```

**Step 5**: `DeployerBuilder::build()` calls the bootstrap helper:

```rust
// PROPOSED
use crate::bootstrap::sdk::{default_clock, default_repository_provider};

pub fn build(self) -> Result<Deployer, DeployerBuildError> {
    let working_dir = self.working_dir.ok_or(DeployerBuildError::MissingWorkingDir)?;
    let repository_provider = default_repository_provider(DEFAULT_LOCK_TIMEOUT);
    let data_dir = working_dir.join("data");
    let repository = repository_provider.create(data_dir.clone());
    let clock = default_clock();
    // ...
}
```

Optionally, `DeployerBuilder` can accept a custom `RepositoryProvider` for testing:

```rust
pub fn repository_provider(mut self, provider: Arc<dyn RepositoryProvider>) -> Self {
    self.repository_provider = Some(provider);
    self
}
```

#### Rationale

- The `bootstrap` layer is explicitly allowed to cross-cut all layers for dependency injection — this is the same pattern the CLI uses via `Container`
- `DeployerBuilder` remains ergonomic (zero-config default) while gaining testability (injectable provider)
- `Deployer` no longer has a compile-time dependency on any infrastructure type

#### Benefits

- ✅ Clean layer boundary — SDK imports only from `application` and `bootstrap`
- ✅ Testable — consumers can inject a mock `RepositoryProvider` for unit testing
- ✅ Consistent — matches the CLI's bootstrap pattern
- ✅ No functional change to end-user behavior

#### Implementation Checklist

- [ ] Add `RepositoryProvider` trait to `src/application/traits.rs` (or a new file)
- [ ] Implement `RepositoryProvider` for `RepositoryFactory` in the infrastructure layer
- [ ] Create `src/bootstrap/sdk.rs` with `default_repository_provider()` and `default_clock()`
- [ ] Register `sdk` module in `src/bootstrap/mod.rs`
- [ ] Replace `Arc<RepositoryFactory>` with `Arc<dyn RepositoryProvider>` in `Deployer` struct
- [ ] Update `DeployerBuilder::build()` to use bootstrap helpers instead of direct infrastructure imports
- [ ] Remove infrastructure imports (`RepositoryFactory`, `SystemClock`) from `builder.rs` and `deployer.rs`
- [ ] Optionally: add `repository_provider()` builder method for dependency injection
- [ ] Verify all tests pass
- [ ] Verify all SDK examples compile
- [ ] Run linter and fix any issues

#### Testing Strategy

- All existing tests continue to pass (default wiring unchanged)
- SDK examples compile and work identically
- Optionally: add a unit test that injects a mock `RepositoryProvider` to verify dependency injection works

---

## 📈 Timeline

- **Start Date**: TBD (after Plans 1–3 are complete)
- **Actual Completion**: TBD

## 🔍 Review Process

### Approval Criteria

- [ ] Technical feasibility validated
- [ ] Aligns with [Development Principles](../../development-principles.md)
- [ ] Aligns with [SDK Interface Design ADR](../../decisions/sdk-presentation-layer-interface-design.md)
- [ ] Implementation plan is clear and actionable

### Completion Criteria

- [ ] Both proposals implemented
- [ ] All tests passing (`cargo test --workspace`)
- [ ] All linters passing (`cargo run --bin linter all`)
- [ ] SDK examples compile without importing any `crate::domain` or `crate::infrastructure` type
- [ ] No domain types (`RepositoryError`, `StateTypeError`, `ReleaseStep`) in the SDK's public error surface
- [ ] No infrastructure imports (`RepositoryFactory`) in the SDK package
- [ ] ADR updated to reflect the fixes
- [ ] Changes committed and pushed

## 📚 Related Documentation

- [SDK Interface Design ADR](../../decisions/sdk-presentation-layer-interface-design.md)
- [Development Principles](../../development-principles.md)
- [DDD Layer Placement Guide](../../contributing/ddd-layer-placement.md)
- [Codebase Architecture](../../codebase-architecture.md)
- [Error Handling Guide](../../contributing/error-handling.md)
- [Presentation CLI/SDK Separation](presentation-cli-sdk-separation.md) (Plan 1)
- [Extract SDK Workspace Package](extract-sdk-workspace-package.md) (Plan 2)
- [Extract Shared Types Package](extract-shared-types-package.md) (Plan 3)

## 💡 Notes

- **Proposals #0 and #1 are independent** of each other — they can be implemented in any order or in parallel
- **Proposal #0 is the larger** — it touches 9 error enums but is mechanical (swap imports, add `From` impls)
- `EnvironmentName` / `EnvironmentNameError` will be in `torrust-deployer-types` after Plan 3 — they are simple value objects, not domain types, so no DDD violation
- `SystemClock` is a shared/cross-cutting type, not infrastructure — it will be in `torrust-deployer-types` after Plan 3. Only `RepositoryFactory` is a genuine infrastructure violation.
- `RegisterCommandHandlerError` and `RenderCommandHandlerError` also have domain types, but are NOT re-exported from the SDK — they are out of scope for this plan

---

**Created**: 2026-02-24
**Last Updated**: 2026-02-24
**Status**: 📋 Planning
