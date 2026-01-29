# Production Considerations

This document outlines issues that must be addressed before using the backup
sidecar PoC in a production environment.

## Security Issues

### Critical

| Issue                     | Current State                | Production Fix                                                             |
| ------------------------- | ---------------------------- | -------------------------------------------------------------------------- |
| **MySQL password in env** | Plain text in docker-compose | Use Docker secrets or external secret manager (Vault, AWS Secrets Manager) |
| **No encryption at rest** | Backups stored in plain gzip | Encrypt with GPG or age before writing to disk                             |
| **.env contains secrets** | Backed up in plain text      | Exclude secrets from backup or encrypt the backup archive                  |

### High

| Issue                                | Current State                                | Production Fix                                                   |
| ------------------------------------ | -------------------------------------------- | ---------------------------------------------------------------- |
| **Root container**                   | Container runs as root                       | Create dedicated `backup` user with uid matching app user (1000) |
| **Backup file permissions**          | Root-owned files, potentially world-readable | Set explicit permissions (600), run container as app user        |
| **No backup integrity verification** | No checksums generated                       | Generate SHA256 sums for each backup file                        |

### Medium

| Issue                  | Current State                         | Production Fix                                       |
| ---------------------- | ------------------------------------- | ---------------------------------------------------- |
| **Broad source mount** | Mounts entire `/opt/torrust` as `:ro` | Mount only specific directories that need backing up |
| **No TLS for MySQL**   | Uses `--ssl=0`                        | Enable TLS for MySQL connections                     |

## Performance Issues

| Issue                       | Current State                               | Production Fix                                        |
| --------------------------- | ------------------------------------------- | ----------------------------------------------------- |
| **No I/O throttling**       | Unrestricted disk/network I/O during backup | Use `ionice`/`nice` or cgroups to limit I/O priority  |
| **Full copy every time**    | Copies all config files each backup cycle   | Use `rsync` with checksums to copy only changed files |
| **Unbounded backup growth** | No retention policy, backups accumulate     | Implement rotation (keep last N backups or N days)    |
| **No parallel operations**  | Sequential MySQL then config backup         | Could parallelize if backup window is tight           |

## Reliability Issues

| Issue                      | Current State                           | Production Fix                                                   |
| -------------------------- | --------------------------------------- | ---------------------------------------------------------------- |
| **No retry on failure**    | Script exits on first error (`set -e`)  | Add retry logic with exponential backoff for transient failures  |
| **No alerting**            | Failures are silent (only in logs)      | Send alerts on backup failure (email, Slack, PagerDuty, webhook) |
| **No health endpoint**     | Cannot query backup service status      | Expose HTTP `/health` and `/ready` endpoints                     |
| **No backup verification** | Assumes backup file is valid            | Periodically verify backups can be restored (restore test)       |
| **No locking mechanism**   | Could conflict with concurrent restores | Use `flock` or similar to prevent concurrent operations          |
| **No graceful shutdown**   | SIGTERM kills script immediately        | Trap signals, complete current backup before exiting             |

## Operational Issues

| Issue                       | Current State                       | Production Fix                                                   |
| --------------------------- | ----------------------------------- | ---------------------------------------------------------------- |
| **No retention policy**     | Backups accumulate indefinitely     | Auto-delete backups older than N days or keep only N most recent |
| **No size monitoring**      | Disk could fill up unnoticed        | Monitor backup size, alert when approaching disk limits          |
| **No backup catalog**       | Just files on disk, no metadata     | Create manifest file with timestamp, size, checksum per backup   |
| **No off-site copy**        | All backups stored locally          | Push copies to S3, GCS, or remote server for disaster recovery   |
| **No restore automation**   | Manual restore process only         | Provide automated restore script with validation                 |
| **No scheduling precision** | Sleep-based timing drifts over time | Use cron or systemd timer for precise scheduling                 |

## Technical Debt

| Issue                    | Current State                                 | Production Fix                                                             |
| ------------------------ | --------------------------------------------- | -------------------------------------------------------------------------- |
| **Bash implementation**  | Hard to test, limited error handling          | Consider Rust implementation with proper error types                       |
| **Plain text logging**   | Simple echo-based logs                        | Structured JSON logging for log aggregation (Loki, ELK)                    |
| **No metrics**           | Cannot monitor backup duration/size           | Expose Prometheus metrics (`backup_duration_seconds`, `backup_size_bytes`) |
| **No config validation** | Silently uses defaults for missing vars       | Validate required env vars at startup, fail fast with clear errors         |
| **No version pinning**   | Uses `default-mysql-client` package           | Pin specific package versions in Dockerfile for reproducibility            |
| **Hardcoded paths**      | `/backups/mysql`, `/backups/config` in script | Make all paths configurable via environment variables                      |

## Recommended Production Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    Backup Sidecar (Production)              │
├─────────────────────────────────────────────────────────────┤
│  Security:                                                  │
│  - Runs as non-root user (uid=1000, matching app user)     │
│  - Uses Docker secrets for database credentials             │
│  - Encrypts backups with GPG/age before storage             │
│  - Generates SHA256 checksums for integrity                 │
│  - Restrictive file permissions (600)                       │
├─────────────────────────────────────────────────────────────┤
│  Reliability:                                               │
│  - Retry with exponential backoff on transient failures     │
│  - Graceful shutdown (trap SIGTERM, complete current work)  │
│  - Health and readiness endpoints                           │
│  - Backup verification (periodic restore tests)             │
├─────────────────────────────────────────────────────────────┤
│  Operations:                                                │
│  - Configurable retention policy (N backups or N days)      │
│  - Off-site sync to S3/GCS/remote server                    │
│  - Alerting on failures (webhook, email, Slack)             │
│  - Backup catalog with metadata                             │
├─────────────────────────────────────────────────────────────┤
│  Observability:                                             │
│  - Structured JSON logging                                  │
│  - Prometheus metrics endpoint                              │
│  - Backup size and duration tracking                        │
└─────────────────────────────────────────────────────────────┘
```

## Priority Matrix

### Must Have (Before Production)

1. Run as non-root user
2. Encrypt backups at rest
3. Implement retention policy
4. Add alerting on failure
5. Secure credential management

### Should Have (Soon After)

1. Off-site backup copies
2. Health endpoints
3. Structured logging
4. Prometheus metrics
5. Backup verification tests

### Nice to Have (Future)

1. Rust implementation
2. Incremental backups
3. Point-in-time recovery
4. Web UI for backup management
5. Automated restore testing

## References

- [Docker Secrets](https://docs.docker.com/engine/swarm/secrets/)
- [age encryption](https://github.com/FiloSottile/age)
- [restic backup tool](https://restic.net/) - inspiration for paths-file approach
- [Prometheus metrics best practices](https://prometheus.io/docs/practices/naming/)
