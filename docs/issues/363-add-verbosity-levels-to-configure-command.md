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

**Status**: 🔄 **IN PROGRESS** — Phase 1 and 2 complete (handler + step-level verbosity), Phase 3 pending (documentation)

**Latest Commits**:

- `666ca363` - "feat: [#363] add verbosity levels to configure command" (Phase 1)
- `206d6137` - "feat: [#363] add step-level verbosity (VeryVerbose and Debug levels)" (Phase 2)

**Branch**: `363-add-verbosity-configure`
**PR**: [#TBD](https://github.com/torrust/torrust-tracker-deployer/pull/TBD) _(not yet created)_

## Current Implementation Status

**What Works** (commit `206d6137`):

- ✅ **Normal (default)**: Shows 3 main workflow phases
- ✅ **Verbose (`-v`)**: Shows all 4 configuration steps (Docker, Docker Compose, Security Updates, Firewall)
- ✅ **VeryVerbose (`-vv`)**: Shows detail messages (versions, status, configurations) with 📋 icon
- ✅ **Debug (`-vvv`)**: Shows debug messages (Ansible commands, working directories) with 🔍 icon

**What's Missing**:

- ❌ **Documentation**: User guide not updated with verbosity examples (Phase 3 pending)

**Next Steps**: Complete Phase 3 (update documentation with verbosity examples and verify help text).

## Overview

Add graduated verbosity levels (`-v`, `-vv`, `-vvv`) to the `configure` command to give users control over the amount of user-facing progress detail displayed. This applies the same verbosity pattern established in the provision command (task 8.1) to the configure command, which handles system configuration via Ansible playbooks.

**Important**: This feature controls **only user-facing output** (via `UserOutput`), not internal logging (which remains controlled by `RUST_LOG` environment variable). The two systems are intentionally kept separate as documented in the UX research.

## Goals

- [x] Reuse `CommandProgressListener` infrastructure from provision command ✅ **DONE**
- [x] Apply same four verbosity levels to configure command ✅ **DONE**
- [x] Show configuration steps, Ansible operations, and system changes at different detail levels ✅ **DONE**
- [x] Maintain backward compatibility (default = Normal level) ✅ **DONE**
- [x] Keep user output completely separate from tracing logs ✅ **DONE**
- [ ] Update user documentation with verbosity examples ❌ **PENDING** (Phase 3)
- [ ] Add help text examples to configure command ❌ **PENDING** (Phase 3)

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

- [x] Pass `CommandProgressListener` to configure command handler ✅ **DONE**
- [x] Add `on_step_started()` calls for configuration steps ✅ **DONE**
- [x] Add `on_detail()` calls for Ansible operation context ✅ **DONE**
- [x] Add `on_debug()` calls for technical Ansible details ✅ **DONE**
- [ ] Update user guide documentation ❌ **PENDING**
- [ ] Verify help text shows verbosity options ❌ **PENDING**

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
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: e2e-deployment (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
⏳   ✓ Infrastructure configured (took 34.1s)
✅ Environment 'e2e-deployment' configured successfully
```

#### Verbose Level (-v)

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: e2e-deployment (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
📋   [Step 1/4] Installing Docker...
📋   [Step 2/4] Installing Docker Compose...
📋   [Step 3/4] Configuring automatic security updates...
📋   [Step 4/4] Configuring firewall (UFW)...
⏳   ✓ Infrastructure configured (took 34.1s)
✅ Environment 'e2e-deployment' configured successfully
```

#### VeryVerbose Level (-vv)

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: e2e-deployment (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
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
⏳   ✓ Infrastructure configured (took 34.1s)
✅ Environment 'e2e-deployment' configured successfully
...
```

#### Debug Level (-vvv)

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: e2e-deployment (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
📋   [Step 1/4] Installing Docker...
🔍      → Ansible working directory: ./build/e2e-deployment/ansible
🔍      → Executing playbook: ansible-playbook install-docker.yml -i inventory.ini
🔍      → Playbook completed successfully
      → Installing Docker Engine from official repository
      → Docker version: 24.0.7
📋   [Step 2/4] Installing Docker Compose...
🔍      → Ansible working directory: ./build/e2e-deployment/ansible
🔍      → Executing playbook: ansible-playbook install-docker-compose.yml -i inventory.ini
🔍      → Playbook completed successfully
      → Installing Docker Compose plugin
      → Compose version: 2.23.3
...
⏳   ✓ Infrastructure configured (took 34.1s)
✅ Environment 'e2e-deployment' configured successfully
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

> **Implementation Status**: 🟡 **MOSTLY COMPLETE** (Phase 1 and 2 done, Phase 3 pending)
>
> **Current State** (commit `206d6137`):
>
> - ✅ **Phase 1 Complete**: Handler accepts listener, reports 4 configuration steps at Verbose (-v) level
> - ✅ **Phase 2 Complete**: Steps accept listener and emit detail/debug messages for VeryVerbose (-vv) and Debug (-vvv)
> - ❌ **Phase 3 Pending**: Documentation not yet updated
>
> **What works now**:
>
> - Normal (default): Shows 3 main phases (validate, create handler, configure)
> - Verbose (`-v`): Shows 4 configuration steps (Docker, Docker Compose, Security Updates, Firewall)
> - VeryVerbose (`-vv`): Shows detail messages (versions, status, configurations)
> - Debug (`-vvv`): Shows debug messages (Ansible commands, working directories)
>
> **What doesn't work yet**:
>
> - Documentation: User guide not updated with verbosity examples

### Phase 1: Handler Integration (1-2 hours) ✅ **COMPLETE**

- [x] Task 1.1: Update `ConfigureCommandHandler::execute()` to accept `Option<&dyn CommandProgressListener>`
- [x] Task 1.2: Add `on_step_started()` calls for each configuration step in the handler
- [x] Task 1.3: Pass listener through to individual configuration steps ✅ **COMPLETED in Phase 2**
- [x] Task 1.4: Update controller to pass listener from `UserOutput` to handler

**Rationale**: Same pattern as provision command. Handler orchestrates steps and emits step progress.

**Completed in commit**: `666ca363` - "feat: [#363] add verbosity levels to configure command"

### Phase 2: Step-Level Progress (2-3 hours) ✅ **COMPLETE**

- [x] Task 2.1: Update configuration step `execute()` methods to accept `listener` parameter
- [x] Task 2.2: Add `on_detail()` calls in each step:
  - Step 1 (Install Docker): Docker version, installation source
  - Step 2 (Install Docker Compose): Compose version
  - Step 3 (Configure Security Updates): Update configuration status, unattended-upgrades settings
  - Step 4 (Configure Firewall): UFW policies, SSH access preservation, firewall status
- [x] Task 2.3: Add `on_debug()` calls for Ansible execution:
  - Working directory
  - Playbook command (`ansible-playbook ...`)
  - Playbook execution status
- [x] Task 2.4: Manual test with `-vv` and `-vvv` flags

**Rationale**: Steps report context around Infrastructure calls (Ansible execution). Infrastructure layer remains opaque per DDD rules.

**Completed in commit**: `206d6137` - "feat: [#363] add step-level verbosity (VeryVerbose and Debug levels)"

**Files Modified**:

- `src/application/command_handlers/configure/handler.rs` - Pass listener to steps
- `src/application/steps/software/docker.rs` - Accept listener, emit detail/debug
- `src/application/steps/software/docker_compose.rs` - Accept listener, emit detail/debug
- `src/application/steps/system/configure_security_updates.rs` - Accept listener, emit detail/debug
- `src/application/steps/system/configure_firewall.rs` - Accept listener, emit detail/debug

### Phase 3: Testing and Documentation (1-2 hours) ⚠️ **IN PROGRESS**

- [x] Task 3.1: Manual testing with all verbosity levels (`-v`, `-vv`, `-vvv`)
- [x] Task 3.2: Verify output readability at all levels
- [ ] Task 3.3: Update user guide documentation (`docs/user-guide/commands/configure.md`)
- [ ] Task 3.4: Verify help text includes verbosity examples (should already be present from global flags)

**Rationale**: Same validation as provision command. Ensure clean, readable output.

**Status**: Implementation and testing complete. Documentation updates pending.

**Total Estimated Time**: 4-7 hours (4-5 hours completed, 1-2 hours remaining)

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.
>
> **Current Status** (commit `206d6137`): ✅ 18/22 criteria complete (82%)

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [x] No unused dependencies: `cargo machete`
- [x] All existing tests pass
- [x] No clippy warnings

**Task-Specific Criteria**:

- [x] Configure command accepts `-v`, `-vv`, `-vvv` flags (global flags, already works)
- [x] Default behavior (no flags) remains unchanged from current output
- [x] Verbose level (`-v`) shows all 4 configuration steps
- [x] VeryVerbose level (`-vv`) shows Ansible task details (versions, ports, status)
- [x] Debug level (`-vvv`) shows Ansible commands and working directories
- [x] User output stays completely separate from tracing logs
- [x] `RUST_LOG` continues to control logging independently
- [ ] Help text clearly explains verbosity levels (already present in global help) - **NOT VERIFIED**
- [x] Output remains clean and readable at all verbosity levels
- [x] Channel separation maintained (stdout for results, stderr for progress)
- [x] Pattern matches provision command implementation (consistency)

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
