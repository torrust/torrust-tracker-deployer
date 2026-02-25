# Decision: SDK Discarded — Typestate Pattern at SDK Layer

## Status

Rejected

## Date

2026-02-24

## Context

The domain layer uses a phantom typestate pattern (`Environment<Created>`,
`Environment<Provisioned>`, etc.) to prevent invalid state transitions. A
proposal was made to mirror this pattern at the SDK layer with
presentation-level state types enforcing deployment order at compile time.

## Decision

Rejected. SDK operations take `&EnvironmentName` and return `()` or simple
result types. The application layer enforces ordering at runtime with clear
errors.

## Consequences

Callers can call operations in any order. Invalid ordering is caught at
runtime by the application layer with informative error messages. This
supports the primary use case: AI agents and automation that resume from
any state across process restarts.

## Alternatives Considered

### SDK-Layer Typestate

```rust
pub struct SdkEnvironment<S> { name: EnvironmentName, _state: PhantomData<S> }

async fn provision(&self, env: SdkEnvironment<Created>)
    -> Result<SdkEnvironment<Provisioned>, ...>
```

**Why rejected:**

1. **The resume problem breaks the guarantee.** A user who ran `provision`
   in a previous process has no `SdkEnvironment<Provisioned>` value. They
   would always need a `load_as_provisioned(name)` escape hatch, undermining
   the compile-time guarantee.

2. **Friction without proportional benefit.** Ordering is already enforced
   at runtime by the application layer. Duplicating it adds complexity
   without eliminating runtime errors.

3. **Replicates domain internals in the presentation layer.** Maintaining
   two parallel state hierarchies violates DRY.

4. **Forces callers to thread typed values.** Storing typed environments in
   collections, passing across async boundaries, or serializing becomes
   awkward. The name string is the natural identifier.

## Related Decisions

- [SDK Presentation Layer Interface Design](sdk-presentation-layer-interface-design.md)

## References

- Original discussion in `docs/experiments/sdk/discarded-ideas.md` (removed)
