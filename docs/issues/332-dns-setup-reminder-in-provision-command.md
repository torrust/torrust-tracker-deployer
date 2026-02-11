# Add DNS Setup Reminder in Provision Command

**Issue**: #332  
**Status**: Open  
**Related Roadmap Section**: 10. Improve usability (UX)

## Summary

Add a reminder message in the `provision` command output to inform users they need to configure DNS to point their domains to the newly assigned server IP.

## Context

When the `provision` command completes, the user receives a new IP address. If they have configured domains for any services, they need to update their DNS records to point to this IP. Currently, this information is only shown in the `show` command output when TLS proxy is enabled.

**Current behavior:**

- `show` command displays DNS setup info only when `use_tls_proxy: true`
- `provision` command shows the IP but no DNS reminder

**Proposed behavior:**

- `provision` command shows a DNS reminder when ANY service has a domain configured (with or without TLS)
- The reminder is proactive, appearing when the IP is first assigned

## Why This Matters

- The IP is new at `provision` time - user hasn't had time to configure DNS yet
- It's a proactive reminder, not a troubleshooting message
- Prevents confusion when services fail due to DNS misconfiguration
- Domains are optional, so the reminder should only appear when domains are actually configured

## Proposed Output

When at least one domain is configured in the environment:

```text
✅ Environment 'my-environment' provisioned successfully

Instance Connection Details:
  IP Address:        XX.XX.XX.XX
  ...

⚠️  DNS Setup Required:
  Your configuration uses custom domains. Remember to update your DNS records
  to point your domains to the server IP: XX.XX.XX.XX

  Configured domains:
    - http.tracker.example.com
    - api.tracker.example.com
    - grafana.example.com
```

## Implementation Notes

- Check all services for domain configuration (HTTP trackers, API, health check, Grafana)
- Collect all unique domains
- Only show the reminder if at least one domain is configured
- Message should be informative but not alarming (it's expected that DNS isn't set up yet)
- **List all domains** - helps users know exactly what to configure
- **Reuse existing view pattern** - The DNS setup view from `show` command can be adapted
- **No TLS differentiation needed** - DNS setup is the same whether using HTTP or HTTPS

## 🏗️ Architecture Requirements

**DDD Layers**: Application + Presentation

**Module Paths**:

- `src/application/command_handlers/show/info/tracker.rs` - Domain information extraction (reuse `ServiceInfo`, `TlsDomainInfo`)
- `src/presentation/controllers/provision/handler.rs` - Add DNS reminder display after connection details
- `src/presentation/views/commands/provision/dns_reminder.rs` - New view for DNS setup reminder (similar to `show/environment_info/https_hint.rs`)

**Patterns**:

- **Application Layer**: Reuse existing `ServiceInfo::from_tracker_config()` to extract domain information
- **Presentation Layer (MVC)**:
  - Controller (`provision/handler.rs`) orchestrates the workflow
  - View (`provision/dns_reminder.rs`) renders the DNS reminder output
- **View Reusability**: Follow the same pattern as `show` command's `HttpsHintView` but adapted for all domains (not just TLS)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (presentation depends on application, not vice versa)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Follow MVC pattern: Controller orchestrates, View renders

### Architectural Constraints

- [ ] No business logic in presentation layer - domain extraction stays in application layer
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Output handling follows project conventions (see [docs/contributing/output-handling.md](../contributing/output-handling.md)) - use `UserOutput` methods, never `println!`

### Anti-Patterns to Avoid

- ❌ Duplicating domain extraction logic - reuse `ServiceInfo`
- ❌ Mixing concerns across layers - keep view rendering in presentation
- ❌ Direct stdout/stderr access - always use `UserOutput`

## Implementation Plan

### Phase 1: Create DNS Reminder View

- [ ] Create `src/presentation/views/commands/provision/dns_reminder.rs`
- [ ] Add `DnsReminderView` struct with `render()` method
- [ ] Extract all domains from `ServiceInfo` (HTTP trackers, API, health check, Grafana)
- [ ] Format output similar to proposed output above
- [ ] Add unit tests for the view

### Phase 2: Integrate into Provision Controller

- [ ] Update `src/presentation/controllers/provision/handler.rs`
- [ ] After displaying connection details, extract domain information
- [ ] Use `ServiceInfo::from_tracker_config()` to get domains
- [ ] Call `DnsReminderView::render()` if domains are present
- [ ] Display using `UserOutput` methods

### Phase 3: Update Show Command (Optional Enhancement)

- [ ] Update `show` command to display DNS info for ALL domains (not just TLS)
- [ ] Remove TLS-only restriction from `HttpsHintView`
- [ ] Reuse DNS reminder view logic

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All unit tests pass for new view
- [ ] E2E tests pass (no regressions in provision command)

**Task-Specific Criteria**:

- [ ] DNS reminder appears after `provision` command when domains are configured
- [ ] DNS reminder lists ALL configured domains (HTTP trackers, API, health check, Grafana)
- [ ] DNS reminder does NOT appear when no domains are configured
- [ ] Output format matches proposed output (warning icon, clear message, domain list)
- [ ] View follows MVC pattern (controller orchestrates, view renders)
- [ ] No business logic in presentation layer
- [ ] Uses `UserOutput` methods (no direct `println!`)

## Reference Outputs

See captured command outputs for comparison:

- Without TLS: [lxd-local-example.md](../reference/command-outputs/lxd-local-example.md)
- With TLS: [lxd-local-https-example.md](../reference/command-outputs/lxd-local-https-example.md)

## Related

- Issue: [#332](https://github.com/torrust/torrust-tracker-deployer/issues/332)
- Roadmap: [#1](https://github.com/torrust/torrust-tracker-deployer/issues/1) (Section 10.1 - Improve usability)
- Specification: docs/issues/332-dns-setup-reminder-in-provision-command.md
