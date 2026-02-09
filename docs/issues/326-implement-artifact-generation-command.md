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
- [ ] Reuse existing template rendering infrastructure from release/configure commands
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

```bash
# Primary usage - generate from existing environment
torrust-tracker-deployer render --env-name <name> --output-dir <path>

# Alternative - generate from environment config file (requires IP)
torrust-tracker-deployer render --env-file <path> --ip <ip-address> --output-dir <path>
```

**Design Decisions**:

1. **`--output-dir` is REQUIRED** (not optional)
   - **Rationale**: Avoid conflicts with internal `build/` directory used by the deployer for actual deployment operations
   - **Separation of concerns**: Internal build artifacts (deployment automation) vs user-facing generated artifacts (manual inspection/deployment)
   - **User gets full control**: Explicit output location prevents accidental overwrites of deployer's internal state

2. **Instance IP address is REQUIRED** for Ansible inventory template (`templates/ansible/inventory.yml.tera`)
   - When using `--env-name`: IP is read from `data/{env}/environment.json` (infrastructure outputs)
   - When using `--env-file`: IP must be provided via `--ip` flag (user must know target IP in advance)

**Supported Input Modes**:

1. **`--env-name`** (existing environment):
   - Use case: "I have a deployed environment, regenerate artifacts"
   - Use case: "Show me what was deployed"
   - Reads from: `data/{env}/environment.json` (includes IP from infrastructure outputs)
   - State: Must exist in data directory
   - IP: Automatically extracted from environment data

2. **`--env-file` + `--ip`** (preview mode):
   - Use case: "Generate artifacts from this config before deploying" (user knows target IP)
   - Use case: "Preview what would be deployed to this specific server"
   - Reads from: User-provided config file (e.g., `envs/my-config.json`) + explicit IP
   - State: No environment created
   - IP: User-provided (must be known in advance - e.g., pre-provisioned instance)
   - Combines with `validate` workflow: `validate` → `render --ip <ip>` → manual deployment

### Output Format

**Simplified Output** - Only show output directory path and target IP. No need to list individual files since:

- All templates are always rendered (predictable output)
- Listing files would require updating specification every time templates are added
- User can easily explore the output directory themselves

When generation succeeds, show:

```text
✓ Generated deployment artifacts for environment: my-env

Artifacts written to: /path/to/output/
Target instance IP: 203.0.113.42

Next steps:
  1. Review generated artifacts in /path/to/output/
  2. Copy artifacts to your target server (203.0.113.42)
  3. Execute deployment manually:
     - Run Ansible playbooks from ansible/ directory
     - Deploy docker-compose stack from docker-compose/ directory
```

### Error Scenarios

| Error Condition | Message | Exit Code |
|--------------------------------------|----------------------------------------------------------------------|-----------||
| Missing `--output-dir` | "Output directory is required: --output-dir <path>" | 1 |
| Environment name doesn't exist | "Environment 'my-env' not found in data/" | 1 |
| Config file doesn't exist | "Configuration file not found: {path}" | 1 |
| Config file invalid | "Invalid configuration: {validation errors}" | 1 |
| Missing `--ip` with `--env-file` | "IP address required when using --env-file: --ip <ip-address>" | 1 |
| Invalid IP address format | "Invalid IP address format: {ip}" | 1 |
| Template rendering fails | "Failed to render {template}: {error}" | 1 |
| Output directory not writable | "Cannot write to output directory: {path}" | 1 |
| Output directory already exists | "Output directory already exists: {path} (use --force to overwrite)" | 1 |

## Implementation Plan

### Phase 1: Application Layer - Command Handler (2-3 hours)

- [ ] Create `src/application/command_handlers/render/mod.rs`
- [ ] Create `RenderCommandHandler` struct
- [ ] Define `RenderInput` (environment name OR config file path + IP, plus output directory)
- [ ] Define `RenderOutput` (output directory path + target IP)
- [ ] Define `RenderCommandHandlerError` enum
- [ ] Implement dual input modes:
  - [ ] From environment data (IP extracted from infrastructure outputs)
  - [ ] From config file + explicit IP parameter
- [ ] Add IP address validation
- [ ] Add comprehensive error messages with actionable instructions
- [ ] Add unit tests for command handler logic

### Phase 2: Template Orchestration (3-4 hours)

- [ ] Identify all existing template renderers:
  - [ ] `TofuTemplateRenderer` (infrastructure)
  - [ ] `AnsibleProjectGenerator` (configuration)
  - [ ] `DockerComposeRenderer` (application stack)
  - [ ] Tracker config renderer (from release command)
  - [ ] Monitoring config renderers (Prometheus/Grafana)
  - [ ] Caddy config renderer
  - [ ] Backup config renderer
- [ ] Create orchestration logic to invoke ALL renderers (no conditional logic)
- [ ] Ensure all artifacts write to user-specified output directory subdirectories
- [ ] Pass IP address to Ansible inventory template renderer
- [ ] Add progress indicators using `UserOutput`
- [ ] Add integration tests verifying all templates rendered with correct IP

### Phase 3: Presentation Layer - CLI Command (2 hours)

- [ ] Create `src/presentation/controllers/render/mod.rs`
- [ ] Create `RenderCommandController`
- [ ] Add CLI argument parsing:
  - [ ] `--env-name <name>` (mutually exclusive with `--env-file`)
  - [ ] `--env-file <path>` (mutually exclusive with `--env-name`, requires `--ip`)
  - [ ] `--ip <ip-address>` (required when using `--env-file`, forbidden with `--env-name`)
  - [ ] `--output-dir <path>` (REQUIRED always)
  - [ ] `--force` (optional, overwrites existing output directory)
- [ ] Implement input validation:
  - [ ] Ensure `--env-name` and `--env-file` are mutually exclusive
  - [ ] Ensure `--ip` is present when using `--env-file`
  - [ ] Ensure `--ip` is NOT present when using `--env-name` (would be confusing)
  - [ ] Validate IP address format
  - [ ] Ensure `--output-dir` is always provided
- [ ] Connect to `RenderCommandHandler`
- [ ] Format output using `UserOutput` (progress, success, output directory path, target IP)
- [ ] Handle errors with clear messages
- [ ] Add help text and examples
- [ ] Add unit tests for controller logic

### Phase 4: Documentation and Testing (2-3 hours)

- [ ] Create user guide: `docs/user-guide/commands/render.md`
  - [ ] Overview and use cases
  - [ ] Command syntax and options
  - [ ] Examples for common scenarios
  - [ ] Explanation of generated artifacts
  - [ ] Integration with manual deployment workflow
- [ ] Update `docs/console-commands.md` with render command
- [ ] Update roadmap (`docs/roadmap.md`) - mark task 9.2 complete
- [ ] Add E2E tests:
  - [ ] Generate from existing environment (with `--env-name`)
  - [ ] Generate from config file (with `--env-file` + `--ip`)
  - [ ] Verify all expected files created
  - [ ] Verify output directory structure
  - [ ] Verify Ansible inventory contains correct IP address
  - [ ] Test error conditions:
    - [ ] Missing `--output-dir`
    - [ ] Missing `--ip` when using `--env-file`
    - [ ] Invalid IP format
    - [ ] Missing environment
    - [ ] Invalid config
  - [ ] Test `--force` flag (overwrite existing output directory)
- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`

### Phase 5: Polish and Review (1-2 hours)

- [ ] Review generated artifact quality
- [ ] Verify no external operations triggered
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

1. **Separate Output from Internal Build**: The `--output-dir` parameter is **required** to avoid conflicts with the deployer's internal `build/` directory. This separation ensures:
   - Internal deployment automation uses `build/` without user interference
   - User-facing artifact generation goes to explicit, user-chosen location
   - No risk of overwriting deployer's internal state
   - Clear separation between "deployer internals" and "user artifacts"

2. **Instance IP is Essential**: Templates (especially Ansible inventory) require the target instance IP address:
   - When using `--env-name`: IP is extracted from stored environment data (infrastructure outputs)
   - When using `--env-file`: IP must be provided via `--ip` flag (user must know target IP)
   - This aligns with the `register` command pattern (for pre-provisioned instances)

3. **No State Requirements (with caveats)**: Unlike other commands, render can work from existing environment OR just a config file:
   - With `--env-name`: Full environment state provides everything (config + IP)
   - With `--env-file` + `--ip`: No environment state needed, but user must know target IP
   - This maximizes flexibility for different workflows

4. **Reuse Over Reinvention**: This command is essentially "run all template renderers but don't execute anything." We should reuse existing renderer infrastructure rather than duplicating logic.

5. **All Templates Always Rendered**: Current implementation renders ALL templates regardless of configuration:
   - Simplifies rendering logic (no conditional checks)
   - User gets complete artifact set
   - Templates for optional services (MySQL, HTTPS, etc.) are still generated
   - May change in the future if inter-template dependencies emerge
   - Only dynamic value is instance IP (from provision output or `--ip` flag)
   - All other values come from environment configuration

6. **Simplified Output**: Show only output directory path and IP, not individual file list:
   - All templates are always rendered (predictable output)
   - Avoids maintenance burden of updating file list as templates change
   - User can explore output directory directly

7. **AI Agent Friendly**: This command enables AI agents to inspect what would be deployed without executing deployment. Pairs well with `validate` for complete preview workflow.

8. **Manual Deployment Bridge**: For organizations with strict change control, this enables: generate → review → approve → manual deploy.

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
