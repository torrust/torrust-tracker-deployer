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

- [x] Generate all deployment artifacts without executing deployment commands ✅
- [x] **Reuse existing template rendering infrastructure from release/configure commands** ✅ (Phase 0 complete - 8 rendering services extracted)
- [x] Support all artifact types: docker-compose, tracker config, Ansible playbooks, Caddy, monitoring ✅
- [x] Provide clear output showing what was generated and where ✅
- [x] Enable "inspect before deploy" workflow for cautious administrators ✅
- [x] Make the tool more AI-agent friendly (dry-run artifact inspection) ✅

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

**CRITICAL DESIGN DECISION - Output Directory Separation**:

The `render` command **MUST use a separate output directory** from the standard `build/{env}/` location to avoid ambiguity and data loss.

**Problem Identified** (why `--output-dir` is required):

1. User creates environment
2. User runs `render --env-name env --instance-ip 192.168.1.100` (preview with fake IP)
3. User runs `provision env` (generates artifacts with real IP to `build/env/`)

**Without separate output**: The provision command **silently overwrites** the render output, destroying the preview artifacts the user generated. No record of what was previewed.

**Solution**: Require `--output-dir` flag for explicit separation:

- **Render command** writes to user-specified directory (e.g., `./preview/`, `/tmp/artifacts/`)
- **Provision command** writes to `build/{env}/` (standard deployer location)
- Clear mental model: render = "preview mode" (your location), provision = "deployment mode" (deployer location)
- No conflicts, no data loss, no ambiguity

```bash
# Mode 1: Generate from existing Created environment
torrust-tracker-deployer render --env-name <name> --instance-ip <ip> --output-dir <path>

# Mode 2: Generate from config file (no environment creation)
torrust-tracker-deployer render --env-file <path> --instance-ip <ip> --output-dir <path>
```

**Design Decisions**:

1. **IP address is ALWAYS REQUIRED** (via `--instance-ip` flag)
   - **Rationale**: User must explicitly specify target infrastructure IP
   - Even when using `--env-name`, IP is required (environment hasn't been provisioned yet)
   - Ansible inventory template requires IP address for all templates

2. **Output directory is ALWAYS REQUIRED** (via `--output-dir` flag)
   - **Rationale**: Prevents ambiguity and data loss (see "Problem Identified" above)
   - Render writes to user-specified location (preview artifacts)
   - Provision writes to `build/{env}/` (deployment artifacts)
   - Clear separation between preview mode and deployment mode
   - User controls where preview artifacts are stored

3. **Works with any environment state** (when using `--env-name`)
   - **Created state**: Generate preview before provisioning
   - **Provisioned state**: Generate preview with different IP or configuration
   - **Use case**: "Generate artifacts to preview/inspect at any time"
   - No state restrictions - pure read-only operation

**Supported Input Modes**:

1. **`--env-name` + `--instance-ip` + `--output-dir`** (existing environment):
   - Use case: "I created an environment, generate artifacts before provisioning"
   - Use case: "Preview what will be deployed to this IP"
   - Use case: "Generate artifacts with different IP for comparison"
   - Reads from: `data/{env}/environment.json` (environment configuration)
   - State: Any state (Created, Provisioned, etc.) - pure read-only operation
   - IP: User-provided via `--instance-ip` flag
   - Output: User-specified via `--output-dir` flag

2. **`--env-file` + `--instance-ip` + `--output-dir`** (config file mode):
   - Use case: "Generate artifacts from this config before creating environment"
   - Use case: "I want artifacts only, no deployer state management"
   - Reads from: User-provided config file (e.g., `envs/my-config.json`)
   - State: No environment created
   - IP: User-provided via `--instance-ip` flag
   - Output: User-specified via `--output-dir` flag

### Output Format

**Simplified Output** - Only show output directory path and target IP.

**When generation succeeds**:

```text
✓ Generated deployment artifacts

Artifacts written to: /home/user/preview-artifacts/
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
  1. Review generated artifacts in /home/user/preview-artifacts/
  2. Either:
     a. Continue with deployer workflow: provision → configure → release → run
        (artifacts will be regenerated in build/{env}/ with actual IP)
     b. Use these artifacts for manual deployment to 203.0.113.42
```

### Error Scenarios

| Error Condition                      | Message                                                              | Exit Code |
| ------------------------------------ | -------------------------------------------------------------------- | --------- |
| Missing `--instance-ip`              | "Instance IP is required: --instance-ip <ip-address>"                | 1         |
| Missing `--output-dir`               | "Output directory is required: --output-dir <path>"                  | 1         |
| Invalid IP address format            | "Invalid IP address format: {ip}"                                    | 1         |
| Output directory exists (no --force) | "Output directory already exists: {path}. Use --force to overwrite." | 1         |
| Environment name doesn't exist       | "Environment 'my-env' not found in data/"                            | 1         |
| Config file doesn't exist            | "Configuration file not found: {path}"                               | 1         |
| Config file invalid                  | "Invalid configuration: {validation errors}"                         | 1         |
| Template rendering fails             | "Failed to render {template}: {error}"                               | 1         |
| Output directory not writable        | "Cannot write to output directory: {path}"                           | 1         |

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

### Phase 1: Presentation Layer Stub ✅ COMPLETED

**Goal**: Make command runnable with routing and empty implementation.

- [x] Add CLI command variant in `src/presentation/input/cli/commands.rs`
  - [x] `Render { env_name: Option<String>, env_file: Option<PathBuf>, instance_ip: String, output_dir: PathBuf, force: bool }`
- [x] Add routing in `src/presentation/dispatch/router.rs`
- [x] Create controller in `src/presentation/controllers/render/handler.rs`
  - [x] Show progress steps
  - [x] Input validation
- [x] Define presentation errors in `src/presentation/controllers/render/errors.rs`
  - [x] With `.help()` methods
- [x] Wire in `src/bootstrap/container.rs`

**Status**: Initial implementation complete. Requires update for `--output-dir` flag.

**Commit**: Part of implementation commits

### Phase 2: Application Layer - Command Handler ✅ COMPLETED

**Goal**: Implement business logic for artifact generation.

- [x] Create `src/application/command_handlers/render/mod.rs`
- [x] Create `RenderCommandHandler` struct
- [x] Define `RenderInput` (environment name OR config file + IP + output dir)
- [x] Define `RenderOutput` (output path + target IP)
- [x] Define `RenderCommandHandlerError` enum with detailed help messages
- [x] Implement dual input modes:
  - [x] From environment data
  - [x] From config file + IP parameter
- [x] Add IP address validation (Ipv4Addr parsing)
- [x] Add comprehensive error messages with actionable instructions
- [x] Add unit tests for command handler logic

**Status**: Initial implementation complete. Requires update for `--output-dir` handling and output directory validation.

**Commit**: Part of implementation commits

### Phase 3: Template Orchestration ✅ COMPLETED

- [x] Identify all existing template renderers:
  - [x] `TofuTemplateRenderer` (infrastructure)
  - [x] `AnsibleProjectGenerator` (configuration)
  - [x] `DockerComposeRenderer` (application stack)
  - [x] Tracker config renderer (from release command)
  - [x] Monitoring config renderers (Prometheus/Grafana)
  - [x] Caddy config renderer (if HTTPS)
  - [x] Backup config renderer (if backups)
- [x] Create orchestration logic to invoke ALL renderers
- [x] Ensure all artifacts write to target directory subdirectories
- [x] Pass IP address to Ansible inventory template renderer
- [x] Add progress indicators using `UserOutput`
- [x] Add integration tests verifying all templates rendered with correct IP

**Status**: Initial implementation complete. Requires update to use `--output-dir` instead of `build/{env}/`.

**Commit**: Part of implementation commits

### Phase 4: Documentation and Testing (2-3 hours) ✅ Complete

**Goal**: Complete user documentation and E2E tests.

- [x] Create user guide: `docs/user-guide/commands/render.md`
  - [x] Overview and use cases ("preview before provision")
  - [x] Command syntax and options
  - [x] Examples for common scenarios
  - [x] Explanation of generated artifacts
  - [x] Integration with deployment workflow
- [x] Update `docs/console-commands.md` with render command
- [x] Update roadmap (`docs/roadmap.md`) - mark task 9.2 complete
- [x] Add E2E tests:
  - [x] Generate from Created environment (`--env-name` + `--ip`)
  - [x] Show message for Provisioned environment (handled via state checking)
  - [x] Generate from config file (`--env-file` + `--ip`)
  - [x] Verify all expected files created in `build/{env}/`
  - [x] Verify Ansible inventory contains correct IP address
  - [x] Test error conditions:
    - [x] Missing `--ip` flag (handled by clap validation)
    - [x] Invalid IP format
    - [x] Missing environment
    - [x] Invalid config file
- [x] Add manual E2E test documentation: `docs/e2e-testing/manual/render-verification.md`
- [x] Run pre-commit checks: `./scripts/pre-commit.sh`

**Commits**:

- `test: [#326] add E2E blackbox tests and manual test documentation for render command` (37cbe240)
- `docs: [#326] add render command user guide and update documentation` (689094fb)

**Manual E2E Test**: Successfully completed (2026-02-10)

- Validated artifact generation with test IP
- Compared rendered vs provisioned artifacts (only expected differences: IP, timestamps, terraform state)
- Validated Docker Compose byte-for-byte identical
- Confirmed state machine enforcement
- Result: ✅ ALL VALIDATIONS PASSED

### Phase 5: Output Directory Fix (2-3 hours) 🔧 IN PROGRESS

**Goal**: Fix critical design flaw - add required `--output-dir` flag to prevent ambiguity.

**Problem Discovered**: Without separate output directory, render artifacts conflict with provision artifacts:

1. User renders with test IP to `build/env/`
2. User provisions with real IP to `build/env/` (overwrites render output)
3. Result: Silent data loss, no record of preview

**Solution**: Require `--output-dir` flag for explicit separation.

- [ ] Update CLI command struct to include `output_dir: PathBuf` (required)
- [ ] Add `--force` flag for overwrite behavior
- [ ] Update command handler to accept output directory
- [ ] Add output directory validation (exists check, writability)
- [ ] Update template orchestration to write to user-specified directory
- [ ] Update all documentation:
  - [ ] User guide (`docs/user-guide/commands/render.md`)
  - [ ] Console commands (`docs/console-commands.md`)
  - [ ] Manual E2E test guide (`docs/e2e-testing/manual/render-verification.md`)
- [ ] Update E2E tests to provide `--output-dir`
- [ ] Re-run manual E2E test with new interface
- [ ] Update acceptance criteria to match corrected design

**Commit**: `fix: [#326] require --output-dir flag to prevent artifact conflicts`

---

### Phase 6: Final Polish and Review (1-2 hours)

**Goal**: Final quality checks and refinements after output-dir fix.

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

- [ ] Command generates all artifacts for given environment (from `--env-name` + `--instance-ip` + `--output-dir`)
- [ ] Command generates all artifacts from config file (from `--env-file` + `--instance-ip` + `--output-dir`)
- [ ] Both input modes produce identical output for same configuration and IP
- [ ] `--output-dir` is REQUIRED (command fails without it)
- [ ] `--instance-ip` is REQUIRED (command fails without it)
- [ ] `--force` flag enables overwriting existing output directory
- [ ] Without `--force`, fails if output directory exists (prevents accidental overwrites)
- [ ] IP address is validated for correct format
- [ ] IP address is correctly written to Ansible inventory template
- [ ] ALL templates are ALWAYS generated (no conditional rendering)
- [ ] All artifact types present: infrastructure, Ansible, docker-compose, tracker, monitoring, Caddy, backup
- [ ] Artifacts written to user-specified output directory (NOT `build/` directory)
- [ ] Output directory path and target IP shown in success message (no file list)
- [ ] No remote operations executed (no SSH, no Ansible playbook runs)
- [ ] No environment state modifications
- [ ] Command works from existing environment OR just config file + IP
- [ ] Clear success message with output path and target IP (simplified, no file list)
- [ ] Clear error messages for all failure scenarios

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

1. **IP Address Always Required**: User must explicitly provide target IP via `--instance-ip` flag:
   - Even when using `--env-name` (makes preview requirements explicit)
   - Even for provisioned environments (allows preview with different IPs)
   - This makes the command's requirements explicit and avoids confusion
   - Aligns with "preview before provision" use case

2. **Output Directory Always Required**: User must explicitly provide output location via `--output-dir` flag:
   - **Critical for avoiding ambiguity**: Without this, render artifacts conflict with provision artifacts
   - **Scenario**: User renders with test IP, then provisions with real IP - without separate directories, provision silently destroys render output
   - **Solution**: Render writes to user-specified preview location, provision writes to `build/{env}/`
   - **Mental model**: Render = "preview mode" (your location), Provision = "deployment mode" (deployer location)
   - User controls where preview artifacts are stored
   - No overlap, no data loss, clear separation of concerns

3. **No State Restrictions**: The render command works with environments in any state:
   - **Created** = Preview before provisioning (primary use case)
   - **Provisioned** = Generate preview with different IP or inspect configuration
   - **Any state** = Pure read-only operation, no state modifications
   - Use case: "Show me what would be deployed to this IP" at any time

4. **Instance IP is Essential**: Templates (especially Ansible inventory) require the target instance IP address:
   - When using `--env-name` + `--instance-ip`: User provides target IP for preview
   - When using `--env-file` + `--instance-ip`: User provides target IP for preview
   - This aligns with the `register` command pattern (for pre-provisioned instances)
   - Enables "what-if" scenarios: "What would artifacts look like for IP X?"

5. **Reuse Over Reinvention**: This command is essentially "run all template renderers but don't execute anything." We reuse existing renderer infrastructure rather than duplicating logic (8 rendering services from Phase 0).

6. **All Templates Always Rendered**: Current implementation renders ALL templates regardless of configuration:
   - Simplifies rendering logic (no conditional checks)
   - User gets complete artifact set
   - Templates for optional services (MySQL, HTTPS, etc.) are still generated
   - May change in the future if inter-template dependencies emerge
   - Only dynamic value is instance IP (user-provided via `--ip` flag)
   - All other values come from environment configuration

7. **Simplified Output**: Show only output directory path and IP, not individual file list:
   - All templates are always rendered (predictable output)
   - Avoids maintenance burden of updating file list as templates change
   - User can explore output directory directly

8. **Artifact Separation Prevents Data Loss**:
   - **Without `--output-dir`**: Provision silently overwrites render output (data loss)
   - **With `--output-dir`**: Render in separate location, provision to `build/{env}/` (no conflicts)
   - User maintains preview history if desired (can keep multiple render outputs)

9. **AI Agent Friendly**: This command enables AI agents to inspect what would be deployed without executing deployment. Pairs well with `validate` for complete preview workflow.

10. **Manual Deployment Bridge**: For organizations with strict change control, this enables: generate → review → approve → manual deploy to target infrastructure.

### Future Enhancements

- **`--format` flag**: Support different output formats (directory structure, tarball, zip)
- **`--filter` flag**: Generate only specific artifact categories (e.g., `--filter=ansible,docker`)
- **Diff mode**: Compare generated artifacts with currently deployed version
- **Template variables export**: Output a JSON/YAML file with all template variables for inspection

### Open Questions

1. **Command Name**: ✅ RESOLVED - Using `render` (aligns with internal architecture)

2. **Overwrite Behavior**: ✅ RESOLVED - Require `--force` flag (safer, more explicit)
   - Prevents accidental overwrites
   - Clear user intent required

3. **Output Structure**: ✅ RESOLVED - Subdirectories per artifact type (matches template source structure)
   - Easier to navigate and understand
   - Matches deployment directory structure

4. **Should `--output-dir` have a default value?**: ✅ RESOLVED - NO, always require explicit `--output-dir`
   - **Rationale**: Prevents ambiguity and data loss (see "Design Considerations" section)
   - User must consciously choose output location
   - Clear separation: render (preview) vs provision (deployment)

5. **IP validation strictness**: ✅ RESOLVED - Accept only IP addresses (strict validation)
   - Flag name: `--instance-ip` (clear intent)
   - Validation: IPv4 address format only
   - Future: Could extend to support hostnames if needed
