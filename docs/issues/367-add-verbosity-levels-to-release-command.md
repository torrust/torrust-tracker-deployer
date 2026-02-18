# Add Verbosity Levels to Release Command

**Issue**: #367
**Parent Epic**: #362 - Add levels of verbosity (Roadmap Section 8)
**Related**:

- [Roadmap Section 8](../roadmap.md#8-add-levels-of-verbosity)
- [Task 8.1 - Provision Command Verbosity (Completed)](./drafts/add-verbosity-levels-to-provision-command.md) — reference implementation
- [Task 8.2 - Configure Command Verbosity (Completed)](./363-add-verbosity-levels-to-configure-command.md) — reference implementation
- [UX Research - Console Output & Logging Strategy](../research/UX/console-output-logging-strategy.md)
- [UX Research - User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md)
- [Generic Command Progress Listener for Verbosity](./drafts/generic-command-progress-listener-for-verbosity.md) — architectural design for the `CommandProgressListener` trait
- [Progress Reporting in Application Layer](../features/progress-reporting-in-application-layer/README.md)

**Status**: 🔜 **NOT STARTED** — Ready for implementation

**Branch**: TBD
**PR**: TBD

## Current Implementation Status

**Pending Implementation**:

- [ ] **Normal (default)**: Show 2 main workflow phases (validate + release)
- [ ] **Verbose (`-v`)**: Show all 7 service-specific release steps
- [ ] **VeryVerbose (`-vv`)**: Show detail messages (files deployed, templates rendered, paths)
- [ ] **Debug (`-vvv`)**: Show debug messages (Ansible commands, working directories, full outputs)
- [ ] **Documentation**: User guide update with verbosity examples and patterns

## Overview

Add graduated verbosity levels (`-v`, `-vv`, `-vvv`) to the `release` command to give users control over the amount of user-facing progress detail displayed. This applies the same verbosity pattern established in the provision and configure commands (tasks 8.1 and 8.2) to the release command, which handles software deployment via template rendering and file transfers.

**Important**: This feature controls **only user-facing output** (via `UserOutput`), not internal logging (which remains controlled by `RUST_LOG` environment variable). The two systems are intentionally kept separate as documented in the UX research.

## Goals

- [ ] Reuse `CommandProgressListener` infrastructure from provision/configure commands
- [ ] Apply same four verbosity levels to release command
- [ ] Show service-specific release steps, template rendering, file deployments at different detail levels
- [ ] Maintain backward compatibility (default = Normal level)
- [ ] Keep user output completely separate from tracing logs
- [ ] Update user documentation with verbosity examples
- [ ] Help text examples already present (global flags)

## 🏗️ Architecture Requirements

**DDD Layer**: Application (`src/application/command_handlers/release/`)

**Module Paths**:

- Command handler: `src/application/command_handlers/release/handler.rs`
- Release workflow: `src/application/command_handlers/release/workflow.rs`
- Service steps: `src/application/command_handlers/release/steps/` (tracker, prometheus, grafana, mysql, backup, caddy, compose)
- Progress listener: `src/application/traits/command_progress_listener.rs` (already exists)
- CLI integration: `src/presentation/controllers/release/handler.rs`

**Pattern**: Same as provision/configure commands - use `CommandProgressListener` trait for progress reporting

### Existing Infrastructure (from Tasks 8.1 & 8.2)

The following components **already exist** from previous verbosity implementations:

- ✅ `CommandProgressListener` trait in `src/application/traits/command_progress_listener.rs`
  - `on_step_started(step_number, total_steps, step_name)`
  - `on_detail(message)` for contextual details
  - `on_debug(message)` for technical implementation details
  - `on_success(operation_name, duration)`
- ✅ `VerboseProgressListener` in `src/presentation/views/messages/verbose_progress_listener.rs`
  - Converts progress events to user-facing messages based on verbosity level
- ✅ CLI verbosity flags already wired through `ExecutionContext`
- ✅ Four verbosity levels: Normal, Verbose (-v), VeryVerbose (-vv), Debug (-vvv)

**What's needed for release command**:

- [ ] Pass `CommandProgressListener` to release command handler
- [ ] Add `on_step_started()` calls for all 7 service-specific release steps
- [ ] Add `on_detail()` calls for template rendering, file deployments, and service operations
- [ ] Add `on_debug()` calls for Ansible commands, working directories, full outputs
- [ ] Update user guide documentation
- [ ] Verify help text shows verbosity options (already present from global flags)

### Module Structure Requirements

- [ ] Follow same pattern as provision/configure commands (reference implementations)
- [ ] Handler emits step-level progress events
- [ ] Workflow coordinates service steps and emits progress for each service
- [ ] Service steps emit detail and debug events
- [ ] Infrastructure layer (Ansible, template rendering) remains unaware of listener

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
- ❌ **Don't** duplicate the pattern - reuse what provision/configure commands established
- ❌ **Don't** change the verbosity flag names or meanings

## Specifications

### Verbosity Level Behaviors

Same four levels as provision and configure commands:

| Level           | Flag      | User Output Behavior                                          | Use Case                         |
| --------------- | --------- | ------------------------------------------------------------- | -------------------------------- |
| **Normal**      | (default) | Essential progress and results                                | Regular users, normal operations |
| **Verbose**     | `-v`      | + Service-specific release steps (tracker, prometheus, etc.)  | Users wanting more visibility    |
| **VeryVerbose** | `-vv`     | + Template rendering, file paths, deployment results          | Troubleshooting release issues   |
| **Debug**       | `-vvv`    | + Ansible commands executed, working directories, full output | Deep troubleshooting, debugging  |

### CLI Interface

```bash
# Default (Normal) - Essential progress only
torrust-tracker-deployer release my-env

# Verbose - Show service-specific release steps
torrust-tracker-deployer release my-env -v

# Very Verbose - Show template rendering and file deployment details
torrust-tracker-deployer release my-env -vv

# Debug - Show Ansible commands and full output
torrust-tracker-deployer release my-env -vvv
```

### Example Output Progression

#### Normal Level (Default)

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: my-env (took 0ms)
⏳ [2/2] Releasing application...
⏳   ✓ Application released (took 45.8s)
✅ Environment 'my-env' released successfully
```

**Behavior**: Shows only the 2 main presentation-layer workflow phases. No visibility into which services are being released or what files are being deployed.

#### Verbose Level (`-v`)

```text
⏳ [2/2] Releasing application...
📋   [Step 1/7] Releasing Tracker service...
📋   [Step 2/7] Releasing Prometheus service...
📋   [Step 3/7] Releasing Grafana service...
📋   [Step 4/7] Releasing MySQL service...
📋   [Step 5/7] Releasing Backup service...
📋   [Step 6/7] Releasing Caddy service...
📋   [Step 7/7] Deploying Docker Compose configuration...
⏳   ✓ Application released (took 43.2s)
```

**Behavior**: Shows all 7 service-specific release steps. User can see which service is being processed but not the detailed operations. The 📋 icon indicates detail-level messages (VeryVerbose).

**Service Steps**:

1. Tracker - storage creation, database init, config rendering/deployment
2. Prometheus - config rendering/deployment
3. Grafana - dashboard provisioning, config deployment
4. MySQL - database setup (if enabled)
5. Backup - backup container config deployment (if enabled)
6. Caddy - reverse proxy config deployment (if HTTPS enabled)
7. Docker Compose - compose file rendering and deployment

#### VeryVerbose Level (`-vv`)

```text
📋   [Step 1/7] Releasing Tracker service...
📋      → Creating storage directories: /opt/torrust/storage/tracker/{lib,log,etc}
📋      → Initializing database: tracker.db
📋      → Rendering tracker.toml from template
📋      → Deploying config to /opt/torrust/storage/tracker/etc/tracker.toml
📋   [Step 2/7] Releasing Prometheus service...
📋      → Rendering prometheus.yml from template
📋      → Deploying config to /opt/torrust/storage/prometheus/etc/prometheus.yml
📋   [Step 3/7] Releasing Grafana service...
📋      → Rendering dashboards from templates
📋      → Deploying 2 dashboard files
📋      → Deploying datasource configuration
📋   [Step 7/7] Deploying Docker Compose configuration...
📋      → Rendering docker-compose.yml (7 services enabled)
📋      → Rendering .env file (12 environment variables)
📋      → Deploying to /opt/torrust/docker-compose.yml
```

**Behavior**: Shows detailed context for each service release:

- File paths where artifacts are deployed
- Number of files/resources being processed
- Template rendering operations
- Storage directory creation
- Database initialization details

**Message Categories**:

- Template rendering operations
- File deployment paths
- Directory creation
- Service-specific operations (DB init, dashboard provisioning)
- Resource counts (files, variables, services)

#### Debug Level (`-vvv`)

```text
📋   [Step 1/7] Releasing Tracker service...
🔍      → Ansible working directory: ./build/my-env/ansible
🔍      → Executing playbook: ansible-playbook create-tracker-storage.yml -i inventory.ini
📋      → Creating storage directories: /opt/torrust/storage/tracker/{lib,log,etc}
🔍      → Executing playbook: ansible-playbook init-tracker-database.yml -i inventory.ini
📋      → Initializing database: tracker.db
📋      → Rendering tracker.toml from template
📋      → Template source: ./templates/tracker/tracker.toml.tera
📋      → Template output: ./build/my-env/tracker/tracker.toml
🔍      → Executing playbook: ansible-playbook deploy-tracker-config.yml -i inventory.ini
📋      → Deploying config to /opt/torrust/storage/tracker/etc/tracker.toml
```

**Behavior**: Shows technical implementation details:

- Ansible commands being executed
- Working directories for operations
- Playbook names
- Source and destination paths for all operations
- Full command-line invocations

**Additional Debug Information**:

- Ansible playbook execution commands
- Working directory paths
- Template source paths (before rendering)
- Template output paths (after rendering)
- Remote deployment paths

### Symbol Legend

| Symbol | Level       | Meaning                          |
| ------ | ----------- | -------------------------------- |
| ⏳     | Normal+     | Operation in progress            |
| ✅     | Normal+     | Operation completed successfully |
| 📋     | VeryVerbose | Detailed contextual information  |
| 🔍     | Debug       | Technical implementation detail  |

## 📋 Implementation Plan

### Phase 1: Handler-Level Verbosity (Core Integration)

**Goal**: Enable handler to accept listener and emit progress for the 7 service-specific release steps

**Files to Modify**:

- `src/application/command_handlers/release/handler.rs`
- `src/application/command_handlers/release/workflow.rs`
- `src/presentation/controllers/release/handler.rs`
- `src/testing/e2e/tasks/run_release_command.rs` (E2E test compatibility)

**Tasks**:

- [ ] Task 1.1: Update `ReleaseCommandHandler::execute()` to accept optional `&dyn CommandProgressListener`
- [ ] Task 1.2: Add `TOTAL_RELEASE_STEPS` constant (7 services)
- [ ] Task 1.3: Add `on_step_started()` calls before each service release in `workflow::execute()`
- [ ] Task 1.4: Pass listener from controller to handler
- [ ] Task 1.5: Update E2E tests to pass `None` for listener (backward compatibility)
- [ ] Task 1.6: Test Normal and Verbose levels work correctly

**Rationale**: Matches the pattern established in provision and configure commands. Handler orchestrates high-level workflow, emitting progress for each major service step.

**Estimated Time**: 3-4 hours

### Phase 2: Step-Level Verbosity (Detail and Debug Messages)

**Goal**: Add detail and debug progress reporting to service-specific release steps

**Files to Modify**:

- `src/application/command_handlers/release/steps/tracker.rs`
- `src/application/command_handlers/release/steps/prometheus.rs`
- `src/application/command_handlers/release/steps/grafana.rs`
- `src/application/command_handlers/release/steps/mysql.rs`
- `src/application/command_handlers/release/steps/backup.rs`
- `src/application/command_handlers/release/steps/caddy.rs`
- `src/application/command_handlers/release/steps/compose.rs`
- `src/application/command_handlers/release/workflow.rs`

**Tasks**:

- [ ] Task 2.1: Update all 7 service release functions to accept optional `&dyn CommandProgressListener`
- [ ] Task 2.2: Add `on_detail()` calls for:
  - Storage directory creation
  - Database initialization
  - Template rendering operations
  - File deployment paths
  - Service-specific operations
- [ ] Task 2.3: Add `on_debug()` calls for:
  - Ansible command execution
  - Working directories
  - Template source/output paths
  - Playbook names
- [ ] Task 2.4: Pass listener from workflow to all service steps
- [ ] Task 2.5: Test VeryVerbose and Debug levels work correctly

**Rationale**: Service steps are the natural place to report detailed progress. They know what files are being rendered, where they're deployed, and what Ansible commands execute. Following the same pattern as configure command.

**Estimated Time**: 5-7 hours (7 service modules need updates)

### Phase 3: Documentation and Validation

**Goal**: Document the verbosity feature in user-facing documentation

**Files to Modify**:

- `docs/user-guide/commands/release.md`
- `docs/issues/365-add-verbosity-levels-to-release-command.md` (this file)

**Tasks**:

- [ ] Task 3.1: Add "Verbosity Levels" section to release.md
- [ ] Task 3.2: Include examples for all 4 verbosity levels (Normal, Verbose, VeryVerbose, Debug)
- [ ] Task 3.3: Document use cases for each level
- [ ] Task 3.4: Add examples showing combined usage with other flags
- [ ] Task 3.5: Verify help text includes verbosity examples (already present from global flags)
- [ ] Task 3.6: Update this issue spec with implementation status

**Rationale**: Same validation as provision/configure commands. Ensure clean, readable output and comprehensive documentation.

**Estimated Time**: 2-3 hours

**Total Estimated Time**: 10-14 hours

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] No unused dependencies: `cargo machete`
- [ ] All existing tests pass
- [ ] No clippy warnings

**Task-Specific Criteria**:

- [ ] Release command accepts `-v`, `-vv`, `-vvv` flags (global flags, already works)
- [ ] Default behavior (no flags) remains unchanged from current output
- [ ] Verbose level (`-v`) shows all 7 service-specific release steps
- [ ] VeryVerbose level (`-vv`) shows template rendering, file deployments, storage operations
- [ ] Debug level (`-vvv`) shows Ansible commands, working directories, full paths
- [ ] User output stays completely separate from tracing logs
- [ ] `RUST_LOG` continues to control logging independently
- [ ] Help text clearly explains verbosity levels (verified in global flags)
- [ ] Output remains clean and readable at all verbosity levels
- [ ] Channel separation maintained (stdout for results, stderr for progress)
- [ ] Pattern matches provision/configure command implementations (consistency)

**Documentation Criteria**:

- [ ] User guide (`docs/user-guide/commands/release.md`) updated with verbosity section
- [ ] Examples provided for all 4 verbosity levels
- [ ] Symbol legend explained (⏳ ✅ 📋 🔍)
- [ ] Use cases documented for each verbosity level
- [ ] Combined flag usage examples provided

**Out of Scope**:

- [ ] Quiet mode (`-q`) - defer to future work
- [ ] Silent mode - defer to future work
- [ ] Per-step timing breakdown - not needed, total duration is sufficient

## Related Documentation

- [Roadmap Section 8 - Add levels of verbosity](../roadmap.md#8-add-levels-of-verbosity)
- [Task 8.1 - Provision Command Verbosity (Reference Implementation)](./drafts/add-verbosity-levels-to-provision-command.md)
- [Task 8.2 - Configure Command Verbosity (Reference Implementation)](./363-add-verbosity-levels-to-configure-command.md)
- [PR #361 - Add verbosity levels to provision command](https://github.com/torrust/torrust-tracker-deployer/pull/361)
- [PR #364 - Add verbosity levels to configure command](https://github.com/torrust/torrust-tracker-deployer/pull/364)
- [UX Research - Console Output & Logging Strategy](../research/UX/console-output-logging-strategy.md)
- [UX Research - User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md)
- [Contributing - Output Handling](../contributing/output-handling.md)
- [Development Principles - Observability](../development-principles.md)
- [Generic Command Progress Listener ADR](./drafts/generic-command-progress-listener-for-verbosity.md)
- [User Guide - Release Command](../user-guide/commands/release.md)

## Notes

### Design Decisions

1. **Why reuse CommandProgressListener?**
   - Already proven pattern from provision and configure commands
   - Consistent user experience across all commands
   - No need to reinvent the wheel
   - Makes the pattern truly reusable across the codebase

2. **Why not pass listener to Infrastructure layer?**
   - Violates DDD dependency rules (Infrastructure shouldn't depend on Application)
   - Service steps can report context around Infrastructure calls
   - Pragmatic approach: report before/after Infrastructure operations

3. **Why same verbosity levels and flags?**
   - Users expect consistency across commands
   - Already documented in global help text
   - Reduces cognitive load
   - Provides predictable user experience

4. **Why focus on release command?**
   - One of the three most complex, time-consuming commands
   - Multiple service deployments can take 30-60 seconds
   - Template rendering and file transfers benefit from progress visibility
   - Users need to know which service is being processed

5. **Why 7 service steps instead of grouping?**
   - Each service is a distinct deployment operation
   - Users want to know which specific service is being processed
   - Troubleshooting requires knowing which service failed
   - Matches the actual implementation in `workflow::execute()`

### Release Workflow Details

The release command executes 7 service-specific release operations:

1. **Tracker** (`tracker::release`):
   - Create storage directories
   - Initialize database
   - Render tracker.toml template
   - Deploy configuration to remote

2. **Prometheus** (`prometheus::release`):
   - Render prometheus.yml template
   - Deploy configuration to remote

3. **Grafana** (`grafana::release`):
   - Render dashboard templates
   - Render datasource configuration
   - Deploy dashboards and provisioning to remote

4. **MySQL** (`mysql::release`):
   - Conditional: only if database type is MySQL
   - Database setup operations

5. **Backup** (`backup::release`):
   - Conditional: only if backup is enabled
   - Deploy backup container configuration

6. **Caddy** (`caddy::release`):
   - Conditional: only if HTTPS is enabled
   - Render Caddyfile template
   - Deploy reverse proxy configuration

7. **Docker Compose** (`compose::release`):
   - Render docker-compose.yml template
   - Render .env file
   - Deploy compose files to remote

### Reference Implementations

The provision command (task 8.1, PR #361) and configure command (task 8.2, PR #364) serve as references:

- Same `CommandProgressListener` trait
- Same verbosity levels and symbols (⏳ ✅ 📋 🔍)
- Same handler → workflow/steps → listener flow
- Same documentation patterns

Review those implementations for architectural patterns, code structure, and documentation examples.

### Testing Strategy

- **Unit tests**: Not required - verbosity is presentation concern
- **E2E tests**: Update existing E2E tests to pass `None` for listener (backward compatibility)
- **Manual testing**: Test all 4 verbosity levels with real release operations
- **Verification**: Compare output against documented examples in user guide
