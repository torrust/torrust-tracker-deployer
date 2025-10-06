# Decision: Error Context Strategy in Commands

## Status

Accepted

## Date

2025-10-06

## Context

Commands need to persist error context when failures occur. We need to decide what information to store and where.

### The Core Problem

When a command like `ProvisionCommand` fails:

- We need to know **what** failed (which step)
- We need to know **why** it failed (error details)
- We need **complete information** for debugging
- We need **actionable guidance** for users

### Key Requirements

1. Type-safe error handling (not string-based)
2. Complete error information (full error chain)
3. No serialization constraints on error types
4. Error history preservation (multiple attempts)
5. Actionable user guidance

## Decision

**Use structured error context with independent trace files**.

### Architecture

**State File** (`data/{env}/state.json`) contains:

- Enum-based context (type-safe)
- Essential metadata (timing, step, kind)
- Reference to trace file

**Trace Files** (`data/{env}/traces/{timestamp}-{command}.log`) contain:

- Complete error chain (all nested errors)
- Root cause analysis
- Suggested actions
- Full debugging context

### Key Innovation: `Traceable` Trait

Instead of requiring errors to implement `Serialize`, we use a custom `Traceable` trait:

```rust
trait Traceable {
    fn trace_format(&self) -> String;
}
```

This allows:

- Custom formatting per error type
- No serialization constraints
- Errors can contain non-serializable data (file handles, sockets, etc.)

### Storage Details

**Location**: `data/{env}/traces/` (not `build/`)

- `data/` = internal application state
- `build/` = generated artifacts (OpenTofu, Ansible configs)

**Naming**: `{timestamp}-{command}.log` (timestamp first)

- Example: `20251003-103045-provision.log`
- Chronological sorting by default
- Easy to find latest/oldest

**Generation**: Only on command failure (not success)

- Traces are for debugging failures
- Success cases don't need error details
- Keeps trace directory focused

## Rationale

### Why Separate Trace Files?

✅ **No Serialization Constraints**

- Error types don't need `Serialize + Deserialize`
- Can use custom `Traceable` trait instead
- Errors can contain non-serializable data

✅ **Complete Error History**

- Multiple failures preserved
- Not overwritten on retry
- Full audit trail maintained

✅ **Type-Safe Context**

- Pattern matching on enums
- Compile-time guarantees
- No string typos

✅ **Independent Concern**

- Trace generation separate from state management
- Any command can generate traces
- Flexible implementation

### Why `data/` Directory?

The `data/` vs `build/` distinction is important:

- **`data/{env}/`** = Internal application state

  - `state.json` - current environment state
  - `traces/` - error trace history
  - Managed by the application
  - Should be backed up

- **`build/{env}/`** = Generated artifacts
  - OpenTofu `.tf` files
  - Ansible inventory files
  - Can be regenerated from templates
  - Safe to delete

### Why Timestamp-First Naming?

`{timestamp}-{command}.log` format enables:

```bash
# List all traces chronologically
ls data/e2e-full/traces/
20251003-103045-provision.log
20251003-105230-provision.log
20251003-110015-configure.log

# Find latest trace
ls -1 data/e2e-full/traces/ | tail -1

# Find all provision failures
ls data/e2e-full/traces/*-provision.log
```

## Alternatives Considered

### 1. String-Based Context (Current Phase 5 Implementation)

```rust
pub struct ProvisionFailed {
    failed_step: String,
}
```

**Pros**: Simple, easy to implement
**Cons**: No type safety, no error details, typo-prone
**Verdict**: Good for MVP, insufficient for production

### 2. Full Error Serialization

```rust
pub struct ProvisionFailed {
    error: ProvisionCommandError,  // Requires Serialize
}
```

**Pros**: Complete information, type-safe
**Cons**: Forces all errors to be serializable, tight coupling
**Verdict**: Too restrictive

### 3. Captured Logs

```rust
pub struct ProvisionFailureContext {
    captured_logs: Vec<LogRecord>,
}
```

**Pros**: Complete execution context
**Cons**: Complex, indirect, format-dependent
**Verdict**: Error trace is more direct

### 4. Error Reference (Trace ID Only)

```rust
pub struct ProvisionFailureContext {
    trace_id: String,  // Search logs manually
}
```

**Pros**: Minimal state
**Cons**: Requires log retention, not self-contained
**Verdict**: Trace files provide better UX

## Consequences

### Positive

✅ Type-safe error handling with enums
✅ Complete error information preserved
✅ No serialization constraints via `Traceable` trait
✅ Error history maintained
✅ Better user experience
✅ Independent trace management

### Negative

⚠️ Additional files to manage
⚠️ Two-phase storage (state + traces)
⚠️ Initial implementation effort (~10-13 hours)

### Neutral

ℹ️ Will need trace cleanup policy eventually
ℹ️ Requires writable filesystem

## Implementation Plan

This will be implemented in a **separate refactor after Phase 5**:

1. Complete Phase 5 with string-based approach
2. Define `ProvisionFailureContext` struct with enums
3. Define `TraceId` newtype (wrapping `Uuid`) for type safety
4. Define `Traceable` trait for error formatting
5. Implement trace file writer with `PathBuf` for file paths
6. Update commands to generate traces on failure
7. Add tests and documentation

**Refactor plan**: `docs/refactors/error-context-with-trace-files.md`

## Related Decisions

- [Command State Return Pattern](./command-state-return-pattern.md)
- [Type Erasure for Environment States](./type-erasure-for-environment-states.md)
- [Actionable Error Messages](./actionable-error-messages.md)

## Summary

We chose **structured context + independent trace files** because it:

1. Provides type-safe pattern matching (enums)
2. Captures complete error chains (`Traceable` trait, not `Serialize`)
3. Maintains lightweight state files
4. Preserves error history
5. Stores traces in `data/` (internal state), not `build/` (artifacts)
6. Uses timestamp-first naming for easy sorting
7. Generates traces only on failure

This balances simplicity, completeness, and usability better than alternatives.
