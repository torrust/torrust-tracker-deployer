# Phase 5: Backup Maintenance (Packaging & Retention)

**Status**: ✅ Complete
**Date**: 2026-01-29

## Goal

Add a maintenance phase that runs after each backup cycle to compress older
backups and apply retention policies.

## Design Decisions

### Two-Phase Approach

The backup process now has two logical phases:

1. **Backup Phase** (Phase 3-4): Generate raw dumps + copy configs (fast)
2. **Maintenance Phase** (this phase): Compress + cleanup (can be slower)

This separation ensures:

- Minimal database lock time during dump
- Compression doesn't block backup operations
- Retention cleanup is logically separate

### No Overlap Concern

The current script is sequential - the loop only repeats after the previous
backup completes:

```bash
while true; do
    sleep "${interval}"   # Wait first
    run_backup            # Blocks until complete (including maintenance)
done
```

If a backup takes longer than the interval, the next backup simply starts
later. No concurrent backups are possible with this design.

### Why Not Restic (Yet)

For the first release, simple bash-based compression and retention is
sufficient. Restic adds value when you need:

- Incremental/deduplicated backups
- Encryption at rest
- Cloud storage backends (S3, B2, etc.)
- Sophisticated retention policies

Advanced users can integrate restic later if needed.

## Checklist

- [ ] Add `run_maintenance()` function after backup cycle
- [ ] Implement `compress_old_backups()` - gzip config files older than 1 hour
- [ ] Implement `apply_retention_policy()` - delete backups older than N days
- [ ] Add `BACKUP_RETENTION_DAYS` environment variable (default: 7)
- [ ] Document the maintenance behavior

## Implementation

### New Functions

```bash
run_maintenance() {
    compress_old_backups
    apply_retention_policy
}

compress_old_backups() {
    # Compress config files older than 1 hour (MySQL already compressed)
    find /backups/config -type f ! -name "*.gz" -mmin +60 -exec gzip {} \;
}

apply_retention_policy() {
    local days="${BACKUP_RETENTION_DAYS:-7}"
    find /backups/mysql -name "*.sql.gz" -mtime +"${days}" -delete
    find /backups/config -type f -mtime +"${days}" -delete
}
```

### Environment Variables

| Variable                | Default | Description                              |
| ----------------------- | ------- | ---------------------------------------- |
| `BACKUP_RETENTION_DAYS` | 7       | Delete backups older than this many days |

## Validation

**Test retention policy** (with short interval for testing):

```bash
# Set retention to 0 days to test immediate cleanup
docker exec backup-test env BACKUP_RETENTION_DAYS=0 \
  find /backups/mysql -name "*.sql.gz" -mtime +0 -delete

# Verify old backups are removed
ls -la /opt/torrust/storage/backup/lib/mysql/
```

**Test compression**:

```bash
# Check for gzipped config files
find /opt/torrust/storage/backup/lib/config -name "*.gz"
```

## Issues Encountered

<!-- Will be populated if issues arise -->

## Next Steps

Proceed to [Phase 6: Restore Validation](06-restore-validation.md).
