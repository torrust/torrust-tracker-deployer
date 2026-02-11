# Improve Run Command Output with Service URLs

**Issue**: #334
**Parent**: #1 (Project Roadmap)
**Related Roadmap Section**: 10. Improve usability (UX)

## Overview

Enhance the `run` command output to display service URLs immediately after services start, plus a hint about the `show` command for full details. This improves actionability by giving users immediate access to their services without requiring a separate command.

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation  
**Module Path**: `src/presentation/controllers/run/` and `src/presentation/views/commands/shared/`  
**Pattern**: View composition with shared service URL views

### Module Structure Requirements

- [ ] Create shared view module: `src/presentation/views/commands/shared/service_urls/`
- [ ] Extract URL rendering logic from `show` command views
- [ ] Reuse shared views in both `run` and `show` commands
- [ ] Follow DDD layer separation (see [`docs/codebase-architecture.md`](../codebase-architecture.md))

### Architectural Constraints

- [ ] Reuse service URL rendering logic from `show` command
- [ ] Show subset of information: only service URLs (no SSH, internal ports)
- [ ] Include DNS note for TLS environments
- [ ] Error handling follows project conventions (see [`docs/contributing/error-handling.md`](../contributing/error-handling.md))
- [ ] Output handling follows project conventions (see [`docs/contributing/output-handling.md`](../contributing/output-handling.md))

### Anti-Patterns to Avoid

- ❌ Duplicating URL rendering logic between commands
- ❌ Mixing business logic with presentation formatting
- ❌ Using `println!` or `eprintln!` instead of `UserOutput`
- ❌ Showing internal-only services (localhost addresses) without context

## Context

Currently, the `run` command output is minimal:

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/2] Running application services...
⏳   ✓ Services started (took 22.9s)
✅ Run command completed for 'lxd-local-example'
```

In contrast, the `show` command provides rich information about service URLs, connection details, and DNS instructions.

**Problem**: After running `run`, users want to immediately access their services but must run `show` separately to find the URLs.

## Proposed Output

After `run` succeeds, show the most actionable information (service URLs) plus a hint:

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/2] Running application services...
⏳   ✓ Services started (took 22.9s)
✅ Run command completed for 'lxd-local-example'

Services are now accessible:
  Tracker (UDP):  udp://10.140.190.188:6969/announce
  Tracker (HTTP): http://10.140.190.188:7070/announce
  API:            http://10.140.190.188:1212/api
  Health Check:   http://10.140.190.188:1313/health_check
  Grafana:        http://10.140.190.188:3000/

Tip: Run 'torrust-tracker-deployer show lxd-local-example' for full details.
```

For TLS-enabled environments:

```text
✅ Run command completed for 'lxd-local-https-example'

Services are now accessible:
  Tracker (UDP):  udp://udp.tracker.local:6969/announce
  Tracker (HTTP): https://http.tracker.local/announce
  API:            https://api.tracker.local/api
  Health Check:   https://health.tracker.local/health_check
  Grafana:        https://grafana.tracker.local/

Note: HTTPS services require DNS configuration. See 'show' command for details.

Tip: Run 'torrust-tracker-deployer show lxd-local-https-example' for full details.
```

**Rules for URL Display**:

- ✅ Include Health Check URL if publicly exposed (not localhost)
- ❌ Don't show Prometheus (internal only, not exposed)
- ❌ Don't show localhost-only services without SSH tunnel instructions
- ✅ Match "show" command output format for consistency

## Rationale

1. **Actionable** - User immediately knows where to access services
2. **Not overwhelming** - Doesn't duplicate the full `show` output (omits SSH details, internal ports, etc.)
3. **Educational** - Teaches users about the `show` command
4. **Follows project principles** - "Actionability: The system must always tell users how to continue"

The `run` command is the moment users want to _use_ the services, so showing URLs immediately is high value.

## Implementation Plan

### Phase 1: Extract Shared View Components

**Goal**: Create reusable view components for service URL rendering

- [ ] Create `src/presentation/views/commands/shared/` directory
- [ ] Create `src/presentation/views/commands/shared/service_urls/` module
- [ ] Extract URL formatting logic from existing views:
  - [ ] `TrackerServicesView` → `ServiceUrlsView`
  - [ ] Filter logic for publicly accessible services
  - [ ] DNS hint rendering (for TLS environments)
- [ ] Add unit tests for shared views
- [ ] Update `show` command to use shared views (refactor without breaking existing behavior)

**Files to Create**:

- `src/presentation/views/commands/shared/mod.rs`
- `src/presentation/views/commands/shared/service_urls/mod.rs`
- `src/presentation/views/commands/shared/service_urls/tracker.rs`
- `src/presentation/views/commands/shared/service_urls/grafana.rs`
- `src/presentation/views/commands/shared/service_urls/dns_hint.rs`

**Files to Modify**:

- `src/presentation/views/commands/show/environment_info/mod.rs` (use shared views)
- `src/presentation/views/commands/show/environment_info/tracker_services.rs` (delegate to shared view)
- `src/presentation/views/commands/show/environment_info/grafana.rs` (delegate to shared view)

### Phase 2: Enhance Run Command Output

**Goal**: Add service URLs to run command completion message

- [ ] Modify `RunCommandController::complete_workflow()` in `src/presentation/controllers/run/handler.rs`
- [ ] Load environment info after services start (reuse logic from `show` command handler)
- [ ] Render service URLs using shared views from Phase 1
- [ ] Add DNS hint for TLS environments
- [ ] Add tip about `show` command
- [ ] Ensure output goes to stdout (not stderr) using `ProgressReporter::result()`

**Files to Modify**:

- `src/presentation/controllers/run/handler.rs`
  - Add method to load environment info after services start
  - Add method to render service URLs summary
  - Update `complete_workflow()` to include service URLs

### Phase 3: Testing & Documentation

**Goal**: Ensure quality and document the changes

- [ ] Add unit tests for new shared views
- [ ] Add integration tests for run command output
- [ ] Update E2E tests to verify service URLs in run command output
- [ ] Update documentation:
  - [ ] [`docs/user-guide/commands/run.md`](../user-guide/commands/run.md) - document new output format
  - [ ] [`docs/console-commands.md`](../console-commands.md) - update run command example
  - [ ] Update reference outputs in [`docs/issues/reference/command-outputs/`](reference/command-outputs/)

**Time Estimate**: 4-6 hours

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

**Output Requirements**:

- [ ] Run command displays service URLs after success message
- [ ] Output includes all publicly accessible services (UDP tracker, HTTP tracker, API, Grafana)
- [ ] Health Check URL included only if publicly exposed (not localhost)
- [ ] Prometheus not shown (internal only)
- [ ] Localhost-only services not shown (or shown with SSH tunnel hint if needed)
- [ ] TLS environments show DNS configuration note
- [ ] Tip about `show` command always displayed

**Code Quality**:

- [ ] Shared view module created in `src/presentation/views/commands/shared/service_urls/`
- [ ] URL rendering logic extracted and reused from `show` command
- [ ] No duplication between `run` and `show` command views
- [ ] Uses `UserOutput` methods (no `println!` or `eprintln!`)
- [ ] Output goes to stdout via `ProgressReporter::result()`
- [ ] Follows module organization conventions (see [`docs/contributing/module-organization.md`](../contributing/module-organization.md))

**Testing**:

- [ ] Unit tests for shared views
- [ ] Integration tests for run command output
- [ ] E2E tests verify service URLs in output
- [ ] Tests cover both HTTP and HTTPS scenarios

**Documentation**:

- [ ] User guide updated with new output examples
- [ ] Console commands documentation updated
- [ ] Reference outputs updated

## Related Documentation

- Codebase Architecture: [`docs/codebase-architecture.md`](../codebase-architecture.md)
- Show Command Implementation: [`src/presentation/controllers/show/handler.rs`](../../src/presentation/controllers/show/handler.rs)
- Service URL Views: [`src/presentation/views/commands/show/environment_info/tracker_services.rs`](../../src/presentation/views/commands/show/environment_info/tracker_services.rs)
- Output Handling Conventions: [`docs/contributing/output-handling.md`](../contributing/output-handling.md)
- Module Organization: [`docs/contributing/module-organization.md`](../contributing/module-organization.md)
- Reference Outputs:
  - Without TLS: [`docs/issues/reference/command-outputs/lxd-local-example.md`](reference/command-outputs/lxd-local-example.md)
  - With TLS: [`docs/issues/reference/command-outputs/lxd-local-https-example.md`](reference/command-outputs/lxd-local-https-example.md)

## Notes

**Design Decision**: Extract shared views to avoid duplication between `run` and `show` commands. This follows DRY principle and ensures consistent formatting.

**User Experience**: The `run` command output should be immediately actionable - users should be able to copy-paste URLs and start using services. The `show` command provides additional context (SSH details, internal ports, DNS instructions) for users who need more information.

**Future Considerations**: If we add more commands that need to display service URLs (e.g., a `status` command), they should also use the shared views module.
