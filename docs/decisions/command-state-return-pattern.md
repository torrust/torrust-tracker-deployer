# Decision: Command State Return Pattern

## Status

Accepted

## Date

2025-10-03

## Context

In Phase 5 of the environment state management feature, we need to integrate type-safe state transitions into our command handlers (`ProvisionCommand` and `ConfigureCommand`). This raises a fundamental architectural question: **Should commands return the transformed `Environment<S>` state, or should they operate as pure command handlers that only persist state via the repository?**

### The Problem

We have two competing patterns:

1. **Typed State Returns**: Commands accept `Environment<S>` and return `Environment<T>`

   - Example: `ProvisionCommand::execute(Environment<Created>) -> Result<Environment<Provisioned>, Error>`

2. **Pure Command Handler**: Commands accept `Environment<S>`, persist state internally, return void
   - Example: `ProvisionCommand::execute(Environment<Created>) -> Result<(), Error>`

Both patterns can work, but they have different implications for type safety, data flow, and future extensibility.

### Why This Matters

- We've invested 4 phases of work building a sophisticated type-state pattern for compile-time state validation
- Commands orchestrate complex multi-step workflows with clear state progressions
- We want to enable future command chaining and orchestration with compile-time guarantees
- The repository layer exists for persistence, not as the primary data flow mechanism

## Decision

**We will use typed state returns**: Commands accept and return strongly-typed `Environment<S>` states.

```rust
impl ProvisionCommand {
    pub async fn execute(
        &self,
        environment: Environment<Created>,
    ) -> Result<Environment<Provisioned>, ProvisionCommandError> {
        // Transition to intermediate state
        let environment = environment.start_provisioning();
        self.persist_state(&environment)?; // Persistence is secondary

        // Execute provisioning steps...
        let provisioned = self.execute_steps(&environment).await?;

        // Persist final state
        self.persist_state(&provisioned)?;

        // Return transformed state
        Ok(provisioned)
    }
}
```

### Key Principles

1. **Commands are state transformations**: `Environment<S>` → `Environment<T>`
2. **Repository is for persistence**: Save/load state, but not primary data flow
3. **Type safety is paramount**: Leverage compile-time guarantees from type-state pattern
4. **Data flow is explicit**: Input → transform → output (no hidden state)

## Consequences

### Positive

✅ **Compile-Time Safety**: Invalid state transitions are prevented at compile time

- Cannot call `ConfigureCommand` on an `Environment<Created>` (not yet provisioned)
- Cannot call `ProvisionCommand` on an already `Environment<Provisioned>` instance
- Impossible to forget a state transition

✅ **Clear Data Flow**: Easy to understand what's happening

```rust
let created = Environment::new(...);
let provisioned = provision_command.execute(created).await?;
let configured = configure_command.execute(provisioned).await?;
```

✅ **No Repeated Parsing**: Avoid pattern matching on `AnyEnvironmentState`

- Without typed returns: Load from repository → match on `AnyEnvironmentState` → extract typed state
- With typed returns: State already typed, no parsing needed

✅ **Future Orchestration**: Enables fluent command chaining

```rust
// Future possibility:
let workflow = Workflow::new()
    .then(provision_command)
    .then(configure_command)
    .then(deploy_command);

let final_state = workflow.execute(created).await?;
```

✅ **Type-State Pattern Reaches Full Potential**: Commands leverage all the work from Phases 1-4

### Negative

⚠️ **Deviates from Pure CQS**: Commands traditionally shouldn't return values in strict Command/Query Separation

- However, CQRS patterns allow commands to return acknowledgments/identifiers
- State transformation is a valid command output in functional paradigms

⚠️ **Commands Return Values**: Not traditional "fire and forget" command handlers

- However, this is intentional - we want the transformed state for chaining

### Neutral

ℹ️ **Repository is Secondary**: State persistence happens alongside transformation

- This is by design - persistence is a cross-cutting concern, not the primary data flow
- Failed persistence is logged but doesn't fail the command (state is still valid in memory)

## Alternatives Considered

### Alternative 1: Pure Command Handler Pattern

```rust
impl ProvisionCommand {
    pub async fn execute(
        &self,
        environment: Environment<Created>,
    ) -> Result<(), ProvisionCommandError> {
        let environment = environment.start_provisioning();
        self.repository.save(&environment.into_any())?;

        // Execute steps...

        let provisioned = environment.complete_provisioning(ip);
        self.repository.save(&provisioned.into_any())?;

        // No return - caller must load from repository
    }
}

// Caller must load state
provision_command.execute(created).await?;
let state = repository.load(&env_name)?.expect("Must exist");
let provisioned = match state {
    AnyEnvironmentState::Provisioned(env) => env,
    _ => return Err("Wrong state!"), // Runtime error!
};
```

**Why Rejected**:

- ❌ Loses compile-time type safety (runtime pattern matching required)
- ❌ Awkward data flow (caller must reload what command just created)
- ❌ Repository becomes central to data flow (not just persistence)
- ❌ Makes command chaining difficult
- ❌ Doesn't leverage the type-state pattern we built in Phases 1-4

### Alternative 2: Hybrid - Store Environment in Command

```rust
pub struct ProvisionCommand {
    environment: RefCell<Option<Environment<Provisioning>>>,
    // ...
}

impl ProvisionCommand {
    pub async fn execute(&self, environment: Environment<Created>) -> Result<(), Error> {
        let provisioning = environment.start_provisioning();
        *self.environment.borrow_mut() = Some(provisioning);

        // Execute...

        let provisioned = self.environment.borrow().as_ref().unwrap().complete_provisioning(ip);
        self.repository.save(&provisioned.into_any())?;
    }

    pub fn get_result(&self) -> Environment<Provisioned> {
        // Complex extraction logic...
    }
}
```

**Why Rejected**:

- ❌ Interior mutability complexity (`RefCell`, borrowing rules)
- ❌ Unclear ownership semantics
- ❌ Still requires separate getter method
- ❌ Makes command non-`Send` (problematic for async)
- ❌ More complex than straightforward transformation

### Alternative 3: Builder Pattern with Fluent API

```rust
provision_command
    .with_environment(created)
    .execute()
    .await?
    .get_provisioned_environment();
```

**Why Rejected**:

- ❌ More complex API than direct transformation
- ❌ Still needs to return state somehow
- ❌ Doesn't solve the fundamental return question

## Related Decisions

- [Type Erasure for Environment States](./type-erasure-for-environment-states.md) - How we handle serialization while maintaining type safety
- [Actionable Error Messages](./actionable-error-messages.md) - Error handling approach for commands
- Phase 1-4 implementation of type-state pattern in `Environment<S>`

## References

- **Type-State Pattern in Rust**: <https://cliffle.com/blog/rust-typestate/>

  - Demonstrates how to use Rust's type system for state machines
  - Our pattern follows this approach for environment lifecycle

- **CQRS Flexibility**: <https://martinfowler.com/bliki/CQRS.html>

  - While pure CQS says commands return void, CQRS patterns often return acknowledgments
  - Command can return identifiers or confirmation objects

- **Functional Programming Perspective**:

  - State transitions as pure transformations: `S -> T`
  - Commands as functions that transform state
  - Side effects (persistence) are secondary concerns

- **Rust Ownership Model**:

  - Returning transformed data is idiomatic in Rust
  - Ownership transfer makes data flow explicit
  - No implicit state mutations

- **Phase 1-4 Implementation**:
  - `docs/features/environment-state-management/feature-description.md`
  - `src/domain/environment/mod.rs` - Type-state implementation
  - `src/infrastructure/persistence/` - Repository layer

## Implementation Notes

### Persistence Error Handling

Persistence failures are logged but don't fail the command:

```rust
if let Err(e) = self.persist_state(&environment) {
    warn!(
        "Failed to persist state: {}. Command execution continues.",
        e
    );
}
```

**Rationale**: The in-memory state transformation is valid even if persistence fails. We log for observability but don't block the workflow.

### Command Chaining Pattern

This decision enables future orchestration:

```rust
// Phase 5 Subtasks 3-4: Individual commands return typed states
let provisioned = provision_cmd.execute(created).await?;
let configured = configure_cmd.execute(provisioned).await?;

// Future: Orchestration layer with compile-time guarantees
let workflow = Orchestrator::new()
    .step(provision_cmd)    // Requires Created, produces Provisioned
    .step(configure_cmd)    // Requires Provisioned, produces Configured
    .step(deploy_cmd);      // Requires Configured, produces Deployed

workflow.execute(created).await?; // Type-checked at compile time
```

### Backward Compatibility

Commands still work with existing E2E tests by extracting values from returned states:

```rust
// Old pattern (Phase 5 Subtask 1):
let ip_address = provision_command.execute(&ssh_credentials).await?;

// New pattern (Phase 5 Subtask 3+):
let provisioned = provision_command.execute(environment).await?;
let ip_address = provisioned.instance_ip(); // Getter method
```
