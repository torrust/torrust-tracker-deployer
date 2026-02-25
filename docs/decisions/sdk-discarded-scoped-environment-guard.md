# Decision: SDK Discarded — Scoped Environment Guard (RAII Auto-Cleanup)

## Status

Rejected

## Date

2026-02-24

## Context

During SDK development, a `ScopedEnvironment` struct was proposed that would
wrap an environment name and automatically call `deployer.purge(name)` on
drop, preventing leaked local state in test harnesses and short-lived scripts.

## Decision

Rejected. The SDK provides `destroy` and `purge` as explicit, composable
operations. Callers implement their own cleanup wrappers if needed.

## Consequences

Callers must explicitly clean up environments. This is more verbose but gives
full control over error handling and cleanup policy.

## Alternatives Considered

### Scoped Environment Guard

A `ScopedEnvironment` that auto-purges on drop.

**Why rejected:**

1. **Wrong layer for lifecycle policy.** The `Deployer` is a command facade —
   it executes operations, it does not own lifecycle decisions. Whether to
   purge, keep, or inspect a failed environment is a business decision that
   belongs to the caller:
   - A user may want to inspect a failed environment's `data/` directory
   - A user may want to destroy infrastructure but keep local artifacts
   - A user may be passing the environment name across processes or restarts

2. **The destroy vs. purge ambiguity.** A cleanup guard would need to do both
   `destroy` (infrastructure teardown via OpenTofu) and `purge` (local state
   removal) in the correct order. This is a multi-step workflow that can fail
   at each step, and those failures cannot be propagated from `Drop`.

3. **`Drop` errors are silently swallowed.** Rust's `Drop` trait returns `()`.
   Any purge failure during drop can only be logged, not propagated. This
   creates a false sense of safety.

## Related Decisions

- [SDK Presentation Layer Interface Design](sdk-presentation-layer-interface-design.md)

## References

- Original discussion in `docs/experiments/sdk/discarded-ideas.md` (removed)
