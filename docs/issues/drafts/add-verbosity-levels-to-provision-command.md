# Add Verbosity Levels to Provision Command

**Issue**: TBD (Draft)
**Parent Epic**: TBD - Add levels of verbosity (Roadmap Section 8)
**Related**:

- [Roadmap Section 8](../../roadmap.md#8-add-levels-of-verbosity)
- [UX Research - Console Output & Logging Strategy](../../research/UX/console-output-logging-strategy.md)
- [UX Research - User Output vs Logging Separation](../../research/UX/user-output-vs-logging-separation.md)
- [Generic Command Progress Listener for Verbosity](generic-command-progress-listener-for-verbosity.md) — architectural design for the `CommandProgressListener` trait that enables application-layer progress reporting
- [Progress Reporting in Application Layer](../../features/progress-reporting-in-application-layer/README.md) — feature doc describing the broader problem of reporting progress from inside command handlers

**Status**: 🚧 **DRAFT** - Exploration phase, no issue opened yet

## Overview

Add graduated verbosity levels (`-v`, `-vv`, `-vvv`) to the `provision` command to give users control over the amount of user-facing progress detail displayed. This will allow users to see more detailed intermediate steps during provisioning operations without being overwhelmed by information in normal usage.

**Important**: This feature controls **only user-facing output** (via `UserOutput`), not internal logging (which remains controlled by `RUST_LOG` environment variable). The two systems are intentionally kept separate as documented in the UX research.

## Goals

- [ ] Add CLI verbosity flags (`-v`, `-vv`, `-vvv`) to control user output detail
- [ ] Wire verbosity level through execution context to controllers
- [ ] Implement graduated verbosity levels for provision command
- [ ] Maintain backward compatibility (default = Normal level)
- [ ] Keep user output completely separate from tracing logs

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

- ❌ CLI flags to capture user's desired verbosity level
- ❌ Wiring from CLI args → ExecutionContext → Controllers
- ❌ Progressive detail levels for progress messages

### Module Structure Requirements

- [ ] Add verbosity flags to `GlobalArgs` (global for all commands)
- [ ] Update `ExecutionContext` to carry verbosity level
- [ ] Update `UserOutput` construction to use CLI-provided verbosity
- [ ] Add verbosity-aware progress messages to provision workflow

### Architectural Constraints

- [ ] Verbosity flags control **only UserOutput** (user-facing messages)
- [ ] **Do not** mix verbosity with tracing logs (logs use `RUST_LOG`)
- [ ] Follow separation documented in [user-output-vs-logging-separation.md](../../research/UX/user-output-vs-logging-separation.md)
- [ ] Maintain channel separation (stdout for results, stderr for progress)
- [ ] Backward compatible (default = Normal level, existing output unchanged)

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
⏳   ✓ Environment name validated: verbosity-test-provision (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
📋   [Step 1/9] Rendering OpenTofu templates...
🔍      → Template source: templates/tofu
🔍      → Template destination: build/verbosity-test-provision/tofu
📋      → Template directory: build/verbosity-test-provision/tofu
🔍      → Rendering main.tf from template
📋      → Generated main.tf
📋   [Step 2/9] Initializing OpenTofu...
🔍      → Command: cd build/verbosity-test-provision/tofu && tofu init
🔍      → Exit code: 0
📋      → Initialized OpenTofu backend
📋   [Step 3/9] Validating infrastructure configuration...
🔍      → Command: cd build/verbosity-test-provision/tofu && tofu validate
🔍      → Output: Success! The configuration is valid.
📋      → Configuration is valid ✓
📋   [Step 4/9] Planning infrastructure changes...
🔍      → Command: cd build/verbosity-test-provision/tofu && tofu plan
🔍      → Output: Plan: 3 to add, 0 to change, 0 to destroy.
📋      → Plan: 3 to add, 0 to change, 0 to destroy
📋   [Step 5/9] Applying infrastructure changes...
🔍      → Command: cd build/verbosity-test-provision/tofu && tofu apply -auto-approve
🔍      → Output: lxd_instance.vm: Creating...
🔍      → Output: lxd_instance.vm: Creation complete after 5s [id=torrust-tracker-vm-verbosity-test-provision]
📋      → Creating lxd_instance.vm...
📋      → Instance created successfully
📋   [Step 6/9] Retrieving instance information...
🔍      → Command: cd build/verbosity-test-provision/tofu && tofu output -json
🔍      → Parsed instance IP from output: 10.140.190.235
📋      → Instance IP: 10.140.190.235
📋   [Step 7/9] Rendering Ansible templates...
🔍      → Template source: templates/ansible
🔍      → Template destination: build/verbosity-test-provision/ansible
🔍      → Injecting runtime parameter: instance_ip=10.140.190.235
📋      → Template directory: build/verbosity-test-provision/ansible
📋      → Generated inventory and playbooks
📋   [Step 8/9] Waiting for SSH connectivity...
🔍      → Max attempts: 30, timeout per attempt: 5s
📋      → Attempt 1/30: Testing connection to 10.140.190.235:22
🔍      → Command: ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no torrust@10.140.190.235 echo ok
🔍      → Exit code: 255 (connection refused)
📋      → Attempt 2/30: Testing connection to 10.140.190.235:22
🔍      → Exit code: 0 (success)
📋      → SSH connection established ✓
📋   [Step 9/9] Waiting for cloud-init completion...
🔍      → Command: ansible-playbook -i build/verbosity-test-provision/ansible/inventory.yml playbooks/wait_cloud_init.yml
🔍      → Output: TASK [Wait for cloud-init] *****
📋      → Cloud-init status: running
🔍      → Waiting 2s before retry...
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

- [ ] Task 2.0.1: Create `CommandProgressListener` trait in `src/application/ports/progress.rs` with methods: `on_step_started()`, `on_step_completed()`, `on_detail()`, `on_debug()`
- [ ] Task 2.0.2: Create `NullProgressListener` (no-op) in the same module for tests and backward compatibility
- [ ] Task 2.0.3: Create `DetailMessage` in `src/presentation/views/messages/detail.rs` (VerbosityLevel::Verbose, 📋 symbol)
- [ ] Task 2.0.4: Create `DebugDetailMessage` in `src/presentation/views/messages/debug_detail.rs` (VerbosityLevel::Debug, 🔍 symbol)
- [ ] Task 2.0.5: Add `.detail()` and `.debug_detail()` methods to `UserOutput`
- [ ] Task 2.0.6: Create `VerboseProgressListener` in `src/presentation/views/progress/verbose_listener.rs` implementing `CommandProgressListener` using `UserOutput`
- [ ] Task 2.0.7: Add `listener` parameter to `ProvisionCommandHandler.execute()` (optional, backward compatible)
- [ ] Task 2.0.8: Wire controller to create `VerboseProgressListener` and pass to handler
- [ ] Task 2.0.9: Write unit tests (trait implementation, message types, null listener)
- [ ] Task 2.0.10: Commit Phase 2.0 changes

**Rationale**: Builds the complete infrastructure before adding any progress calls. All existing behavior remains unchanged because no `on_*()` calls are emitted yet.

#### Phase 2A: Verbose Level (`-v`) - Core Step Messages (1-1.5 hours)

**Goal**: Show the 9 ProvisionStep values as progress messages via `on_step_started()`

- [ ] Task 2A.1: Add `listener.on_step_started()` calls at the start of each of the 9 steps in `ProvisionCommandHandler`:
  - [ ] Step 1/9: Rendering OpenTofu templates
  - [ ] Step 2/9: Initializing OpenTofu
  - [ ] Step 3/9: Validating infrastructure configuration
  - [ ] Step 4/9: Planning infrastructure changes
  - [ ] Step 5/9: Applying infrastructure changes
  - [ ] Step 6/9: Retrieving instance information
  - [ ] Step 7/9: Rendering Ansible templates
  - [ ] Step 8/9: Waiting for SSH connectivity
  - [ ] Step 9/9: Waiting for cloud-init completion
- [ ] Task 2A.2: Create `RecordingProgressListener` for test assertions
- [ ] Task 2A.3: Write unit tests verifying the handler emits correct step events
- [ ] Task 2A.4: Manual test with `-v` flag to verify output
- [ ] Task 2A.5: Commit Phase 2A changes

**Rationale**: Minimum useful enhancement. Users with `-v` see what the command is doing step-by-step. Tests verify event emission without presentation dependencies.

#### Phase 2B: VeryVerbose Level (`-vv`) - Add Context (1-1.5 hours)

**Goal**: Pass listener to Steps and add `on_detail()` calls with context (file paths, results, counts)

- [ ] Task 2B.1: Add `listener` parameter to Step `execute()` methods (optional, backward compatible)
- [ ] Task 2B.2: Pass listener from handler to each Step
- [ ] Task 2B.3: Add `listener.on_detail()` calls within each step:
  - [ ] Step 1: Template directory path, generated file names
  - [ ] Step 2: OpenTofu backend initialization status
  - [ ] Step 3: Configuration validation result
  - [ ] Step 4: Resource change counts (add/change/destroy)
  - [ ] Step 5: Resource creation/modification details
  - [ ] Step 6: Retrieved instance IP address
  - [ ] Step 7: Ansible template directory, generated files
  - [ ] Step 8: Retry attempt numbers, connection status
  - [ ] Step 9: Cloud-init status checks
- [ ] Task 2B.4: Manual test with `-vv` flag to verify output
- [ ] Task 2B.5: Commit Phase 2B changes

**Rationale**: Steps wrap Infrastructure calls and report using input parameters and return values. No DDD boundary violations — both handler and Steps are in the Application layer.

#### Phase 2C: Debug Level (`-vvv`) - Technical Details (1-1.5 hours)

**Goal**: Add `on_debug()` calls with technical details (commands, exit codes, raw output)

- [ ] Task 2C.1: Add `listener.on_debug()` calls in Steps with technical details:
  - [ ] Commands executed (full command strings)
  - [ ] Exit codes from external tools
  - [ ] Raw output from tools (relevant excerpts)
  - [ ] Template source/destination paths
  - [ ] Runtime parameters injected
  - [ ] Timeout values and retry configurations
- [ ] Task 2C.2: Manual test with `-vvv` flag to verify output
- [ ] Task 2C.3: Review for information overload (ensure readability)
- [ ] Task 2C.4: Commit Phase 2C changes

**Rationale**: Steps report around Infrastructure calls using return values. Infrastructure layer stays opaque (it cannot receive the listener due to DDD dependency rules). For retry loops inside Infrastructure (e.g., SSH connectivity), the pragmatic approach is to report before/after the call; moving the loop into the Step is a future improvement.

**Note**: See the [Nested Progress Reporting Analysis](generic-command-progress-listener-for-verbosity.md#nested-progress-reporting-analysis) for details on how the listener flows through layers and the Infrastructure boundary edge case.

### Phase 3: Testing and Documentation (2-3 hours)

- [ ] Task 3.1: Manual testing with `-v`, `-vv`, `-vvv` flags
- [ ] Task 3.2: Verify output readability at all levels
- [ ] Task 3.3: Update user guide documentation
- [ ] Task 3.4: Add examples to `--help` output
- [ ] Task 3.5: Consider extending to other commands (future work)

**Total Estimated Time**: 10-14 hours for provision command exploration

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] No unused dependencies: `cargo machete`
- [ ] All existing tests pass
- [ ] No clippy warnings

**Task-Specific Criteria**:

- [ ] CLI accepts `-v`, `-vv`, `-vvv` flags (counted verbosity)
- [ ] Default behavior (no flags) remains unchanged from current output
- [ ] Verbose level (`-v`) shows detailed progress messages
- [ ] Debug level (`-vvv`) shows technical details for troubleshooting
- [ ] User output stays completely separate from tracing logs
- [ ] `RUST_LOG` continues to control logging independently
- [ ] Help text clearly explains verbosity levels and their difference from logging
- [ ] Output remains clean and readable at all verbosity levels
- [ ] Channel separation maintained (stdout for results, stderr for progress)

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
