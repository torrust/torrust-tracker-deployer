# Conclusions

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Summary

This document summarizes the key findings from the database backup strategies
research and provides the **recommended production solution**.

## ⭐ Recommended Solution: Maintenance Window Backup

After extensive testing and analysis, we recommend the **maintenance-window
hybrid approach** for production deployments. This solution combines:

1. **Container portability** - 95%+ of backup logic in Docker container
2. **Host-level orchestration** - Simple crontab + script for scheduling
3. **Works for any database size** - No SQLite locking issues

### Architecture

```text
Crontab (3:00 AM) → maintenance-backup.sh → Stop Tracker → Run Backup Container → Start Tracker
```

### Why This Approach

| Factor              | Decision                                                      |
| ------------------- | ------------------------------------------------------------- |
| Database size       | Works for any size (17GB in ~90s vs 16+ hours with `.backup`) |
| Portability         | Container image is portable, only ~50 lines of host script    |
| Complexity          | Lower than always-running sidecar with locking workarounds    |
| Downtime            | Acceptable (~90s at 3 AM for most trackers)                   |
| Deployer automation | Could automate crontab installation in "Configure" phase      |

### Solution Comparison

| Approach                     | Small DB (< 1GB) | Large DB (> 1GB) | Host Setup | Downtime |
| ---------------------------- | ---------------- | ---------------- | ---------- | -------- |
| **Sidecar (always running)** | ✅ Works         | ❌ Unusable      | None       | None     |
| **Maintenance Window**       | ✅ Works         | ✅ Works         | Minimal    | ~90s     |
| **Exclude Statistics**       | ✅ Works         | ⚠️ Size only     | None       | None     |

**Recommendation**: Use maintenance-window for all production deployments.
The sidecar-only approach is acceptable for small databases but lacks
scalability.

See [Maintenance Window Solution](solutions/maintenance-window/) for
implementation details and tested artifacts.

## SQLite Backup

| Finding            | Recommendation                                                              |
| ------------------ | --------------------------------------------------------------------------- |
| Safe backup method | Use `.backup` command (Online Backup API)                                   |
| WAL mode           | Optional - not required for safe backups, helps read performance under load |
| Torrust Live Demo  | Currently uses unsafe `cp`, should switch to `.backup`                      |
| Verification       | Use `PRAGMA integrity_check` after restore                                  |

**Key insight**: The `.backup` command is safe during concurrent writes because it uses SQLite's Online Backup API, which handles page-level consistency internally.

### ⚠️ Critical Finding: Large Database Limitations

**Real-world testing on the 17GB Torrust Demo production database revealed that
the `.backup` command is impractical for large databases under load.**

| Database Size | `.backup` Viability | Recommendation                       |
| ------------- | ------------------- | ------------------------------------ |
| < 1 GB        | ✅ Works well       | Use `.backup` (sidecar container OK) |
| 1 GB - 10 GB  | ⚠️ Slow             | Consider maintenance window          |
| > 10 GB       | ❌ Unusable         | **Must use alternatives**            |

**Test results (17GB database)**:

- `.backup` ran for 16+ hours, stalled at 10% (~37 MB/hour)
- Disk capable of 445 MB/s - SQLite locking is the bottleneck
- Simple `cp` with tracker stopped: **72 seconds** (complete backup)

**Implication for backup sidecar container**:

The backup sidecar container approach (using `.backup` inside a Docker container)
is **only practical for small databases (< 1GB)**. For larger databases:

1. **Maintenance window backup**: Stop the tracker, copy files, restart (~90
   seconds downtime for 17GB)
2. **Filesystem snapshots**: Use LVM/ZFS snapshots (instant, no downtime)
3. **Litestream**: Continuous replication to S3 (near-zero RPO)
4. **Migrate to MySQL**: Better tooling for large databases

See [Large Database Backup](sqlite/large-database-backup.md) for detailed
analysis and recommendations.

## Backup Tool

| Finding          | Recommendation                                          |
| ---------------- | ------------------------------------------------------- |
| Recommended tool | Restic - mature, encrypted, deduplicated, Docker-native |
| Backup approach  | Two-phase: DB dump first, then file backup              |
| Docker image     | Custom image needed (add `sqlite3` and `mysql-client`)  |
| Alternatives     | Kopia (newer, more features, less mature)               |

**Key insight**: Backup tools like Restic backup **files**, not databases. Databases require a pre-hook script to create a dump file first.

## Architecture Pattern

| Finding    | Recommendation                                  |
| ---------- | ----------------------------------------------- |
| Pattern    | External Tool (Restic container)                |
| Design     | Centralized backup container in Docker Compose  |
| Philosophy | Use well-tested tools instead of custom scripts |

## Scope Decision

**Document and recommend, but don't automate in the deployer yet.**

### Rationale

| Factor                            | Implication                                                                                                     |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| **Opinionated**                   | Backup strategies vary widely based on user preferences                                                         |
| **Provider-dependent**            | Cloud providers offer native backup/snapshot tools (DigitalOcean snapshots, AWS EBS snapshots, Hetzner backups) |
| **Infrastructure vs Application** | Some users prefer infrastructure-level backups over application-level                                           |
| **Complexity**                    | Automating backups adds significant configuration surface area                                                  |

### Recommended Approach

1. ✅ **Document best practices** - Completed in this research
2. ✅ **Implement manually in Torrust Live Demo** - Use `.backup` command (issues created: #85, #86)
3. ❌ **Don't automate in deployer** - Keep deployer focused on core deployment
4. 📄 **Provide templates/examples** - Reference for users who want to implement backups

This keeps the deployer focused on core deployment functionality while users retain flexibility to choose their preferred backup strategy based on their infrastructure and requirements.

## Pending Research

The following areas were identified but not fully researched:

- [x] MySQL backup strategies (mysqldump) - Implemented in backup container
- [ ] Complete storage folder backup approaches
- [ ] Selective files backup strategies
- [ ] Off-site backup synchronization (S3, rsync)
- [ ] Backup encryption at rest

These may be addressed in future research if needed.

## References

- [Maintenance Window Solution](solutions/maintenance-window/) ⭐ **Recommended**
- [Sidecar Container Solution](solutions/sidecar-container/) (small DBs only)
- [SQLite Backup Approaches](databases/sqlite/backup-approaches.md)
- [SQLite Large Database Backup](databases/sqlite/large-database-backup.md) ⚠️ **Critical
  for databases > 1GB**
- [Container Backup Architectures](architectures/container-patterns.md)
- [Restic Evaluation](tools/restic.md)
- [Restic vs Kopia Comparison](tools/restic-vs-kopia.md)
