# Add Verbosity Levels to Configure Command

**Issue**: #363
**Parent Epic**: #362 - Add levels of verbosity (Roadmap Section 8)
**Related**:

- [Roadmap Section 8](../roadmap.md#8-add-levels-of-verbosity)
- [Task 8.1 - Provision Command Verbosity (Completed)](./drafts/add-verbosity-levels-to-provision-command.md) — reference implementation
- [UX Research - Console Output & Logging Strategy](../research/UX/console-output-logging-strategy.md)
- [UX Research - User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md)
- [Generic Command Progress Listener for Verbosity](./drafts/generic-command-progress-listener-for-verbosity.md) — architectural design for the `CommandProgressListener` trait
- [Progress Reporting in Application Layer](../features/progress-reporting-in-application-layer/README.md)

**Status**: 📋 NOT STARTED

## Overview

Add graduated verbosity levels (`-v`, `-vv`, `-vvv`) to the `configure` command to give users control over the amount of user-facing progress detail displayed. This applies the same verbosity pattern established in the provision command (task 8.1) to the configure command, which handles system configuration via Ansible playbooks.

**Important**: This feature controls **only user-facing output** (via `UserOutput`), not internal logging (which remains controlled by `RUST_LOG` environment variable). The two systems are intentionally kept separate as documented in the UX research.

## Goals

- [ ] Reuse `CommandProgressListener` infrastructure from provision command
- [ ] Apply same four verbosity levels to configure command
- [ ] Show configuration steps, Ansible operations, and system changes at different detail levels
- [ ] Maintain backward compatibility (default = Normal level)
- [ ] Keep user output completely separate from tracing logs
- [ ] Update user documentation with verbosity examples
- [ ] Add help text examples to configure command

## 🏗️ Architecture Requirements

**DDD Layer**: Application (`src/application/command_handlers/configure/`)

**Module Paths**:

- Command handler: `src/application/command_handlers/configure/handler.rs`
- Configuration steps: `src/application/steps/configuration/` (various Ansible playbook steps)
- Progress listener: `src/application/traits/command_progress_listener.rs` (already exists)
- CLI integration: `src/presentation/controllers/configure/handler.rs`

**Pattern**: Same as provision command - use `CommandProgressListener` trait for progress reporting

### Existing Infrastructure (from Task 8.1)

The following components **already exist** from the provision command implementation:

- ✅ `CommandProgressListener` trait in `src/application/traits/command_progress_listener.rs`
  - `on_step_started(step_number, total_steps, step_name)`
  - `on_detail(message)` for contextual details
  - `on_debug(message)` for technical implementation details
  - `on_success(operation_name, duration)`
- ✅ `UserOutputProgressListener` in `src/presentation/views/messages/user_output_progress_listener.rs`
  - Converts progress events to user-facing messages based on verbosity level
- ✅ CLI verbosity flags already wired through `ExecutionContext`
- ✅ Four verbosity levels: Normal, Verbose (-v), VeryVerbose (-vv), Debug (-vvv)

**What's needed for configure command**:

- [ ] Pass `CommandProgressListener` to configure command handler
- [ ] Add `on_step_started()` calls for configuration steps
- [ ] Add `on_detail()` calls for Ansible operation context
- [ ] Add `on_debug()` calls for technical Ansible details
- [ ] Update user guide documentation
- [ ] Verify help text shows verbosity options

### Module Structure Requirements

- [ ] Follow same pattern as provision command (reference implementation)
- [ ] Handler emits step-level progress events
- [ ] Steps emit detail and debug events
- [ ] Infrastructure layer (Ansible) remains unaware of listener

### Architectural Constraints

- [ ] Verbosity flags control **only UserOutput** (user-facing messages)
- [ ] **Do not** mix verbosity with tracing logs (logs use `RUST_LOG`)
- [ ] Follow separation documented in [user-output-vs-logging-separation.md](../research/UX/user-output-vs-logging-separation.md)
- [ ] Maintain channel separation (stdout for results, stderr for progress)
- [ ] Backward compatible (default = Normal level, existing output unchanged)
- [ ] Reuse `CommandProgressListener` trait (do not create command-specific variants)

### Anti-Patterns to Avoid

- ❌ **Don't** create command-specific progress listener traits
- ❌ **Don't** pass listener to Infrastructure layer (violates DDD dependency rules)
- ❌ **Don't** duplicate the pattern - reuse what provision command established
- ❌ **Don't** change the verbosity flag names or meanings

## Specifications

### Verbosity Level Behaviors

Same four levels as provision command:

| Level           | Flag      | User Output Behavior                          | Use Case                         |
| --------------- | --------- | --------------------------------------------- | -------------------------------- |
| **Normal**      | (default) | Essential progress and results                | Regular users, normal operations |
| **Verbose**     | `-v`      | + Configuration steps, playbook names         | Users wanting more visibility    |
| **VeryVerbose** | `-vv`     | + Ansible task details, file paths, results   | Troubleshooting configuration    |
| **Debug**       | `-vvv`    | + Ansible commands executed, playbook outputs | Deep troubleshooting, debugging  |

### CLI Interface

```bash
# Default (Normal) - Essential progress only
torrust-tracker-deployer configure my-env

# Verbose - Show configuration steps
torrust-tracker-deployer configure my-env -v

# Very Verbose - Show Ansible task details
torrust-tracker-deployer configure my-env -vv

# Debug - Show Ansible commands and full output
torrust-tracker-deployer configure my-env -vvv
```

### Example Output Progression

#### Normal Level (Default)

```text
⏳ [2/3] Configuring provisioned infrastructure...
⏳   ✓ Infrastructure configured (took 18.3s)
```

#### Verbose Level (-v)

```text
⏳ [2/3] Configuring provisioned infrastructure...
📋   [Step 1/4] Installing Docker...
📋   [Step 2/4] Installing Docker Compose...
📋   [Step 3/4] Configuring automatic security updates...
📋   [Step 4/4] Configuring firewall (UFW)...
⏳   ✓ Infrastructure configured (took 18.3s)
```

#### VeryVerbose Level (-vv)

```text
⏳ [2/3] Configuring provisioned infrastructure...
📋   [Step 1/4] Installing Docker...
      → Installing Docker Engine from official repository
      → Docker version: 24.0.7
📋   [Step 2/4] Installing Docker Compose...
      → Installing Docker Compose plugin
      → Compose version: 2.23.3
📋   [Step 3/4] Configuring automatic security updates...
      → Configuring unattended-upgrades for automatic security patches
      → Update configuration status: enabled
📋   [Step 4/4] Configuring firewall (UFW)...
      → Configuring UFW with restrictive default policies
      → Allowing SSH access before enabling firewall
      → Firewall status: active
...
```

#### Debug Level (-vvv)

```text
📋   [Step 1/4] Installing Docker...
🔍      → Ansible working directory: ./build/my-env/ansible
🔍      → Executing playbook: ansible-playbook install-docker.yml -i inventory.ini
🔍      → Playbook completed successfully
      → Installing Docker Engine from official repository
      → Docker version: 24.0.7
```

## Configuration Steps Overview

The configure command executes 4 configuration steps via Ansible playbooks:

1. **Install Docker** - Install Docker Engine from official repository
2. **Install Docker Compose** - Install Docker Compose plugin
3. **Configure Automatic Security Updates** - Set up unattended-upgrades for security patches
4. **Configure Firewall (UFW)** - Set up UFW with restrictive policies while maintaining SSH access

Each step should emit:

- Always: `on_step_started()` for step header
- At Normal: Only step headers (already shown by step started)
- At Verbose (-v): Step completion (already handled by success message)
- At VeryVerbose (-vv): `on_detail()` for operation context (versions, ports, status)
- At Debug (-vvv): `on_debug()` for Ansible commands, playbook paths, working directories

## Implementation Plan

### Phase 1: Handler Integration (1-2 hours)

- [ ] Task 1.1: Update `ConfigureCommandHandler::execute()` to accept `Option<&dyn CommandProgressListener>`
- [ ] Task 1.2: Add `on_step_started()` calls for each configuration step in the handler
- [ ] Task 1.3: Pass listener through to individual configuration steps
- [ ] Task 1.4: Update controller to pass listener from `UserOutput` to handler

**Rationale**: Same pattern as provision command. Handler orchestrates steps and emits step progress.

### Phase 2: Step-Level Progress (2-3 hours)

- [ ] Task 2.1: Update configuration step `execute()` methods to accept `listener` parameter
- [ ] Task 2.2: Add `on_detail()` calls in each step:
  - Step 1 (Install Docker): Docker version, installation source
  - Step 2 (Install Docker Compose): Compose version
  - Step 3 (Configure Security Updates): Update configuration status, unattended-upgrades settings
  - Step 4 (Configure Firewall): UFW policies, SSH access preservation, firewall status
- [ ] Task 2.3: Add `on_debug()` calls for Ansible execution:
  - Working directory
  - Playbook command (`ansible-playbook ...`)
  - Playbook execution status
- [ ] Task 2.4: Manual test with `-vv` and `-vvv` flags

**Rationale**: Steps report context around Infrastructure calls (Ansible execution). Infrastructure layer remains opaque per DDD rules.

### Phase 3: Testing and Documentation (1-2 hours)

- [ ] Task 3.1: Manual testing with all verbosity levels (`-v`, `-vv`, `-vvv`)
- [ ] Task 3.2: Verify output readability at all levels
- [ ] Task 3.3: Update user guide documentation (`docs/user-guide/commands/configure.md`)
- [ ] Task 3.4: Verify help text includes verbosity examples (should already be present from global flags)

**Rationale**: Same validation as provision command. Ensure clean, readable output.

**Total Estimated Time**: 4-7 hours

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] No unused dependencies: `cargo machete`
- [ ] All existing tests pass
- [ ] No clippy warnings

**Task-Specific Criteria**:

- [ ] Configure command accepts `-v`, `-vv`, `-vvv` flags (global flags, should already work)
- [ ] Default behavior (no flags) remains unchanged from current output
- [ ] Verbose level (`-v`) shows all 4 configuration steps
- [ ] VeryVerbose level (`-vv`) shows Ansible task details (versions, ports, status)
- [ ] Debug level (`-vvv`) shows Ansible commands and working directories
- [ ] User output stays completely separate from tracing logs
- [ ] `RUST_LOG` continues to control logging independently
- [ ] Help text clearly explains verbosity levels (already present in global help)
- [ ] Output remains clean and readable at all verbosity levels
- [ ] Channel separation maintained (stdout for results, stderr for progress)
- [ ] Pattern matches provision command implementation (consistency)

**Out of Scope**:

- [ ] Quiet mode (`-q`) - defer to future work
- [ ] Silent mode - defer to future work
- [ ] Per-step timing breakdown - not needed, total duration is sufficient

## Related Documentation

- [Roadmap Section 8 - Add levels of verbosity](../roadmap.md#8-add-levels-of-verbosity)
- [Task 8.1 - Provision Command Verbosity (Reference Implementation)](./drafts/add-verbosity-levels-to-provision-command.md)
- [PR #361 - Add verbosity levels to provision command](https://github.com/torrust/torrust-tracker-deployer/pull/361)
- [UX Research - Console Output & Logging Strategy](../research/UX/console-output-logging-strategy.md)
- [UX Research - User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md)
- [Contributing - Output Handling](../contributing/output-handling.md)
- [Development Principles - Observability](../development-principles.md)
- [Generic Command Progress Listener ADR](./drafts/generic-command-progress-listener-for-verbosity.md)
- [User Guide - Configure Command](../user-guide/commands/configure.md)

## Notes

### Design Decisions

1. **Why reuse CommandProgressListener?**
   - Already proven pattern from provision command
   - Consistent user experience across commands
   - No need to reinvent the wheel
   - Makes future command verbosity additions trivial

2. **Why not pass listener to Infrastructure layer?**
   - Violates DDD dependency rules (Infrastructure shouldn't depend on Application)
   - Steps can report context around Infrastructure calls
   - Pragmatic approach: report before/after Infrastructure operations

3. **Why same verbosity levels and flags?**
   - Users expect consistency across commands
   - Already documented in global help text
   - Reduces cognitive load

4. **Why focus on configure command?**
   - One of the three most complex, time-consuming commands
   - Ansible operations can take 15-30 seconds
   - Users benefit from progress visibility during long operations

### Reference Implementation

The provision command implementation (task 8.1, PR #361) serves as the reference for this task:

- Same `CommandProgressListener` trait
- Same verbosity levels and symbols (⏳ ✅ 📋 🔍)
- Same handler → steps → listener flow
- Same documentation patterns

Review that implementation for architectural patterns, code structure, and documentation examples.
