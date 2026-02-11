# Add DNS Resolution Check to Test Command

**Issue**: #336
**Parent Epic**: N/A (Standalone task)
**Related**: Roadmap #1 - Section 10.3: Improve usability (UX)

## Overview

Add an optional DNS resolution check to the `test` command that verifies configured domains resolve to the expected instance IP. This helps users identify DNS configuration issues early while keeping infrastructure tests decoupled from DNS setup.

## Goals

- [ ] Add DNS resolution check to `test` command
- [ ] Display advisory warnings (not failures) when domains don't resolve correctly
- [ ] Show resolved IP and expected IP for troubleshooting
- [ ] Only check when domains are configured
- [ ] Distinguish between service tests (using internal IP) and DNS checks (using system DNS)

## 🏗️ Architecture Requirements

**DDD Layer**: Application (Steps) + Infrastructure (DNS resolution)
**Module Path**: `src/application/steps/` + `src/infrastructure/dns/`
**Pattern**: Step + Infrastructure Service

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] DNS resolution logic in infrastructure layer (`src/infrastructure/dns/`)
- [ ] DNS check orchestration and result types in application layer (`src/application/command_handlers/test/`)
- [ ] DNS warning rendering in presentation layer (`src/presentation/controllers/test/`)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))

### Architectural Constraints

- [ ] DNS resolution belongs in infrastructure layer (external system interaction)
- [ ] Test command handler returns structured `TestResult` with DNS warnings (application layer)
- [ ] `TestCommandHandler` must NOT use `UserOutput` — return data, let presentation display it
- [ ] Presentation controller renders DNS warnings from `TestResult` (follows `ListCommandHandler` → `EnvironmentList` pattern)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Output uses `UserOutput` methods only in presentation layer (see [docs/contributing/output-handling.md](../docs/contributing/output-handling.md))

### Anti-Patterns to Avoid

- ❌ Using `println!` or direct stdout/stderr access (use `UserOutput` in presentation layer)
- ❌ Passing `UserOutput` to command handlers in the application layer
- ❌ Making DNS checks required/strict (should be advisory warnings)
- ❌ Mixing DNS resolution logic in application layer
- ❌ Failing infrastructure tests due to DNS issues

## Specifications

### Context

### Context and Current Behavior

The `test` command validates infrastructure health by making actual requests to services. However, these tests use the instance IP directly (resolved internally) rather than relying on external DNS resolution. This is intentional - it decouples infrastructure tests from user's DNS configuration.

**Current behavior:**

- Tests resolve domains internally using the known instance IP
- Tests pass regardless of external DNS configuration
- User may not realize their DNS isn't configured correctly

**Problem:** A user could have all tests pass but still be unable to access services via domain names because their DNS isn't configured.

### Proposed Behavior

Add a DNS resolution check as an advisory warning (not a failure):

```text
⏳ [3/3] Testing infrastructure...
⏳   ✓ Cloud-init completed
⏳   ✓ Docker running
⏳   ✓ Docker Compose running
⏳   ✓ HTTP tracker responding
⏳   ✓ API responding
⚠️   DNS check: http.tracker.local does not resolve (expected: 10.140.190.254)
⚠️   DNS check: api.tracker.local does not resolve (expected: 10.140.190.254)
⏳   ✓ Infrastructure tests passed (with warnings)
✅ Infrastructure validation completed successfully for 'lxd-local-https-example'
```

When DNS is correctly configured:

```text
⏳   ✓ DNS check: http.tracker.local → 10.140.190.254 ✓
⏳   ✓ DNS check: api.tracker.local → 10.140.190.254 ✓
```

### Design Decisions

1. **Warning, not failure** - DNS issues shouldn't fail infrastructure tests because:
   - DNS propagation can take time
   - Local `.local` domains use `/etc/hosts`, not real DNS
   - User may intentionally be testing without DNS

2. **Separate from service tests** - This is a distinct check from "is the service running":
   - Service tests: verify the application works (using internal IP resolution)
   - DNS check: verify external access will work (using system DNS)

3. **Only when domains configured** - Skip the check entirely if no domains are defined

4. **Return structured result, don't use `UserOutput` in the application layer** - The
   `TestCommandHandler` must return a structured `TestResult` type containing DNS warnings,
   following the same pattern as `ListCommandHandler` which returns `EnvironmentList`. The
   presentation layer is responsible for rendering the warnings to the user.

   **Rationale**: In this project, "command handlers" handle user commands (not strictly CQRS
   commands). Some are pure commands that modify state (`create`, `provision`), while others
   are read-only queries (`list`, `show`, `test`). The `ListCommandHandler` already returns
   a structured DTO (`EnvironmentList`) that the presentation layer renders — the test command
   should follow this established pattern.

   **Why not pass `UserOutput` to the handler?**
   - `UserOutput` is a presentation concern — no other command handler uses it
   - Passing it to the application layer breaks DDD layer separation
   - It makes the handler harder to test (needs to mock output instead of asserting on return values)
   - The handler should produce data; the controller should display it

   **Implementation pattern** (consistent with `ListCommandHandler` → `EnvironmentList`):

   ```rust
   // Application layer: returns structured data
   pub async fn execute(&self, env_name: &EnvironmentName)
       -> Result<TestResult, TestCommandHandlerError>

   // TestResult contains advisory DNS warnings
   pub struct TestResult {
       pub dns_warnings: Vec<DnsWarning>,
   }

   pub struct DnsWarning {
       pub domain: DomainName,
       pub expected_ip: IpAddr,
       pub issue: DnsIssue,
   }

   pub enum DnsIssue {
       ResolutionFailed(String),
       IpMismatch { resolved_ips: Vec<IpAddr> },
   }

   // Presentation layer: renders the warnings
   let result = handler.execute(&env_name).await?;
   for warning in &result.dns_warnings {
       self.progress.output().lock().borrow_mut().warn(&format!("DNS check: {warning}"));
   }
   ```

### Implementation Details

- Use system DNS resolution (not internal resolution)
- Check each configured domain (HTTP trackers, API, health check, Grafana)
- Compare resolved IP with expected instance IP
- Report as warnings, not errors
- Include expected IP in warning message for troubleshooting

### Special Considerations

- **Local domains (.local)**: These typically use mDNS or `/etc/hosts`, not DNS servers
- **DNS propagation**: New records can take minutes to hours to propagate
- **Multiple IPs**: Some domains may legitimately resolve to different IPs (load balancers, CDNs)

## Implementation Plan

### Phase 1: Add DNS Resolution Infrastructure (2-3 hours)

- [ ] Create `src/infrastructure/dns/resolver.rs` module
- [ ] Implement system DNS resolution using Rust's `std::net::ToSocketAddrs` or `trust-dns-resolver` crate
- [ ] Add error types for DNS resolution failures
- [ ] Add unit tests for DNS resolver

### Phase 2: Add DNS Check to Test Command Handler (2-3 hours)

- [ ] Add `TestResult`, `DnsWarning`, `DnsIssue` types to `src/application/command_handlers/test/`
- [ ] Change `TestCommandHandler.execute()` return type from `Result<(), ...>` to `Result<TestResult, ...>`
- [ ] Add DNS check logic in handler: extract domains from tracker config, use `DnsResolver`, collect warnings
- [ ] Handle cases where no domains are configured (return empty warnings)
- [ ] Update presentation controller to render DNS warnings from `TestResult`
- [ ] Remove any `UserOutput` parameter from `TestCommandHandler.execute()`

### Phase 3: Integration and Testing (1-2 hours)

- [ ] Update E2E tests to verify DNS check behavior
- [ ] Test with domains that don't resolve (should show warnings)
- [ ] Test with correctly configured domains (should show success)
- [ ] Test with no domains configured (should skip check)
- [ ] Update documentation to explain DNS check behavior

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] DNS resolution check is added to `test` command
- [ ] DNS check shows warnings (not failures) when domains don't resolve
- [ ] DNS check shows success when domains resolve correctly
- [ ] DNS check is skipped when no domains are configured
- [ ] Output uses `UserOutput` methods (no `println!` or direct stdout/stderr)
- [ ] Error messages include expected IP for troubleshooting
- [ ] E2E tests verify DNS check behavior in all scenarios
- [ ] Documentation explains DNS check purpose and behavior

## Related Documentation

- [docs/user-guide/commands/test.md](../user-guide/commands/test.md) - Test command documentation
- [docs/contributing/output-handling.md](../contributing/output-handling.md) - Output conventions
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error handling patterns
- [docs/codebase-architecture.md](../codebase-architecture.md) - DDD architecture
- [docs/reference/command-outputs/lxd-local-example.md](../reference/command-outputs/lxd-local-example.md) - Example without TLS
- [docs/reference/command-outputs/lxd-local-https-example.md](../reference/command-outputs/lxd-local-https-example.md) - Example with TLS

## Notes

### Open Questions

1. Should there be a flag to make DNS checks strict (fail on mismatch)?
   - **Recommendation**: Start with warnings only, add `--strict-dns` flag later if needed
2. Should we check from the deployer machine or from the target instance?
   - **Recommendation**: Check from deployer machine (that's where the user will access services)
3. How to handle `.local` domains (mDNS vs /etc/hosts)?
   - **Recommendation**: Check using system DNS, let OS handle mDNS/hosts lookup

### Future Enhancements

- Add `--skip-dns-check` flag to disable DNS checks entirely
- Add support for checking CNAME records (not just A records)
- Add support for checking multiple IPs (for load balancers)
