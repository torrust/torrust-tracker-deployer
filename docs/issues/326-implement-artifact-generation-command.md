# Implement Artifact Generation Command

**Issue**: #326
**Parent Epic**: #1 - Roadmap
**Related**:

- Roadmap task: 9.2 - Implement artifact generation command
- `validate` command: #TBD (similar read-only, no deployment pattern)
- Template system architecture: `docs/contributing/templates/template-system-architecture.md`
- Release command: `src/application/command_handlers/release/`

## Overview

Add a command that generates all deployment artifacts (docker-compose files, tracker configuration, Ansible playbooks, Caddy configuration, etc.) to the `build/` directory without executing any deployment operations. This enables users who want the generated configuration files but prefer to handle deployment manually.

This complements the `validate` command (which validates input without generating artifacts) and the `register` command concept (for pre-provisioned instances). Together, these three commands form a "lightweight adoption" trilogy:

- **`validate`** - Check configuration without side effects (already implemented)
- **`render`** (or similar) - Generate artifacts without deployment (this task)
- **`register`** - Use deployer with existing infrastructure (future)

## Goals

- [ ] Generate all deployment artifacts without executing deployment commands
- [x] **Reuse existing template rendering infrastructure from release/configure commands** ✅ (Phase 0 complete - 8 rendering services extracted)
- [ ] Support all artifact types: docker-compose, tracker config, Ansible playbooks, Caddy, monitoring
- [ ] Provide clear output showing what was generated and where
- [ ] Enable "inspect before deploy" workflow for cautious administrators
- [ ] Make the tool more AI-agent friendly (dry-run artifact inspection)

## 🏗️ Architecture Requirements

**DDD Layers**: Application + Infrastructure (template rendering)

**Module Paths**:

- `src/application/command_handlers/render/` (or `generate`) - Command handler (new)
- `src/presentation/controllers/render/` - CLI controller (new)
- `src/infrastructure/external_tools/ansible/template/renderer/` - Reuse existing
- `src/infrastructure/external_tools/docker/template/renderer/` - Reuse existing
- `src/infrastructure/external_tools/tofu/template/renderer/` - Maybe reuse (OpenTofu)

**Pattern**: Command Handler + Template Renderers (reuse from Release/Configure)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Application layer orchestrates template generation (no business logic in CLI)
- [ ] Infrastructure layer handles actual template rendering (Tera, file I/O)
- [ ] No remote operations (pure local artifact generation)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Must reuse existing template renderers (no duplication)
- [ ] No state transitions - operates on any environment state
- [ ] No remote operations - purely local file generation
- [ ] Read-only access to environment data (`data/{env}/environment.json`)
- [ ] Output to `build/{env}/` directory (standard build output location)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] User output through `UserOutput` abstraction (see [docs/contributing/output-handling.md](../contributing/output-handling.md))

### Anti-Patterns to Avoid

- ❌ Duplicating template rendering logic
- ❌ Executing remote operations (Ansible, SSH, etc.)
- ❌ Modifying environment state
- ❌ Requiring specific environment state (must work from any state)
- ❌ Hardcoding build output paths

## Specifications

### Command Name Decision

**Candidates**: `render`, `generate`, `export`, `prepare`, `scaffold`

**Recommendation**: **`render`**

**Rationale**:

| Candidate  | Pros                                | Cons                                       |
| ---------- | ----------------------------------- | ------------------------------------------ |
| `render`   | Technical accuracy (Tera rendering) | May sound overly technical to some         |
| `generate` | Clear intent, common in CLI tools   | Generic, could mean many things            |
| `export`   | Implies taking something out        | Suggests data export rather than templates |
| `prepare`  | Clear staging intent                | Vague about what preparation means         |
| `scaffold` | Common in Rails/web frameworks      | Implies new project structure creation     |

**`render`** aligns with:

- Internal architecture (Tera template renderer)
- Technical audience (system administrators)
- Precision (we're rendering templates, not exporting data)
- Project terminology already in code (`ProjectGenerator`, `TemplateRenderer`)

**Alternative**: If `render` is too technical, `generate` is the next best choice.

### Artifacts to Generate

**Important**: ALL templates are ALWAYS rendered, regardless of environment configuration or whether they're needed for the specific deployment. This is the current implementation approach:

- Simplifies the rendering logic (no conditional checks needed)
- User gets a complete set of artifacts and decides what to use
- Templates that reference optional services (MySQL, HTTPS, etc.) are still generated
- May change in the future if template dependencies grow or conditional rendering becomes necessary

**Note on Dynamic Values**:

- Instance IP is the ONLY value obtained dynamically (from `provision` command or `--ip` flag)
- All other template values come from user input in the environment configuration
- No inter-template dependencies currently exist (each template is self-contained)

Generated artifacts:

#### 1. Infrastructure Templates (OpenTofu)

**Source**: `templates/tofu/`
**Output**: `<output-dir>/tofu/`
**Generated by**: `TofuTemplateRenderer`

Files:

- `main.tf` - Infrastructure definition
- `variables.tf` - Input variables
- `outputs.tf` - Infrastructure outputs

**Note**: These are useful even after provisioning (for documentation, drift detection, or reprovisioning).

#### 2. Configuration Management (Ansible)

**Source**: `templates/ansible/`
**Output**: `<output-dir>/ansible/`
**Generated by**: `AnsibleProjectGenerator`

Files:

- `playbooks/*.yml` - All configuration playbooks
- `inventory/hosts.ini` - Target hosts
- `ansible.cfg` - Ansible configuration
- `group_vars/all.yml` - Variables

Playbooks include:

- System updates
- Security hardening (UFW)
- Docker installation
- SSH key setup
- User management

#### 3. Docker Compose Stack

**Source**: `templates/docker-compose/`
**Output**: `<output-dir>/docker-compose/`
**Generated by**: `DockerComposeRenderer`

Files:

- `docker-compose.yml` - Complete stack definition
- `.env` - Environment variables for services

Services included (based on config):

- Tracker (always)
- MySQL (if configured)
- Prometheus (if configured)
- Grafana (if configured)
- Caddy (if HTTPS configured)
- Backup (if backup configured)

#### 4. Tracker Configuration

**Source**: `templates/tracker/`
**Output**: `<output-dir>/tracker/`
**Generated by**: Part of release template rendering

Files:

- `tracker.toml` - Tracker configuration
- Includes database, API, HTTP/UDP settings

#### 5. Monitoring Configuration

**Source**: `templates/prometheus/`, `templates/grafana/`
**Output**: `<output-dir>/prometheus/`, `<output-dir>/grafana/`
**Generated by**: Monitoring template renderers

Files:

- `prometheus.yml` - Prometheus configuration
- `grafana/dashboards/*.json` - Grafana dashboards
- `grafana/datasources/*.yml` - Data sources

#### 6. Reverse Proxy Configuration

**Source**: `templates/caddy/`
**Output**: `<output-dir>/caddy/`
**Generated by**: Caddy template renderer

Files:

- `Caddyfile` - Caddy configuration with TLS
- HTTPS configuration for all HTTP services

#### 7. Backup Configuration

**Source**: `templates/backup/`
**Output**: `<output-dir>/backup/`
**Generated by**: Backup template renderer

Files:

- `backup.sh` - Backup script
- `Dockerfile` - Backup container definition
- Crontab configuration

### Command Interface

**Critical Design Rule**: The `render` command is ONLY for environments in the **Created** state (initial state before provisioning). Once an environment is provisioned, artifacts already exist in `build/{env}/` and should not be regenerated.

```bash
# Mode 1: Generate from existing Created environment
torrust-tracker-deployer render --env-name <name> --ip <ip-address>

# Mode 2: Generate from config file (no environment creation)
torrust-tracker-deployer render --env-file <path> --ip <ip-address>
```

**Design Decisions**:

1. **IP address is ALWAYS REQUIRED** (via `--ip` flag)
   - **Rationale**: User must explicitly specify target infrastructure IP
   - Even when using `--env-name`, IP is required (environment hasn't been provisioned yet)
   - Ansible inventory template requires IP address for all templates

2. **Only works with "Created" state environments** (when using `--env-name`)
   - **Created state**: Initial state after `create environment`, before `provision`
   - **Provisioned+**: Artifacts already exist in `build/{env}/`, just show location message
   - **Use case**: "Generate artifacts to preview before provisioning"

3. **Artifacts written to `build/{env}/`** (standard build directory)
   - Consistent with rest of deployer (provision, configure, release all write here)
   - No `--output-dir` flag needed (always uses standard location)
   - User can copy from `build/{env}/` if needed for manual deployment

**Supported Input Modes**:

1. **`--env-name` + `--ip`** (existing Created environment):
   - Use case: "I created an environment, generate artifacts before provisioning"
   - Use case: "Preview what will be deployed to this IP"
   - Reads from: `data/{env}/environment.json` (environment configuration)
   - State: Must be in "Created" state (will show message if already provisioned)
   - IP: User-provided via `--ip` flag

2. **`--env-file` + `--ip`** (preview mode without environment):
   - Use case: "Generate artifacts from this config before creating environment"
   - Use case: "I want artifacts only, no deployer state management"
   - Reads from: User-provided config file (e.g., `envs/my-config.json`)
   - State: No environment created
   - IP: User-provided via `--ip` flag

### Output Format

**Simplified Output** - Only show build directory path and target IP.

**When generation succeeds** (Created state or config file mode):

```text
✓ Generated deployment artifacts for environment: my-env

Artifacts written to: build/my-env/
Target instance IP: 203.0.113.42

Generated artifacts:
  • OpenTofu infrastructure definitions
  • Ansible playbooks and inventory
  • Docker Compose stack configuration
  • Tracker configuration (tracker.toml)
  • Monitoring configuration (Prometheus, Grafana)
  • Reverse proxy configuration (Caddy - if HTTPS configured)
  • Backup configuration (if backups configured)

Next steps:
  1. Review generated artifacts in build/my-env/
  2. Either:
     a. Continue with deployer: provision → configure → release → run
     b. Copy artifacts to your target server (203.0.113.42) for manual deployment
```

**When environment already provisioned**:

```text
ℹ Environment 'my-env' is already provisioned (state: Provisioned)

  Generated artifacts are available at: build/my-env/
  Target instance IP: 203.0.113.42

  The provision command automatically generates all artifacts.
  No need to run 'render' for provisioned environments.

  If you need to regenerate artifacts after provisioning:
    - Modify configuration in data/my-env/environment.json
    - Run: configure --env-name my-env
          (or appropriate workflow command)
```

### Error Scenarios

| Error Condition | Message | Exit Code |
|--------------------------------------|----------------------------------------------------------------------|-----------||
| Missing `--ip` | "IP address is required: --ip <ip-address>" | 1 |
| Invalid IP address format | "Invalid IP address format: {ip}" | 1 |
| Environment name doesn't exist | "Environment 'my-env' not found in data/" | 1 |
| Environment not in Created state | (Show message: artifacts already at build/{env}/) | 0 |
| Config file doesn't exist | "Configuration file not found: {path}" | 1 |
| Config file invalid | "Invalid configuration: {validation errors}" | 1 |
| Template rendering fails | "Failed to render {template}: {error}" | 1 |
| Build directory not writable | "Cannot write to build directory: {path}" | 1 |

## Implementation Plan

**Approach**: Outside-In Development (Presentation → Application → Infrastructure)

Following `.github/skills/add-new-command/skill.md` for testability at each phase.

### Phase 0: Template Rendering Services Refactor (PREREQUISITE) ✅ COMPLETED

**Goal**: Extract reusable template rendering services to enable clean render command implementation.

**Status**: ✅ Completed - See refactor plan: `docs/refactors/plans/extract-template-rendering-services.md`

**What was completed**:

- [x] Created `src/application/services/rendering/` module with 8 rendering services
- [x] Moved AnsibleTemplateService into rendering module
- [x] Created 4 simple services: OpenTofu, Tracker, Prometheus, Grafana (Phase 1)
- [x] Created 3 complex services: DockerCompose, Caddy, Backup (Phase 2)
- [x] Refactored render handler to delegate to services (Phase 3)
- [x] Refactored 6 Steps to delegate to services (Phase 4)
- [x] Removed ~750 lines of duplicated rendering logic
- [x] All tests passing (2190 tests)
- [x] All linters passing

**Commits**:

- Phase 0: 3ecf94bc - Rendering module established
- Phase 1: d217e149 - Simple services created
- Phase 2: 901113e4 - Complex services created
- Phase 3: 3e14bea6 - Handler refactored
- Phase 4: 463e7933 - Steps refactored
- Docs: 30b9001d - Refactor plan updated

**Impact**: The render command can now cleanly reuse these services without any code duplication. Each service has a clear API accepting explicit domain types (not `Environment<S>`), making them perfect for the render command's needs.

---

### Phase 1: Presentation Layer Stub (2 hours)

**Goal**: Make command runnable with routing and empty implementation.

- [ ] Add CLI command variant in `src/presentation/input/cli/commands.rs`
  - [ ] `Render { env_name: Option<String>, env_file: Option<PathBuf>, ip: String }`
- [ ] Add routing in `src/presentation/dispatch/router.rs`
- [ ] Create controller stub in `src/presentation/controllers/render/handler.rs`
  - [ ] Show progress steps (stub)
  - [ ] Basic input validation
- [ ] Define presentation errors in `src/presentation/controllers/render/errors.rs`
  - [ ] With `.help()` methods
- [ ] Wire in `src/bootstrap/container.rs`

**Test Phase 1**:

```bash
cargo run -- render --help           # Help text displays
cargo run -- render --env-name test  # Error: missing --ip
cargo run -- render --env-name test --ip 10.0.0.1  # Stub runs, shows progress
```

**Commit**: `feat: [#326] add render command presentation layer stub`

### Phase 2: Application Layer - Command Handler (3 hours)

**Goal**: Implement business logic for artifact generation.

- [ ] Create `src/application/command_handlers/render/mod.rs`
- [ ] Create `RenderCommandHandler` struct
- [ ] Define `RenderInput` (environment name OR config file + IP)
- [ ] Define `RenderOutput` (build path + target IP)
- [ ] Define `RenderCommandHandlerError` enum with detailed help messages
- [ ] Implement dual input modes:
  - [ ] From environment data (Created state check)
  - [ ] From config file + IP parameter
- [ ] Add state validation (Created state only)
- [ ] Add IP address validation (Ipv4Addr parsing)
- [ ] Add comprehensive error messages with actionable instructions
- [ ] Add unit tests for command handler logic

**Test Phase 2**:

```bash
# Valid Created state environment
cargo run -- render --env-name created-env --ip 10.0.0.1

# Already provisioned environment (show message)
cargo run -- render --env-name provisioned-env --ip 10.0.0.1

# Config file mode
cargo run -- render --env-file envs/test.json --ip 10.0.0.1
```

**Commit**: `feat: [#326] implement render command handler with state validation`

### Phase 3: Template Orchestration (3-4 hours)

- [ ] Identify all existing template renderers:
  - [ ] `TofuTemplateRenderer` (infrastructure)
  - [ ] `AnsibleProjectGenerator` (configuration)
  - [ ] `DockerComposeRenderer` (application stack)
  - [ ] Tracker config renderer (from release command)
  - [ ] Monitoring config renderers (Prometheus/Grafana)
  - [ ] Caddy config renderer (if HTTPS)
  - [ ] Backup config renderer (if backups)
- [ ] Create orchestration logic to invoke ALL renderers
- [ ] Ensure all artifacts write to `build/{env}/` subdirectories
- [ ] Pass IP address to Ansible inventory template renderer
- [ ] Add progress indicators using `UserOutput`
- [ ] Add integration tests verifying all templates rendered with correct IP

**Commit**: `feat: [#326] implement template rendering orchestration`

### Phase 4: Documentation and Testing (2-3 hours)

**Goal**: Complete user documentation and E2E tests.

- [ ] Create user guide: `docs/user-guide/commands/render.md`
  - [ ] Overview and use cases ("preview before provision")
  - [ ] Command syntax and options
  - [ ] Examples for common scenarios
  - [ ] Explanation of generated artifacts
  - [ ] Integration with deployment workflow
- [ ] Update `docs/console-commands.md` with render command
- [ ] Update roadmap (`docs/roadmap.md`) - mark task 9.2 complete
- [ ] Add E2E tests:
  - [ ] Generate from Created environment (`--env-name` + `--ip`)
  - [ ] Show message for Provisioned environment
  - [ ] Generate from config file (`--env-file` + `--ip`)
  - [ ] Verify all expected files created in `build/{env}/`
  - [ ] Verify Ansible inventory contains correct IP address
  - [ ] Test error conditions:
    - [ ] Missing `--ip` flag
    - [ ] Invalid IP format
    - [ ] Missing environment
    - [ ] Invalid config file
- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`

**Commit**: `docs: [#326] add render command documentation and tests`

### Phase 5: Polish and Review (1-2 hours)

**Goal**: Final quality checks and refinements.

- [ ] Review generated artifact quality
- [ ] Verify no external operations triggered (templates only)
- [ ] Test with various configuration combinations:
  - [ ] LXD provider
  - [ ] Hetzner provider
  - [ ] MySQL vs SQLite
  - [ ] HTTPS enabled/disabled
  - [ ] Backup enabled/disabled
  - [ ] Monitoring enabled/disabled
- [ ] Verify output messages are clear and actionable
- [ ] Ensure consistent with project conventions
- [ ] Final documentation review

**Total Estimated Time**: 10-14 hours

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [ ] Command generates all artifacts for given environment (from `--env-name` + `--output-dir`)
- [ ] Command generates all artifacts from config file (from `--env-file` + `--ip` + `--output-dir`)
- [ ] Both input modes produce identical output for same configuration and IP
- [ ] `--output-dir` is REQUIRED (command fails without it)
- [ ] `--ip` is REQUIRED when using `--env-file` (command fails without it)
- [ ] `--ip` is FORBIDDEN when using `--env-name` (command fails if provided - would be confusing)
- [ ] IP address is validated for correct format
- [ ] IP address is correctly written to Ansible inventory template
- [ ] ALL templates are ALWAYS generated (no conditional rendering)
- [ ] All artifact types present: infrastructure, Ansible, docker-compose, tracker, monitoring, Caddy, backup
- [ ] Artifacts written to user-specified output directory (NOT internal `build/` directory)
- [ ] Output directory path and target IP shown in success message (no file list)
- [ ] No remote operations executed (no SSH, no Ansible playbook runs)
- [ ] No environment state modifications
- [ ] Command works from existing environment OR just config file + IP
- [ ] Clear success message with output path and target IP (simplified, no file list)
- [ ] Clear error messages for all failure scenarios
- [ ] `--force` flag overwrites existing output directory
- [ ] Without `--force`, fails if output directory exists

**Code Quality**:

- [ ] Follows DDD layer placement rules
- [ ] Reuses existing template renderers (no duplication)
- [ ] Error handling follows project conventions
- [ ] User output through `UserOutput` abstraction
- [ ] Proper module organization
- [ ] Comprehensive unit tests (handler and controller)
- [ ] Integration tests for template rendering
- [ ] E2E tests for complete workflow

**Documentation**:

- [ ] User guide created: `docs/user-guide/commands/render.md`
- [ ] Console commands reference updated: `docs/console-commands.md`
- [ ] Roadmap updated (task 9.2 marked complete)
- [ ] Help text includes examples
- [ ] Generated artifacts explained in documentation

**Testing**:

- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass
- [ ] Tested with all provider types
- [ ] Tested with all service configurations
- [ ] Tested error scenarios

## Related Documentation

- Template system: [`docs/contributing/templates/template-system-architecture.md`](../contributing/templates/template-system-architecture.md)
- Tera templates: [`docs/contributing/templates/tera.md`](../contributing/templates/tera.md)
- DDD layer placement: [`docs/contributing/ddd-layer-placement.md`](../contributing/ddd-layer-placement.md)
- Error handling: [`docs/contributing/error-handling.md`](../contributing/error-handling.md)
- Output handling: [`docs/contributing/output-handling.md`](../contributing/output-handling.md)
- Release command (reference): `src/application/command_handlers/release/`
- Configure command (reference): `src/application/command_handlers/configure/`
- Validate command (similar pattern): `#TBD`

## Notes

### Design Considerations

1. **IP Address Always Required**: User must explicitly provide target IP via `--ip` flag:
   - Even when using `--env-name` (environment hasn't been provisioned yet, so no infrastructure IP available)
   - This makes the command's requirements explicit and avoids confusion
   - Aligns with "preview before provision" use case

2. **Created State Only**: The render command is specifically for environments in the "Created" state:
   - **Created** = initial state after `create environment`, before `provision`
   - **Provisioned+** = artifacts already exist in `build/{env}/`, no need to regenerate
   - Use case: "Show me what will be deployed before I provision infrastructure"
   - If environment is already provisioned, command shows informational message pointing to existing artifacts

3. **Standard Build Directory**: Artifacts written to `build/{env}/` (not user-specified location):
   - Consistent with rest of deployer (provision, configure, release all use build/)
   - Simplifies implementation (no --output-dir flag needed)
   - User can copy from build/ if needed for manual deployment
   - No risk of conflicts since directory is environment-specific

4. **Instance IP is Essential**: Templates (especially Ansible inventory) require the target instance IP address:
   - When using `--env-name` + `--ip`: User provides target IP (environment not provisioned yet)
   - When using `--env-file` + `--ip`: User provides target IP (no environment created)
   - This aligns with the `register` command pattern (for pre-provisioned instances)

5. **Reuse Over Reinvention**: This command is essentially "run all template renderers but don't execute anything." We should reuse existing renderer infrastructure rather than duplicating logic.

6. **All Templates Always Rendered**: Current implementation renders ALL templates regardless of configuration:
   - Simplifies rendering logic (no conditional checks)
   - User gets complete artifact set
   - Templates for optional services (MySQL, HTTPS, etc.) are still generated
   - May change in the future if inter-template dependencies emerge
   - Only dynamic value is instance IP (user-provided via `--ip` flag)
   - All other values come from environment configuration

7. **Simplified Output**: Show only build directory path and IP, not individual file list:
   - All templates are always rendered (predictable output)
   - Avoids maintenance burden of updating file list as templates change
   - User can explore build directory directly

8. **AI Agent Friendly**: This command enables AI agents to inspect what would be deployed without executing deployment. Pairs well with `validate` for complete preview workflow.

9. **Manual Deployment Bridge**: For organizations with strict change control, this enables: generate → review → approve → manual deploy.

### Future Enhancements

- **`--format` flag**: Support different output formats (directory structure, tarball, zip)
- **`--filter` flag**: Generate only specific artifact categories (e.g., `--filter=ansible,docker`)
- **Diff mode**: Compare generated artifacts with currently deployed version
- **Template variables export**: Output a JSON/YAML file with all template variables for inspection

### Open Questions

1. **Command Name**: Final decision between `render`, `generate`, or other?
   - Recommendation: `render` (aligns with internal architecture)
   - Alternative: `generate` (more user-friendly)

2. **Overwrite Behavior**: Should `--force` be required or should we prompt?
   - Recommendation: Require `--force` (safer, more explicit)
   - Pro: Prevents accidental overwrites
   - Con: Extra flag for users

3. **Output Structure**: Should we create subdirectories per artifact type or flat structure?
   - Recommendation: Subdirectories (matches template source structure)
   - Easier to navigate and understand
   - Matches deployment directory structure

4. **Should `--output-dir` have a default value?**
   - Recommendation: NO - always require explicit `--output-dir`
   - Rationale: User must consciously choose output location
   - Alternative: Could default to `./artifacts/` but explicit is safer

5. **IP validation strictness**: Should we validate IP format strictly or allow hostnames?
   - Recommendation: Allow both IP addresses and hostnames (Ansible supports both)
   - Update flag name to `--host` or `--target-host` if supporting hostnames
   - Alternative: Keep `--ip` strict, add separate `--hostname` flag
