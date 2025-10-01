# Type Erasure Pattern in Rust

## Overview

**Type Erasure** is a design pattern that allows handling values of different compile-time types uniformly at runtime by "erasing" their specific type information into a common representation.

In Rust, the most common implementation uses an **enum with variants** that wrap different generic instantiations of the same base type.

## The Pattern

### Problem

When working with generic types like `Container<T>`, each instantiation (`Container<TypeA>`, `Container<TypeB>`) is a **distinct type** at compile time. This creates challenges:

- ❌ Cannot store different instantiations in the same collection
- ❌ Cannot serialize/deserialize without knowing the concrete type at compile time
- ❌ Cannot pass through non-generic interfaces (trait objects, function parameters)
- ❌ Cannot inspect or manipulate at runtime without type information

### Solution: Type Erasure Enum

Create an enum that can hold any instantiation of the generic type:

```rust
// Generic type with different instantiations
struct Container<T> {
    data: String,
    marker: PhantomData<T>,
}

// Type erasure enum
enum AnyContainer {
    TypeA(Container<TypeA>),
    TypeB(Container<TypeB>),
    TypeC(Container<TypeC>),
}
```

### Key Characteristics

1. **Discriminator**: The enum variant name acts as a type discriminator (similar to a `type` column in database Single Table Inheritance)
2. **Bidirectional Conversion**: Can convert from typed → erased and back
3. **Serialization**: Enum variants serialize naturally with serde
4. **Runtime Inspection**: Can inspect and categorize without pattern matching

## Implementation in This Project

### Where We Use It

**Environment State Management** (`src/domain/environment_state.rs`)

We use type erasure for the `Environment<S>` generic type, where `S` represents different state markers:

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

### Why We Need It

- **Persistence**: Save environments to disk (JSON) without knowing their state at compile time
- **Loading**: Restore environments from disk and recover their typed state
- **Collections**: Store environments in different states in the same `Vec<AnyEnvironmentState>`
- **Runtime Inspection**: Check environment state in status commands, logging, error handling
- **State Repository**: Implement generic storage interface that works with all states

### Conversion Pattern

**Type Erasure (Typed → Runtime)**:

```rust
impl Environment<Created> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Created(self)
    }
}

// Usage
let env: Environment<Created> = create_environment();
let any_env: AnyEnvironmentState = env.into_any();
// Can now serialize, store, or handle generically
```

**Type Restoration (Runtime → Typed)**:

```rust
impl AnyEnvironmentState {
    pub fn try_into_created(self) -> Result<Environment<Created>, StateTypeError> {
        match self {
            AnyEnvironmentState::Created(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "created",
                actual: other.state_name(),
            }),
        }
    }
}

// Usage
let any_env: AnyEnvironmentState = load_from_disk()?;
let env: Environment<Created> = any_env.try_into_created()?;
// Back to compile-time type safety!
```

### Introspection Helpers

Helper methods enable runtime inspection without pattern matching:

```rust
impl AnyEnvironmentState {
    pub fn name(&self) -> &EnvironmentName { /* ... */ }
    pub fn state_name(&self) -> &'static str { /* ... */ }
    pub fn is_error_state(&self) -> bool { /* ... */ }
    pub fn is_terminal_state(&self) -> bool { /* ... */ }
}

// Usage - no pattern matching needed
if any_env.is_error_state() {
    eprintln!("Environment {} failed", any_env.name());
}
```

## Database Analogy

This pattern is conceptually identical to **Single Table Inheritance** in databases:

| Database Pattern               | Rust Type Erasure Enum                    |
| ------------------------------ | ----------------------------------------- |
| `type` column (discriminator)  | Enum variant name                         |
| Single table for all subtypes  | Single enum type                          |
| Rows with different types      | Enum instances with different variants    |
| `WHERE type = 'TypeA'`         | `match any { TypeA(_) => ... }`           |
| NULL columns for unused fields | Type-specific fields only in that variant |
| INSERT/UPDATE preserves type   | Serialization preserves variant           |
| SELECT restores typed object   | `try_into_type_a()` restores typed value  |

## Benefits

✅ **Serialization**: Works seamlessly with serde  
✅ **Exhaustiveness**: Compiler ensures all variants are handled  
✅ **Performance**: No dynamic dispatch overhead (unlike trait objects)  
✅ **Type Safety**: Pattern matching is exhaustive-checked  
✅ **Ergonomics**: Helper methods avoid repetitive pattern matching  
✅ **Maintainability**: Centralized logic for common operations

## Alternatives Considered

### Trait Objects (`Box<dyn Trait>`)

```rust
// ❌ Doesn't work well
let container: Box<dyn Container> = ...;
```

**Problems**:

- Cannot serialize without custom machinery
- Loses concrete type information completely
- Runtime overhead (dynamic dispatch)
- Cannot downcast reliably

### Keep Everything Generic

```rust
// ❌ Doesn't scale
fn load<S>() -> Container<S> { ... }
```

**Problems**:

- Caller must specify `S` at compile time
- Cannot load from disk (don't know `S` beforehand)
- Cannot store mixed types

## Related Documentation

- [Decision Record: Type Erasure for Environment States](../decisions/type-erasure-for-environment-states.md) - Why we chose this approach
- [Feature: Environment State Management](../features/environment-state-management/implementation-plan/README.md) - How this fits into the larger feature
- [Phase 2 Plan](../features/environment-state-management/implementation-plan/phase-2-serialization.md) - Detailed implementation plan

## References

- [Rust Design Patterns: Type Erasure](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
- [Serde: Enum Representations](https://serde.rs/enum-representations.html)
- [Database: Single Table Inheritance](https://en.wikipedia.org/wiki/Single_Table_Inheritance)
