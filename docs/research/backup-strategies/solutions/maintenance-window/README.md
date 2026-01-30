# Maintenance Window Backup

**Status**: 🔬 Proposed

## Summary

Host-level backup with a scheduled maintenance window that stops the tracker,
performs a full database copy, and restarts the service.

## Problem

Container-based backup using SQLite `.backup` is impractical for large databases
(> 1GB) due to locking overhead. The backup sidecar cannot safely stop the
tracker to perform a cold copy.

## Solution

Use host-level orchestration (crontab + bash script) to:

1. Stop the Docker Compose stack
2. Copy database files directly (cold backup)
3. Restart the Docker Compose stack
4. Optionally upload to off-site storage

## Trade-offs

| Advantage                            | Disadvantage                               |
| ------------------------------------ | ------------------------------------------ |
| Complete backup with all data        | Brief service interruption (~90s for 17GB) |
| Simple and reliable (cp cannot fail) | Not portable with Docker stack             |
| No SQLite locking issues             | Requires host-level access                 |
| Predictable backup duration          | Not suitable for pure PaaS deployments     |

## Implementation Notes

- Schedule during low-traffic window (e.g., 3:00 AM)
- Tested: 72 seconds for 17GB SQLite database on SSD
- Can integrate with Restic for off-site encrypted backups
