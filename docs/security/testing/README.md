# Testing Artifacts Security

This directory covers security for Docker images used only in automated tests or local development.
These are [Priority 4](../README.md) — the lowest-risk surface, as they never run in production.

## Scope

- `torrust/tracker-ssh-server` — SSH server used in E2E integration tests
- `torrust/tracker-provisioned-instance` — Ubuntu VM simulation used in E2E deployment workflow tests

## Scan History

See [`scans/`](scans/) for historical scan results.

## When to Re-scan

Scan testing artifacts when:

- A test image uses a base image with known CRITICALs
- An artifact is promoted to be used in the deployer itself (moves to Priority 3)
- An artifact is deployed to production (moves to Priority 1)
