# Decision: Track Cargo.lock for Application Reproducibility

## Status

✅ Accepted

## Date

2026-04-10

## Context

The repository is a Rust workspace that includes reusable packages and executable
application binaries. CI workflows clone the repository and run tasks that depend
on deterministic dependency resolution.

`Cargo.lock` was excluded via `.gitignore`, so GitHub runners did not receive it
on checkout. This caused workflow failures and made dependency resolution
non-deterministic across developer machines and CI runs.

For Rust libraries published to crates.io, omitting `Cargo.lock` is often
recommended. For applications, committing `Cargo.lock` is recommended to keep
builds reproducible.

This repository is app-first from an operations perspective: users clone the repo
and run deployer binaries and automation workflows directly.

## Decision

Stop ignoring `Cargo.lock` and commit it to the repository.

Specifically:

- Remove the `Cargo.lock` ignore rule from `.gitignore`.
- Keep `Cargo.lock` versioned in Git so all environments resolve to the same
  dependency graph.

## Consequences

Positive:

- Reproducible dependency resolution in local development and CI.
- GitHub Actions runners receive `Cargo.lock` after checkout, preventing missing
  lockfile failures.
- Fewer "works on my machine" differences caused by floating transitive versions.

Negative / Risks:

- Lockfile updates create larger diffs and may need periodic refreshes.
- Contributors touching dependencies may need to resolve lockfile merge conflicts.

## Alternatives Considered

1. Keep ignoring `Cargo.lock` and generate it in CI.

Rejected because:

- CI and local builds can drift as transitive dependencies change.
- Adds avoidable complexity to workflows and troubleshooting.

1. Keep ignoring `Cargo.lock` because the workspace includes library crates.

Rejected because:

- The repository's primary usage is as a runnable application and deployment tool.
- Application reproducibility is more important than library-only conventions.

## Related Decisions

- [SDK Package Naming](./sdk-package-naming.md)
- [Application-Layer Progress Reporting Trait](./application-layer-progress-reporting-trait.md)

## References

- Rust Cargo book: https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html
- Workflow affected by lockfile presence:
  - `.github/workflows/cargo-security-audit.yml`
  - `.github/workflows/test-dependency-installer.yml`
  - `.github/workflows/container.yaml`
