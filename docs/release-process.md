# Release Process

This document defines the standard release process for the Torrust Tracker Deployer
repository. Following these steps ensures that releases are predictable, auditable,
and consistent across Git tags, Docker images, and published crates.

## Overview

Releasing consists of these mandatory steps, executed **in order**:

1. Update version in all relevant `Cargo.toml` files
2. Commit the version bump (`release: version vX.Y.Z`)
3. Push the release commit to `main`
4. Create and push the annotated, signed release tag (`vX.Y.Z`)
5. Create and push the release branch (`releases/vX.Y.Z`)
6. Wait for GitHub Actions to publish release artifacts (Docker image, crate)
7. Create the GitHub release from the tag

Do not skip or reorder steps. Each step is a prerequisite for the next.

## Naming Conventions

| Artifact         | Convention              | Example           |
| ---------------- | ----------------------- | ----------------- |
| Git tag          | `vX.Y.Z`                | `v1.2.3`          |
| Release branch   | `releases/vX.Y.Z`       | `releases/v1.2.3` |
| Docker image tag | `X.Y.Z` (no `v` prefix) | `1.2.3`           |
| Crate version    | `X.Y.Z` (no `v` prefix) | `1.2.3`           |

> **Important**: Docker release tags must use bare semver (`X.Y.Z`). Never publish
> release Docker tags with the `v` prefix (e.g., `v1.2.3`).

## Files to Update for Each Release

Update the `version` field in all four manifests:

| File                                       | Crate                                           |
| ------------------------------------------ | ----------------------------------------------- |
| `Cargo.toml` (workspace root)              | `torrust-tracker-deployer`                      |
| `packages/deployer-types/Cargo.toml`       | `torrust-tracker-deployer-types`                |
| `packages/dependency-installer/Cargo.toml` | `torrust-tracker-deployer-dependency-installer` |
| `packages/sdk/Cargo.toml`                  | `torrust-tracker-deployer-sdk`                  |

All four must carry the same non-prefixed semver version string (e.g., `1.2.3`).

Also ensure that every internal path dependency in each manifest declares an explicit
`version` constraint that matches the release version, for example:

```toml
torrust-tracker-deployer-types = { path = "packages/deployer-types", version = "1.2.3" }
```

crates.io rejects packages with path-only dependencies (no `version` field) because
consumers resolving the crate from the registry cannot follow local paths.

## Pre-Flight Checklist

Run these checks before starting any release actions:

**Git state:**

- [ ] You are on the `main` branch with a clean working tree (`git status`)
- [ ] The branch is up to date with `origin/main` (`git pull --ff-only`)

**GitHub Environments:**

- [ ] GitHub Environment `dockerhub-torrust` exists and contains:
  - `DOCKER_HUB_ACCESS_TOKEN` — secret
  - `DOCKER_HUB_USERNAME` — variable (value: `torrust`)
- [ ] GitHub Environment `crates-io` exists and contains:
  - `CARGO_REGISTRY_TOKEN` — secret with `publish-new` and `publish-update` scopes
    for **all four** crate names:
    - `torrust-tracker-deployer-types`
    - `torrust-tracker-deployer-dependency-installer`
    - `torrust-tracker-deployer`
    - `torrust-tracker-deployer-sdk`

  > A token scoped to only one crate name will cause HTTP 403 Forbidden for all
  > others. Create a single token covering all four names.

**Permissions:**

- [ ] You have push access to `main`, and can push tags and release branches
- [ ] You have access to the `dockerhub-torrust` and `crates-io` environments

**Crate metadata** (verify for all four crates before first publish, or whenever
adding a new publishable crate):

- [ ] Each `Cargo.toml` has `description`, `license`, `repository`, and `readme`
- [ ] Each internal path dependency also declares an explicit `version` constraint
- [ ] All four crate names follow the `torrust-tracker-deployer-*` namespace
      (crate names are permanent on crates.io — audit before first publish)
- [ ] `Cargo.lock` is committed and up to date

## Release Steps

### Step 1 — Update Versions

Edit the `version` field in all four `Cargo.toml` files. Also update the `version`
constraint on every internal path dependency in each manifest to match the new
release version:

```bash
# Edit these four files:
#   Cargo.toml
#   packages/deployer-types/Cargo.toml
#   packages/dependency-installer/Cargo.toml
#   packages/sdk/Cargo.toml
#
# In each, change: version = "X.Y.Z-dev" (or current dev version)
#               to: version = "X.Y.Z"
#
# Also update the version constraint on internal path deps, for example:
#   torrust-tracker-deployer-types = { path = "...", version = "X.Y.Z" }
```

Verify the workspace compiles and tests pass after the version change:

```bash
cargo build
cargo test
```

### Step 2 — Create the Release Commit

Stage all four manifests **and `Cargo.lock`**, then create a signed commit:

```bash
git add Cargo.toml \
        packages/deployer-types/Cargo.toml \
        packages/dependency-installer/Cargo.toml \
        packages/sdk/Cargo.toml \
        Cargo.lock
git commit -S -m "release: version vX.Y.Z"
```

The commit subject must follow the pattern `release: version vX.Y.Z` so releases
are easily identifiable in the git log.

### Step 3 — Push to `main`

```bash
git push origin main
```

Wait for the CI pipeline on `main` to pass before continuing.

### Step 4 — Create and Push the Release Tag

Create an annotated, signed tag from the release commit:

```bash
git tag -s -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

Verify the tag:

```bash
git tag -v vX.Y.Z
```

### Step 5 — Create and Push the Release Branch

Create the release branch from the same commit (already at `HEAD` of `main`):

```bash
git checkout -b releases/vX.Y.Z
git push origin releases/vX.Y.Z
```

This push triggers the GitHub Actions workflows that publish the Docker image and
the crate.

### Step 6 — Wait for CI Artifacts

Monitor the following workflows in GitHub Actions:

- **Container** workflow — publishes the Docker image tagged `X.Y.Z` to Docker Hub
- **Publish Crate** workflow — publishes all four crates to crates.io in dependency
  order:
  1. `torrust-tracker-deployer-types`
  2. `torrust-tracker-deployer-dependency-installer`
  3. `torrust-tracker-deployer`
  4. `torrust-tracker-deployer-sdk`

  Each crate's dry-run step runs only after its prerequisites are available on
  crates.io. Do not attempt to manually publish out of order.

Both workflows must succeed before moving to step 7. See
[Finalization Gates](#finalization-gates) below.

### Step 7 — Create the GitHub Release

Once both workflows have passed:

1. Go to **GitHub → Releases → Draft a new release**
2. Select tag `vX.Y.Z`
3. Write release notes (highlights, breaking changes, upgrade instructions)
4. Publish the release

## Finalization Gates

All of the following must be confirmed before marking the release as complete:

- [ ] Release commit is on `main` and CI passed
- [ ] Tag `vX.Y.Z` is pushed and signed
- [ ] Branch `releases/vX.Y.Z` is pushed
- [ ] Container workflow completed successfully (Docker image `X.Y.Z` published)
- [ ] Publish Crate workflow completed successfully (crate `X.Y.Z` on crates.io)
- [ ] GitHub release created from tag `vX.Y.Z`

## Docker Image Verification

After the Container workflow completes:

```bash
# Pull and inspect the published image
docker pull torrust/tracker-deployer:X.Y.Z
docker image inspect torrust/tracker-deployer:X.Y.Z

# Confirm the version and tools
docker run --rm torrust/tracker-deployer:X.Y.Z --version || true
docker run --rm --entrypoint tofu torrust/tracker-deployer:X.Y.Z version
```

## Crate Verification

After the Publish Crate workflow completes, verify all four crates are indexed
(crates.io indexing may take a few minutes after publish):

```bash
for crate in \
  torrust-tracker-deployer-types \
  torrust-tracker-deployer-dependency-installer \
  torrust-tracker-deployer \
  torrust-tracker-deployer-sdk; do
  status=$(curl -s -o /dev/null -w "%{http_code}" \
    "https://crates.io/api/v1/crates/$crate/X.Y.Z")
  echo "$crate: HTTP $status"
done
```

All four should return `HTTP 200`. If any return 404, wait a few minutes and retry.

**docs.rs** pages may take minutes to hours to become available after a first
publish, especially for a new crate. The Publish Crate workflow prints the URLs
as informational output. Verify them manually when convenient:

- `https://docs.rs/torrust-tracker-deployer-types/X.Y.Z`
- `https://docs.rs/torrust-tracker-deployer-dependency-installer/X.Y.Z`
- `https://docs.rs/torrust-tracker-deployer/X.Y.Z`
- `https://docs.rs/torrust-tracker-deployer-sdk/X.Y.Z`

A 404 on docs.rs is **not** a release failure. The crate is published; docs.rs
builds in its own queue.

## Failure Handling and Recovery

### Partial-Failure Action Matrix

| Failure point                                             | Action                                                                                 |
| --------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Docker failed, crate not started                          | Fix the Docker workflow and re-run the Container workflow on the same release branch   |
| Docker passed, crate failed before upload                 | Fix the issue and re-run the Publish Crate workflow on the same release branch         |
| Crate published, later step (e.g., GitHub release) failed | Do not republish. Proceed with follow-up patch release if the crate artifact is broken |

### Re-Run Rules

**Docker publication** is safely re-runnable for the same release branch. Pushing the
same Docker tag twice with identical content is idempotent.

**Crate publication** must detect previously published versions:

- `cargo publish` will fail with a clear error if the version is already on crates.io
- Do not attempt to republish the same version; instead, cut a patch release

**Tag and branch creation** must verify that refs do not already exist:

```bash
# Check before creating the tag
git ls-remote --tags origin vX.Y.Z

# Check before creating the release branch
git ls-remote --heads origin releases/vX.Y.Z
```

If a ref already exists, **do not force-push**. Investigate the previous state and
determine whether the release partially succeeded.

### Crate Rollback and Yank Policy

Yanking a published crate is a last resort, not a routine operation.

Use `cargo yank` **only** for:

- A critical security vulnerability in the published version
- A broken build that prevents dependents from compiling
- Corruption that makes the crate entirely unusable

```bash
# Yank a specific version (prevents new Cargo.lock pins; existing users keep it)
cargo yank --version X.Y.Z torrust-tracker-deployer-sdk
```

After yanking, cut a patch release (`X.Y.Z+1`) with a fix and document the
remediation in its release notes.

Never yank for minor issues. Prefer a follow-up patch release instead.

### Tag and Branch Rollback

If the release commit has not been pushed to `main` yet, you can reset locally:

```bash
# Delete the local tag
git tag -d vX.Y.Z

# Delete the local branch
git branch -d releases/vX.Y.Z
```

Once a tag or branch is pushed and CI has run, **do not delete** the remote ref
without coordinating with maintainers. Deleting a published release ref can break
CI re-runs and audit trails.

## Related Documentation

- [Branching conventions](contributing/branching.md)
- [Commit process](contributing/commit-process.md)
- [Docker workflow](../.github/workflows/container.yaml)
- [Crate publish workflow](../.github/workflows/publish-crate.yaml)
- [Roadmap](roadmap.md)
