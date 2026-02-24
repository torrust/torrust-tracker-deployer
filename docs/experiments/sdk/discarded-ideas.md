# SDK — Discarded Ideas

This document records SDK design ideas that were **explicitly considered and
rejected**. Keeping them here avoids re-litigating the same debates and
documents the reasoning for future contributors.

## Scoped Environment Guard (RAII Auto-Cleanup)

**Original proposal (Task 10 in phase-2-improvements.md):**

A `ScopedEnvironment` struct that wraps `Environment<Created>` and
automatically calls `deployer.purge(name)` on drop, preventing leaked
local state in test harnesses and short-lived scripts.

**Why it was rejected:**

1. **Wrong layer for lifecycle policy.** The `Deployer` is a command facade
   — it executes operations, it does not own lifecycle decisions. Whether to
   purge, keep, or inspect a failed environment is a business decision that
   belongs to the caller. Baking auto-purge into the SDK makes an assumption
   that doesn't hold universally:
   - A user may want to inspect a failed environment's `data/` directory
     before cleaning it
   - A user may want to destroy infrastructure but keep local artifacts for
     debugging
   - A user may be passing the environment name across processes or restarts

2. **The destroy vs. purge ambiguity.** A cleanup guard would need to do both
   `destroy` (infrastructure teardown via OpenTofu) *and* `purge` (local state
   removal) in the correct order. This is not merely a cleanup — it's a
   multi-step workflow that can fail at each step, and those failures cannot
   be propagated from `Drop` (which returns `()`).

3. **`Drop` errors are silently swallowed.** Rust's `Drop` trait returns `()`.
   Any purge failure during drop can only be logged, not propagated. This
   creates a false sense of safety: the guard appears to guarantee cleanup, but
   purge could silently fail.

**Correct alternative:** Let callers implement their own cleanup wrappers if
needed. The SDK provides `destroy` and `purge` as explicit, composable
operations. A user who wants RAII cleanup can write a thin wrapper in their
own codebase with full control over the error handling policy.

---

## SDK-Layer Typestate Pattern (Compile-Time Ordering Enforcement)

**Proposal:**

Mirror the domain's `Environment<State>` phantom type pattern at the SDK
layer, creating presentation-layer state types that enforce the deployment
order at compile time:

```rust
pub struct SdkEnvironment<S> { name: EnvironmentName, _state: PhantomData<S> }

async fn provision(&self, env: SdkEnvironment<Created>)
    -> Result<SdkEnvironment<Provisioned>, ...>

fn configure(&self, env: SdkEnvironment<Provisioned>)
    -> Result<SdkEnvironment<Configured>, ...>
```

**Why it was rejected:**

1. **The resume problem breaks the guarantee.** The SDK is primarily for
   automation — AI agents, scripts, CI pipelines that get interrupted. A user
   who ran `provision` in a previous process run has no `SdkEnvironment<Provisioned>`
   value. They would always need a `deployer.load_as_provisioned(name)` escape
   hatch, which undermines the compile-time guarantee entirely.

2. **Friction without proportional benefit.** The ordering contract is already
   enforced at runtime by the application layer (the command handlers return
   clear errors like "environment is not in Provisioned state"). Duplicating
   this at the SDK presentation layer adds complexity without eliminating
   runtime errors — the `load_as_*` escape hatches still fail at runtime.

3. **Replicates domain internals in the presentation layer.** The domain
   already has a rich typestate pattern. Mirroring it in the SDK means
   maintaining two parallel state hierarchies in sync. This violates DRY and
   creates a maintenance burden.

4. **Forces callers to thread typed values.** Storing `SdkEnvironment<T>` in
   a `HashMap`, passing it across async task boundaries, or serialising it
   becomes awkward. The name string is the natural identifier for workflows
   that span process restarts, databases, or message queues.

**Correct alternative:** Return `()` from operations. The name is already
known to the caller. The application layer enforces ordering at runtime with
clear, informative errors. See the ADR
[SDK Presentation Layer Interface Design](../../decisions/sdk-presentation-layer-interface-design.md)
for the full reasoning.

---

## Fluent Interface (Method Chaining)

**Proposal:**

Return `&self` (or a builder-style value) from operations so callers can
chain deployment steps:

```rust
deployer
    .provision(&name)?
    .configure(&name)?
    .release(&name)?
    .run_services(&name)?;
```

**Why it was rejected:**

No compile-time ordering enforcement — a caller can chain `configure` after a
failed `provision` and only discover the error at runtime. This combines the
ergonomic cost of chaining (methods must all share compatible return types)
with none of the safety benefit. The typestate approach at least gives
compile-time guarantees in the non-resume case; the fluent interface gives
neither.
