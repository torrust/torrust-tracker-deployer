# Security Overview

This directory documents security considerations for the Torrust Tracker Deployer project, organized by priority level.

## Priority Levels

Security effort should be distributed according to exposure and risk. The highest-priority areas are those that directly affect end users in production.

### Priority 1 — Production Environment (Critical)

**Directory**: [`production/`](production/)

The most critical security surface: the Docker images, OS packages, system dependencies, and server configuration that the deployer deploys to production.

These are exposed to the internet and run continuously. Any vulnerability here directly affects tracker users.

**Scope**:

- Service container images: `caddy`, `prom/prometheus`, `grafana/grafana`, `mysql`
- Backup service container: `torrust/tracker-backup`
- OS base layers of the provisioned VM
- Server configuration (TLS, SSH access policies)

**Scan history**: [`production/scans/`](production/scans/)

---

### Priority 2 — User Workflow Security (Important)

**Directory**: [`user-security/`](user-security/)

How users interact with the deployer affects the security of their deployments. Mistakes here can expose secrets or production credentials.

**Scope**:

- Sharing secrets with AI coding agents during deployment
- SSH access controls and key management
- Safe handling of deployment credentials (`envs/*.json`)

**Documents**:

- [AI Agents and Secrets](user-security/ai-agents-and-secrets.md) — risks when using cloud-based AI agents during deployments
- [SSH Root Access on Hetzner](user-security/ssh-root-access-hetzner.md) — SSH key behavior and hardening guidance

---

### Priority 3 — Deployer Tooling Security (Standard)

**Directory**: [`deployer/`](deployer/)

The deployer itself — its Rust binary, container images, and bundled tools (OpenTofu, Ansible). This is a **lower-risk surface** because:

- Users run the deployer locally for minutes at a time
- It is not exposed to the internet during normal use
- It runs in a controlled local or CI environment

This priority increases if the deployer is ever embedded in a long-running service (e.g., a web application that calls the deployer on demand).

**Scope**:

- The deployer container image: `torrust/tracker-deployer` (Rust binary + OpenTofu + Ansible)
- Rust dependency vulnerabilities (`cargo audit` / RustSec)
- Bundled tool vulnerabilities: OpenTofu, Ansible

**Subdirectories**:

- [`deployer/docker/`](deployer/docker/) — Docker image scans
- [`deployer/dependencies/`](deployer/dependencies/) — Rust dependency audits

---

### Priority 4 — Testing Artifacts (Low)

**Directory**: [`testing/`](testing/)

Docker images and other artifacts used only in automated tests or local development. These never run in production and have a minimal attack surface.

**Scope**:

- `torrust/tracker-ssh-server` — SSH server used in E2E integration tests
- `torrust/tracker-provisioned-instance` — Ubuntu VM simulation used in E2E deployment workflow tests

**Scan history**: [`testing/scans/`](testing/scans/)

---

## Scan Tooling

| Tool        | Purpose                   | Run Command                                    |
| ----------- | ------------------------- | ---------------------------------------------- |
| Trivy       | Docker image CVE scanning | `trivy image --severity HIGH,CRITICAL <image>` |
| cargo-audit | Rust dependency audits    | `cargo audit`                                  |

## Current Security Status

### Production Images

See [`production/scans/README.md`](production/scans/README.md) for the latest status of all production-deployed images.

### Deployer Images

See [`deployer/docker/scans/README.md`](deployer/docker/scans/README.md) for the latest status of deployer-internal images.

### Rust Dependencies

See [`deployer/dependencies/README.md`](deployer/dependencies/README.md) for the latest cargo-audit report.

## Related Documentation

- [Docker Image Scanning Guide](production/README.md)
- [Dependency Security Reports](deployer/dependencies/README.md)
