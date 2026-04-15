# Deployer Tooling Security

This directory covers security for the deployer's own tools and container images.
These are [Priority 3](../README.md) — a lower-risk surface because the deployer runs locally for minutes at a time and is not exposed to the internet.

> **Note**: This priority increases if the deployer is ever embedded in a long-running service
> (e.g., a web application that provisions environments on demand).

## Subdirectories

### [`docker/`](docker/)

Security scans for Docker images used by the deployer itself:

- `torrust/tracker-deployer` — the deployer container (Rust binary + OpenTofu + Ansible)
- `torrust/tracker-backup` — backup helper container
- `torrust/tracker-ssh-server` — SSH server used in local testing

### [`dependencies/`](dependencies/)

Rust dependency security audits via `cargo audit`:

- Tracks RustSec advisories for the deployer's Cargo.lock
- Records remediation actions and accepted risks

## Relationship to Priority 1

Vulnerabilities in deployer tooling are less urgent than production image vulnerabilities
because the deployer is a short-lived local tool. However, CRITICAL CVEs in tools like
OpenTofu or Ansible should still be tracked and addressed when upstream fixes are available.
