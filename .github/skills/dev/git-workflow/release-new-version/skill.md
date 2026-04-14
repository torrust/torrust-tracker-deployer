---
name: release-new-version
description: Guide for releasing a new version of the deployer using the standard branch/tag workflow. Covers version bump, signed release commit, pushing main, creating signed tag, creating release branch, and verifying Docker + crate publication workflows. Use when asked to "release", "cut a version", "publish a new version", or "create release vX.Y.Z".
metadata:
  author: torrust
  version: "1.0"
---

# Release New Version

This skill provides the canonical workflow to release a new version of the Torrust Tracker Deployer.

Primary reference: [`docs/release-process.md`](../../../../../docs/release-process.md)

## Release Order (Mandatory)

Execute these steps in order:

1. Update versions in manifests
2. Create release commit
3. Push release commit to `main`
4. Create and push signed tag `vX.Y.Z`
5. Create and push release branch `releases/vX.Y.Z`
6. Verify release workflows
7. Create GitHub release

Do not reorder these steps.

## Version and Naming Rules

- Git tag: `vX.Y.Z`
- Release branch: `releases/vX.Y.Z`
- Docker release tag: `X.Y.Z` (no `v` prefix)
- Crate version: `X.Y.Z`

## Pre-Flight Checklist

Before starting:

- [ ] Clean working tree (`git status`)
- [ ] Up to date with `origin/main`
- [ ] GitHub environment `dockerhub-torrust` configured
- [ ] GitHub environment `crates-io` configured with `CARGO_REGISTRY_TOKEN` scoped to
      **all four** crate names (`torrust-tracker-deployer-types`,
      `torrust-tracker-deployer-dependency-installer`, `torrust-tracker-deployer`,
      `torrust-tracker-deployer-sdk`) — a token scoped to only one will cause 403 on the others
- [ ] Releaser has permissions for `main`, tags, and release branches
- [ ] All four `Cargo.toml` files have `description`, `license`, `repository`, `readme`
- [ ] Every internal path dependency also declares an explicit `version` constraint
- [ ] `Cargo.lock` is committed and up to date

## Commands

### 1) Update versions

Update `version` in all four manifests, and update the `version` constraint on every
internal path dependency in each file:

- `Cargo.toml`
- `packages/deployer-types/Cargo.toml`
- `packages/dependency-installer/Cargo.toml`
- `packages/sdk/Cargo.toml`

### 2) Commit and push

```bash
git add Cargo.toml \
        packages/deployer-types/Cargo.toml \
        packages/dependency-installer/Cargo.toml \
        packages/sdk/Cargo.toml \
        Cargo.lock
git commit -S -m "release: version vX.Y.Z"
git push origin main
```

### 3) Tag and release branch

```bash
git tag -s -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z

git checkout -b releases/vX.Y.Z
git push origin releases/vX.Y.Z
```

### 4) Verify workflows

- **Container** workflow: publishes Docker image from release branch
- **Publish Crate** workflow: publishes all four crates in dependency order:
  1. `torrust-tracker-deployer-types`
  2. `torrust-tracker-deployer-dependency-installer`
  3. `torrust-tracker-deployer`
  4. `torrust-tracker-deployer-sdk`

  Each crate's dry-run runs only after its prerequisites are indexed on crates.io.
  Do not attempt to publish out of order.

Workflow files:

- `.github/workflows/container.yaml`
- `.github/workflows/publish-crate.yaml`

### 5) Create GitHub release

Create the release manually from tag `vX.Y.Z` after both workflows pass.

## Failure Handling

- Docker failed, crate not started: fix Docker workflow and rerun on same release branch
- Docker passed, crate failed before upload: fix issue and rerun crate workflow on same release branch
- Crate already published: do not republish same version; cut a patch release
- Ref already exists (tag/branch): stop and investigate partial release state before continuing

## Quick Validation

```bash
# Verify refs exist remotely
git ls-remote --tags origin vX.Y.Z
git ls-remote --heads origin releases/vX.Y.Z
```

For full operational guidance, troubleshooting, and rollback/yank policy, use [`docs/release-process.md`](../../../../../docs/release-process.md).
