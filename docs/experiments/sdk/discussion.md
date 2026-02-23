# SDK Exploration: Discussion

## Goal

Expose a Rust SDK (library) from the Torrust Tracker Deployer so that:

1. **Other Rust projects** can programmatically create, provision, configure,
   release, run, destroy, and query deployment environments — without shelling
   out to the CLI binary.
2. **AI agents** can build custom deployment workflows in Rust, composing
   deployer operations with their own logic, error handling, and progress
   reporting.

The SDK should feel like a first-class Rust library: typed inputs and outputs,
structured errors, trait-based extension points, and ergonomic construction.

## Why

### Current state

The deployer already ships a `lib` crate (`torrust_tracker_deployer_lib`) that
exposes all internal modules as `pub`. However, there is no curated public API
surface — consumers would need to wire `Arc<dyn EnvironmentRepository>`,
`Arc<dyn Clock>`, and other infrastructure dependencies manually, replicating
what the CLI bootstrap code does.

### Problems this solves

| Problem                                                                   | How the SDK helps                                                    |
| ------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| **CLI-only access** — programmatic consumers must parse stdout/stderr     | SDK returns typed `Result<Environment<S>, E>` values                 |
| **No composability** — CLI commands are atomic, no mid-workflow branching | SDK consumers call individual operations and branch on results       |
| **Fragile integration** — shelling out breaks on output format changes    | SDK has a stable Rust API with semver guarantees                     |
| **AI agent friction** — agents must generate CLI strings and parse text   | Agents call Rust methods, get structured data, match on typed errors |

## How: Architecture Placement

### The SDK as a presentation concern

In our DDD layered architecture, the SDK is an **alternative delivery
mechanism** — analogous to the CLI. Both sit in the **Presentation Layer**:

```text
Presentation Layer
├── CLI delivery     (input/ → dispatch/ → controllers/ → views/)
└── SDK delivery     (sdk/)                                        ← NEW
    │
    ▼
Application Layer    (command_handlers/)   ← SHARED, unchanged
    │
    ▼
Domain Layer         (environment/, template/, topology/, ...)
```

The SDK facade lives in `src/presentation/sdk/` and depends on:

- **Application layer** — command handlers (the use cases)
- **Domain layer** — types that flow in/out of command handlers

It does NOT depend on the CLI presentation code (input, dispatch, controllers,
views).

### Why Presentation and not Application?

The `Deployer` facade is about **how external code enters the system** — a
delivery concern. The application layer already has its delivery-agnostic API:
the command handlers. Adding a convenience facade on top is presentation logic,
just as the CLI controllers are.

### Why not a separate crate?

For the proof of concept, keeping it in the same crate is simpler:

- No workspace restructuring
- Domain types are directly available (no re-export indirection)
- Can evolve the API surface before committing to a separate package

A future decision may split it into a `torrust-tracker-deployer-sdk` crate
once the API stabilizes.

## Design Decisions

### Decision 1: Expose command handlers through a facade, not directly

**Context**: Command handlers are already delivery-agnostic, but constructing
them requires wiring `Arc<dyn EnvironmentRepository>`, `Arc<dyn Clock>`, etc.

**Decision**: Provide a `Deployer` struct that encapsulates dependency wiring
and exposes one method per command handler.

**Rationale**: SDK consumers should not need to understand the internal
dependency graph. The facade hides construction complexity while preserving
full access to every operation.

### Decision 2: Builder pattern for Deployer construction

**Context**: The `Deployer` needs at minimum a `working_dir` path. Optional
customizations include progress listeners, custom clocks (for testing), etc.

**Decision**: Use a `DeployerBuilder` with sensible defaults.

**Rationale**: Follows Rust conventions. Allows SDK consumers to start simple
(`Deployer::builder().working_dir(path).build()`) and customize later.

### Decision 3: Re-export domain types, don't wrap them

**Context**: SDK consumers need types like `EnvironmentName`,
`EnvironmentCreationConfig`, `Environment<Created>`, etc.

**Decision**: Re-export existing domain and application types from the SDK
module. Do not create SDK-specific wrapper types.

**Rationale**: Avoids duplication and conversion overhead. The domain types
are already well-designed — they are the shared language of the system.

### Decision 4: Steps (Level 2) remain internal

**Context**: The three-level architecture has Command Handlers (L1) → Steps
(L2) → Remote Actions (L3). Should the SDK expose steps for fine-grained
composition?

**Decision**: For the initial SDK, expose only command handlers (L1). Steps
remain `pub(crate)` implementation details.

**Rationale**: Steps are coupled to infrastructure internals (Ansible template
paths, OpenTofu state, SSH sessions). Exposing them would create a brittle API
surface. If fine-grained composition is needed later, we can promote specific
steps to the public API with proper abstraction.

### Decision 5: Keep presentation-layer placement as provisional

**Context**: There is uncertainty about whether `src/presentation/sdk/` is the
right long-term location. Some arguments favor a top-level `src/sdk/` or even
a separate crate.

**Decision**: Start with `src/presentation/sdk/` for the proof of concept.
Revisit after the API stabilizes.

**Rationale**: The argument for presentation-layer placement is solid (it's a
delivery mechanism), but we don't need to commit permanently. The proof of
concept will reveal whether the placement causes friction.

## Open Questions

1. **Should the SDK expose the `EnvironmentRepository` trait?** — Allowing
   custom storage backends is powerful but adds API surface. For the PoC,
   we use the default filesystem repository.

2. **Should error types expose `.help()` methods?** — These are designed for
   human-readable guidance. Programmatic consumers may prefer structured
   error data instead. Both could coexist.

3. **Async vs sync API** — Some command handlers are async (Provision, Release,
   Register, Render, Test) and some are sync. Should the SDK normalize to
   all-async? For the PoC, we preserve the existing signatures.

4. **Versioning strategy** — When should the SDK surface get semver stability
   guarantees? Not during the PoC, but this needs a decision before publishing.

5. **Feature flags** — Should the SDK be behind a cargo feature flag to avoid
   increasing compile times for CLI-only users? Worth considering if the SDK
   pulls in additional dependencies.
