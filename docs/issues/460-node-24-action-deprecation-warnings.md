# Update GitHub Actions to Node.js 24 Compatible Versions

**Issue**: #460
**Parent Epic**: N/A
**Related**: N/A

## Overview

Several GitHub Actions workflows produce deprecation warnings because some actions
still run on Node.js 20. Starting **June 2nd, 2026**, GitHub will force all actions
to run with Node.js 24 by default, and Node.js 20 will be removed from runners on
**September 16th, 2026**.

Each affected action needs to be reviewed: in some cases a newer version with
Node.js 24 support exists and can be adopted; in other cases no compatible release
exists yet and the issue must be tracked until one does.

Reference: <https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/>

## Goals

- [ ] Identify which affected actions have Node.js 24-compatible releases available
- [ ] Update workflow files to use compatible versions where possible
- [ ] Track actions that have no compatible release yet, and re-check periodically
- [ ] Eliminate all Node.js 20 deprecation warnings from CI runs

## Affected Actions by Workflow

### `backup-container.yaml` — Backup Container

| Action                       | Current Version | Node.js 24? |
| ---------------------------- | --------------- | ----------- |
| `docker/setup-buildx-action` | `@v3`           | TBD         |
| `docker/build-push-action`   | `@v6`           | TBD         |
| `docker/login-action`        | `@v3`           | TBD         |
| `docker/metadata-action`     | `@v5`           | TBD         |

### `container.yaml` — Container

| Action                       | Current Version | Node.js 24? |
| ---------------------------- | --------------- | ----------- |
| `docker/setup-buildx-action` | `@v3`           | TBD         |
| `docker/build-push-action`   | `@v6`           | TBD         |
| `docker/login-action`        | `@v3`           | TBD         |
| `docker/metadata-action`     | `@v5`           | TBD         |

### `cargo-security-audit.yml` — Cargo Security Audit

| Action                | Current Version | Node.js 24? |
| --------------------- | --------------- | ----------- |
| `rustsec/audit-check` | `@v2.0.0`       | TBD         |

### `docker-security-scan.yml` — Docker Security Scan

| Action                      | Current Version | Node.js 24? |
| --------------------------- | --------------- | ----------- |
| `aquasecurity/trivy-action` | `@0.35.0`       | TBD         |

> **Note**: The warning in this workflow shows `actions/cache@0400d5f...` running on
> Node.js 20. This is a **transitive dependency** used internally by
> `aquasecurity/trivy-action`. Updating Trivy to a newer release should resolve it.

### `test-e2e-deployment.yml` — E2E Deployment Workflow Tests

| Action                       | Current Version | Node.js 24? |
| ---------------------------- | --------------- | ----------- |
| `docker/setup-buildx-action` | `@v3`           | TBD         |

### `dependabot-updates` — Dependabot (GitHub-managed)

| Action                     | Current Version | Node.js 24? |
| -------------------------- | --------------- | ----------- |
| `github/dependabot-action` | `@main`         | TBD         |

> **Note**: This workflow is **managed entirely by GitHub** and is not present in
> this repository. We cannot update it directly. The warning may resolve
> automatically when GitHub updates their internal Dependabot runner, or it may
> require a GitHub support request.

## Implementation Plan

### Phase 1: Research available updates

- [ ] Check latest releases of `docker/setup-buildx-action`, `docker/build-push-action`, `docker/login-action`, `docker/metadata-action` for Node.js 24 support
- [ ] Check latest release of `rustsec/audit-check` for Node.js 24 support
- [ ] Check latest release of `aquasecurity/trivy-action` for Node.js 24 support (resolves transitive `actions/cache` warning)
- [ ] Investigate `github/dependabot-action` — determine if this is fully GitHub-managed and no action is needed from our side

### Phase 2: Apply available updates

- [ ] Update all docker action versions in `backup-container.yaml` where newer Node.js 24 compatible versions are available
- [ ] Update all docker action versions in `container.yaml` where newer Node.js 24 compatible versions are available
- [ ] Update `docker/setup-buildx-action` in `test-e2e-deployment.yml`
- [ ] Update `rustsec/audit-check` in `cargo-security-audit.yml`
- [ ] Update `aquasecurity/trivy-action` in `docker-security-scan.yml`

### Phase 3: Handle actions with no available update

- [ ] For any action without a Node.js 24-compatible release, open a follow-up tracking note or issue
- [ ] Document the status and re-check schedule

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] No Node.js 20 deprecation warnings appear in any of the affected workflow runs
- [ ] All updated action versions are pinned correctly and tested
- [ ] Any action that cannot be updated is documented with a follow-up plan

## Related Documentation

- [GitHub blog: Deprecation of Node 20 on Actions Runners](https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/)
- Affected workflow runs:
  - [backup-container.yaml run #24191868780](https://github.com/torrust/torrust-tracker-deployer/actions/runs/24191868780)
  - [cargo-security-audit.yml run #24455465380](https://github.com/torrust/torrust-tracker-deployer/actions/runs/24455465380)
  - [container.yaml run #24455465394](https://github.com/torrust/torrust-tracker-deployer/actions/runs/24455465394)
  - [dependabot-updates run #24389583837](https://github.com/torrust/torrust-tracker-deployer/actions/runs/24389583837)
  - [docker-security-scan.yml run #24445697392](https://github.com/torrust/torrust-tracker-deployer/actions/runs/24445697392)
  - [test-e2e-deployment.yml run #24455481734](https://github.com/torrust/torrust-tracker-deployer/actions/runs/24455481734)

## Notes

- The `docker/*` actions appear in both `backup-container.yaml` and `container.yaml` with the same versions. They should be updated together.
- The `dependabot-updates` warning may resolve itself without any action on our part — GitHub is likely already working on updating the internal runner.
- Setting `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` in the workflow environment is available as a temporary opt-in to test compatibility before the forced migration.
