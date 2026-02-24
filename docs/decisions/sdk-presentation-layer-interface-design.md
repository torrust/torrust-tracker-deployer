# Decision: SDK Presentation Layer Interface Design

## Status

Accepted

## Date

2026-02-24

## Context

The SDK (`src/presentation/sdk/`) is a programmatic delivery mechanism
that sits in the Presentation Layer alongside the CLI. It wraps the same
application-layer command handlers that the CLI uses, providing a
friendlier API for AI agents, scripts, and automation pipelines.

During Phase 2 of the SDK proof-of-concept, three design questions arose:

1. **What should SDK operation methods return?** The current implementation
   returns domain types (`Environment<Created>`, `Environment<Provisioned>`,
   etc.) which leaks internal DDD layers through the public API.

2. **Should the SDK enforce deployment ordering at compile time?** The
   domain layer uses a phantom typestate pattern to prevent invalid state
   transitions. Should the SDK mirror this?

3. **Should operations take or return a typed environment value, or use
   the environment name string as the primary identifier?**

### The Primary Use Case

The SDK is designed for AI agents and automation pipelines. Key
characteristics of these consumers:

- They execute operations by name, resuming from any state across process
  restarts
- They need to be "agile" — low friction, no forced pattern matching on
  internal state types
- They work in a REPL-like interaction model: call operations, inspect
  results, react to errors
- The application layer already validates state preconditions and provides
  clear runtime errors

## Decision

**SDK operations use `EnvironmentName` as input and return `()` on
success.** The deployment ordering contract is documented, not enforced
at compile time. The SDK does not expose or replicate domain types.

```rust
// Callers provide a name, get back unit on success or a typed error
async fn provision(&self, name: &EnvironmentName) -> Result<(), ProvisionCommandHandlerError>
fn configure(&self, name: &EnvironmentName) -> Result<(), ConfigureCommandHandlerError>
async fn release(&self, name: &EnvironmentName) -> Result<(), ReleaseCommandHandlerError>
```

For operations that produce data consumers need (like `show`, `list`,
`test`), the SDK returns purpose-built result types from the application
layer (`EnvironmentInfo`, `EnvironmentList`, `TestResult`), not domain
entities.

`EnvironmentName` is a simple value object with no internal logic to
protect. It is re-exported from the SDK public surface for convenience.
Domain entities (`Environment<T>`, state types) are not re-exported.

## Consequences

### Positive

- **No domain leaking.** The public SDK surface contains only
  presentation-layer and application-layer types.
- **Resume-friendly.** Any automation that stores an environment name
  (database, config file, environment variable) can reconstruct the full
  workflow state and call the next operation without holding a typed value
  from a previous process run.
- **Low friction for AI agents.** Agents call operations by name, in
  sequence. The application layer returns informative errors if the order
  is wrong.
- **Single source of truth for ordering.** The application-layer command
  handlers enforce state preconditions. The SDK does not duplicate this
  logic.
- **Simpler API surface.** No generic type parameters on SDK method
  signatures.

### Negative

- **No compile-time ordering guarantees.** A consumer can accidentally
  call `configure` before `provision` and receive a runtime error.
  Mitigation: the runtime error is clear and informative; documentation
  shows the correct order; examples demonstrate the full workflow.

## Alternatives Considered

### SDK-Layer Typestate Pattern

Mirror the domain's `Environment<State>` phantom type pattern at the SDK
layer:

```rust
pub struct SdkEnvironment<S> { name: EnvironmentName, _state: PhantomData<S> }

async fn provision(&self, env: SdkEnvironment<Created>)
    -> Result<SdkEnvironment<Provisioned>, ...>
```

**Rejected because:**

1. **The resume problem.** A user resuming after a process restart has no
   `SdkEnvironment<Provisioned>` value. They always need a
   `load_as_provisioned(name)` escape hatch, which undermines the
   compile-time guarantee and still fails at runtime anyway.
2. **Replicates domain internals.** Two parallel state hierarchies (domain
   and SDK) must be kept in sync — a maintenance burden that violates DRY.
3. **Friction for the primary use case.** Storing typed values in
   `HashMap`s, passing them across async tasks, or serialising them is
   awkward compared to working with plain names.

Full reasoning in
[`docs/experiments/sdk/discarded-ideas.md`](../experiments/sdk/discarded-ideas.md).

### Fluent Interface

Return `&self` or a builder-like value to allow method chaining:

```rust
deployer.provision(&name)?.configure(&name)?.release(&name)?;
```

**Rejected because:** provides no compile-time safety benefit while adding
ergonomic constraints. Ordering errors still surface at runtime. Full
reasoning in
[`docs/experiments/sdk/discarded-ideas.md`](../experiments/sdk/discarded-ideas.md).

### Return Domain `Environment<State>` Types

Keep the current approach of returning `Environment<Created>`, etc.

**Rejected because:** violates the DDD layer boundary — domain types are
internal implementation details, not public API contracts. Changes to
domain internals would become breaking changes in the public SDK. The
returned typed value is not used as input to the next call (methods take
`&EnvironmentName`, not `&Environment<T>`), so it provides no type safety
benefit anyway.

## Related Decisions

- [Command State Return Pattern](./command-state-return-pattern.md) —
  the domain/application-layer decision to use typed state returns in
  command handlers (this ADR intentionally does *not* mirror that pattern
  at the presentation layer)
- [Configuration DTO Layer Placement](./configuration-dto-layer-placement.md) —
  related decision about keeping DTOs in the application layer

## References

- [`src/presentation/sdk/`](../../src/presentation/sdk/) — SDK implementation
- [`docs/experiments/sdk/phase-2-improvements.md`](../experiments/sdk/phase-2-improvements.md) — Phase 2 task list
- [`docs/experiments/sdk/discarded-ideas.md`](../experiments/sdk/discarded-ideas.md) — Full catalogue of discarded SDK design ideas
