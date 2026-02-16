# Add Verbosity Levels to Provision Command

**Issue**: TBD (Draft)
**Parent Epic**: TBD - Add levels of verbosity (Roadmap Section 8)
**Related**: [Roadmap Section 8](../../roadmap.md#8-add-levels-of-verbosity), [UX Research - Console Output & Logging Strategy](../../research/UX/console-output-logging-strategy.md), [UX Research - User Output vs Logging Separation](../../research/UX/user-output-vs-logging-separation.md)

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

#### Phase 2: Add Progressive Detail to Provision Workflow

Update `ProvisionCommandController` to emit detail messages at appropriate verbosity levels:

```rust
// src/presentation/controllers/provision/handler.rs

// In the validation step:
user_output.detail("Checking environment name format...");
// validation logic
user_output.detail(&format!("Environment name '{}' is valid", env_name));

// In the handler creation step:
user_output.detail("Initializing provision command handler...");
user_output.debug(&format!("Parameters: env_name={}, clock={:?}", env_name, clock));

// In the provisioning step:
user_output.detail("Rendering OpenTofu templates...");
user_output.debug(&format!("Template source: {}", template_source));
user_output.detail("Applying infrastructure changes...");
user_output.debug(&format!("Command: {}", tofu_command));
```

#### Phase 3: Test and Refine

1. Manual testing with different verbosity levels
2. Verify output formatting is clean and readable
3. Ensure no information overload at any level
4. Validate that `-vvv` provides enough detail for troubleshooting

## Implementation Plan

### Phase 1: CLI Flags and Wiring (2-3 hours)

- [ ] Task 1.1: Add `verbosity` field to `GlobalArgs` with `ArgAction::Count`
- [ ] Task 1.2: Add `verbosity_level()` method to convert count to enum
- [ ] Task 1.3: Add verbosity field to `ExecutionContext`
- [ ] Task 1.4: Wire CLI args → ExecutionContext → UserOutput construction
- [ ] Task 1.5: Write unit tests for verbosity level conversion

### Phase 2: Provision Command Detail Messages (3-4 hours)

- [ ] Task 2.1: Add `.detail()` messages for Verbose level (validation step)
- [ ] Task 2.2: Add `.detail()` messages for Verbose level (handler creation step)
- [ ] Task 2.3: Add `.detail()` messages for Verbose level (provisioning step)
- [ ] Task 2.4: Add `.debug()` messages for Debug level (technical details)
- [ ] Task 2.5: Review message wording for clarity and consistency

### Phase 3: Testing and Documentation (2-3 hours)

- [ ] Task 3.1: Manual testing with `-v`, `-vv`, `-vvv` flags
- [ ] Task 3.2: Verify output readability at all levels
- [ ] Task 3.3: Update user guide documentation
- [ ] Task 3.4: Add examples to `--help` output
- [ ] Task 3.5: Consider extending to other commands (future work)

**Total Estimated Time**: 7-10 hours for provision command exploration

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
