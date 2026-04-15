# Production Image Scan Results

Historical security scan results for Docker images deployed to production by the deployer.

## Current Status Summary

| Image              | Version | HIGH | CRITICAL | Status                                   | Last Scan    | Details                                                                    |
| ------------------ | ------- | ---- | -------- | ---------------------------------------- | ------------ | -------------------------------------------------------------------------- |
| `caddy`            | 2.11.2  | 10   | 2        | ⚠️ CRITICAL pending upstream              | Apr 15, 2026 | [View](caddy.md)                                                           |
| `prom/prometheus`  | v3.11.2 | 4    | 0        | ✅ Remediated                            | Apr 14, 2026 | [View](prometheus.md)                                                      |
| `grafana/grafana`  | 12.4.2  | 4    | 0        | ⚠️ Accepted risk (OS `<no-dsa>`)          | Apr 8, 2026  | [View](grafana.md)                                                         |
| `mysql`            | 8.4     | 9    | 1        | ⚠️ Accepted risk (gosu/mysqlsh, not core) | Apr 15, 2026 | [View](mysql.md)                                                           |
| `torrust/tracker-backup` | trixie | 6  | 0        | ⚠️ Accepted risk (Debian `<no-dsa>`)    | Apr 15, 2026 | [View](torrust-tracker-backup.md)                                          |

## Scanning Instructions

See [`../README.md`](../README.md) for Trivy usage and remediation workflow.

## Scan Archives

Each file contains the complete scan history for a service:

- [caddy.md](caddy.md) — Caddy TLS termination proxy
- [prometheus.md](prometheus.md) — Prometheus monitoring
- [grafana.md](grafana.md) — Grafana dashboards
- [mysql.md](mysql.md) — MySQL tracker database
- [torrust-tracker-backup.md](torrust-tracker-backup.md) — Backup service container
