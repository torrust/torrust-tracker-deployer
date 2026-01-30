# Backup Solutions

This directory contains proposed solutions for implementing database backups
in Torrust Tracker deployments.

## Solutions

| Solution                                  | Status         | Description                                        |
| ----------------------------------------- | -------------- | -------------------------------------------------- |
| [Sidecar Container](sidecar-container/)   | ⭐ Recommended | Dedicated backup container in Docker Compose stack |
| [Exclude Statistics](exclude-statistics/) | 🔬 Proposed    | Backup only essential data, exclude large stats    |
| [Maintenance Window](maintenance-window/) | 🔬 Proposed    | Host-level script: stop tracker, copy DB, restart  |

> **Note**: The sidecar container approach is only practical for databases < 1GB.
> For larger databases, consider the alternative solutions.

## Selection Criteria

When evaluating backup solutions, we considered:

1. **Portability** - Solution should move with the deployment stack
2. **Single Responsibility** - Each container should do one thing well
3. **Minimal Coupling** - Avoid tight coupling to host VM
4. **Consistency** - Work identically for SQLite and MySQL
5. **Simplicity** - Easy to understand and maintain

## Rejected Approaches

| Approach                    | Why Rejected                                                 |
| --------------------------- | ------------------------------------------------------------ |
| Backup inside app container | Violates single-process principle, needs supervisord         |
| Backup scripts on host VM   | Not portable, tied to specific VM, harder to version control |
| Manual backups              | Not reliable, human error prone                              |

## Implementation Status

- [ ] Sidecar container pattern documented
- [ ] Docker Compose templates created
- [ ] Restic integration tested
- [ ] Cron scheduling validated
