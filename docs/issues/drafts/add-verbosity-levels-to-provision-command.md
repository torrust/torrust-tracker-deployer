# Add Verbosity Levels to Provision Command

**Issue**: TBD (Draft)
**Parent Epic**: TBD - Add levels of verbosity (Roadmap Section 8)
**Related**:

- [Roadmap Section 8](../../roadmap.md#8-add-levels-of-verbosity)
- [UX Research - Console Output & Logging Strategy](../../research/UX/console-output-logging-strategy.md)
- [UX Research - User Output vs Logging Separation](../../research/UX/user-output-vs-logging-separation.md)
- [Generic Command Progress Listener for Verbosity](generic-command-progress-listener-for-verbosity.md) — architectural design for the `CommandProgressListener` trait that enables application-layer progress reporting
- [Progress Reporting in Application Layer](../../features/progress-reporting-in-application-layer/README.md) — feature doc describing the broader problem of reporting progress from inside command handlers

**Status**: 🚧 **IN PROGRESS** - Core implementation complete (Phase 1 & 2), documentation pending (Phase 3)

## Overview

Add graduated verbosity levels (`-v`, `-vv`, `-vvv`) to the `provision` command to give users control over the amount of user-facing progress detail displayed. This will allow users to see more detailed intermediate steps during provisioning operations without being overwhelmed by information in normal usage.

**Important**: This feature controls **only user-facing output** (via `UserOutput`), not internal logging (which remains controlled by `RUST_LOG` environment variable). The two systems are intentionally kept separate as documented in the UX research.

## Goals

- [x] Add CLI verbosity flags (`-v`, `-vv`, `-vvv`) to control user output detail
- [x] Wire verbosity level through execution context to controllers
- [x] Implement graduated verbosity levels for provision command
- [x] Maintain backward compatibility (default = Normal level)
- [x] Keep user output completely separate from tracing logs
- [ ] Update user documentation with verbosity examples
- [ ] Add help text examples for verbosity flags

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (`src/presentation/`)

**Module Paths**:

- CLI arguments: `src/presentation/input/cli/args.rs`
- Execution context: `src/presentation/context/execution.rs`
- Controller: `src/presentation/controllers/provision/handler.rs`
- Views: `src/presentation/views/` (already has `VerbosityLevel` enum)

**Pattern**: Global CLI flags → Execution Context → Controller → UserOutput

### Existing Infrastructure

The following components **already exist** and need to be wired together:

- ✅ `VerbosityLevel` enum in `src/presentation/views/verbosity.rs`
  - `Silent`, `Quiet`, `Normal`, `Verbose`, `VeryVerbose`, `Debug`
- ✅ `UserOutput` supports verbosity filtering
- ✅ `VerbosityFilter` implements filtering logic

**What's missing**:

- ✅ CLI flags to capture user's desired verbosity level
- ✅ Wiring from CLI args → ExecutionContext → Controllers
- ✅ Progressive detail levels for progress messages
- ❌ User guide documentation
- ❌ Help text examples

### Module Structure Requirements

- [x] Add verbosity flags to `GlobalArgs` (global for all commands)
- [x] Update `ExecutionContext` to carry verbosity level
- [x] Update `UserOutput` construction to use CLI-provided verbosity
- [x] Add verbosity-aware progress messages to provision workflow

### Architectural Constraints

- [x] Verbosity flags control **only UserOutput** (user-facing messages)
- [x] **Do not** mix verbosity with tracing logs (logs use `RUST_LOG`)
- [x] Follow separation documented in [user-output-vs-logging-separation.md](../../research/UX/user-output-vs-logging-separation.md)
- [x] Maintain channel separation (stdout for results, stderr for progress)
- [x] Backward compatible (default = Normal level, existing output unchanged)

### Anti-Patterns to Avoid

- ❌ **Don't** redirect tracing output to users based on verbosity
- ❌ **Don't** make tracing logs conditional on user verbosity flags
- ❌ **Don't** duplicate information between user output and logs
- ❌ **Don't** expose internal log format to users

## Specifications

### Verbosity Level Behaviors

Based on UX research in [console-output-logging-strategy.md](../../research/UX/console-output-logging-strategy.md):

| Level           | Flag      | User Output Behavior                    | Use Case                         |
| --------------- | --------- | --------------------------------------- | -------------------------------- |
| **Normal**      | (default) | Essential progress and results          | Regular users, normal operations |
| **Verbose**     | `-v`      | + Detailed progress, intermediate steps | Users wanting more visibility    |
| **VeryVerbose** | `-vv`     | + Decision points, retry attempts       | Troubleshooting common issues    |
| **Debug**       | `-vvv`    | + Technical details, commands executed  | Deep troubleshooting, debugging  |

**Important**: `Quiet` (`-q`) and `Silent` modes are out of scope for this initial implementation. Focus on the common case: users wanting more detail, not less.

### CLI Interface

```bash
# Normal verbosity (default) - unchanged from current behavior
torrust-tracker-deployer provision my-env

# Verbose - show detailed progress
torrust-tracker-deployer provision my-env -v
torrust-tracker-deployer provision my-env --verbose

# Very verbose - include decisions and retries
torrust-tracker-deployer provision my-env -vv

# Debug - maximum detail for troubleshooting
torrust-tracker-deployer provision my-env -vvv
```

### Example Output Progression

#### Normal Level (`VerbosityLevel::Normal`) - Default Behavior

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: verbosity-test-provision (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
⏳   ✓ Infrastructure provisioned (took 27.0s)
✅ Environment 'verbosity-test-provision' provisioned successfully


Instance Connection Details:
  IP Address:        10.140.190.235
  SSH Port:          22
  SSH Private Key:   /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa torrust@10.140.190.235 -p 22

⚠️  DNS Setup Required:
  Your configuration uses custom domains. Remember to update your DNS records
  to point your domains to the server IP: 10.140.190.235

  Configured domains:
    - tracker1.example.com
    - tracker2.example.com
    - api.example.com
    - grafana.example.com
    - health.example.com
```

#### Verbose Level (`VerbosityLevel::Verbose` / `-v`)

Shows the 9 individual **steps** from the Command→Steps architecture:

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: verbosity-test-provision (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
📋   [Step 1/9] Rendering OpenTofu templates...
📋   [Step 2/9] Initializing OpenTofu...
📋   [Step 3/9] Validating infrastructure configuration...
📋   [Step 4/9] Planning infrastructure changes...
📋   [Step 5/9] Applying infrastructure changes...
📋   [Step 6/9] Retrieving instance information...
📋   [Step 7/9] Rendering Ansible templates...
📋   [Step 8/9] Waiting for SSH connectivity...
📋   [Step 9/9] Waiting for cloud-init completion...
⏳   ✓ Infrastructure provisioned (took 27.0s)
✅ Environment 'verbosity-test-provision' provisioned successfully


Instance Connection Details:
  IP Address:        10.140.190.235
  SSH Port:          22
  SSH Private Key:   /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa torrust@10.140.190.235 -p 22

⚠️  DNS Setup Required:
  Your configuration uses custom domains. Remember to update your DNS records
  to point your domains to the server IP: 10.140.190.235

  Configured domains:
    - tracker1.example.com
    - tracker2.example.com
    - api.example.com
    - grafana.example.com
    - health.example.com
```

#### Very Verbose Level (`VerbosityLevel::VeryVerbose` / `-vv`)

Shows step details with additional context (file paths, results, retry attempts):

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: verbosity-test-provision (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
📋   [Step 1/9] Rendering OpenTofu templates...
📋      → Template directory: build/verbosity-test-provision/tofu
📋      → Generated main.tf
📋   [Step 2/9] Initializing OpenTofu...
📋      → Initialized OpenTofu backend
📋   [Step 3/9] Validating infrastructure configuration...
📋      → Configuration is valid ✓
📋   [Step 4/9] Planning infrastructure changes...
📋      → Plan: 3 to add, 0 to change, 0 to destroy
📋   [Step 5/9] Applying infrastructure changes...
📋      → Creating lxd_instance.vm...
📋      → Instance created successfully
📋   [Step 6/9] Retrieving instance information...
📋      → Instance IP: 10.140.190.235
📋   [Step 7/9] Rendering Ansible templates...
📋      → Template directory: build/verbosity-test-provision/ansible
📋      → Generated inventory and playbooks
📋   [Step 8/9] Waiting for SSH connectivity...
📋      → Attempt 1/30: Testing connection to 10.140.190.235:22
📋      → Attempt 2/30: Testing connection to 10.140.190.235:22
📋      → SSH connection established ✓
📋   [Step 9/9] Waiting for cloud-init completion...
📋      → Cloud-init status: running
📋      → Cloud-init status: done ✓
⏳   ✓ Infrastructure provisioned (took 27.0s)
✅ Environment 'verbosity-test-provision' provisioned successfully


Instance Connection Details:
  IP Address:        10.140.190.235
  SSH Port:          22
  SSH Private Key:   /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa torrust@10.140.190.235 -p 22

⚠️  DNS Setup Required:
  Your configuration uses custom domains. Remember to update your DNS records
  to point your domains to the server IP: 10.140.190.235

  Configured domains:
    - tracker1.example.com
    - tracker2.example.com
    - api.example.com
    - grafana.example.com
    - health.example.com
```

#### Debug Level (`VerbosityLevel::Debug` / `-vvv`)

Shows technical implementation details (commands, parameters, raw output):

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: verbosity-demo (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
📋   [Step 1/9] Rendering OpenTofu templates...
🔍      → Template generator: torrust_tracker_deployer_lib::infrastructure::templating::tofu::template::common::renderer::project_generator::TofuProjectGenerator
📋      → Generated OpenTofu configuration files
📋   [Step 2/9] Initializing OpenTofu...
🔍      → Working directory: ./build/verbosity-demo/tofu/lxd
🔍      → Executing: tofu init
🔍      → Command completed successfully
📋      → Initialized OpenTofu backend
📋   [Step 3/9] Validating infrastructure configuration...
🔍      → Working directory: ./build/verbosity-demo/tofu/lxd
🔍      → Executing: tofu validate
🔍      → Validation output: Success! The configuration is valid.
📋      → Configuration is valid ✓
📋   [Step 4/9] Planning infrastructure changes...
🔍      → Working directory: ./build/verbosity-demo/tofu/lxd
🔍      → Executing: tofu plan -var-file=variables.tfvars
📋      → Plan: 2 to add, 0 to change, 0 to destroy.
📋   [Step 5/9] Applying infrastructure changes...
🔍      → Working directory: ./build/verbosity-demo/tofu/lxd
🔍      → Executing: tofu apply -var-file=variables.tfvars -auto-approve
📋      → Infrastructure resources created successfully
📋   [Step 6/9] Retrieving instance information...
🔍      → Working directory: ./build/verbosity-demo/tofu/lxd
🔍      → Executing: tofu output -json
🔍      → Instance name: torrust-tracker-vm-verbosity-demo
📋      → Instance IP: 10.140.190.7
📋   [Step 7/9] Rendering Ansible templates...
🔍      → Template directory: ./data/verbosity-demo/templates
🔍      → Build directory: ./build/verbosity-demo/ansible
🔍      → Instance IP: 10.140.190.7
📋      → Template directory: ./build/verbosity-demo/ansible
📋      → Generated inventory and playbooks
📋   [Step 8/9] Waiting for SSH connectivity...
🔍      → SSH target: torrust@10.140.190.7:22
🔍      → Private key: /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa
📋      → Testing connection to 10.140.190.7:22
📋      → SSH connection established ✓
📋   [Step 9/9] Waiting for cloud-init completion...
🔍      → Ansible working directory: ./build/verbosity-demo/ansible
🔍      → Executing: ansible-playbook wait-cloud-init.yml
🔍      → Playbook completed successfully
📋      → Cloud-init status: done ✓
⏳   ✓ Infrastructure provisioned (took 25.9s)
✅ Environment 'verbosity-demo' provisioned successfully


Instance Connection Details:
  IP Address:        10.140.190.7
  SSH Port:          22
  SSH Private Key:   /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa torrust@10.140.190.7 -p 22
```

**Legend**:

- ⏳ = Major step progress (all levels)
- ✅ = Success message (all levels)
- 📋 = Detailed progress (Verbose+)
- 🔍 = Technical details (Debug only)

### Implementation Approach

#### Phase 1: Add CLI Flags (Minimal wiring)

1. Add verbosity counting flag to `GlobalArgs`:

```rust
// src/presentation/input/cli/args.rs

#[derive(clap::Args, Debug, Clone)]
pub struct GlobalArgs {
    // ... existing fields ...

    /// Increase verbosity of user-facing output
    ///
    /// Controls the amount of detail shown during operations:
    /// - Default: Essential progress and results
    /// - -v: Detailed progress including intermediate steps
    /// - -vv: Very detailed including decisions and retries
    /// - -vvv: Maximum detail for troubleshooting
    ///
    /// Note: This controls user-facing messages only. For internal
    /// logging verbosity, use the RUST_LOG environment variable.
    ///
    /// Examples:
    ///   provision my-env        # Normal verbosity
    ///   provision my-env -v     # Verbose
    ///   provision my-env -vv    # Very verbose
    ///   provision my-env -vvv   # Debug
    #[arg(
        short = 'v',
        long = "verbose",
        action = clap::ArgAction::Count,
        global = true
    )]
    pub verbosity: u8,
}

impl GlobalArgs {
    /// Convert CLI verbosity count to VerbosityLevel
    pub fn verbosity_level(&self) -> VerbosityLevel {
        match self.verbosity {
            0 => VerbosityLevel::Normal,      // Default
            1 => VerbosityLevel::Verbose,     // -v
            2 => VerbosityLevel::VeryVerbose, // -vv
            _ => VerbosityLevel::Debug,       // -vvv or more
        }
    }
}
```

1. Update `ExecutionContext` to carry verbosity:

```rust
// src/presentation/context/execution.rs

pub struct ExecutionContext {
    // ... existing fields ...
    verbosity: VerbosityLevel,
}

impl ExecutionContext {
    pub fn verbosity(&self) -> VerbosityLevel {
        self.verbosity
    }
}
```

1. Update application bootstrap to use CLI verbosity:

```rust
// src/bootstrap/app.rs or wherever UserOutput is created

let verbosity = args.verbosity_level();
let user_output = Arc::new(ReentrantMutex::new(RefCell::new(
    UserOutput::new(verbosity)
)));
```

#### Phase 2: Add Progressive Detail Using `CommandProgressListener`

> **Architecture Decision**: Progress from inside the application layer is reported via a generic `CommandProgressListener` trait (defined in Application, implemented in Presentation). This follows the Dependency Inversion Principle and avoids violating DDD layer rules. See [Generic Command Progress Listener for Verbosity](generic-command-progress-listener-for-verbosity.md) for the full architectural design.

This phase has 4 sub-phases:

#### Phase 2.0: Build the listener infrastructure

Create the trait, message types, and wiring before adding any progress calls.

#### Phase 2A: Verbose level (`-v`)

Add `on_step_started()` calls in the provision command handler for the 9 steps.

#### Phase 2B: VeryVerbose level (`-vv`)

Pass the listener to Steps and add `on_detail()` calls with context (file paths, results, retry counts).

#### Phase 2C: Debug level (`-vvv`)

Add `on_debug()` calls with technical details (commands, exit codes, raw output).

Example of how the handler reports progress:

```rust
// src/application/command_handlers/provision/handler.rs

async fn provision_infrastructure(
    &self,
    environment: &Environment<Provisioning>,
    listener: Option<&dyn CommandProgressListener>,
) -> StepResult<IpAddr, ProvisionCommandHandlerError, ProvisionStep> {
    let total_steps = 9;

    // Phase 2A: Step-level progress (Verbose / -v)
    let current_step = ProvisionStep::RenderOpenTofuTemplates;
    if let Some(l) = listener {
        l.on_step_started(1, total_steps, "Rendering OpenTofu templates");
    }
    self.render_opentofu_templates(&tofu_template_renderer).await
        .map_err(|e| (e, current_step))?;

    // Phase 2B/2C: Steps receive listener for detail/debug
    let current_step = ProvisionStep::OpenTofuApply;
    if let Some(l) = listener {
        l.on_step_started(5, total_steps, "Applying infrastructure changes");
    }
    ApplyInfrastructureStep::new(Arc::clone(&opentofu_client))
        .execute(listener)?;
    // ...
}
```

Example of how the controller creates and passes the listener:

```rust
// src/presentation/controllers/provision/handler.rs

let listener = VerboseProgressListener::new(self.progress.user_output_ref());
let provisioned = handler.execute(env_name, Some(&listener)).await?;
```

#### Phase 3: Test and Refine

1. Manual testing with different verbosity levels
2. Verify output formatting is clean and readable
3. Ensure no information overload at any level
4. Validate that `-vvv` provides enough detail for troubleshooting

## Implementation Plan

### Phase 1: CLI Flags and Wiring (2-3 hours)

- [x] Task 1.1: Add `verbosity` field to `GlobalArgs` with `ArgAction::Count`
- [x] Task 1.2: Add `verbosity_level()` method to convert count to enum
- [x] Task 1.3: Wire CLI args → ExecutionContext → UserOutput construction
- [x] Task 1.4: Write unit tests for verbosity level conversion
- [x] Task 1.5: Update doc examples to include verbosity field

**Status**: ✅ **COMPLETED** - Commit `cde6050e`

### Phase 2: Provision Command Detail Messages (5-7 hours)

> **Architecture**: Uses the `CommandProgressListener` trait pattern. See [Generic Command Progress Listener for Verbosity](generic-command-progress-listener-for-verbosity.md) for the full design, including nesting analysis and DDD layer compliance.

**Incremental implementation in 4 sub-phases**:

#### Phase 2.0: Listener Infrastructure (1.5-2 hours)

**Goal**: Build the `CommandProgressListener` trait, new message types, and presentation-layer listener implementation

- [x] Task 2.0.1: Create `CommandProgressListener` trait in `src/application/traits/progress.rs` with methods: `on_step_started()`, `on_step_completed()`, `on_detail()`, `on_debug()`
- [x] Task 2.0.2: Create `NullProgressListener` (no-op) in the same module for tests and backward compatibility
- [x] Task 2.0.3: Create `DetailMessage` in `src/presentation/views/messages/detail.rs` (VerbosityLevel::Verbose, 📋 symbol)
- [x] Task 2.0.4: Create `DebugDetailMessage` in `src/presentation/views/messages/debug_detail.rs` (VerbosityLevel::Debug, 🔍 symbol)
- [x] Task 2.0.5: Add `.detail()` and `.debug_detail()` methods to `UserOutput`
- [x] Task 2.0.6: Create `VerboseProgressListener` in `src/presentation/views/progress/verbose_listener.rs` implementing `CommandProgressListener` using `UserOutput`
- [x] Task 2.0.7: Add `listener` parameter to `ProvisionCommandHandler.execute()` (optional, backward compatible)
- [x] Task 2.0.8: Wire controller to create `VerboseProgressListener` and pass to handler
- [x] Task 2.0.9: Write unit tests (trait implementation, message types, null listener)
- [x] Task 2.0.10: Commit Phase 2.0 changes

**Status**: ✅ **COMPLETED** - Commit `136dba25`

**Rationale**: Builds the complete infrastructure before adding any progress calls. All existing behavior remains unchanged because no `on_*()` calls are emitted yet.

#### Phase 2A: Verbose Level (`-v`) - Core Step Messages (1-1.5 hours)

**Goal**: Show the 9 ProvisionStep values as progress messages via `on_step_started()`

- [x] Task 2A.1: Add `listener.on_step_started()` calls at the start of each of the 9 steps in `ProvisionCommandHandler`:
  - [x] Step 1/9: Rendering OpenTofu templates
  - [x] Step 2/9: Initializing OpenTofu
  - [x] Step 3/9: Validating infrastructure configuration
  - [x] Step 4/9: Planning infrastructure changes
  - [x] Step 5/9: Applying infrastructure changes
  - [x] Step 6/9: Retrieving instance information
  - [x] Step 7/9: Rendering Ansible templates
  - [x] Step 8/9: Waiting for SSH connectivity
  - [x] Step 9/9: Waiting for cloud-init completion
- [x] Task 2A.2: Create `RecordingProgressListener` for test assertions
- [x] Task 2A.3: Write unit tests verifying the handler emits correct step events
- [x] Task 2A.4: Manual test with `-v` flag to verify output
- [x] Task 2A.5: Commit Phase 2A changes

**Status**: ✅ **COMPLETED** - Commit `136dba25`

**Rationale**: Minimum useful enhancement. Users with `-v` see what the command is doing step-by-step. Tests verify event emission without presentation dependencies.

#### Phase 2B: VeryVerbose Level (`-vv`) - Add Context (1-1.5 hours)

**Goal**: Pass listener to Steps and add `on_detail()` calls with context (file paths, results, counts)

- [x] Task 2B.1: Add `listener` parameter to Step `execute()` methods (optional, backward compatible)
- [x] Task 2B.2: Pass listener from handler to each Step
- [x] Task 2B.3: Add `listener.on_detail()` calls within each step:
  - [x] Step 1: Template directory path, generated file names
  - [x] Step 2: OpenTofu backend initialization status
  - [x] Step 3: Configuration validation result
  - [x] Step 4: Resource change counts (add/change/destroy)
  - [x] Step 5: Resource creation/modification details
  - [x] Step 6: Retrieved instance IP address
  - [x] Step 7: Ansible template directory, generated files
  - [x] Step 8: Retry attempt numbers, connection status
  - [x] Step 9: Cloud-init status checks
- [x] Task 2B.4: Manual test with `-vv` flag to verify output
- [x] Task 2B.5: Commit Phase 2B changes
- [x] Task 2B.6: Fix verbosity filtering (DetailMessage → VeryVerbose, add StepProgressMessage for Verbose)

**Status**: ✅ **COMPLETED** - Commits `136dba25` (initial), `5b67e6a6` (filtering fix)

**Actual Output Examples**:

<details>
<summary><strong>Verbose Level (-v)</strong> - Shows step headers only</summary>

```text
⏳ [3/3] Provisioning infrastructure...
📋   [Step 1/9] Rendering OpenTofu templates...
📋   [Step 2/9] Initializing OpenTofu...
📋   [Step 3/9] Validating infrastructure configuration...
📋   [Step 4/9] Planning infrastructure changes...
📋   [Step 5/9] Applying infrastructure changes...
📋   [Step 6/9] Retrieving instance information...
📋   [Step 7/9] Rendering Ansible templates...
📋   [Step 8/9] Waiting for SSH connectivity...
📋   [Step 9/9] Waiting for cloud-init completion...
⏳   ✓ Infrastructure provisioned (took 26.1s)
```

</details>

<details>
<summary><strong>VeryVerbose Level (-vv)</strong> - Shows step headers + detail messages</summary>

```text
⏳ [3/3] Provisioning infrastructure...
📋   [Step 1/9] Rendering OpenTofu templates...
📋      → Generated OpenTofu configuration files
📋   [Step 2/9] Initializing OpenTofu...
📋      → Initialized OpenTofu backend
📋   [Step 3/9] Validating infrastructure configuration...
📋      → Configuration is valid ✓
📋   [Step 4/9] Planning infrastructure changes...
📋      → Plan: 2 to add, 0 to change, 0 to destroy.
📋   [Step 5/9] Applying infrastructure changes...
📋      → Infrastructure resources created successfully
📋   [Step 6/9] Retrieving instance information...
📋      → Instance IP: 10.140.190.59
📋   [Step 7/9] Rendering Ansible templates...
📋      → Template directory: ./build/verbosity-test-provision/ansible
📋      → Generated inventory and playbooks
📋   [Step 8/9] Waiting for SSH connectivity...
📋      → Testing connection to 10.140.190.59:22
📋      → SSH connection established ✓
📋   [Step 9/9] Waiting for cloud-init completion...
📋      → Cloud-init status: done ✓
⏳   ✓ Infrastructure provisioned (took 25.3s)
```

</details>

**Implementation Notes**:

- Created `StepProgressMessage` with `VerbosityLevel::Verbose` for step headers
- Changed `DetailMessage.required_verbosity()` from `Verbose` to `VeryVerbose`
- Added `UserOutput.step_progress()` method for step headers
- Updated `VerboseProgressListener.on_step_started()` to use `step_progress()`
- All 2292 library tests pass with filtering changes

**Rationale**: Steps wrap Infrastructure calls and report using input parameters and return values. No DDD boundary violations — both handler and Steps are in the Application layer.

#### Phase 2C: Debug Level (`-vvv`) - Technical Details (1-1.5 hours)

**Goal**: Add `on_debug()` calls with technical details (commands, exit codes, raw output)

- [x] Task 2C.1: Add `listener.on_debug()` calls in Steps with technical details:
  - [x] Commands executed (full command strings)
  - [x] Exit codes from external tools
  - [x] Raw output from tools (relevant excerpts)
  - [x] Template source/destination paths
  - [x] Runtime parameters injected
  - [x] Timeout values and retry configurations
- [x] Task 2C.2: Manual test with `-vvv` flag to verify output
- [x] Task 2C.3: Review for information overload (ensure readability)
- [x] Task 2C.4: Commit Phase 2C changes
- [x] Task 2C.5: Update documentation with actual output example

**Status**: ✅ **COMPLETED** - Commits `ebd2cdf1` (implementation), `df42958b` (documentation)

**Actual Output Example**:

<details>
<summary><strong>Debug Level (-vvv)</strong> - Shows step headers + details + technical debug info</summary>

```text
⏳ [3/3] Provisioning infrastructure...
📋   [Step 1/9] Rendering OpenTofu templates...
🔍      → Template generator: torrust_tracker_deployer_lib::infrastructure::templating::tofu::template::common::renderer::project_generator::TofuProjectGenerator
📋      → Generated OpenTofu configuration files
📋   [Step 2/9] Initializing OpenTofu...
🔍      → Working directory: ./build/verbosity-test-debug/tofu/lxd
🔍      → Executing: tofu init
🔍      → Command completed successfully
📋      → Initialized OpenTofu backend
📋   [Step 3/9] Validating infrastructure configuration...
🔍      → Working directory: ./build/verbosity-test-debug/tofu/lxd
🔍      → Executing: tofu validate
🔍      → Validation output: Success! The configuration is valid.
📋      → Configuration is valid ✓
📋   [Step 4/9] Planning infrastructure changes...
🔍      → Working directory: ./build/verbosity-test-debug/tofu/lxd
🔍      → Executing: tofu plan -var-file=variables.tfvars
📋      → Plan: 2 to add, 0 to change, 0 to destroy.
📋   [Step 5/9] Applying infrastructure changes...
🔍      → Working directory: ./build/verbosity-test-debug/tofu/lxd
🔍      → Executing: tofu apply -var-file=variables.tfvars -auto-approve
📋      → Infrastructure resources created successfully
📋   [Step 6/9] Retrieving instance information...
🔍      → Working directory: ./build/verbosity-test-debug/tofu/lxd
🔍      → Executing: tofu output -json
🔍      → Instance name: torrust-tracker-vm-verbosity-test-debug
📋      → Instance IP: 10.140.190.78
📋   [Step 7/9] Rendering Ansible templates...
🔍      → Template directory: ./data/verbosity-test-debug/templates
🔍      → Build directory: ./build/verbosity-test-debug/ansible
🔍      → Instance IP: 10.140.190.78
📋      → Template directory: ./build/verbosity-test-debug/ansible
📋      → Generated inventory and playbooks
📋   [Step 8/9] Waiting for SSH connectivity...
🔍      → SSH target: torrust@10.140.190.78:22
🔍      → Private key: /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-02/fixtures/testing_rsa
📋      → Testing connection to 10.140.190.78:22
📋      → SSH connection established ✓
📋   [Step 9/9] Waiting for cloud-init completion...
🔍      → Ansible working directory: ./build/verbosity-test-debug/ansible
🔍      → Executing: ansible-playbook wait-cloud-init.yml
🔍      → Playbook completed successfully
📋      → Cloud-init status: done ✓
⏳   ✓ Infrastructure provisioned (took 25.8s)
```

</details>

**Rationale**: Steps report around Infrastructure calls using return values. Infrastructure layer stays opaque (it cannot receive the listener due to DDD dependency rules). For retry loops inside Infrastructure (e.g., SSH connectivity), the pragmatic approach is to report before/after the call; moving the loop into the Step is a future improvement.

**Note**: See the [Nested Progress Reporting Analysis](generic-command-progress-listener-for-verbosity.md#nested-progress-reporting-analysis) for details on how the listener flows through layers and the Infrastructure boundary edge case.

### Phase 3: Testing and Documentation (2-3 hours)

- [x] Task 3.1: Manual testing with `-v`, `-vv`, `-vvv` flags
- [x] Task 3.2: Verify output readability at all levels
- [ ] Task 3.3: Update user guide documentation
- [ ] Task 3.4: Add examples to `--help` output
- [ ] Task 3.5: Consider extending to other commands (future work)

**Status**: 🚧 **IN PROGRESS** - Core implementation complete, documentation pending

**Completed**:

- Manual testing verified all verbosity levels work correctly
- Output formatting is clean and readable at all levels
- Implementation plan updated with actual output examples

**Remaining**:

- User guide documentation (Task 3.3)
- Help text examples (Task 3.4)
- Extension to other commands (Task 3.5 - future work)

**Total Estimated Time**: 10-14 hours for provision command exploration

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [x] No unused dependencies: `cargo machete`
- [x] All existing tests pass
- [x] No clippy warnings

**Task-Specific Criteria**:

- [x] CLI accepts `-v`, `-vv`, `-vvv` flags (counted verbosity)
- [x] Default behavior (no flags) remains unchanged from current output
- [x] Verbose level (`-v`) shows detailed progress messages
- [x] Very Verbose level (`-vv`) shows contextual details
- [x] Debug level (`-vvv`) shows technical details for troubleshooting
- [x] User output stays completely separate from tracing logs
- [x] `RUST_LOG` continues to control logging independently
- [ ] Help text clearly explains verbosity levels and their difference from logging
- [x] Output remains clean and readable at all verbosity levels
- [x] Channel separation maintained (stdout for results, stderr for progress)

**Out of Scope**:

- [ ] Quiet mode (`-q`) - defer to future work
- [ ] Silent mode - defer to future work
- [ ] Verbosity for other commands - defer to future work (after validating approach)

## Related Documentation

- [Roadmap Section 8 - Add levels of verbosity](../../roadmap.md#8-add-levels-of-verbosity)
- [UX Research - Console Output & Logging Strategy](../../research/UX/console-output-logging-strategy.md)
- [UX Research - User Output vs Logging Separation](../../research/UX/user-output-vs-logging-separation.md)
- [Contributing - Output Handling](../../contributing/output-handling.md)
- [Development Principles - Observability](../../development-principles.md)
- [Console App Output Patterns](../../research/UX/console-app-output-patterns.md)

## Notes

### Exploration Branch

This is being explored in the `explore-verbosity-levels-provision` branch before opening an issue. The goal is to validate the approach, understand implementation complexity, and ensure the UX feels right before committing to a full rollout across all commands.

### Design Decisions

1. **Why start with provision?**
   - Provision is a long-running command where users benefit most from progress visibility
   - Provides a representative example for other commands
   - Allows validating the pattern before wider rollout

2. **Why separate from logging?**
   - Different audiences: end users vs developers/operators
   - Different purposes: progress vs debugging
   - Allows independent evolution of each system
   - Follows established best practices (see UX research)

3. **Why not implement quiet mode yet?**
   - Most users want more detail, not less
   - Default level already provides minimal essential output
   - Quiet mode requires more thought about what's "essential"
   - Can be added later if there's demand

### Future Work

After validating the approach with provision:

- Extend to other commands (configure, release, run, destroy)
- Consider global verbosity settings (config file)
- Possibly add quiet mode (`-q`) if requested
- Document pattern for adding verbosity to new commands

### Anti-Pattern Warning

**DO NOT** do this:

```rust
// ❌ WRONG - Don't conditionally enable logging based on user verbosity
if verbosity >= VerbosityLevel::Debug {
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_writer(std::io::stderr)
            .finish()
    )?;
}
```

**Why?** This mixes user output with internal logging, breaking the separation principle. Logs should always be available (in files) and their level controlled by `RUST_LOG`, not by user-facing verbosity flags.
