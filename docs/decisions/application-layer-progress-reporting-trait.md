# Decision: Application-Layer Progress Reporting Trait

## Status

Accepted

## Date

2026-02-17

## Context

Command handlers in the application layer need to report progress during long-running operations (provisioning infrastructure, configuring services, etc.). Users need feedback that the system is working, especially for operations that take 30+ seconds.

### The Problem

Without progress reporting, long-running commands appear "frozen" to users:

```bash
$ torrust-tracker-deployer provision my-env
⏳ [3/3] Provisioning infrastructure...
# User sees nothing for 30+ seconds...
✅ Environment 'my-env' provisioned successfully
```

The command handler executes 9 distinct steps (render templates, OpenTofu init/validate/plan/apply, get instance info, render Ansible templates, wait for SSH, wait for cloud-init), but the user only sees the start and end states.

### Architectural Constraints

1. **DDD Layering**: Application layer command handlers must not depend on presentation layer types (`UserOutput`)
2. **Dependency Direction**: Dependencies must flow inward (Presentation → Application → Domain)
3. **Testability**: Progress reporting must be verifiable in unit tests without UI dependencies
4. **Multiple Commands**: The solution must work for all commands (provision, configure, run, test), not just one
5. **Backward Compatibility**: Existing E2E tests must continue working without modification

### Infrastructure Project Considerations

This project deals heavily with network concepts (SSH ports, socket addresses, port bindings). Using the term "ports" for DDD architectural boundaries would cause confusion, so we use "traits" instead.

## Decision

We implement **trait-based progress reporting** using the Dependency Inversion Principle:

### Core Architecture

```rust
// Application layer defines the interface it needs
// Location: src/application/traits/progress.rs
pub trait CommandProgressListener: Send + Sync {
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str);
    fn on_step_completed(&self, step_number: usize, description: &str);
    fn on_detail(&self, message: &str);
    fn on_debug(&self, message: &str);
}
```

### Implementation Strategy

#### 1. Trait Definition (Application Layer)

The trait lives in `src/application/traits/` (not "ports" to avoid network port confusion). It defines what progress events command handlers need to emit.

#### 2. Handler Integration

Command handlers accept an optional listener:

```rust
pub async fn execute(
    &self,
    env_name: &EnvironmentName,
    listener: Option<&dyn CommandProgressListener>,
) -> Result<Environment<Provisioned>, Error> {
    // Notify progress at each step
    if let Some(l) = listener {
        l.on_step_started(1, TOTAL_STEPS, "Rendering OpenTofu templates");
    }
    // ... execute step ...
    if let Some(l) = listener {
        l.on_step_completed(1, "Rendering OpenTofu templates");
    }
    Ok(provisioned)
}
```

#### 3. Presentation Layer Implementation

The presentation layer implements the trait using `UserOutput`:

```rust
// Location: src/presentation/views/progress/verbose_listener.rs
pub struct VerboseProgressListener {
    output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
}

impl CommandProgressListener for VerboseProgressListener {
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str) {
        let mut output = self.output.lock().borrow_mut();
        output.detail(&format!("  [Step {}/{}] {}...", step_number, total_steps, description));
    }
    // ... other methods ...
}
```

#### 4. Controller Wiring

Controllers create the listener and pass it to handlers:

```rust
// Only create listener if verbosity is Verbose or higher
let listener = if self.progress.verbosity() >= VerbosityLevel::Verbose {
    Some(VerboseProgressListener::new(self.progress.output().clone()))
} else {
    None
};

let provisioned = handler.execute(env_name, listener.as_ref().map(|l| l as &dyn CommandProgressListener)).await?;
```

#### 5. Test Double for Verification

A recording listener enables testing:

```rust
// Location: src/testing/recording_progress_listener.rs
pub struct RecordingProgressListener {
    events: Arc<Mutex<Vec<ProgressEvent>>>,
}

// Test usage
let listener = RecordingProgressListener::new();
handler.execute(env_name, Some(&listener)).await?;
assert_eq!(listener.event_count(), 9); // All steps reported
```

### Verbosity Mapping

The trait methods map to user-facing verbosity levels:

- `on_step_started` / `on_step_completed` → **Verbose** (`-v`)
- `on_detail` → **VeryVerbose** (`-vv`)
- `on_debug` → **Debug** (`-vvv`)

The application layer reports everything. The presentation layer's `VerboseProgressListener` implementation filters based on the user's chosen verbosity level via `UserOutput`.

### Why `Option<&dyn CommandProgressListener>`?

Using `Option` provides backward compatibility:

- Controllers that don't support progress pass `None`
- E2E tests that directly call handlers pass `None`
- No changes needed for non-CLI entry points

## Consequences

### Benefits

1. **Clean Architecture**: Application layer doesn't depend on presentation types
2. **Testable**: Unit tests can verify progress events without UI
3. **Reusable**: Same trait works for all commands (provision, configure, run, test)
4. **Backward Compatible**: Existing code works unchanged by passing `None`
5. **Flexible**: Different implementations possible (JSON output, web UI, TUI)
6. **Type-Safe**: Compiler ensures all steps are reported correctly
7. **Clear Semantics**: `None` = silent execution, `Some` = progress reporting

### Trade-offs

1. **Boilerplate**: Every command handler needs to check `if let Some(l) = listener`
2. **Trait Object Overhead**: Small runtime cost for dynamic dispatch (negligible for I/O-bound operations)
3. **Callback Pattern**: Handlers must remember to call listener methods at appropriate points
4. **Optional Complexity**: The `Option` wrapper adds cognitive load (but enables backward compatibility)

### Risks

1. **Forgotten Notifications**: Developers might forget to add progress calls when adding new steps
   - _Mitigation_: Code review checklist, unit tests verify step count
2. **Inconsistent Descriptions**: Step descriptions might not match actual operations
   - _Mitigation_: Keep step descriptions as string literals next to the operation code
3. **Nesting Complexity**: Sub-steps within steps need careful `on_detail`/`on_debug` usage
   - _Mitigation_: Documentation and examples in architectural design doc

## Alternatives Considered

### Alternative 1: Direct `UserOutput` Dependency

```rust
pub async fn execute(
    &self,
    env_name: &EnvironmentName,
    output: &Arc<ReentrantMutex<RefCell<UserOutput>>>,
) -> Result<Environment<Provisioned>, Error>
```

**Rejected because:**

- Violates DDD layering (Application → Presentation dependency)
- Makes command handlers untestable without full presentation layer setup
- Couples application logic to specific user interface
- Cannot support non-CLI execution contexts (web UI, API)

### Alternative 2: Channels or Event Bus

```rust
let (tx, rx) = mpsc::channel();
handler.execute(env_name, tx).await?;
// Separate task consumes progress events
```

**Rejected because:**

- Over-engineering for synchronous command execution
- Adds complexity (channel lifecycle, error handling across channels)
- Harder to test (must coordinate goroutines/tasks)
- Not needed for our sequential command execution model

### Alternative 3: Callback Functions

```rust
pub async fn execute<F>(
    &self,
    env_name: &EnvironmentName,
    on_progress: F,
) -> Result<Environment<Provisioned>, Error>
where
    F: Fn(usize, usize, &str),
```

**Rejected because:**

- Less flexible than trait (single callback vs. multiple methods)
- Harder to test (function pointers less inspectable than trait objects)
- No way to distinguish step types (started vs. completed vs. detail vs. debug)
- Callback signature becomes complex for multiple event types

### Alternative 4: Observer Pattern with Registration

```rust
handler.register_observer(observer);
handler.execute(env_name).await?;
```

**Rejected because:**

- Requires mutable state in handler (registration)
- Handler instances would need to be mutable
- Tests would need cleanup (unregister observers)
- More complex than passing trait at call site

## Implementation Learnings

These patterns emerged from implementing Phase 2B (VeryVerbose detail messages) in the provision command:

### Pattern 1: Report After Infrastructure Calls

Steps report detail messages **after** Infrastructure layer calls complete, using return values:

```rust
// Step executes Infrastructure operation first
let output = self.opentofu_client.init()?;

// Then reports using the result
if let Some(l) = listener {
    l.on_detail("Initialized OpenTofu backend");
}
```

**Why**: Infrastructure layer cannot receive the listener (DDD boundary violation). Steps wrap Infrastructure calls and report using inputs/outputs.

**For complex outputs**: Parse the Infrastructure response to extract meaningful information:

```rust
let output = self.opentofu_client.plan()?;

// Parse plan output for resource counts
if let Some(l) = listener {
    if output.contains("Plan:") {
        // Extract "3 to add, 0 to change, 0 to destroy"
        l.on_detail(&parse_plan_summary(&output));
    }
}
```

### Pattern 2: Avoid Private Field Access

Detail messages should use **public APIs** only, not implementation details:

```rust
// ❌ WRONG - Accesses private field
l.on_detail(&format!("Generated files in {}", renderer.build_dir()));

// ✅ CORRECT - Uses generic message
l.on_detail("Generated OpenTofu configuration files");
```

**Why**: Attempting to access private fields/methods causes compilation errors. Keep messages user-focused, not implementation-focused.

### Pattern 3: Helper Methods Reduce Boilerplate

Handlers can use helper methods to reduce repetitive `if let Some` checks:

```rust
impl ProvisionCommandHandler {
    fn notify_step_started(
        listener: Option<&dyn CommandProgressListener>,
        step_number: usize,
        description: &str,
    ) {
        if let Some(l) = listener {
            l.on_step_started(step_number, TOTAL_PROVISION_STEPS, description);
        }
    }
}

// Usage in handler
Self::notify_step_started(listener, 1, "Rendering OpenTofu templates");
```

**Benefit**: Cleaner handler code, consistent step notification pattern.

### Pattern 4: Message Granularity Guidelines

Based on Phase 2B implementation:

| Method      | Information Type                | Examples                                                  |
| ----------- | ------------------------------- | --------------------------------------------------------- |
| `on_detail` | User-facing context and results | "Instance IP: 10.140.190.235", "Configuration is valid ✓" |
| `on_debug`  | Technical details for debugging | "Command: tofu init", "Exit code: 0", raw output excerpts |
| `on_step_*` | High-level workflow progress    | "Rendering OpenTofu templates" (step descriptions)        |

**Guidelines**:

- **Detail messages**: Answer "what happened?" from the user's perspective
- **Debug messages**: Answer "how did it work?" from the developer's perspective
- **Keep it concise**: One line per message, no multiline blocks
- **Use return values**: Report information that the Step already computed

### Pattern 5: Listener Threading Through Layers

The listener parameter flows:

```text
Controller → Handler.execute(listener) → Handler.provision_infrastructure(listener) → Step.execute(listener)
```

**Each layer**:

1. **Receives** `Option<&dyn CommandProgressListener>`
2. **Checks** for presence with `if let Some(l) = listener`
3. **Passes** the same reference to nested calls
4. **Reports** at its level of abstraction

**Key insight**: The Option type means every layer handles the "no listener" case gracefully. No separate code paths needed.

### Pattern 6: Backward Compatibility Validation

Using `Option<&dyn CommandProgressListener>` provided complete backward compatibility:

- **E2E tests**: Pass `None` - no changes needed
- **Direct handler calls**: Can omit listener parameter
- **New code**: Pass `Some(&listener)` for progress reporting

**Validation**: All 2288 library tests passed after Phase 2B without modification.

## Related Decisions

- [Execution Context Wrapper](execution-context-wrapper.md) - How `UserOutput` is passed through controller layer
- [ReentrantMutex UserOutput Pattern](reentrant-mutex-useroutput-pattern.md) - Interior mutability pattern for output
- [User Output Mutex Removal](user-output-mutex-removal.md) - Why `UserOutput` doesn't use `Mutex` directly

## References

- [Feature: Progress Reporting in Application Layer](../features/progress-reporting-in-application-layer/README.md)
- [Architectural Design: Generic Command Progress Listener for Verbosity](../issues/drafts/generic-command-progress-listener-for-verbosity.md)
- [Dependency Inversion Principle (DIP)](https://en.wikipedia.org/wiki/Dependency_inversion_principle)
- [Hexagonal Architecture (Ports and Adapters)](https://alistair.cockburn.us/hexagonal-architecture/)
