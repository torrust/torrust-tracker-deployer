# Decision: Type Erasure for Environment States

## Status

Accepted

## Date

2025-10-01

## Context

We implemented a type-state pattern for `Environment<S>` in Phase 1, where each state (`Created`, `Provisioning`, `Provisioned`, etc.) is represented by a distinct generic type parameter. This provides excellent compile-time type safety and prevents invalid state transitions.

However, this created a new challenge: **how do we handle these typed environments at runtime?**

### The Problem

Each `Environment<S>` instantiation is a **different type** at compile time:

```rust
let env1: Environment<Created> = ...;      // Type A
let env2: Environment<Provisioned> = ...;  // Type B
let env3: Environment<Running> = ...;      // Type C
```

This prevents us from:

1. **Persisting to disk**: Cannot serialize without knowing the concrete type at compile time
2. **Loading from disk**: Cannot deserialize - what type should we create?
3. **Storing in collections**: Cannot have `Vec<Environment<?>>` - what type parameter?
4. **Runtime inspection**: Cannot check state without knowing the type at compile time
5. **Generic interfaces**: Cannot pass through trait objects or non-generic function parameters
6. **State repository**: Cannot implement a generic storage layer that works with all states

### Real-World Scenario

When a user runs `torrust-tracker-deployer status`, we need to:

1. Load all environments from `./data/` directory
2. Check each environment's current state
3. Display status information

**Without type erasure**, this is impossible because:

- We don't know the state types at compile time
- JSON files don't carry Rust type information
- We can't deserialize into `Environment<???>` - what type parameter?

## Decision

We will implement **Type Erasure using an Enum** to enable runtime handling while preserving the ability to restore compile-time type safety when needed.

### Solution: `AnyEnvironmentState` Enum

Create an enum with one variant per state type:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyEnvironmentState {
    Created(Environment<Created>),
    Provisioning(Environment<Provisioning>),
    Provisioned(Environment<Provisioned>),
    Configuring(Environment<Configuring>),
    Configured(Environment<Configured>),
    Releasing(Environment<Releasing>),
    Released(Environment<Released>),
    Running(Environment<Running>),
    ProvisionFailed(Environment<ProvisionFailed>),
    ConfigureFailed(Environment<ConfigureFailed>),
    ReleaseFailed(Environment<ReleaseFailed>),
    RunFailed(Environment<RunFailed>),
    Destroyed(Environment<Destroyed>),
}
```

### How It Works

**1. Type Erasure (Compile-time → Runtime)**:

```rust
// Start with typed environment
let env: Environment<Provisioned> = ...;

// Erase type information for runtime handling
let any_env: AnyEnvironmentState = env.into_any();

// Now can serialize, store, pass through generic interfaces
let json = serde_json::to_string(&any_env)?;
std::fs::write("environment.json", json)?;
```

**2. Type Restoration (Runtime → Compile-time)**:

```rust
// Load from disk (runtime)
let json = std::fs::read_to_string("environment.json")?;
let any_env: AnyEnvironmentState = serde_json::from_str(&json)?;

// Restore type safety (compile-time)
let env: Environment<Provisioned> = any_env.try_into_provisioned()?;

// Now can use type-safe methods again
let configured = env.begin_configure()?;
```

**3. Runtime Inspection**:

```rust
// Can inspect without knowing concrete type
if any_env.is_error_state() {
    eprintln!("Environment '{}' failed at: {}",
        any_env.name(),
        any_env.error_details().unwrap());
}
```

## Consequences

### Positive

✅ **Serialization Support**: Can save/load environments to/from JSON files  
✅ **Runtime Flexibility**: Can store mixed states in collections `Vec<AnyEnvironmentState>`  
✅ **Type Safety Preserved**: Can restore typed `Environment<S>` when needed  
✅ **Exhaustiveness Checking**: Compiler ensures all state variants are handled  
✅ **Zero Runtime Cost**: No dynamic dispatch overhead (unlike trait objects)  
✅ **Ergonomic Inspection**: Helper methods avoid repetitive pattern matching  
✅ **Clear Errors**: Invalid type conversions produce actionable error messages

### Negative

⚠️ **Boilerplate**: Need conversion methods for each state (13 × 2 = 26 methods)  
⚠️ **Enum Maintenance**: Must update enum when adding/removing states  
⚠️ **Type Verbosity**: Type signatures become longer with `AnyEnvironmentState`

### Trade-offs Accepted

- **More code** (conversion methods) for **better type safety** (explicit conversions)
- **Enum maintenance** for **exhaustiveness checking** (compiler catches missing states)
- **Verbose types** for **runtime flexibility** (can serialize and inspect)

## Alternatives Considered

### 1. Trait Objects (`Box<dyn EnvironmentState>`)

```rust
let env: Box<dyn EnvironmentState> = ...;
```

**Rejected because**:

- ❌ Cannot serialize trait objects with serde (requires custom machinery)
- ❌ Loses concrete type information completely (cannot downcast reliably)
- ❌ Runtime overhead from dynamic dispatch
- ❌ Cannot restore typed `Environment<S>` for type-safe transitions

### 2. Keep Everything Generic

```rust
fn load<S>() -> Environment<S> { ... }
fn store<S>(env: Environment<S>) { ... }
```

**Rejected because**:

- ❌ Caller must specify `S` at compile time (defeats the purpose)
- ❌ Cannot load from disk without knowing `S` beforehand
- ❌ Cannot store mixed states in the same collection
- ❌ Cannot implement a truly generic state repository

### 3. Separate Repository Per State

```rust
struct CreatedRepository { ... }
struct ProvisionedRepository { ... }
// ... 13 separate repositories
```

**Rejected because**:

- ❌ Massive code duplication (13 nearly identical implementations)
- ❌ Cannot list all environments across states
- ❌ Complex to manage (need 13 separate directories or tables)
- ❌ Violates DRY principle

### 4. Serialize State to String

```rust
struct SerializedEnvironment {
    state_type: String,  // "created", "provisioned", etc.
    data: serde_json::Value,
}
```

**Rejected because**:

- ❌ Loses type safety (stringly-typed)
- ❌ Error-prone (typos in state names)
- ❌ No exhaustiveness checking
- ❌ Manual serialization/deserialization logic
- ❌ Harder to maintain and debug

## Comparison: Database Single Table Inheritance

This pattern is conceptually identical to **Single Table Inheritance** in databases:

| Database STI                  | Type Erasure Enum                      |
| ----------------------------- | -------------------------------------- |
| `type` column (discriminator) | Enum variant name                      |
| Single table for all types    | Single enum type                       |
| Rows with different types     | Enum instances with different variants |
| `WHERE type = 'Provisioned'`  | `match any { Provisioned(_) => ... }`  |
| NULL for unused columns       | Type-specific fields only in variant   |
| INSERT preserves type         | Serialization preserves variant        |
| SELECT restores typed object  | `try_into_provisioned()` restores type |

## Implementation Plan

This decision is implemented in **Phase 2: Serialization & Type Erasure**, broken into three subtasks:

1. **Subtask 1**: Create `AnyEnvironmentState` enum with 13 variants
2. **Subtask 2**: Implement bidirectional type conversion methods
3. **Subtask 3**: Add introspection helpers and `Display` implementation

See [Phase 2 Plan](../features/environment-state-management/implementation-plan/phase-2-serialization.md) for detailed implementation steps.

## Success Metrics

We'll know this decision is successful when:

- ✅ Can save any environment state to JSON file
- ✅ Can load environments from disk without compile-time type knowledge
- ✅ Can list and inspect all environments regardless of state
- ✅ Can restore typed `Environment<S>` for type-safe operations
- ✅ All state transitions remain type-safe at compile time
- ✅ Error messages are clear when type conversions fail

## Related Documentation

- [Technical: Type Erasure Pattern](../technical/type-erasure-pattern.md) - Detailed pattern explanation
- [Feature: Environment State Management](../features/environment-state-management/implementation-plan/README.md) - Overall feature context
- [Phase 2 Plan](../features/environment-state-management/implementation-plan/phase-2-serialization.md) - Implementation details
- [Error Handling Guide](../contributing/error-handling.md) - How we handle type conversion errors

## References

- [Rust Design Patterns: Newtype](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
- [Serde: Enum Representations](https://serde.rs/enum-representations.html)
- [Type-State Pattern](https://cliffle.com/blog/rust-typestate/)
