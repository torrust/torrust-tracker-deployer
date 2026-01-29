# Sidecar Container Backup - Proof of Concept

## Overview

This folder contains the practical implementation and validation of the
[sidecar container backup solution](../sidecar-container.md).

**Goal**: Validate the sidecar container pattern by implementing it in a real
test environment, progressively adding features and documenting findings.

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

| Phase | Description              | Status         | Document                                                    |
| ----- | ------------------------ | -------------- | ----------------------------------------------------------- |
| 1     | Environment Setup        | ✅ Complete    | [01-environment-setup.md](phases/01-environment-setup.md)   |
| 2     | Minimal Backup Container | ✅ Complete    | [02-minimal-container.md](phases/02-minimal-container.md)   |
| 3     | MySQL Backup             | ✅ Complete    | [03-mysql-backup.md](phases/03-mysql-backup.md)             |
| 4     | Config Files Backup      | 🔲 Not started | [04-config-backup.md](phases/04-config-backup.md)           |
| 5     | Archive Creation         | 🔲 Not started | [05-archive-creation.md](phases/05-archive-creation.md)     |
| 6     | Restore Validation       | 🔲 Not started | [06-restore-validation.md](phases/06-restore-validation.md) |
| 7     | Documentation Update     | 🔲 Not started | [07-documentation.md](phases/07-documentation.md)           |

## Directory Structure

```text
poc/
├── README.md                    # This file - overview and status
├── artifacts/                   # Configuration files and scripts
│   ├── environment-config.json  # Environment configuration
│   ├── Dockerfile               # Backup container Dockerfile
│   ├── entrypoint.sh            # Backup container entrypoint
│   ├── docker-compose-original.yml    # Original docker-compose
│   └── docker-compose-with-backup.yml # With backup service
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

- [Sidecar Container Solution](../sidecar-container.md)
- [MySQL Backup Approaches](../../mysql/backup-approaches.md)
- [Restic Best Practices](../../tools/restic.md#best-practices)
