# Security Scan Results

This directory contains historical security scan results for Docker images used in the deployer.

## Current Status Summary

| Image                      | Version | HIGH | CRITICAL | Status       | Last Scan    | Details                             |
| -------------------------- | ------- | ---- | -------- | ------------ | ------------ | ----------------------------------- |
| `torrust/tracker-deployer` | latest  | 25   | 7        | ⚠️ Monitored | Jan 10, 2026 | [View](torrust-tracker-deployer.md) |
| `torrust/tracker-backup`   | local   | 9    | 2        | ⚠️ Monitored | Feb 2, 2026  | [View](torrust-tracker-backup.md)   |
| `caddy`                    | 2.10    | 3    | 1        | ⚠️ Monitored | Jan 13, 2026 | [View](caddy.md)                    |
| `prom/prometheus`          | v3.5.0  | 0    | 0        | ✅ SECURE    | Dec 29, 2025 | [View](prometheus.md)               |
| `grafana/grafana`          | 12.3.1  | 0    | 0        | ✅ SECURE    | Dec 29, 2025 | [View](grafana.md)                  |
| `mysql`                    | 8.4     | 0    | 0        | ✅ SECURE    | Dec 29, 2025 | [View](mysql.md)                    |

**Overall Status**: ⚠️ Deployer, Backup, and Caddy images have upstream vulnerabilities (backup has fixable OpenSSL issues, others monitoring for releases).

## Scan Archives

Each file contains the complete scan history for a service:

- [torrust-tracker-deployer.md](torrust-tracker-deployer.md) - The deployer Docker image
- [torrust-tracker-backup.md](torrust-tracker-backup.md) - Backup container for tracker data
- [caddy.md](caddy.md) - Caddy TLS termination proxy
- [prometheus.md](prometheus.md) - Prometheus monitoring
- [grafana.md](grafana.md) - Grafana dashboards
- [mysql.md](mysql.md) - MySQL database

## How to Add New Scans

1. Run Trivy scan: `trivy image --severity HIGH,CRITICAL <image-name>`
2. Add results to the appropriate service file
3. Update the summary table above
4. Commit with message: `docs: add security scan for <service> (<date>)`

See [../README.md](../README.md) for detailed scanning instructions.
