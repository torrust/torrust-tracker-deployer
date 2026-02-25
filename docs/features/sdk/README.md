# SDK — Programmatic Deployer Access

Typed Rust SDK for the Torrust Tracker Deployer, enabling other Rust
projects and AI agents to programmatically manage deployment environments
without shelling out to the CLI.

## Overview

The SDK exposes a `Deployer` facade that encapsulates dependency wiring and
provides one method per deployment operation. It sits in the Presentation
Layer alongside the CLI — both share the same Application-layer command
handlers.

### Problems it solves

| Problem                                                               | How the SDK helps                                                    |
| --------------------------------------------------------------------- | -------------------------------------------------------------------- |
| CLI-only access — programmatic consumers must parse stdout/stderr     | Returns typed `Result<T, E>` values                                  |
| No composability — CLI commands are atomic, no mid-workflow branching | Consumers call individual operations and branch on results           |
| Fragile integration — shelling out breaks on output format changes    | Stable Rust API with semver guarantees                               |
| AI agent friction — agents must generate CLI strings and parse text   | Agents call Rust methods, get structured data, match on typed errors |

### Architecture placement

```text
Presentation Layer
├── CLI delivery     (input/ → dispatch/ → controllers/ → views/)
└── SDK delivery     (packages/sdk/)
    │
    ▼
Application Layer    (command_handlers/)   ← SHARED, unchanged
    │
    ▼
Domain Layer         (environment/, template/, topology/, ...)
```

The SDK depends on the Application and Domain layers. It does NOT depend on
the CLI presentation code.

## Package

The SDK lives in [`packages/sdk/`](../../../packages/sdk/) as a workspace
member. See [`packages/sdk/README.md`](../../../packages/sdk/README.md) for
quick-start usage, examples, and API reference.

## Design Decisions

Decisions that shaped the SDK design. For full ADR details, see `docs/decisions/sdk-*.md`.

### Facade over direct handlers

SDK consumers should not need to understand the internal dependency graph.
The `Deployer` struct encapsulates wiring and exposes one method per
command handler.

### Builder pattern for construction

`Deployer::builder().working_dir(path).build()` — sensible defaults,
optional customization. Follows standard Rust conventions.

### Re-export domain types, don't wrap them

Existing domain and application types are re-exported from the SDK module.
No SDK-specific wrapper types. Avoids duplication and conversion overhead.

### Steps (Level 2) remain internal

The three-level architecture (Handlers → Steps → Actions) exposes only
command handlers (Level 1) through the SDK. Steps remain `pub(crate)`.
They are coupled to infrastructure internals and would create a brittle API.

### Name-based operations, not typestate

Operations take `&EnvironmentName` and return `()` or simple result types.
No phantom typestate at the SDK layer. This supports the primary use case:
AI agents and automation that resume from any state across process restarts.
See [SDK Presentation Layer Interface Design](../decisions/sdk-presentation-layer-interface-design.md).

## Related Decisions

- [SDK Presentation Layer Interface Design](../../decisions/sdk-presentation-layer-interface-design.md) — name-based vs typestate design
- [SDK Discarded: Scoped Environment Guard](../../decisions/sdk-discarded-scoped-environment-guard.md) — why RAII auto-cleanup was rejected
- [SDK Discarded: Typestate at SDK Layer](../../decisions/sdk-discarded-typestate-at-sdk-layer.md) — why compile-time ordering was rejected
- [SDK Discarded: Fluent Interface](../../decisions/sdk-discarded-fluent-interface.md) — why method chaining was rejected

## Testing

SDK tests use only the public API (`torrust_tracker_deployer_sdk::*`) and
exercise local operations against temporary workspaces. No infrastructure
required. See [SDK Testing Strategy](../../contributing/testing/sdk/testing-strategy.md).

## Agent Skills

- [Add SDK Method](../../../.github/skills/dev/sdk/add-sdk-method/skill.md)
- [Write SDK Integration Test](../../../.github/skills/dev/sdk/write-sdk-integration-test/skill.md)
- [Add SDK Example](../../../.github/skills/dev/sdk/add-sdk-example/skill.md)

## Status

**Current Phase**: Complete (v1)

**Implemented**:

1. `Deployer` facade with builder
2. All 14 operations (create, show, list, exists, validate, destroy, purge, provision, configure, release, run, test, create-from-file, exists)
3. `EnvironmentCreationConfig` typed builder
4. Curated re-exports in `lib.rs`
5. `SdkError` unified error type
6. `Clone + Send + Sync` support
7. 5 runnable examples
8. 13 integration tests
9. Separate `packages/sdk/` workspace crate

## Future Work

Ideas identified during development. Not prioritized — implement when
real-world usage creates demand.

### High value / low effort

- **Configurable lock timeout** — expose `.lock_timeout(Duration)` on
  `DeployerBuilder` (currently hard-coded at 30 seconds)
- **`create_environment_from_json(&str)` convenience** — symmetric with
  `create_environment_from_file`
- **`#[non_exhaustive]` on public enums** — prevent downstream breakage
  when adding variants to `SdkError`, `CreateEnvironmentFromFileError`, etc.

### Medium value

- **`tracing` instrumentation** — `#[instrument]` spans on SDK methods for
  automatic structured logging
- **`DeployerBuilder::data_directory()` override** — custom data path
  instead of `working_dir.join("data")`
- **Render operation** — expose the CLI `render` command through the SDK
- **Idempotent deploy example** — use `exists()` + `show()` to skip
  completed stages
- **Custom progress listener example** — richer `CommandProgressListener`
  for UIs/dashboards

### Deferred

- **Feature flag gating** — `sdk` cargo feature to gate the SDK module
  for CLI-only consumers
- **Extract to independent crate** — independent versioning, crates.io
  publishing (wait for API stability and community adoption)
