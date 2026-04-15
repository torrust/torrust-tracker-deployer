# Production Image Security

This directory covers security scanning for Docker images that the deployer **deploys to production**.
These are [Priority 1](../README.md) — the highest-risk surface because they run continuously and are exposed to the internet.

## Images Covered

| Image                    | Role                                                                |
| ------------------------ | ------------------------------------------------------------------- |
| `caddy`                  | TLS termination proxy — public-facing                               |
| `prom/prometheus`        | Metrics collection                                                  |
| `grafana/grafana`        | Metrics dashboards                                                  |
| `mysql`                  | Tracker database                                                    |
| `torrust/tracker-backup` | Backup service — runs on a schedule inside the deployed environment |

## Scanning with Trivy

```bash
# Scan a production image
trivy image --severity HIGH,CRITICAL caddy:2.11.2

# Scan and output JSON
trivy image --format json --output report.json caddy:2.11.2

# Scan all severities for a full report
trivy image caddy:2.11.2
```

## When to Act on Findings

**CRITICAL severity**:

1. Check whether the upstream vendor has released a patched image
2. Update the image version in `templates/docker-compose/docker-compose.yml.tera`
3. Re-scan the updated image to confirm the fix
4. Update scan history in `scans/<image>.md`

**HIGH severity**:

1. Check Debian/Alpine security tracker for fix availability
2. If a fix exists, update the image as above
3. If no fix exists (`affected` / `will_not_fix` / `<no-dsa>`), document the accepted risk

## Best Practices

- Pin to specific versions, never `latest`, in production templates
- Prefer official vendor images (`prom`, `grafana`, `mysql`)
- Re-scan after every image version bump
- Monitor vendor security advisories

## Scan History

See [`scans/`](scans/) for per-image scan history and current status.

## References

- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [Debian Security Tracker](https://security-tracker.debian.org/tracker/)
- [Issue #250: Automated Security Scanning](https://github.com/torrust/torrust-tracker-deployer/issues/250)
