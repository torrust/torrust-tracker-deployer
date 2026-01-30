# Phase 7: Documentation Update

**Status**: ✅ Complete
**Date**: 2026-01-30

## Goal

Update research documentation with findings from real-world testing.

## Checklist

- [x] Document final working configuration
- [x] Add lessons learned
- [x] Update implementation status checklist
- [x] Document critical large database findings
- [x] Update preliminary conclusions

## Documentation Updates

### New Documents Created

| Document                                                                       | Description                                             |
| ------------------------------------------------------------------------------ | ------------------------------------------------------- |
| [large-database-backup.md](../../../databases/sqlite/large-database-backup.md) | Critical findings from 17GB production database testing |
| [06-restore-validation.md](06-restore-validation.md)                           | Complete restore procedures and disaster recovery       |

### Documents Updated

| Document                                                               | Changes                                                           |
| ---------------------------------------------------------------------- | ----------------------------------------------------------------- |
| [conclusions.md](../../../conclusions.md)                              | Added critical large database warning, size-based recommendations |
| [sqlite/README.md](../../../databases/sqlite/README.md)                | Added size-based viability table, link to large database findings |
| [backup-approaches.md](../../../databases/sqlite/backup-approaches.md) | Added warning about large database limitations                    |
| [POC README.md](../README.md)                                          | Updated phase status, IP address                                  |

## Lessons Learned

### 1. SQLite `.backup` Does Not Scale

**Finding**: The SQLite `.backup` command is fundamentally unsuitable for large
databases under concurrent load.

| Database Size  | Observed Behavior                        |
| -------------- | ---------------------------------------- |
| 17 GB          | Stalled at 10% after 16+ hours           |
| Effective rate | ~37 MB/hour (disk capable of 445 MB/s)   |
| Cause          | Lock contention, restart-on-modification |

**Implication**: The sidecar container backup pattern only works for databases
< 1 GB. Larger databases require alternative approaches.

### 2. Maintenance Window Backup is Practical

**Finding**: Stopping the application for backup is far more practical than
expected for large databases.

| Metric         | Value       |
| -------------- | ----------- |
| Database size  | 17 GB       |
| Copy time      | 72 seconds  |
| Total downtime | ~90 seconds |
| Speed          | ~240 MB/s   |

For a tracker, 90 seconds of downtime is acceptable for daily backups.

### 3. Disk I/O is Not the Bottleneck

The `dd` test showed 445 MB/s read speed. The SQLite backup mechanism itself
(locking, page copying, restart behavior) is the bottleneck - not disk I/O.

### 4. Unit Tests Catch Real Bugs

The bats-core unit tests caught a real bug: the `is_comment_or_empty` function
didn't handle whitespace-only lines correctly. 44 unit tests now cover the
backup script.

### 5. Test Enforcement in Docker Build Works Well

The multi-stage Dockerfile pattern (test stage creates marker, production stage
requires it via `COPY --from=test`) ensures tests cannot be skipped.

### 6. Restore Testing is Essential

Testing actual restore procedures revealed important details:

- MySQL restore is fast (~15 seconds for small DBs)
- Config restore requires careful permission handling
- Full disaster recovery RTO is achievable in minutes

## Final Configuration

The POC backup sidecar container is complete and tested. See:

- [Dockerfile](../artifacts/backup-container/Dockerfile) - Multi-stage with tests
- [backup.sh](../artifacts/backup-container/backup.sh) - Unified backup script
- [backup_test.bats](../artifacts/backup-container/backup_test.bats) - 44 unit tests

### Configuration Highlights

```yaml
# docker-compose addition
backup:
  build: ./backup
  volumes:
    - tracker_data:/data/tracker:ro
    - mysql_data:/data/mysql:ro
    - backup_storage:/backups
  environment:
    - BACKUP_INTERVAL=120 # 2 minutes for testing
    - MYSQL_HOST=mysql
    - MYSQL_USER=torrust
    - MYSQL_PASSWORD=torrust
    - MYSQL_DATABASE=torrust_tracker
    - RETENTION_DAYS=7
```

### Backup Coverage

| Component      | Method        | Location                   |
| -------------- | ------------- | -------------------------- |
| MySQL database | `mysqldump`   | `/backups/mysql/`          |
| Tracker config | File copy     | `/backups/config/tracker/` |
| Backup script  | 44 unit tests | Run during Docker build    |

## Scope Limitations

### What This POC Validates

- ✅ Sidecar container pattern works for small databases
- ✅ MySQL backup/restore is reliable and fast
- ✅ Config file backup/restore works correctly
- ✅ Unit testing backup scripts is valuable
- ✅ Test enforcement in Docker build prevents regressions

### What This POC Does NOT Validate

- ❌ Large database backup (> 1 GB) - **proven impractical with `.backup`**
- ❌ Remote/off-site backup storage (S3, etc.)
- ❌ Encrypted backups
- ❌ Backup verification automation
- ❌ Alerting on backup failures

## Recommendations

### For Small Deployments (< 1 GB database)

Use the sidecar container pattern as implemented in this POC:

1. Add backup container to docker-compose
2. Mount data volumes read-only
3. Use `.backup` for SQLite, `mysqldump` for MySQL
4. Implement 7-day retention with cleanup

### For Large Deployments (> 1 GB database)

Do NOT use the sidecar container pattern. Instead:

1. **Scheduled maintenance window**: Stop app, copy files, restart
2. **Filesystem snapshots**: LVM/ZFS for instant, consistent snapshots
3. **Litestream**: Continuous SQLite replication to S3
4. **Consider MySQL**: Better tooling for large databases

See [Large Database Backup](../../../databases/sqlite/large-database-backup.md) for
detailed analysis.

## Next Steps

- [x] Cleanup test environment: `cargo run -- destroy manual-test-sidecar-backup`
- [ ] Consider implementing maintenance window backup script
- [ ] Evaluate Litestream for SQLite continuous replication
- [ ] Plan integration into deployer (future task, if deemed valuable)
