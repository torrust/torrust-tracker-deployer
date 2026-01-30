# Sidecar Container Backup - Proof of Concept

**Status**: ✅ POC Complete | ⚠️ **Limited to small databases (< 1GB)**

## Overview

This folder contains the practical implementation and validation of the
sidecar container backup solution (see [design.md](design.md) for details).

**Goal**: Validate the sidecar container pattern by implementing it in a real
test environment, progressively adding features and documenting findings.

## ⚠️ Important Limitation

Real-world testing on a 17GB production database revealed that the sidecar
container pattern (using SQLite `.backup` or running continuously) is **only
practical for databases < 1GB**.

| Database Size | Sidecar Viability | Recommendation                                           |
| ------------- | ----------------- | -------------------------------------------------------- |
| < 1 GB        | ✅ Works          | Sidecar container is acceptable                          |
| > 1 GB        | ❌ Unusable       | Use [Maintenance Window](../maintenance-window/) instead |

**For production deployments, we recommend the [Maintenance Window](../maintenance-window/)
approach regardless of database size** - it's simpler and scales to any size.

## Environment

| Setting         | Value                        |
| --------------- | ---------------------------- |
| Name            | `manual-test-sidecar-backup` |
| Provider        | LXD (local)                  |
| Database        | MySQL                        |
| Backup storage  | Local filesystem only        |
| Backup interval | 2 minutes (for testing)      |
| IP Address      | `10.140.190.35`              |

## Implementation Status

| Phase | Description              | Status      | Document                                                    |
| ----- | ------------------------ | ----------- | ----------------------------------------------------------- |
| 1     | Environment Setup        | ✅ Complete | [01-environment-setup.md](phases/01-environment-setup.md)   |
| 2     | Minimal Backup Container | ✅ Complete | [02-minimal-container.md](phases/02-minimal-container.md)   |
| 3     | MySQL Backup             | ✅ Complete | [03-mysql-backup.md](phases/03-mysql-backup.md)             |
| 4     | Config Files Backup      | ✅ Complete | [04-config-backup.md](phases/04-config-backup.md)           |
| 5     | Backup Maintenance       | ✅ Complete | [05-archive-creation.md](phases/05-archive-creation.md)     |
| 6     | Restore Validation       | ✅ Complete | [06-restore-validation.md](phases/06-restore-validation.md) |
| 7     | Documentation Update     | ✅ Complete | [07-documentation.md](phases/07-documentation.md)           |

**POC Status**: ✅ **Complete** - All phases validated and documented.

## Directory Structure

```text
sidecar-container/
├── README.md                    # This file - overview and status
├── artifacts/                   # Configuration files and scripts
│   ├── environment-config.json  # Environment configuration
│   ├── backup-container/        # Container build context
│   │   ├── Dockerfile
│   │   ├── entrypoint.sh
│   │   └── backup-mysql.sh
│   ├── docker-compose-original.yml    # Original docker-compose
│   ├── docker-compose-with-backup.yml # With backup service
│   ├── mysql_20260129_185824.sql      # Sample backup (empty DB)
│   └── mysql_20260129_190424.sql      # Sample backup (with data)
├── phases/                      # Detailed documentation per phase
│   ├── 01-environment-setup.md
│   ├── 02-minimal-container.md
│   └── ...
└── troubleshooting.md           # Common issues and solutions
```

## Quick Commands

### Connect to Instance

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35
```

### Deployer Commands

```bash
# Provision → Configure → Release → Run
cargo run -- provision manual-test-sidecar-backup
cargo run -- configure manual-test-sidecar-backup
cargo run -- release manual-test-sidecar-backup
cargo run -- run manual-test-sidecar-backup

# Cleanup
cargo run -- destroy manual-test-sidecar-backup
```

### Instance Commands

```bash
cd /opt/torrust
docker compose ps
docker compose logs backup
docker compose exec backup /scripts/backup-all.sh
```

## Findings and Lessons Learned

<!-- Will be populated as we progress through phases -->

## References

- [Sidecar Container Design](design.md)
- [MySQL Backup Approaches](../../databases/mysql/backup-approaches.md)
- [Restic Best Practices](../../tools/restic.md#best-practices)
