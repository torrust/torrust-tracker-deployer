# Release v0.1.0-beta.1: End-to-End Process Validation

**Issue**: #450
**Parent Epic**: N/A (standalone release task)
**Related**:

- `docs/release-process.md`
- `.github/skills/dev/git-workflow/release-new-version/skill.md`
- Issue #448 — Define release process (merged)

## Overview

Execute the first real end-to-end release of the Torrust Tracker Deployer following
the release process defined in issue #448. Version `0.1.0-beta.1` is the first
pre-release version and serves as the practical validation of the entire release
workflow — from version bump to artifact verification.

This task is intentionally broader than a normal release. It treats the release
itself as an audit surface: every step where the documentation is inaccurate,
incomplete, or misleading must be captured and fixed before closing the issue.
The goal is a release process that is fully trustworthy for future stable releases.

## Goals

- [ ] Execute the complete release process for `v0.1.0-beta.1`
- [ ] Verify Docker image is published and pullable from Docker Hub
- [ ] Verify SDK crate is published and visible on crates.io
- [ ] Verify docs.rs builds the published crate
- [ ] Verify GitHub release is created and accessible
- [ ] Document every friction point, error, or inconsistency encountered
- [ ] Fix any release-process issues or documentation gaps found during execution
- [ ] Leave `docs/release-process.md` accurate for future releases

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (CI/CD, release automation), Documentation
**Module Path**: `docs/`, `.github/workflows/`, `Cargo.toml` manifests
**Pattern**: Release workflow and operational guide

### Module Structure Requirements

- [ ] Version updates limited to `Cargo.toml` and `packages/sdk/Cargo.toml`
- [ ] Any documentation fixes go in `docs/release-process.md` or the release skill

### Architectural Constraints

- [ ] Release order from `docs/release-process.md` must be followed exactly
- [ ] Tag and branch naming must follow established conventions (`vX.Y.Z`, `releases/vX.Y.Z`)
- [ ] Docker tag must use bare semver (`0.1.0-beta.1`, no `v` prefix)
- [ ] Pre-release versions are valid on both crates.io and Docker Hub
- [ ] Any workflow fix must not break existing `main` or `develop` behavior

### Anti-Patterns to Avoid

- ❌ Skipping artifact verification and declaring the release done on tag push alone
- ❌ Silently skipping problems found during the release without filing follow-up issues
- ❌ Making workflow changes without verifying the full release pipeline
- ❌ Bumping past `0.1.0-beta.1` to cover up issues instead of documenting them

## Specifications

### 1. Version to Release

- **Version string**: `0.1.0-beta.1`
- **Git tag**: `v0.1.0-beta.1`
- **Release branch**: `releases/v0.1.0-beta.1`
- **Docker tag**: `0.1.0-beta.1` (no `v` prefix)
- **Crate version**: `0.1.0-beta.1`

Pre-release suffixes (`-beta.1`) are valid semver and supported by crates.io and Docker Hub.

### 2. Pre-Flight Checks

Before executing any release step, the following must be confirmed:

**Git state:**

- Clean working tree on `main` that is up to date with `torrust/main`
- The merge commit of PR #448 must be on `main`

**GitHub Environments:**

- `dockerhub-torrust` environment exists with correct credentials:
  - `DOCKER_HUB_ACCESS_TOKEN` (secret)
  - `DOCKER_HUB_USERNAME` (variable = `torrust`)
- `crates-io` environment exists with:
  - `CARGO_REGISTRY_TOKEN` (secret)

**Permissions:**

- Releaser can push to `torrust/main`, tags, and release branches

### 3. Release Execution Steps

Follow `docs/release-process.md` exactly:

1. Update `version` in `Cargo.toml` and `packages/sdk/Cargo.toml` to `0.1.0-beta.1`
2. Run `cargo build && cargo test` to verify workspace health
3. Commit: `git commit -S -m "release: version v0.1.0-beta.1"`
4. Push to `main`: `git push origin main`
5. Create annotated signed tag: `git tag -s -a v0.1.0-beta.1 -m "Release v0.1.0-beta.1"`
6. Push tag: `git push origin v0.1.0-beta.1`
7. Create release branch: `git checkout -b releases/v0.1.0-beta.1`
8. Push release branch: `git push origin releases/v0.1.0-beta.1`
9. Monitor Container and Publish Crate workflows
10. Create GitHub release from tag `v0.1.0-beta.1`

### 4. Artifact Verification

Artifacts must be verified after CI completes, not assumed:

**Docker image:**

```bash
docker pull torrust/tracker-deployer:0.1.0-beta.1
docker image inspect torrust/tracker-deployer:0.1.0-beta.1
docker run --rm --entrypoint tofu torrust/tracker-deployer:0.1.0-beta.1 version
```

**Crate:**

```bash
curl -sf "https://crates.io/api/v1/crates/torrust-tracker-deployer-sdk/0.1.0-beta.1" | jq '.version.num'
```

**docs.rs:**

- Confirm build passes at: `https://docs.rs/torrust-tracker-deployer-sdk/0.1.0-beta.1`

**GitHub release:**

- Confirm it exists and is published (not draft) at `https://github.com/torrust/torrust-tracker-deployer/releases/tag/v0.1.0-beta.1`

### 5. Process Review and Improvement

During execution, document all friction points, errors, and documentation gaps in
a dedicated section of this issue or directly in `docs/release-process.md`.

Categories to watch for:

- Missing or inaccurate steps in the release guide
- Workflow failures due to incorrect configuration
- Environment/permission issues not covered by the pre-flight checklist
- Timing issues (e.g., crates.io indexing delay, docs.rs build delay)
- Any step that required improvising beyond what the docs describe

For each issue found: fix it inline if small (typo, clarification), or file a
follow-up issue if it requires non-trivial work.

## Implementation Plan

### Phase 1: Pre-Flight and Setup (estimated time: 30 minutes)

- [ ] Task 1.1: Verify local workspace is clean and on `torrust/main`
- [ ] Task 1.2: Verify GitHub environments `dockerhub-torrust` and `crates-io` are configured
- [ ] Task 1.3: Confirm releaser has required push permissions
- [ ] Task 1.4: Document any pre-flight issues found

### Phase 2: Execute the Release (estimated time: 1-2 hours)

- [ ] Task 2.1: Update `version` to `0.1.0-beta.1` in both `Cargo.toml` files
- [ ] Task 2.2: Run `cargo build && cargo test` and verify they pass
- [ ] Task 2.3: Create and push the signed release commit to `main`
- [ ] Task 2.4: Create and push annotated signed tag `v0.1.0-beta.1`
- [ ] Task 2.5: Create and push release branch `releases/v0.1.0-beta.1`
- [ ] Task 2.6: Monitor Container and Publish Crate workflows to completion

### Phase 3: Artifact Verification (estimated time: 30 minutes)

- [ ] Task 3.1: Pull and inspect Docker image `torrust/tracker-deployer:0.1.0-beta.1`
- [ ] Task 3.2: Verify crate `torrust-tracker-deployer-sdk@0.1.0-beta.1` is on crates.io
- [ ] Task 3.3: Verify docs.rs build for the published crate version
- [ ] Task 3.4: Create GitHub release from tag `v0.1.0-beta.1`

### Phase 4: Process Review and Cleanup (estimated time: 1 hour)

- [ ] Task 4.1: Collect all issues and friction points encountered during execution
- [ ] Task 4.2: Fix small documentation inconsistencies in `docs/release-process.md`
- [ ] Task 4.3: Fix small skill inaccuracies in `release-new-version/skill.md`
- [ ] Task 4.4: File follow-up issues for any non-trivial problems found
- [ ] Task 4.5: Update release finalization gates to confirm all pass

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Release Execution**:

- [ ] Version `0.1.0-beta.1` is committed and present in both `Cargo.toml` files on `main`
- [ ] Tag `v0.1.0-beta.1` exists and is signed
- [ ] Branch `releases/v0.1.0-beta.1` exists
- [ ] Container workflow completed successfully
- [ ] Publish Crate workflow completed successfully
- [ ] GitHub release `v0.1.0-beta.1` is published (not draft)

**Artifact Verification**:

- [ ] Docker image `torrust/tracker-deployer:0.1.0-beta.1` can be pulled and run
- [ ] Crate `torrust-tracker-deployer-sdk@0.1.0-beta.1` is visible on crates.io
- [ ] docs.rs build page loads for the published version

**Process Quality**:

- [ ] All issues found during the release are documented (inline or filed as follow-ups)
- [ ] `docs/release-process.md` reflects any corrections made
- [ ] No step was silently skipped or improvised without documentation

## Related Documentation

- `docs/release-process.md`
- `.github/workflows/container.yaml`
- `.github/workflows/publish-crate.yaml`
- `.github/skills/dev/git-workflow/release-new-version/skill.md`
- Issue #448 — release process definition (merged)

## Notes

- This is the first real execution of the release process. Treat it as an audit.
- Pre-release semver (`0.1.0-beta.1`) is valid for crates.io and Docker Hub.
- If a blocking issue is found that cannot be fixed quickly, pause the release, file an issue, and continue when resolved — do not rush past it.
- The next release after this will be a stable `0.1.0` once the process is validated.
