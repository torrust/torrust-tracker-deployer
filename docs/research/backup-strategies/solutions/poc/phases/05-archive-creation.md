# Phase 5: Archive Creation

**Status**: 🔲 Not started
**Date**: -

## Goal

Create timestamped tar.gz archives with retention policy.

## Checklist

- [ ] Update script to create staging directory
- [ ] Stage all files (database dump + configs)
- [ ] Create compressed archive
- [ ] Implement 7-day local retention
- [ ] Clean up staging directory after archive

## Artifacts

- Complete backup script: [../artifacts/scripts/backup-all.sh](../artifacts/scripts/backup-all.sh)

## Commands Executed

<!-- Will be populated during implementation -->

## Validation

**Expected**: `backup_YYYYMMDD_HHMMSS.tar.gz` files in `/backups/`:

```bash
ls -la /opt/torrust/backups/
# backup_20260129_183000.tar.gz
# backup_20260129_185000.tar.gz
```

Archive contents:

```bash
tar -tzf /opt/torrust/backups/backup_20260129_183000.tar.gz
```

## Issues Encountered

<!-- Will be populated if issues arise -->

## Next Steps

Proceed to [Phase 6: Restore Validation](06-restore-validation.md).
