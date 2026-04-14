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

- [x] Execute the complete release process for `v0.1.0-beta.1`
- [x] Verify Docker image is published and pullable from Docker Hub
- [x] Verify all 4 crates are published and visible on crates.io
- [~] Verify docs.rs builds the published crates (all 4 were 404 at last check; expected propagation delay — verify manually)
- [x] Verify GitHub release is created and accessible
- [x] Document every friction point, error, or inconsistency encountered (see Execution Log below)
- [x] Fix any release-process issues or documentation gaps found during execution
- [ ] Leave `docs/release-process.md` accurate for future releases (follow-up task)

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (CI/CD, release automation), Documentation
**Module Path**: `docs/`, `.github/workflows/`, `Cargo.toml` manifests
**Pattern**: Release workflow and operational guide

### Module Structure Requirements

- [x] Version updates in all four `Cargo.toml` manifests (`/`, `packages/deployer-types/`, `packages/dependency-installer/`, `packages/sdk/`)
- [ ] Any documentation fixes go in `docs/release-process.md` or the release skill

### Architectural Constraints

- [x] Release order from `docs/release-process.md` must be followed exactly
- [x] Tag and branch naming must follow established conventions (`vX.Y.Z`, `releases/vX.Y.Z`)
- [x] Docker tag must use bare semver (`0.1.0-beta.1`, no `v` prefix)
- [x] Pre-release versions are valid on both crates.io and Docker Hub
- [x] Any workflow fix must not break existing `main` or `develop` behavior

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

**Published crates** (all four workspace members, in dependency order):

1. `torrust-tracker-deployer-types` (`packages/deployer-types/`)
2. `torrust-tracker-deployer-dependency-installer` (`packages/dependency-installer/`)
3. `torrust-tracker-deployer` (workspace root)
4. `torrust-tracker-deployer-sdk` (`packages/sdk/`)

> **Note**: The original spec listed only the SDK crate. All four workspace members are
> publishable and must be released together because each depends on the previous in
> the list. This was discovered during execution.

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

**crates.io token scope** (new — discovered during execution):

The token must have `publish-new` and `publish-update` permissions for **all four**
crate names. A token scoped to only one crate name will cause 403 Forbidden for
the others. Regenerating the token is the only fix.

**Permissions:**

- Releaser can push to `torrust/main`, tags, and release branches

### 3. Release Execution Steps

Follow `docs/release-process.md` exactly:

1. Update `version` in all four `Cargo.toml` manifests to `0.1.0-beta.1`
2. Ensure internal path dependencies also declare an explicit `version` constraint (crates.io requirement)
3. Ensure all publishable crates have `repository` and `readme` fields in their manifests
4. Run `cargo build && cargo test` to verify workspace health
5. Commit `Cargo.lock` together with the version bump commit
6. Commit: `git commit -S -m "release: version v0.1.0-beta.1"`
7. Push to `main`: `git push origin main`
8. Create annotated signed tag: `git tag -s -a v0.1.0-beta.1 -m "Release v0.1.0-beta.1"`
9. Push tag: `git push origin v0.1.0-beta.1`
10. Create release branch: `git checkout -b releases/v0.1.0-beta.1`
11. Push release branch: `git push origin releases/v0.1.0-beta.1`
12. Monitor Container and Publish Crate workflows
13. Create GitHub release from tag `v0.1.0-beta.1`

### 4. Artifact Verification

Artifacts must be verified after CI completes, not assumed:

**Docker image:**

```bash
docker pull torrust/tracker-deployer:0.1.0-beta.1
docker image inspect torrust/tracker-deployer:0.1.0-beta.1
docker run --rm --entrypoint tofu torrust/tracker-deployer:0.1.0-beta.1 version
```

**All four crates (run for each crate name):**

```bash
for crate in \
  torrust-tracker-deployer-types \
  torrust-tracker-deployer-dependency-installer \
  torrust-tracker-deployer \
  torrust-tracker-deployer-sdk; do
  status=$(curl -s -o /dev/null -w "%{http_code}" \
    "https://crates.io/api/v1/crates/$crate/0.1.0-beta.1")
  echo "$crate: HTTP $status"
done
```

**docs.rs** (may take minutes to hours after first publish — non-fatal):

- `https://docs.rs/torrust-tracker-deployer-types/0.1.0-beta.1`
- `https://docs.rs/torrust-tracker-deployer-dependency-installer/0.1.0-beta.1`
- `https://docs.rs/torrust-tracker-deployer/0.1.0-beta.1`
- `https://docs.rs/torrust-tracker-deployer-sdk/0.1.0-beta.1`

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

- [x] Task 1.1: Verify local workspace is clean and on `torrust/main`
- [x] Task 1.2: Verify GitHub environments `dockerhub-torrust` and `crates-io` are configured
- [x] Task 1.3: Confirm releaser has required push permissions
- [x] Task 1.4: Document any pre-flight issues found

### Phase 2: Execute the Release (estimated time: 1-2 hours)

- [x] Task 2.1: Update `version` to `0.1.0-beta.1` in all four `Cargo.toml` files
- [x] Task 2.2: Run `cargo build && cargo test` and verify they pass
- [x] Task 2.3: Create and push the signed release commit to `main`
- [x] Task 2.4: Create and push annotated signed tag `v0.1.0-beta.1`
- [x] Task 2.5: Create and push release branch `releases/v0.1.0-beta.1`
- [x] Task 2.6: Monitor Container and Publish Crate workflows to completion

### Phase 3: Artifact Verification (estimated time: 30 minutes)

- [x] Task 3.1: Pull and inspect Docker image `torrust/tracker-deployer:0.1.0-beta.1`
- [x] Task 3.2: Verify all four crates `@0.1.0-beta.1` are on crates.io (HTTP 200 confirmed)
- [~] Task 3.3: Verify docs.rs build pages — all returned 404 at last check (expected propagation delay; verify manually)
- [x] Task 3.4: Create GitHub release from tag `v0.1.0-beta.1`

### Phase 4: Process Review and Cleanup (estimated time: 1 hour)

- [x] Task 4.1: Collect all issues and friction points encountered during execution (see Execution Log)
- [ ] Task 4.2: Fix small documentation inconsistencies in `docs/release-process.md`
- [ ] Task 4.3: Fix small skill inaccuracies in `release-new-version/skill.md`
- [ ] Task 4.4: File follow-up issues for any non-trivial problems found
- [x] Task 4.5: Update release finalization gates to confirm all pass

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Release Execution**:

- [x] Version `0.1.0-beta.1` is committed and present in all four `Cargo.toml` files on `main`
- [x] Tag `v0.1.0-beta.1` exists and is signed
- [x] Branch `releases/v0.1.0-beta.1` exists
- [x] Container workflow completed successfully
- [x] Publish Crate workflow completed: all four crates published (post-publish visibility step failed due to slow indexing, but publish itself succeeded — fixed in commit `c962e242`)
- [x] GitHub release `v0.1.0-beta.1` is published at https://github.com/torrust/torrust-tracker-deployer/releases/tag/v0.1.0-beta.1

**Artifact Verification**:

- [x] Docker image `torrust/tracker-deployer:0.1.0-beta.1` pulled successfully
- [x] All four crates at `0.1.0-beta.1` returned HTTP 200 from crates.io API
- [~] docs.rs pages were 404 at last check (expected build propagation delay)

**Process Quality**:

- [x] All issues found during the release are documented in the Execution Log section
- [ ] `docs/release-process.md` reflects any corrections made (follow-up)
- [x] No step was silently skipped or improvised without documentation

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

---

## Execution Log

This section records every friction point, error, and fix encountered during the
`v0.1.0-beta.1` beta release. It is the primary input for updating
`docs/release-process.md` and the release skill.

### Issue 1 — Workflow trigger: `paths` filter blocked release branch push

**What happened**: Both `container.yaml` and `publish-crate.yaml` had a `paths:`
filter on their `push` triggers. A release branch push with no matching file changes
did not trigger the workflows.

**Fix**: Removed the `paths:` filter from the release branch `push` trigger in both
workflow files. The `paths:` filter is only appropriate for `develop` and `main`
branches where builds should be skipped on docs-only changes.

**Lesson**: Release branch triggers must never use a `paths:` filter. Any push to a
release branch should unconditionally run release workflows.

### Issue 2 — Version regex rejected pre-release suffix

**What happened**: Both workflows extracted the version from the branch name
`releases/v0.1.0-beta.1` using a regex that did not allow the `-beta.1` suffix.
The version extraction step failed immediately.

**Fix**: Updated the regex in both workflows from a strict semver pattern to one that
accepts an optional pre-release segment:

```text
^releases/v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[a-zA-Z0-9][a-zA-Z0-9.-]*)?$
```

**Lesson**: Version regex in release workflows must accept pre-release suffixes
(`-alpha.1`, `-beta.1`, `-rc.1`) from the start.

### Issue 3 — Spec assumed only SDK crate; all four workspace crates must be published

**What happened**: The original spec mentioned only `torrust-tracker-deployer-sdk` as
the target crate. In practice, the SDK depends on `torrust-tracker-deployer-types`,
`torrust-tracker-deployer-dependency-installer`, and the root
`torrust-tracker-deployer` crate — all of which must also be published to crates.io
before the SDK can be published.

**Fix**: Rewrote `publish-crate.yaml` to publish all four crates in dependency order
with a dry-run for each crate executed after its prerequisites are available on
crates.io:

1. `torrust-tracker-deployer-types`
2. `torrust-tracker-deployer-dependency-installer`
3. `torrust-tracker-deployer`
4. `torrust-tracker-deployer-sdk`

**Lesson**: When a workspace has a publish chain, all members of the chain must be
published together. Map the full dependency graph before first publish.

### Issue 4 — Path dependencies missing explicit `version` constraint

**What happened**: `packages/sdk/Cargo.toml` referenced internal crates with
`path = "..."` only, without a `version` field. crates.io rejects packages that
declare path-only dependencies because path dependencies cannot be resolved by
consumers downloading the crate from the registry.

**Fix**: Added `version = "0.1.0-beta.1"` to every internal path dependency in all
four manifests.

**Lesson**: Every internal path dependency in a publishable crate must also declare
an explicit version constraint. This is enforced at `cargo publish` time.

### Issue 5 — Missing required crates.io metadata fields

**What happened**: `cargo publish --dry-run` failed because `repository` and `readme`
fields were absent from the manifests of the internal crates.

**Fix**: Added `repository` and `readme` metadata to all four `Cargo.toml` files.

**Lesson**: Before the first publish of any crate, verify that `description`,
`license`, `repository`, and `readme` are all present.

### Issue 6 — `CARGO_REGISTRY_TOKEN` scoped to only one crate caused 403 Forbidden

**What happened**: The initial `CARGO_REGISTRY_TOKEN` stored in the `crates-io`
GitHub Environment was scoped to `torrust-tracker-deployer-sdk` only. Publishing
`torrust-tracker-deployer-types` returned HTTP 403 Forbidden.

**Fix**: The token was regenerated on crates.io with `publish-new` and
`publish-update` permissions covering all four crate names, then stored back in the
GitHub Environment secret.

**Lesson**: The crates.io token must explicitly list every crate name it will publish.
This must be a pre-flight check for every release.

### Issue 7 — Internal crate names did not match the project namespace

**What happened**: The two internal crates were originally named
`torrust-deployer-types` and `torrust-dependency-installer` — inconsistent with the
`torrust-tracker-deployer-*` namespace used by the other crates.

**Fix**: Renamed before first publication (since crates.io names are permanent once
published):

- `torrust-deployer-types` → `torrust-tracker-deployer-types`
- `torrust-dependency-installer` → `torrust-tracker-deployer-dependency-installer`

The rename was applied across all source files, docs, and workflows in PR #452
(~65 files) and merged to `main` before the release branch was pushed.

**Lesson**: Audit all crate names against the project namespace convention before
any publish. Renaming after first publish is impossible.

### Issue 8 — Dry-run must run after prerequisites are indexed, not before

**What happened**: The initial workflow ran `cargo publish --dry-run` for the main
crate before `torrust-tracker-deployer-types` and
`torrust-tracker-deployer-dependency-installer` were indexed on crates.io. The
dry-run failed because it could not resolve the dependencies from the registry.

**Fix**: Restructured the workflow so each crate's dry-run step runs only after its
prerequisite crates have been published and the index has been polled for
availability.

**Lesson**: In a multi-crate publish chain, interleave dry-runs between publishes:
publish A → wait → dry-run B → publish B → wait → dry-run C → publish C.

### Issue 9 — Post-publish visibility polling timed out (50 seconds is too short)

**What happened**: The `Verify SDK Is Available on crates.io` step polled with
5 attempts × 10 seconds = 50 seconds. After publishing four new crates in sequence,
crates.io indexing took longer, so all five attempts returned non-200. The workflow
reported `failure` even though all four crates were actually published.

**Fix** (commit `c962e242`): Increased from 5×10s to 18×20s (6 minutes total).

**Lesson**: For first-time crate publication, especially multi-crate chains, allow
at least 5–10 minutes for indexing. Use generous retry windows.

### Issue 10 — docs.rs verification was fatal; docs.rs builds take minutes to hours

**What happened**: The `Verify docs.rs Build for SDK` step used `exit 1` when the
page was not available after 6×20s = 2 minutes. A single docs.rs build can take
well over 2 minutes, especially on a first publish.

**Fix** (commit `c962e242`): Changed `exit 1` to `exit 0` with a warning message
that logs the URL for manual verification.

**Lesson**: docs.rs availability is an informational check, not a correctness gate.
Make it non-fatal.

### Issue 11 — Dynamic container job name broke PR check display

**What happened**: The container workflow job name was
`Publish (${{ needs.context.outputs.type }})`, which rendered as the literal string
`Publish (${{ needs.context.outputs.type }})` in GitHub PR checks.

**Fix**: Renamed the job to the static string `Publish Image`.

**Lesson**: GitHub Actions job names used as PR status check names must be static
strings. Dynamic expressions in job names produce broken check names in the
GitHub UI.

### Issue 12 — `Cargo.lock` was not committed with the version bump

**What happened**: The version bump commit did not include the updated `Cargo.lock`,
causing the lock file to be out of sync on the release branch.

**Fix**: Committed `Cargo.lock` together with the version bump.

**Lesson**: Always stage and commit `Cargo.lock` as part of the version bump commit.

### Summary Table

| #   | Category                           | Severity           | Fixed            | Follow-up needed                 |
| --- | ---------------------------------- | ------------------ | ---------------- | -------------------------------- |
| 1   | Workflow trigger `paths` filter    | Blocking           | Yes              | Document in `release-process.md` |
| 2   | Version regex rejected pre-release | Blocking           | Yes              | Document in `release-process.md` |
| 3   | Spec: only SDK crate listed        | Blocking           | Yes              | Update spec and docs             |
| 4   | Path deps missing `version` field  | Blocking           | Yes              | Add to pre-flight checklist      |
| 5   | Missing crates.io metadata fields  | Blocking           | Yes              | Add to pre-flight checklist      |
| 6   | Token scoped to wrong crate        | Blocking           | Yes              | Add to pre-flight checklist      |
| 7   | Crate names didn't match namespace | High (pre-publish) | Yes (PR #452)    | Add name audit to pre-flight     |
| 8   | Dry-run before deps indexed        | Blocking           | Yes              | Document ordering rule           |
| 9   | Visibility poll only 50 seconds    | Non-blocking       | Yes (`c962e242`) | —                                |
| 10  | docs.rs check was fatal            | False failure      | Yes (`c962e242`) | —                                |
| 11  | Dynamic job name in container      | UI cosmetic        | Yes (PR #452)    | —                                |
| 12  | `Cargo.lock` not in version commit | Low                | Yes              | Add to release steps             |
