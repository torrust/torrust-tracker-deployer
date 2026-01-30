# Backup Solutions

This directory contains proposed solutions for implementing database backups
in Torrust Tracker deployments.

## Solutions

| Solution                                  | Status             | Best For                   |
| ----------------------------------------- | ------------------ | -------------------------- |
| [Maintenance Window](maintenance-window/) | ⭐ **Recommended** | All production deployments |
| [Sidecar Container](sidecar-container/)   | ✅ POC Complete    | Small databases (< 1GB)    |
| [Exclude Statistics](exclude-statistics/) | 🔬 Proposed        | Size optimization only     |

## Recommendation

**Use the Maintenance Window approach for all production deployments.**

The sidecar container approach is elegant but only works for small databases.
Real-world testing on a 17GB production database showed:

- SQLite `.backup`: 16+ hours (stalled at 10%)
- Maintenance window: **90 seconds** (complete backup)

The maintenance-window hybrid approach provides:

- **95%+ portable** - Backup logic in container, only ~50 lines on host
- **Scales to any DB size** - No locking issues
- **Acceptable downtime** - ~90s at 3 AM for most trackers
- **Deployer-compatible** - Could automate crontab in "Configure" phase

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
