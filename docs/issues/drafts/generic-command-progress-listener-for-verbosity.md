# Generic Command Progress Listener for Verbosity

**Issue**: TBD (Draft)
**Parent Epic**: TBD - Add levels of verbosity (Roadmap Section 8)
**Related**:

- [Add Verbosity Levels to Provision Command](add-verbosity-levels-to-provision-command.md)
- [Progress Reporting in Application Layer](../../features/progress-reporting-in-application-layer/README.md)
- [DDD Layer Placement Guide](../../contributing/ddd-layer-placement.md)
- [User Output vs Logging Separation](../../research/UX/user-output-vs-logging-separation.md)

**Status**: 🚧 **DRAFT** - Architectural design, not yet implemented

## Problem

The verbosity feature needs to display information about **internal steps** that execute inside the **application layer** (command handlers). Currently, the presentation layer calls `handler.execute(env_name).await` as a black box and receives only the final result. It has no visibility into the 9 intermediate steps happening inside the provision handler.

The challenge: how do we report progress from the application layer to the presentation layer without violating DDD dependency rules?

## Options Considered

### Option 1: Return Data for Presentation (Pure DDD)

The command handler returns structured data about what happened, and the presentation layer formats it afterward.

```rust
// Application layer returns step results
let result = handler.execute(env_name).await?;
// result contains: Vec<StepResult> with timing, status, details

// Presentation layer formats them
for step in &result.steps {
    user_output.detail(&format!("[Step {}/{}] {}", step.number, step.total, step.description));
}
```

**Pros**: Cleanest DDD separation, no callbacks, easy to test.
**Cons**: Progress appears only AFTER the command completes. Defeats the purpose of real-time progress visibility for long-running operations. Users see a frozen screen for 30+ seconds during provisioning.

### Option 2: Inject UserOutput Into Command Handlers

Pass `UserOutput` (presentation layer service) directly into the application layer.

```rust
// ❌ Application layer depends on Presentation layer
pub async fn execute(
    &self,
    env_name: &EnvironmentName,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
) -> Result<Environment<Provisioned>, ProvisionCommandHandlerError>
```

**Pros**: Simple to implement.
**Cons**: **Violates DDD dependency rule.** Application → Presentation is a forbidden dependency. Makes the application layer untestable without presentation infrastructure. Couples business logic to UI concerns.

### Option 3: Generic Trait-Based Callback (Chosen)

Define a generic trait in the **application layer** that the presentation layer implements. The trait is defined where it's consumed (application), implemented where the UI lives (presentation). Dependencies flow correctly.

**Pros**: DDD-compliant, testable, reusable across all commands, real-time progress.
**Cons**: Slightly more code than Option 2 (one trait + one implementation).

## Chosen Solution: Generic `CommandProgressListener` Trait

### Why Generic Instead of Per-Command?

A per-command listener (e.g., `ProvisionProgressListener`, `ConfigureProgressListener`) was considered but rejected because:

1. **The listener doesn't need type-safe step identity** — it only displays human-readable text
2. **All commands have the same progress reporting needs** — step started, step completed, detail, debug
3. **Command-specific step enums stay internal** — `ProvisionStep`, `ConfigureStep`, etc. remain in their respective command handlers for error tracking; the listener receives only their `.description()` strings
4. **One presentation implementation serves all commands** — no trait proliferation

### Trait Definition

Lives in the **application layer** (where it's consumed):

```rust
// src/application/ports/progress.rs

/// A listener for reporting command progress to the user interface.
///
/// This trait is defined in the application layer and implemented in the
/// presentation layer, following the Dependency Inversion Principle.
/// The application layer depends on this abstraction, not on concrete
/// UI implementations.
///
/// # DDD Layer Placement
///
/// - **Defined in**: Application layer (`src/application/ports/`)
/// - **Implemented in**: Presentation layer (`src/presentation/`)
/// - **Dependency direction**: Presentation → Application (correct)
///
/// # Verbosity Mapping
///
/// The listener methods map to verbosity levels:
/// - `on_step_started` / `on_step_completed` → `VerbosityLevel::Verbose` (`-v`)
/// - `on_detail` → `VerbosityLevel::VeryVerbose` (`-vv`)
/// - `on_debug` → `VerbosityLevel::Debug` (`-vvv`)
///
/// The listener implementation decides what to display based on the
/// user's chosen verbosity level. The application layer does not know
/// or care about verbosity — it simply reports everything.
pub trait CommandProgressListener: Send + Sync {
    /// Called when a step begins execution.
    ///
    /// # Arguments
    /// * `step_number` - 1-based step index
    /// * `total_steps` - Total number of steps in the workflow
    /// * `description` - Human-readable step description
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str);

    /// Called when a step completes successfully.
    ///
    /// # Arguments
    /// * `step_number` - 1-based step index
    /// * `description` - Human-readable step description
    fn on_step_completed(&self, step_number: usize, description: &str);

    /// Reports a contextual detail about the current operation.
    /// Intended for intermediate results, file paths, counts, etc.
    ///
    /// # Arguments
    /// * `message` - Human-readable detail message
    fn on_detail(&self, message: &str);

    /// Reports a technical/debug detail about the current operation.
    /// Intended for commands executed, exit codes, raw output, etc.
    ///
    /// # Arguments
    /// * `message` - Technical detail message
    fn on_debug(&self, message: &str);
}
```

### Why Application Layer (Not Domain or Shared)?

The trait lives in `src/application/ports/` because:

- **It's a use-case concern**: Progress reporting is about orchestrating steps in a workflow, which is the application layer's responsibility
- **Domain layer is too low**: The domain doesn't know about command workflows or steps
- **Shared would be wrong**: This isn't a cross-cutting utility like `Clock` — it's specific to command execution workflows
- **Follows existing patterns**: The project already uses `dyn Clock` and `dyn EnvironmentRepository` as trait-based dependencies injected into handlers

### Dependency Flow

```text
┌─────────────────────────────────────────────┐
│  Presentation Layer                         │
│                                             │
│  VerboseProgressListener                    │
│    impl CommandProgressListener             │
│    uses UserOutput to emit messages         │
│    respects VerbosityLevel for filtering    │
└────────────────┬────────────────────────────┘
                 │ implements trait from
                 ↓
┌─────────────────────────────────────────────┐
│  Application Layer                          │
│                                             │
│  trait CommandProgressListener (defined)     │
│                                             │
│  ProvisionCommandHandler                    │
│    receives &dyn CommandProgressListener    │
│    calls listener.on_step_started(...)      │
│    calls listener.on_detail(...)            │
│    calls listener.on_debug(...)             │
└─────────────────────────────────────────────┘
```

Dependencies flow **inward** (Presentation → Application), never outward. The application layer depends only on its own abstraction.

## Implementation Design

### Application Layer Changes

The command handler gains an optional listener parameter (backward compatible):

```rust
// src/application/command_handlers/provision/handler.rs

pub async fn execute(
    &self,
    env_name: &EnvironmentName,
    listener: Option<&dyn CommandProgressListener>,
) -> Result<Environment<Provisioned>, ProvisionCommandHandlerError> {
    // ...existing code...
}
```

Inside the handler, at each step boundary:

```rust
async fn provision_infrastructure(
    &self,
    environment: &Environment<Provisioning>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<IpAddr, ProvisionCommandHandlerError, ProvisionStep> {
    let total_steps = 9;

    let current_step = ProvisionStep::RenderOpenTofuTemplates;
    if let Some(l) = listener {
        l.on_step_started(1, total_steps, "Rendering OpenTofu templates");
    }
    self.render_opentofu_templates(&tofu_template_renderer)
        .await
        .map_err(|e| (e, current_step))?;
    if let Some(l) = listener {
        l.on_step_completed(1, "Rendering OpenTofu templates");
    }

    let current_step = ProvisionStep::OpenTofuInit;
    if let Some(l) = listener {
        l.on_step_started(2, total_steps, "Initializing OpenTofu");
    }
    // ... and so on for all 9 steps
}
```

The handler reports **everything** — it doesn't know or care about verbosity levels. That filtering happens in the presentation layer implementation.

### Presentation Layer Changes

#### New Message Types

Two new message types are needed for the new verbosity levels:

```rust
// src/presentation/views/messages/detail.rs
pub struct DetailMessage { pub text: String }

impl OutputMessage for DetailMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}", theme.detail_symbol(), self.text)
    }
    fn required_verbosity(&self) -> VerbosityLevel { VerbosityLevel::Verbose }
    fn channel(&self) -> Channel { Channel::Stderr }
    fn type_name(&self) -> &str { "detail" }
}

// src/presentation/views/messages/debug_detail.rs
pub struct DebugDetailMessage { pub text: String }

impl OutputMessage for DebugDetailMessage {
    fn format(&self, theme: &Theme) -> String {
        format!("{} {}", theme.debug_symbol(), self.text)
    }
    fn required_verbosity(&self) -> VerbosityLevel { VerbosityLevel::Debug }
    fn channel(&self) -> Channel { Channel::Stderr }
    fn type_name(&self) -> &str { "debug_detail" }
}
```

#### New UserOutput Methods

```rust
// src/presentation/views/user_output.rs

impl UserOutput {
    /// Emit a detail message (shown at Verbose level and above)
    pub fn detail(&mut self, text: &str) { ... }

    /// Emit a debug detail message (shown at Debug level and above)
    pub fn debug_detail(&mut self, text: &str) { ... }
}
```

#### Listener Implementation

```rust
// src/presentation/views/progress/verbose_listener.rs

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::ports::CommandProgressListener;
use crate::presentation::views::UserOutput;

/// Presentation layer implementation of `CommandProgressListener`.
///
/// Translates application-layer progress events into user-facing output
/// through `UserOutput`. The verbosity filtering is handled automatically
/// by the `VerbosityFilter` inside `UserOutput` — this listener simply
/// emits all messages at their appropriate verbosity level.
pub struct VerboseProgressListener {
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
}

impl VerboseProgressListener {
    pub fn new(user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>) -> Self {
        Self { user_output }
    }

    fn with_output<F>(&self, f: F)
    where
        F: FnOnce(&mut UserOutput),
    {
        let guard = self.user_output.lock();
        let mut output = guard.borrow_mut();
        f(&mut output);
    }
}

impl CommandProgressListener for VerboseProgressListener {
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str) {
        self.with_output(|output| {
            output.detail(&format!(
                "  [Step {step_number}/{total_steps}] {description}..."
            ));
        });
    }

    fn on_step_completed(&self, _step_number: usize, _description: &str) {
        // Steps complete silently — the next on_step_started provides
        // visual progress. Completion details go through on_detail().
    }

    fn on_detail(&self, message: &str) {
        self.with_output(|output| {
            output.detail(&format!("     → {message}"));
        });
    }

    fn on_debug(&self, message: &str) {
        self.with_output(|output| {
            output.debug_detail(&format!("     → {message}"));
        });
    }
}
```

### Controller Wiring

The presentation controller creates the listener and passes it to the handler:

```rust
// src/presentation/controllers/provision/handler.rs

async fn provision_infrastructure(
    &mut self,
    handler: &ProvisionCommandHandler,
    env_name: &EnvironmentName,
) -> Result<Environment<Provisioned>, ProvisionSubcommandError> {
    self.progress
        .start_step(ProvisionStep::ProvisionInfrastructure.description())?;

    // Create the listener for verbose progress reporting
    let listener = VerboseProgressListener::new(self.progress.user_output_ref());

    let provisioned = handler
        .execute(env_name, Some(&listener))
        .await
        .map_err(|source| ProvisionSubcommandError::ProvisionOperationFailed {
            name: env_name.to_string(),
            source: Box::new(source),
        })?;

    self.progress
        .complete_step(Some("Infrastructure provisioned"))?;
    Ok(provisioned)
}
```

### Testing

#### No-Op Listener for Tests

```rust
// src/application/ports/progress.rs (or a testing module)

/// A no-op listener for use in tests or when progress reporting is not needed.
pub struct NullProgressListener;

impl CommandProgressListener for NullProgressListener {
    fn on_step_started(&self, _: usize, _: usize, _: &str) {}
    fn on_step_completed(&self, _: usize, _: &str) {}
    fn on_detail(&self, _: &str) {}
    fn on_debug(&self, _: &str) {}
}
```

#### Recording Listener for Assertions

```rust
// src/testing/ or test modules

/// A recording listener that captures all events for test assertions.
pub struct RecordingProgressListener {
    events: RefCell<Vec<ProgressEvent>>,
}

pub enum ProgressEvent {
    StepStarted { step_number: usize, total_steps: usize, description: String },
    StepCompleted { step_number: usize, description: String },
    Detail(String),
    Debug(String),
}

impl CommandProgressListener for RecordingProgressListener {
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str) {
        self.events.borrow_mut().push(ProgressEvent::StepStarted {
            step_number,
            total_steps,
            description: description.to_string(),
        });
    }
    // ... other methods similarly record events
}
```

This allows unit tests to verify that the handler emits the correct progress events without any presentation layer dependency.

## Module Structure

```text
src/
├── application/
│   ├── ports/
│   │   ├── mod.rs
│   │   └── progress.rs          # CommandProgressListener trait + NullProgressListener
│   └── command_handlers/
│       └── provision/
│           └── handler.rs        # Uses &dyn CommandProgressListener
├── presentation/
│   └── views/
│       ├── messages/
│       │   ├── detail.rs         # DetailMessage (Verbose level, 📋)
│       │   └── debug_detail.rs   # DebugDetailMessage (Debug level, 🔍)
│       ├── progress/
│       │   └── verbose_listener.rs  # VerboseProgressListener
│       └── user_output.rs        # New .detail() and .debug_detail() methods
└── testing/
    └── recording_listener.rs     # RecordingProgressListener for tests
```

## How This Extends to Other Commands

Each command handler can accept the same generic listener:

```rust
// Configure command handler
pub async fn execute(
    &self,
    env_name: &EnvironmentName,
    listener: Option<&dyn CommandProgressListener>,
) -> Result<Environment<Configured>, ConfigureCommandHandlerError> {
    if let Some(l) = listener {
        l.on_step_started(1, 3, "Running configuration playbook");
    }
    // ...
}

// Release command handler
pub async fn execute(
    &self,
    env_name: &EnvironmentName,
    listener: Option<&dyn CommandProgressListener>,
) -> Result<Environment<Released>, ReleaseCommandHandlerError> {
    if let Some(l) = listener {
        l.on_step_started(1, 5, "Uploading Docker Compose configuration");
    }
    // ...
}
```

The same `VerboseProgressListener` in the presentation layer handles all commands with zero additional code.

## Comparison with Existing Patterns

| Pattern                       | Where                              | Example                          | Purpose                     |
| ----------------------------- | ---------------------------------- | -------------------------------- | --------------------------- |
| `dyn Clock`                   | Shared → injected everywhere       | `Arc<dyn Clock>`                 | Abstract time source        |
| `dyn EnvironmentRepository`   | Domain → impl in Infrastructure    | `Arc<dyn EnvironmentRepository>` | Abstract persistence        |
| `dyn CommandProgressListener` | Application → impl in Presentation | `&dyn CommandProgressListener`   | Abstract progress reporting |

All three follow the same Dependency Inversion Principle: the trait is defined where it's consumed, implemented where the concrete behavior lives.

## Relationship to Existing Feature Documentation

This document refines the proposal in [Progress Reporting in Application Layer](../../features/progress-reporting-in-application-layer/README.md), which identified the need for "Passing a `ProgressReporter` trait or callback into the Application Layer" but did not specify the concrete design. This document provides that concrete design.

## Nested Progress Reporting Analysis

### The Four Depth Levels

The architecture naturally produces four depth levels, each mapping to a verbosity level and a layer in the codebase:

```text
Level 0  Presentation Controller    [1/3] Provisioning infrastructure...        Normal (default)
Level 1  Command Handler (steps)    [Step 5/9] Applying infrastructure...       Verbose (-v)
Level 2  Step internals             → Creating lxd_instance.vm...               VeryVerbose (-vv)
Level 3  Infrastructure details     → Command: tofu apply -auto-approve         Debug (-vvv)
```

### How Trait Methods Map to Depth Levels

| Trait Method                      | Depth Level     | Content              | Structured?    |
| --------------------------------- | --------------- | -------------------- | -------------- |
| `on_step_started(n, total, desc)` | 1 (Verbose)     | Step boundaries      | Yes — numbered |
| `on_step_completed(n, desc)`      | 1 (Verbose)     | Step completion      | Yes — numbered |
| `on_detail(msg)`                  | 2 (VeryVerbose) | Context within steps | No — free-form |
| `on_debug(msg)`                   | 3 (Debug)       | Technical details    | No — free-form |

Levels 0-1 use **structured methods** with step numbers and totals. Levels 2-3 use **free-form messages** because sub-tasks within a step don't need `[Sub-step 2/3]` numbering — they need contextual information like "Creating lxd_instance.vm..." or "Attempt 2/30: Testing connection...".

### How the Listener Flows Through Layers

**Handler → Steps** works naturally because both live in the Application layer:

```rust
// Handler (application layer) passes listener to Step (also application layer)
let current_step = ProvisionStep::OpenTofuApply;
if let Some(l) = listener {
    l.on_step_started(5, 9, "Applying infrastructure changes");
}

// Step receives the listener and reports sub-task detail
ApplyInfrastructureStep::new(Arc::clone(opentofu_client))
    .execute(listener)?;  // ← pass it through
```

Inside the Step, Level 2 and Level 3 messages are emitted around Infrastructure calls:

```rust
// Inside ApplyInfrastructureStep (application layer)
impl ApplyInfrastructureStep {
    pub fn execute(
        &self,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<(), CommandError> {
        if let Some(l) = listener {
            l.on_debug(&format!("Command: tofu apply -auto-approve -var-file=variables.tfvars"));
        }

        let output = self.opentofu_client.apply(self.auto_approve, &[...])?;

        if let Some(l) = listener {
            l.on_detail("Instance created successfully");
            l.on_debug(&format!("Exit code: 0, output: {output}"));
        }
        Ok(())
    }
}
```

### Infrastructure Layer Boundary

The `OpenTofuClient` and `SshClient` live in `src/adapters/` (Infrastructure layer). The DDD dependency rule prevents them from receiving the listener:

```text
Infrastructure → Domain    ✅ (allowed)
Infrastructure → Application    ❌ (forbidden)
```

Since `CommandProgressListener` is defined in the Application layer, Infrastructure code **cannot** depend on it. However, this is actually fine because:

1. **Steps wrap Infrastructure calls** — the Step (Application layer) calls `self.opentofu_client.apply()` and reports `on_detail()`/`on_debug()` before and after the call, using the return values
2. **Infrastructure already returns data** — `apply()` returns `Result<String, CommandError>` with the output. The Step has all the information it needs to report progress
3. **The Step sees inputs and outputs** — it knows what command will run (input) and what happened (output), which is exactly the information needed for VeryVerbose and Debug levels

### The Edge Case: Retry Loops in Infrastructure

The one place this gets tricky is retry loops that live inside Infrastructure code. For example, `SshClient.wait_for_connectivity()` has a loop with up to 30 attempts:

```rust
// src/adapters/ssh/client.rs — Infrastructure layer
pub async fn wait_for_connectivity(&self) -> Result<(), SshError> {
    while attempt < max_attempts {
        let result = self.test_connectivity();
        match result {
            Ok(true) => return Ok(()),
            Ok(false) => {
                // Can't call listener here — Infrastructure can't depend on Application
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
    Err(SshError::ConnectivityTimeout { ... })
}
```

If we want to report "Attempt 2/30: Testing connection..." at VeryVerbose level, we have two options:

#### Option A: Refactor the retry loop into the Step (preferred)

Move the loop logic from Infrastructure into the Application layer Step, keeping Infrastructure responsible only for a single attempt:

```rust
// WaitForSSHConnectivityStep (application layer) owns the retry loop
impl WaitForSSHConnectivityStep {
    pub async fn execute(
        &self,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<(), SshError> {
        let ssh_client = SshClient::new(self.ssh_config.clone());

        if let Some(l) = listener {
            l.on_debug(&format!("Max attempts: {max_attempts}, timeout per attempt: {timeout}s"));
        }

        for attempt in 1..=max_attempts {
            if let Some(l) = listener {
                l.on_detail(&format!(
                    "Attempt {attempt}/{max_attempts}: Testing connection to {}:{}",
                    ip, port
                ));
            }

            if ssh_client.test_connectivity()? {
                if let Some(l) = listener {
                    l.on_detail("SSH connection established ✓");
                }
                return Ok(());
            }

            if let Some(l) = listener {
                l.on_debug(&format!("Connection refused, retrying in {delay}s..."));
            }
            tokio::time::sleep(delay).await;
        }

        Err(SshError::ConnectivityTimeout { ... })
    }
}
```

This is the better approach because it also **improves testability** — the retry policy becomes testable at the Step level without needing a real SSH server.

#### Option B: Accept that Infrastructure internals are opaque

Report only before and after the Infrastructure call:

```rust
// Step reports what it can
if let Some(l) = listener {
    l.on_detail(&format!("Waiting for SSH connectivity to {}:{}...", ip, port));
}
ssh_client.wait_for_connectivity().await?;
if let Some(l) = listener {
    l.on_detail("SSH connection established ✓");
}
```

This is pragmatic for the initial implementation. Per-attempt reporting is deferred until the retry loop is refactored.

### Summary: Does This Design Support Nesting?

**Yes.** The design supports all four depth levels because:

- **Level 1** (Steps) — `on_step_started`/`on_step_completed` provide structured step-level progress
- **Levels 2-3** (Sub-tasks and technical details) — `on_detail`/`on_debug` provide free-form reporting within steps
- **Listener passes from Handler → Steps** — both live in the Application layer, no dependency violation
- **Steps report around Infrastructure calls** — using input parameters and return values
- **Infrastructure stays opaque** — which is correct for DDD; the Application layer is the boundary where progress is reported

The main trade-off: retry loops inside Infrastructure code need refactoring (moving the loop into the Step) to enable per-iteration progress reporting. This is a clean refactor that also improves testability, and can be done incrementally per-step as needed.

If sub-step numbering is ever needed in the future (e.g., "[Sub-step 2/5] Uploading file..."), the trait can be extended with an `on_substep_started()` method — but this is not needed now (YAGNI).

## Anti-Patterns to Avoid

- **Do not** create per-command listener traits (e.g., `ProvisionProgressListener`) — use the generic `CommandProgressListener`
- **Do not** pass `UserOutput` into the application layer — this inverts the dependency direction
- **Do not** make the listener aware of command-specific step enums — it receives only strings
- **Do not** make filtering decisions in the listener — `UserOutput.detail()` and `UserOutput.debug_detail()` already respect `VerbosityFilter`
- **Do not** return progress data after completion — the whole point is real-time feedback during long-running operations
- **Do not** pass the listener into Infrastructure layer code — Infrastructure cannot depend on Application layer traits; instead, report progress from the Step before/after Infrastructure calls
