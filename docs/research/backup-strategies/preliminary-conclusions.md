# Preliminary Conclusions

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Summary

This document summarizes the key findings from the database backup strategies research.

## SQLite Backup

| Finding            | Recommendation                                                              |
| ------------------ | --------------------------------------------------------------------------- |
| Safe backup method | Use `.backup` command (Online Backup API)                                   |
| WAL mode           | Optional - not required for safe backups, helps read performance under load |
| Torrust Live Demo  | Currently uses unsafe `cp`, should switch to `.backup`                      |
| Verification       | Use `PRAGMA integrity_check` after restore                                  |

**Key insight**: The `.backup` command is safe during concurrent writes because it uses SQLite's Online Backup API, which handles page-level consistency internally.

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

- [ ] MySQL backup strategies (mysqldump, hot backups)
- [ ] Complete storage folder backup approaches
- [ ] Selective files backup strategies

These may be addressed in future research if needed.

## References

- [SQLite Backup Approaches](sqlite/backup-approaches.md)
- [Container Backup Architectures](container-backup-architectures.md)
- [Restic Evaluation](tools/restic.md)
- [Restic vs Kopia Comparison](tools/restic-vs-kopia.md)
