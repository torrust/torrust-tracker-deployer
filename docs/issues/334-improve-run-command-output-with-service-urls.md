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

- [x] Create shared view module: `src/presentation/views/commands/shared/service_urls/`
- [x] Extract URL rendering logic from `show` command views
- [x] Reuse shared views in both `run` and `show` commands
- [x] Follow DDD layer separation (see [`docs/codebase-architecture.md`](../codebase-architecture.md))

### Architectural Constraints

- [x] Reuse service URL rendering logic from `show` command
- [x] Show subset of information: only service URLs (no SSH, internal ports)
- [x] Include DNS note for TLS environments
- [x] Error handling follows project conventions (see [`docs/contributing/error-handling.md`](../contributing/error-handling.md))
- [x] Output handling follows project conventions (see [`docs/contributing/output-handling.md`](../contributing/output-handling.md))

### Anti-Patterns to Avoid

- ✅ Duplicating URL rendering logic between commands (avoided)
- ✅ Mixing business logic with presentation formatting (avoided)
- ✅ Using `println!` or `eprintln!` instead of `UserOutput` (avoided)
- ✅ Showing internal-only services (localhost addresses) without context (avoided)

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

### Phase 1: Extract Shared View Components ✅ COMPLETE

**Goal**: Create reusable view components for service URL rendering

- [x] Create `src/presentation/views/commands/shared/` directory
- [x] Create `src/presentation/views/commands/shared/service_urls/` module
- [x] Extract URL formatting logic from existing views:
  - [x] `CompactServiceUrlsView` for public service URLs
  - [x] Filter logic for publicly accessible services
  - [x] DNS hint rendering (for TLS environments) via `DnsHintView`
- [x] Add unit tests for shared views (15 tests covering all display logic)

**Files Created**:

- `src/presentation/views/commands/shared/mod.rs`
- `src/presentation/views/commands/shared/service_urls/mod.rs`
- `src/presentation/views/commands/shared/service_urls/compact.rs` (10 tests)
- `src/presentation/views/commands/shared/service_urls/dns_hint.rs` (5 tests)

**Files Modified**:

- `src/presentation/views/commands/mod.rs` (added shared module export)

### Phase 2: Enhance Run Command Output ✅ COMPLETE

**Goal**: Add service URLs to run command completion message

- [x] Modify `RunCommandController::complete_workflow()` in `src/presentation/controllers/run/handler.rs`
- [x] Load environment info after services start (reuse logic from `show` command handler)
- [x] Render service URLs using shared views from Phase 1
- [x] Add DNS hint for TLS environments
- [x] Add tip about `show` command
- [x] Ensure output goes to stdout (not stderr) using `ProgressReporter::result()`
- [x] Add `From<RepositoryError>` conversion for proper error handling

**Files Modified**:

- `src/presentation/controllers/run/handler.rs` ✅ COMPLETE
  - Added method to load environment info after services start
  - Added method to render service URLs summary (`display_service_urls`)
  - Updated `complete_workflow()` to include service URLs
- `src/presentation/controllers/run/errors.rs` ✅ COMPLETE
  - Added `From<RepositoryError>` conversion

### Phase 3: Testing & Documentation ✅ COMPLETE

**Goal**: Ensure quality and document the changes

- [x] Add unit tests for new shared views (15 tests, all passing)
- [x] E2E tests naturally exercise new code path (existing tests pass)
- [x] Update documentation:
  - [x] [`docs/user-guide/commands/run.md`](../user-guide/commands/run.md) - documented new output format with examples
  - [x] [`docs/console-commands.md`](../console-commands.md) - updated run command example output

**Note**: Reference outputs directory doesn't exist in project structure. E2E tests validate functionality, not console output formatting.

**Time Taken**: ~4 hours

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

**Output Requirements**:

- [x] Run command displays service URLs after success message
- [x] Output includes all publicly accessible services (API, HTTP tracker, Grafana)
- [x] Health Check URL included only if publicly exposed (not localhost)
- [x] Prometheus not shown (internal only)
- [x] Localhost-only services not shown
- [x] TLS environments show DNS configuration note
- [x] Tip about `show` command always displayed

**Code Quality**:

- [x] Shared view module created in `src/presentation/views/commands/shared/service_urls/`
- [x] URL rendering logic extracted and reused (CompactServiceUrlsView, DnsHintView)
- [x] No duplication between `run` and `show` command views
- [x] Uses `UserOutput` methods (no `println!` or `eprintln!`)
- [x] Output goes to stdout via `ProgressReporter::result()`
- [x] Follows module organization conventions (see [`docs/contributing/module-organization.md`](../contributing/module-organization.md))

**Testing**:

- [x] Unit tests for shared views (15 tests, all passing)
- [x] E2E tests naturally exercise new code path (existing tests pass)
- [x] Tests cover both HTTP and HTTPS scenarios (via shared view tests)

**Documentation**:

- [x] User guide updated with new output examples
- [x] Console commands documentation updated

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
