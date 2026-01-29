# Sidecar Container Backup - Proof of Concept

> **Note**: This document is a quick reference. For detailed documentation,
> artifacts, and phase-by-phase records, see the [poc/](poc/README.md) folder.

## Quick Status

| Phase | Description              | Status         |
| ----- | ------------------------ | -------------- |
| 1     | Environment Setup        | ✅ Complete    |
| 2     | Minimal Backup Container | 🔲 Not started |
| 3     | MySQL Backup             | 🔲 Not started |
| 4     | Config Files Backup      | 🔲 Not started |
| 5     | Archive Creation         | 🔲 Not started |
| 6     | Restore Validation       | 🔲 Not started |
| 7     | Documentation Update     | 🔲 Not started |

## Documentation Structure

```text
poc/
├── README.md                    # Overview and status tracking
├── artifacts/                   # Configuration files and scripts
│   ├── environment-config.json
│   ├── docker-compose-backup.yml
│   └── scripts/
├── phases/                      # Detailed docs per phase
│   ├── 01-environment-setup.md  # ✅ Complete
│   ├── 02-minimal-container.md
│   └── ...
└── troubleshooting.md           # Common issues
```

## Environment

| Setting    | Value                        |
| ---------- | ---------------------------- |
| Name       | `manual-test-sidecar-backup` |
| IP Address | `10.140.190.35`              |
| Provider   | LXD (local)                  |
| Database   | MySQL                        |

## Quick Commands

```bash
# Connect to instance
ssh -i fixtures/testing_rsa torrust@10.140.190.35

# Run deployer commands
cargo run -- {provision|configure|release|run|destroy} manual-test-sidecar-backup
```

---

## Detailed Documentation

For detailed phase documentation, commands with outputs, and troubleshooting:

- **[poc/README.md](poc/README.md)** - Full status and overview
- **[poc/phases/](poc/phases/)** - Detailed records per phase
- **[poc/artifacts/](poc/artifacts/)** - Configuration files and scripts
- **[poc/troubleshooting.md](poc/troubleshooting.md)** - Common issues

## Findings and Lessons Learned

<!-- Will be populated during implementation -->

## References

- [Sidecar Container Solution](sidecar-container.md)
- [MySQL Backup Approaches](../mysql/backup-approaches.md)
- [Manual E2E Testing Guide](../../../e2e-testing/manual-testing.md)
