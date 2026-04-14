# Define a Standard Release Process (Branch, Tag, Docker Image, Crate)

**Issue**: #448
**Parent Epic**: N/A (standalone release process task)
**Related**:

- `docs/contributing/roadmap-issues.md`
- `docs/contributing/commit-process.md`
- `docs/roadmap.md`

## Overview

Define and document a repeatable release process for this repository so releases are
predictable, auditable, and less error-prone.

This repository already has a container workflow in `.github/workflows/container.yaml`
that publishes Docker images for `main` and `develop`. The new release process should
extend that model to support release branches, while keeping the overall process much
simpler than the Torrust Tracker release process.

The initial release process should include these mandatory steps in order:

1. Update the version in the relevant `Cargo.toml` files
2. Commit the release version
3. Push the release commit to `main`
4. Create the release tag and push it
5. Create the release branch and push it
6. Let GitHub Actions publish release artifacts:
   - Docker image for the release branch
   - Crate for the release branch
7. Create the GitHub release from the tag

## Goals

- [x] Define a single documented release workflow with explicit step order
- [x] Make branch and tag conventions consistent across releases
- [x] Ensure Docker image publication is triggered from release branches
- [x] Ensure crate publication is triggered from release branches
- [x] Define validation and rollback guidance for failed release steps
- [x] Keep the first version of the process intentionally simpler than the tracker repository
- [x] Avoid duplicate Docker release tags for the same version

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (CI/CD and release automation), Documentation
**Module Path**: `docs/`, `.github/workflows/`, and release-related scripts if needed
**Pattern**: Release workflow and operational guide

### Module Structure Requirements

- [x] Keep process documentation in `docs/`
- [x] Keep automation in `.github/workflows/` and/or `scripts/`
- [x] Keep branch and tag naming rules explicit and testable
- [x] Keep artifact version alignment across Git tag, Docker image tag, and crate version

### Architectural Constraints

- [x] Release order must be deterministic and documented
- [x] Tag format must be clearly defined as `vX.Y.Z`
- [x] Release branch format must be clearly defined and compatible with workflow triggers
- [x] Docker publish step must support reproducible release tagging without overloading `main` publish behavior
- [x] Docker release tags must not include the Git tag `v` prefix
- [x] Crate publish step must define pre-checks and ownership requirements
- [x] Docker Hub credentials must separate secrets from non-sensitive variables
- [x] Workflow triggers and branch protections must align with allowed branches (`develop`, `main`, `releases/**/*`)

### Anti-Patterns to Avoid

- ❌ Manual ad-hoc release steps without a checklist
- ❌ Tagging and artifact versions drifting from each other
- ❌ Publishing the same Docker release twice with both `vX.Y.Z` and `X.Y.Z` tags
- ❌ Publishing artifacts without verification or rollback notes
- ❌ Coupling release steps to undocumented local machine state

## Specifications

### 1. Release Branch Strategy

Define how release branches are created, named, and finalized.

- Naming convention: `releases/vX.Y.Z`
- Source branch: create the release branch from the same commit that was pushed to `main`
- The release branch is a publication trigger, not a long-lived development branch
- The release branch name must be parseable by GitHub Actions so release version metadata can be extracted

### 2. Version Update and Release Commit

Define which manifests are updated before the release commit.

- Root `Cargo.toml` version must be updated from the current development version to the release version
- Publishable package versions must also be updated in their own manifests
- The likely first publishable crate is `packages/sdk/Cargo.toml` (`torrust-tracker-deployer-sdk`)
- The release commit should be explicit and traceable, for example: `release: version vX.Y.Z`
- Verify release metadata quality for publishable crates (`description`, `license`, `repository`, `readme`) before publishing

### 3. Tagging Strategy

Define release tag rules and when tags are created.

- Tag format: `vX.Y.Z`
- Annotated and signed tag requirements
- Tag is created from the release commit already pushed to `main`
- Tag, release branch, Docker tags, and crate versions must all refer to the same semantic version
- Git tags keep the `v` prefix, but Docker release tags must use bare semver (`X.Y.Z`)

### 4. Docker Image Publication

Extend `.github/workflows/container.yaml` so release branches also publish Docker images.

- Keep existing behavior for `main` and `develop`
- Add support for `releases/**/*` branch pushes
- Follow the tracker repository pattern for deriving release image tags from the release branch version
- Release branch publication should push versioned tags, not `latest`
- Release branch publication must publish only canonical semver Docker tags such as `1.2.3`
- Do not publish duplicate release image tags with both `v1.2.3` and `1.2.3`
- Verify the image can be pulled and inspected after publication

Environment configuration for Docker publish:

- Use GitHub Environment: `dockerhub-torrust`
- Keep `DOCKER_HUB_ACCESS_TOKEN` as a secret
- Keep `DOCKER_HUB_USERNAME` as a normal environment variable (already set to `torrust` in deployer)
- Do not store `DOCKER_HUB_USERNAME` or `DOCKER_HUB_REPOSITORY_NAME` as secrets
- Repository name can be hardcoded for this repo (`tracker-deployer`) or stored as a non-secret variable

### 5. Library Crate Publication

Add a dedicated workflow for publishing crates from release branches.

- Preferred initial target crate: `torrust-tracker-deployer-sdk`
- Trigger on push to `releases/**/*`
- Run tests and release pre-checks before publication
- Verify packaged contents before publishing (`cargo package --list`) to avoid shipping unintended files
- `cargo publish --dry-run` before real publish
- Post-publish verification (crate visible in registry and installable)
- Verify docs.rs build status for the published version
- Avoid mixing Docker-specific logic into the crate publication workflow

Environment configuration for crate publish:

- Use a dedicated GitHub Environment for crate publication (for example `deployment`)
- Store cargo registry token as a secret only in that environment
- Keep non-sensitive crate metadata as normal variables when needed

### 6. GitHub Release Creation

Define how the GitHub release is created from the pushed tag.

- Keep this step simple for now: create the GitHub release manually from the tag
- Attach release notes manually or with a minimal template
- Do not block Docker or crate publication on a more complex release-notes automation flow

Release finalization gate order:

- Confirm the release commit is pushed to `main`
- Confirm tag `vX.Y.Z` is pushed
- Confirm branch `releases/vX.Y.Z` is pushed
- Confirm Docker release workflow passed
- Confirm crate release workflow passed
- Create/publish GitHub release as final step

### 7. Workflow Separation Strategy

Prefer independent workflows instead of one workflow that publishes all release artifacts.

- Keep Docker publication in `container.yaml` because it already owns Docker build/test/publish logic
- Add a separate release-oriented workflow for crate publication; `deployment.yaml` is probably too vague in this repository
- Prefer a name that reveals the artifact, for example `publish-crate.yaml` or `release-crate.yaml`
- Keep GitHub release creation outside the artifact publication workflows for the first iteration

Reasoning:

- Docker and crate publishing have different credentials, failure modes, and verification steps
- Separate workflows reduce accidental coupling and make reruns more targeted
- The simpler process is easier to debug than one orchestrator workflow with multiple artifact paths

### 8. Failure Handling and Recovery

Define how to proceed when a step fails.

- If Docker publication fails, the release branch can be re-pushed or the workflow can be re-run without changing the tag
- If crate publication fails after tag and branch creation, document whether a version must be abandoned or publication can be retried safely
- Branch/tag rollback guidance
- Docker publish retry policy
- Crate publish partial-failure guidance
- Operator-facing troubleshooting notes

Partial-failure action matrix:

- Docker failed, crate not started: fix Docker workflow and re-run publication on the same release branch
- Docker passed, crate failed before upload: fix issue and re-run crate workflow on the same release branch
- Crate published, later step failed: do not republish same crate version; proceed with follow-up patch release if needed

Idempotency and re-run rules:

- Docker release publication must be safely re-runnable for the same release branch/version
- Crate workflow must detect already-published versions and fail with clear guidance instead of ambiguous errors
- Tag and branch creation steps must check for existing refs and stop with actionable output if refs already exist

Crate rollback/yank policy:

- Never delete published versions (not possible on crates.io); use `cargo yank` only when necessary
- Prefer yanking only for severe release defects (broken build, critical security issue, unusable package)
- After yanking, cut a patch release with a higher version and document remediation in release notes

### 9. Pre-Flight Checks

Define mandatory checks before starting any release actions.

- Verify required GitHub environments exist (`dockerhub-torrust` and crate publish environment)
- Verify required secrets and variables exist in those environments
- Verify the releaser has permission to access protected environments and push required refs
- Verify local workspace is clean and on the expected source branch before version bump/tagging

### 10. Repository Settings Alignment

Define repository settings expectations that release automation depends on.

- Allowed branches for release-related workflows: `develop`, `main`, `releases/**/*`
- Release workflows must be trigger-scoped to those branches; avoid broad wildcard triggers
- Current tracker policy (`10` branches and `0` tags allowed) should be documented as reference, and deployer should adopt equivalent branch scoping for release workflows where applicable

## Implementation Plan

### Phase 1: Define the Manual Release Sequence (estimated time: 2-3 hours)

- [x] Task 1.1: Document the simplified release steps from version bump through GitHub release creation
- [x] Task 1.2: Define version, tag, and release branch naming conventions
- [x] Task 1.3: Specify which `Cargo.toml` files must be updated for each release
- [x] Task 1.4: Add a pre-flight checklist for environments, permissions, and clean git state

### Phase 2: Docker Release Branch Publishing (estimated time: 1-2 hours)

- [x] Task 2.1: Extend `container.yaml` to trigger on `releases/**/*`
- [x] Task 2.2: Add release branch context detection and release image tags
- [x] Task 2.3: Define image verification, credential, and rerun requirements
- [x] Task 2.4: Ensure Docker Hub username/repository are configured as non-secret variables (token remains secret)

### Phase 3: Crate Publishing Workflow (estimated time: 1-2 hours)

- [x] Task 3.1: Create a dedicated workflow for publishing the SDK crate from `releases/**/*`
- [x] Task 3.2: Define package inspection, dry-run, publish, and post-publish verification steps
- [x] Task 3.3: Define dedicated environment and document cargo registry credentials and failure recovery rules
- [x] Task 3.4: Add docs.rs post-publish verification guidance

### Phase 4: Validation and Operational Guidance (estimated time: 2-4 hours)

- [ ] Task 4.1: Validate the end-to-end release flow against a test version
- [x] Task 4.2: Document how maintainers verify Docker image, crate publication, and GitHub release creation
- [x] Task 4.3: Add troubleshooting notes for partial publication failures
- [x] Task 4.4: Add explicit idempotency/re-run guidance and crate yank policy

> Note: The practical end-to-end validation for Task 4.1 is planned as the
> post-merge `0.1.0-beta` release run.

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [x] The documented release process follows this order: version update, release commit, push to `main`, tag push, release branch push, workflow-driven artifact publication, GitHub release creation
- [x] The spec defines explicit finalization gates (main push, tag push, release branch push, Docker pass, crate pass, GitHub release)
- [x] Branch naming and tag naming conventions are documented as `releases/vX.Y.Z` and `vX.Y.Z`
- [x] `container.yaml` is specified to publish Docker images for release branches in addition to existing `main` and `develop` behavior
- [x] The spec explicitly requires Docker release tags to use `X.Y.Z` and forbids `vX.Y.Z` image tags
- [x] A separate crate publication workflow is specified for the SDK crate on `releases/**/*`
- [x] The spec explicitly records the decision to keep Docker and crate publication in independent workflows
- [x] Docker Hub configuration policy is explicit: token is secret, username/repository are variables
- [x] Release workflow branch scope is explicit and aligned with `develop`, `main`, and `releases/**/*`
- [x] Docker publish procedure includes verification and failure handling
- [x] Crate publish procedure includes dry-run and post-publish verification
- [x] Crate publish procedure includes package content inspection before publish
- [x] Crate publish procedure includes docs.rs build verification after publish
- [x] Pre-flight checks are documented for environments, secrets/variables, permissions, and git state
- [x] Partial-failure and re-run rules are documented for Docker and crate workflows
- [x] Crate rollback policy includes explicit yank criteria and patch-release follow-up
- [x] Version consistency rules are documented across Git tags, Docker tags, and crate versions

## Related Documentation

- `docs/contributing/roadmap-issues.md`
- `docs/contributing/commit-process.md`
- `docs/roadmap.md`
- https://raw.githubusercontent.com/torrust/torrust-linting/refs/heads/main/skills/publish-rust-crate/SKILL.md

## Notes

- Keep the first iteration focused on one release path that can be executed by maintainers without additional assumptions.
- Start with the SDK crate only unless additional crates are explicitly marked for publication.
- Do not import the tracker repository's full staging and develop branch merge-back process into this repository yet.
- Guard against the tracker bug described in `torrust/torrust-tracker#1029`: Docker release tags should not be published with the `v` prefix.
