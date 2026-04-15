# Release v0.1.0: Stable End-to-End Execution

**Issue**: #462
**Parent Epic**: N/A
**Related**: [#459](https://github.com/torrust/torrust-tracker-deployer/issues/459) (beta.2 release validation),
[#450](https://github.com/torrust/torrust-tracker-deployer/issues/450) (beta.1 release validation),
[#448](https://github.com/torrust/torrust-tracker-deployer/issues/448) (release process definition),
[#452](https://github.com/torrust/torrust-tracker-deployer/pull/452) (crate rename to tracker-deployer namespace)

## Overview

Execute the first stable release of the Torrust Tracker Deployer (`v0.1.0`),
validating the full release workflow from version bump through release publication.

This release is the GA cut after beta validation. The objective is to execute the
canonical process cleanly, ensure all release artifacts are published and verifiable,
and capture any friction that still exists in the release workflow documentation,
skill instructions, or CI automation.

## Goals

- [ ] Version `0.1.0` is reflected in all four `Cargo.toml` files on `main`
- [ ] Signed tag `v0.1.0` and release branch `releases/v0.1.0` exist
- [ ] Docker image `torrust/tracker-deployer:0.1.0` is published to Docker Hub
- [ ] All four crates are published to crates.io at version `0.1.0`
- [ ] GitHub release `v0.1.0` is published (not draft)
- [ ] Any new friction points discovered are documented and filed as follow-ups

## Specifications

### Version Bump Scope

Update `version` field to `0.1.0` in all four manifests:

| File                                       | Crate                                           |
| ------------------------------------------ | ----------------------------------------------- |
| `Cargo.toml` (workspace root)              | `torrust-tracker-deployer`                      |
| `packages/deployer-types/Cargo.toml`       | `torrust-tracker-deployer-types`                |
| `packages/dependency-installer/Cargo.toml` | `torrust-tracker-deployer-dependency-installer` |
| `packages/sdk/Cargo.toml`                  | `torrust-tracker-deployer-sdk`                  |

Also update every internal path dependency `version` constraint to `0.1.0`.

### Publish Order (crates.io dependency order)

1. `torrust-tracker-deployer-types`
2. `torrust-tracker-deployer-dependency-installer`
3. `torrust-tracker-deployer`
4. `torrust-tracker-deployer-sdk`

Each crate's dry-run step in CI runs only after its prerequisites are indexed on
crates.io - do not attempt to publish out of order.

## Implementation Plan

### Phase 1: Pre-Flight and Setup

- [ ] Task 1.1: Verify local workspace is clean and on `torrust/main` up-to-date
      with `origin/main` (`git status`, `git pull --ff-only`)
- [ ] Task 1.2: Confirm GitHub environments `dockerhub-torrust` and `crates-io` are
      correctly configured for stable release publication
- [ ] Task 1.3: Confirm releaser has push access to `main`, tags, and release branches
- [ ] Task 1.4: Document any pre-flight issues found

### Phase 2: Execute the Release

- [ ] Task 2.1: Update `version` to `0.1.0` in all four `Cargo.toml` files
      and in every internal path dependency constraint
- [ ] Task 2.2: Run `cargo build && cargo test` and verify they pass
- [ ] Task 2.3: Run `./scripts/pre-commit.sh` and verify all checks pass
- [ ] Task 2.4: Create and push the signed release commit to `main`
      (`git commit -S -m "release: version v0.1.0"`)
- [ ] Task 2.5: Create and push annotated signed tag `v0.1.0`
- [ ] Task 2.6: Create and push release branch `releases/v0.1.0`
- [ ] Task 2.7: Monitor Container and Publish Crate workflows to completion

### Phase 3: Artifact Verification

- [ ] Task 3.1: Pull and inspect Docker image `torrust/tracker-deployer:0.1.0`
- [ ] Task 3.2: Verify all four crates at `0.1.0` are visible on crates.io
- [ ] Task 3.3: Verify docs.rs build for the published crate versions
- [ ] Task 3.4: Create GitHub release from tag `v0.1.0`

### Phase 4: Process Review and Fixes

- [ ] Task 4.1: Collect every friction point, error, or unexpected step encountered
      during the release - no matter how small
- [ ] Task 4.2: For each issue found, fix the root cause immediately in the relevant
      artifact:
  - `docs/release-process.md` - if a step was wrong, missing, or misleading
  - `.github/skills/dev/git-workflow/release-new-version/skill.md` - if the skill
    diverged from reality
  - `scripts/` - if a helper script failed or was absent
  - CI workflow files (`.github/workflows/`) - if a workflow behaved unexpectedly
  - Any other documentation, template, or configuration that contributed to the
    friction
- [ ] Task 4.3: File follow-up issues for any non-trivial problems that cannot be
      fixed inline (e.g., upstream blockers, larger refactors)
- [ ] Task 4.4: Re-run `./scripts/pre-commit.sh` after any fixes to confirm nothing
      was broken
- [ ] Task 4.5: Confirm all finalization gates below are met

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Release Execution**:

- [ ] Version `0.1.0` is committed and present in all four `Cargo.toml` files on `main`
- [ ] Tag `v0.1.0` exists and is signed
- [ ] Branch `releases/v0.1.0` exists
- [ ] Container workflow completed successfully
- [ ] Publish Crate workflow completed successfully
- [ ] GitHub release `v0.1.0` is published (not draft)

**Artifact Verification**:

- [ ] Docker image `torrust/tracker-deployer:0.1.0` can be pulled and run
- [ ] All four crates at `0.1.0` are visible on crates.io:
  - [ ] `torrust-tracker-deployer-types@0.1.0`
  - [ ] `torrust-tracker-deployer-dependency-installer@0.1.0`
  - [ ] `torrust-tracker-deployer@0.1.0`
  - [ ] `torrust-tracker-deployer-sdk@0.1.0`
- [ ] docs.rs build page loads for the published versions

**Process Quality**:

- [ ] Every friction point or error encountered is documented (inline comment or
      filed follow-up issue)
- [ ] Every fixable problem is fixed in the relevant artifact - documentation,
      skill, script, or workflow - not just noted
- [ ] No step was silently skipped or improvised without documentation
- [ ] `docs/release-process.md` and the release skill accurately reflect how the
      release was actually executed

## Related Documentation

- [Release Process](../release-process.md)
- [Release Skill](../../.github/skills/dev/git-workflow/release-new-version/skill.md)
- [beta.2 release issue #459](https://github.com/torrust/torrust-tracker-deployer/issues/459)

## Notes

This is a stable release (`0.1.0`), not a pre-release. Cargo users should receive
this version by default through normal semver resolution, and release notes should
highlight upgrade guidance from `0.1.0-beta.2` where relevant.
