# Automate Cargo Audit Security Scanning and Dependency Remediation

**Issue**: #439
**Parent Epic**: N/A (standalone security task)
**Related**:

- Existing Docker security workflow: `.github/workflows/docker-security-scan.yml`
- Docker scan reports index: `docs/security/docker/scans/README.md`
- RustSec audit action: https://github.com/rustsec/audit-check

## Overview

Introduce a Rust dependency security workflow based on `cargo audit` that runs periodically and can be triggered manually, document scan results under `docs/security/dependencies`, and remediate vulnerabilities where feasible.

This task also defines a clear process for unresolved findings: if vulnerabilities cannot be fixed quickly (for example, blocked by upstream releases), create follow-up issues with context, impact, and tracking details.

## Goals

- [ ] Add an automated GitHub Actions workflow for periodic Rust dependency security scans
- [ ] Produce a manually generated dependency security report in `docs/security/dependencies`
- [ ] Fix dependency vulnerabilities where updates or replacements are available and safe
- [ ] Open follow-up issue(s) for findings blocked by upstream or high-effort refactors

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (CI/CD), Documentation, and dependency management
**Module Path**: `.github/workflows/`, `docs/security/dependencies/`, Cargo workspace manifests and lockfile
**Pattern**: Security scanning workflow + remediation and tracking process

### Module Structure Requirements

- [ ] Keep CI logic inside `.github/workflows/`
- [ ] Keep human-readable scan reports under `docs/security/dependencies/`
- [ ] Keep dependency updates consistent across workspace crates
- [ ] Reference related issue(s) and report files for traceability

### Architectural Constraints

- [ ] Workflow should support periodic scanning and manual execution (`workflow_dispatch`)
- [ ] Workflow should follow the style and clarity of `.github/workflows/docker-security-scan.yml`
- [ ] Dependency reports must be reproducible from documented commands
- [ ] Vulnerability handling must be actionable (fix now or tracked follow-up)

### Anti-Patterns to Avoid

- ❌ Silent failures where scan output is not discoverable
- ❌ Ad-hoc local-only fixes without documentation
- ❌ Leaving unresolved vulnerabilities without a tracking issue
- ❌ Mixing unrelated refactors into security remediation commits

## Specifications

### 1. Add Scheduled Cargo Audit Workflow

Create a new workflow in `.github/workflows/` that:

- Runs on `schedule` (periodic scans), `workflow_dispatch`, and optionally on dependency file changes (`Cargo.toml`, `Cargo.lock`)
- Uses `RustSec/audit-check@v2.0.0` (or current stable release) with `token: ${{ secrets.GITHUB_TOKEN }}`
- Uses explicit permissions required by the action (`issues: write`, `checks: write`, and minimum required repository permissions)
- Documents why scheduled execution is needed (new advisories may appear without repository changes)

Implementation notes from RustSec action documentation:

- Scheduled workflows can create issues for new advisories
- Non-scheduled runs should still fail checks when vulnerabilities are found
- The action supports `ignore` and `working-directory` inputs when needed

### 2. Generate Manual Dependency Security Report

Re-run `cargo audit` manually and document results in a new report under:

- `docs/security/dependencies/README.md` (index and process)
- One date-stamped report file (for example `docs/security/dependencies/scans/2026-04-10-cargo-audit.md`)

Report format should mirror Docker security documentation conventions:

- Scan date, tool version, command used
- Summary counts by severity/status
- Detailed findings with package, advisory ID, status, and recommended fix
- Risk notes for unmaintained crates and transitive dependencies
- Next actions and owner tracking

### 3. Remediate Security Findings

Attempt practical fixes for current findings, including:

- Upgrading vulnerable dependencies to patched versions
- Updating direct dependencies to versions that pull secure transitives
- Replacing unmaintained dependencies when viable and low-risk
- Regenerating lockfile and validating build/tests/lints after updates

Expected validation:

- `cargo audit`
- `cargo build`
- `cargo test`
- `./scripts/pre-commit.sh`

### 4. Create Follow-up Issues for Hard Blockers

If a vulnerability cannot be resolved quickly:

- Create a follow-up issue per blocker (or one grouped issue with clear subtasks)
- Include advisory ID(s), affected dependency tree, why blocked, and mitigation options
- Add review cadence and closure criteria (for example, upgrade when upstream releases fix)
- Link follow-up issue(s) from the main issue specification/report

## Implementation Plan

### Phase 1: CI Workflow Setup (estimated time: 1-2 hours)

- [ ] Task 1.1: Create `.github/workflows/cargo-audit.yml`
- [ ] Task 1.2: Configure schedule + manual trigger + permissions
- [ ] Task 1.3: Validate workflow configuration and alignment with existing workflow style

### Phase 2: Manual Security Reporting (estimated time: 1-2 hours)

- [ ] Task 2.1: Run `cargo audit` manually and capture results
- [ ] Task 2.2: Create `docs/security/dependencies/` index and scan report
- [ ] Task 2.3: Cross-link report from security documentation as needed

### Phase 3: Dependency Remediation (estimated time: 2-6 hours)

- [ ] Task 3.1: Identify direct vs transitive upgrade paths
- [ ] Task 3.2: Apply safe dependency updates/replacements
- [ ] Task 3.3: Re-run build, tests, lint, and `cargo audit`

### Phase 4: Follow-up Tracking (estimated time: 0.5-1 hour)

- [ ] Task 4.1: Create issue(s) for unresolved advisories/blockers
- [ ] Task 4.2: Link follow-up issue(s) in main issue and report docs
- [ ] Task 4.3: Document mitigation strategy and revisit timeline

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] New workflow exists and runs on schedule + manual dispatch
- [ ] Workflow uses RustSec audit action with appropriate permissions and token configuration
- [ ] Manual dependency security report exists in `docs/security/dependencies/` and follows documented format
- [ ] `cargo audit` was re-run and latest results are documented
- [ ] Feasible dependency vulnerabilities are remediated and validated
- [ ] Unresolved vulnerabilities have linked follow-up issue(s) with actionable next steps

## Related Documentation

- `docs/security/docker/scans/README.md`
- `.github/workflows/docker-security-scan.yml`
- `docs/contributing/roadmap-issues.md`
- RustSec audit-check docs: https://github.com/rustsec/audit-check

## Notes

- Keep the first implementation focused on actionable security outcomes; avoid broad CI refactoring.
- If dependency remediation impacts runtime behavior, document risk and testing scope explicitly.
