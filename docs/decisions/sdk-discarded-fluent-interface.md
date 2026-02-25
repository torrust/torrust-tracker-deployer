# Decision: SDK Discarded — Fluent Interface (Method Chaining)

## Status

Rejected

## Date

2026-02-24

## Context

A fluent interface was proposed where SDK operations return `&self` (or a
builder-style value) so callers can chain deployment steps:

```rust
deployer
    .provision(&name)?
    .configure(&name)?
    .release(&name)?
    .run_services(&name)?;
```

## Decision

Rejected. Operations are independent method calls that return their own
result types.

## Consequences

Each operation is called separately with explicit error handling. This is
more verbose but each step's result is independently inspectable.

## Alternatives Considered

### Fluent chaining

**Why rejected:**

No compile-time ordering enforcement — a caller can chain `configure` after
a failed `provision` and only discover the error at runtime. This combines
the ergonomic cost of chaining (methods must all share compatible return
types) with none of the safety benefit. The typestate approach at least
gives compile-time guarantees in the non-resume case; the fluent interface
gives neither.

## Related Decisions

- [SDK Presentation Layer Interface Design](sdk-presentation-layer-interface-design.md)
- [SDK Discarded: Typestate at SDK Layer](sdk-discarded-typestate-at-sdk-layer.md)

## References

- Original discussion in `docs/experiments/sdk/discarded-ideas.md` (removed)
