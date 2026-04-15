# Security Scan Results

This directory contains historical security scan results for Docker images used in the deployer.

## Current Status Summary

| Image                                  | Version | HIGH | CRITICAL | Status                                 | Last Scan    | Details                                         |
| -------------------------------------- | ------- | ---- | -------- | -------------------------------------- | ------------ | ----------------------------------------------- |
| `torrust/tracker-deployer`             | trixie  | 46   | 1        | ⚠️ CRITICAL blocked (OpenTofu grpc-go) | Apr 15, 2026 | [View](torrust-tracker-deployer.md)             |
| `torrust/tracker-backup`               | trixie  | 6    | 0        | ⚠️ Accepted risk (Debian `<no-dsa>`)   | Apr 15, 2026 | [View](torrust-tracker-backup.md)               |
| `torrust/tracker-ssh-server`           | 3.23.3  | 0    | 0        | ✅ Remediated (vuln scan)              | Apr 8, 2026  | [View](torrust-ssh-server.md)                   |
| `torrust/tracker-provisioned-instance` | 24.04   | 0    | 0        | ✅ Remediated (vuln scan)              | Apr 8, 2026  | [View](torrust-tracker-provisioned-instance.md) |
| `caddy`                                | 2.11.2  | 10   | 2        | ⚠️ Partial remediation                 | Apr 15, 2026 | [View](caddy.md)                                |
| `prom/prometheus`                      | v3.11.2 | 4    | 0        | ✅ Remediated                          | Apr 14, 2026 | [View](prometheus.md)                           |
| `grafana/grafana`                      | 12.4.2  | 4    | 0        | ⚠️ Partial remediation                 | Apr 8, 2026  | [View](grafana.md)                              |
| `mysql`                                | 8.4     | 9    | 1        | ⚠️ Accepted risk (gosu)                | Apr 15, 2026 | [View](mysql.md)                                |

**Overall Status**: ⚠️ **CVE database update detected** - Most images still show increased vulnerability counts from previous scans (Feb-Dec 2025). Deployer has a first remediation pass applied (49 HIGH -> 44 HIGH, with 1 CRITICAL still open).

## Scan Archives

Each file contains the complete scan history for a service:

- [torrust-tracker-deployer.md](torrust-tracker-deployer.md) - Deployer (base: rust:trixie, **updated from bookworm**)
- [torrust-tracker-backup.md](torrust-tracker-backup.md) - Backup container (base: debian:trixie-slim, **updated**)
- [torrust-ssh-server.md](torrust-ssh-server.md) - SSH test server (base: alpine:3.23.3, **new**)
- [torrust-tracker-provisioned-instance.md](torrust-tracker-provisioned-instance.md) - Ubuntu VM simulation (base: ubuntu:24.04, **new**)
- [caddy.md](caddy.md) - Caddy TLS termination proxy
- [prometheus.md](prometheus.md) - Prometheus monitoring
- [grafana.md](grafana.md) - Grafana dashboards
- [mysql.md](mysql.md) - MySQL database

## Build & Scan All Images

To build and scan all Torrust Tracker Deployer images:

```bash
# Build all images
docker build --target release --tag torrust/tracker-deployer:local --file docker/deployer/Dockerfile .
docker build --tag torrust/tracker-backup:local docker/backup/
docker build --tag torrust/tracker-ssh-server:local docker/ssh-server/
docker build --tag torrust/tracker-provisioned-instance:local docker/provisioned-instance/

# Run scans on all images
trivy image --severity HIGH,CRITICAL torrust/tracker-deployer:local
trivy image --severity HIGH,CRITICAL torrust/tracker-backup:local
trivy image --severity HIGH,CRITICAL torrust/tracker-ssh-server:local
trivy image --severity HIGH,CRITICAL torrust/tracker-provisioned-instance:local
```

## Scanning Standards

All scans use:

- **Tool**: Trivy (latest)
- **Severity Filter**: HIGH and CRITICAL only (MEDIUM and LOW omitted for brevity)
- **Update Frequency**: On every push (GitHub Actions), weekly schedules, and manual verification
- **Documentation**: Each scan includes context on image purpose, vulnerability analysis, and mitigation strategies

## How to Add New Scans

1. Build image: `docker build --tag <image-name>:local <dockerfile-path>`
2. Run Trivy scan: `trivy image --severity HIGH,CRITICAL <image-name>:local`
3. Create or update scan file in this directory
4. Update the summary table above
5. Commit with message: `docs: add security scan for <image-name> (<date>)` or `docs: [#<issue>] update security scans`

See [../README.md](../README.md) for detailed scanning instructions and best practices.

## Image Purpose & Risk Context

Each image serves a different purpose with different security contexts:

| Image                    | Purpose                                  | Runtime             | Network Exposure  | Data Access        | Risk Level |
| ------------------------ | ---------------------------------------- | ------------------- | ----------------- | ------------------ | ---------- |
| **Deployer**             | CLI tool for infrastructure provisioning | User's machine / CI | None              | SSH keys only      | LOW        |
| **Backup**               | Database backup container                | Controlled schedule | Internal only     | Read access to DB  | MEDIUM     |
| **SSH Server**           | E2E testing SSH connectivity             | CI test environment | Test network only | Test data only     | NEGLIGIBLE |
| **Provisioned Instance** | E2E deployment workflow testing          | CI test environment | Test network only | Test data only     | NEGLIGIBLE |
| **Caddy**                | TLS termination and reverse proxy        | Production optional | Public internet   | Configuration only | MEDIUM     |
| **Prometheus**           | Metrics collection                       | Infrastructure      | Internal network  | Metrics only       | LOW        |
| **Grafana**              | Metrics visualization                    | Infrastructure      | Internal network  | Read-only graphs   | LOW        |
| **MySQL**                | Database storage                         | Infrastructure      | Internal network  | Application data   | HIGH       |

## Security Updates Schedule

- **Deployer image**: Rebuilt whenever Rust or Debian releases updates (typically monthly)
- **Backup image**: Rebuilt with base OS updates (tied to Debian release cycle)
- **SSH/Provisioned**: Rebuilt on every CI run (via GitHub Actions)
- **Monitoring images**: Scanned weekly, rebuilt when security advisories issued

## References

- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [OWASP Docker Security](https://owasp.org/www-community/attacks/Docker_Escapes)
- [CIS Docker Benchmark](https://www.cisecurity.org/benchmark/docker)
- [GitHub Actions Docker Security](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)
