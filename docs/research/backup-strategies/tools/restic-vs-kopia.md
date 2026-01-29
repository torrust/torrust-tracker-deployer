# Restic vs Kopia Comparison

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Overview

Both Restic and Kopia are modern, open-source backup tools written in Go. They share many features but have different philosophies and target audiences.

| Tool                          | First Release | Philosophy                |
| ----------------------------- | ------------- | ------------------------- |
| [Restic](https://restic.net/) | 2015          | Simple, do one thing well |
| [Kopia](https://kopia.io/)    | 2019          | Feature-rich, all-in-one  |

## Feature Comparison

| Feature                 | Restic                      | Kopia                           |
| ----------------------- | --------------------------- | ------------------------------- |
| **Language**            | Go                          | Go                              |
| **Single Binary**       | ✅ Yes                      | ✅ Yes                          |
| **Deduplication**       | ✅ Content-defined chunking | ✅ Content-defined chunking     |
| **Encryption**          | AES-256 (mandatory)         | AES-256 or ChaCha20 (mandatory) |
| **Compression**         | zstd (since v0.14)          | pgzip, s2, zstd                 |
| **Local Backups**       | ✅ Yes                      | ✅ Yes                          |
| **SFTP**                | ✅ Yes                      | ✅ Yes                          |
| **S3**                  | ✅ Yes                      | ✅ Yes                          |
| **Backblaze B2**        | ✅ Yes                      | ✅ Yes                          |
| **Azure Blob**          | ✅ Yes                      | ✅ Yes                          |
| **Google Cloud**        | ✅ Yes                      | ✅ Yes                          |
| **rclone**              | ✅ Yes                      | ✅ Yes (experimental)           |
| **WebDAV**              | ❌ (via rclone)             | ✅ Native                       |
| **Retention Policies**  | ✅ CLI flags                | ✅ Hierarchical policies        |
| **Integrity Check**     | ✅ `restic check`           | ✅ Built-in verification        |
| **Mount as Filesystem** | ✅ FUSE                     | ✅ FUSE                         |
| **Docker Image**        | ~16 MB                      | ~20 MB                          |

## Key Differentiators

### What Kopia Has That Restic Doesn't

| Feature                    | Description                          | Importance for Torrust      |
| -------------------------- | ------------------------------------ | --------------------------- |
| **Official GUI**           | Full graphical interface             | Low - we use Docker/CLI     |
| **Error Correction (ECC)** | Reed-Solomon protects against bitrot | Medium - nice for large DBs |
| **Repository Server**      | Centralized management with REST API | Low - single-server setup   |
| **Hierarchical Policies**  | Global → host → path inheritance     | Low - simple use case       |
| **WebDAV Native**          | No rclone needed for WebDAV          | Low - we use S3/SFTP        |

### What Restic Has That Kopia Doesn't

| Feature                | Description                            | Importance for Torrust        |
| ---------------------- | -------------------------------------- | ----------------------------- |
| **Maturity**           | 4+ years older, more battle-tested     | High - production use         |
| **Larger Community**   | More tutorials, wrappers, integrations | High - easier troubleshooting |
| **Simpler Model**      | Fewer concepts to learn                | Medium - faster onboarding    |
| **More Documentation** | Extensive official and community docs  | High - self-service           |

## Docker Comparison

Both provide official Docker images based on Alpine Linux.

### Restic

```bash
docker pull restic/restic
# Size: ~16 MB
# Base: Alpine
# Includes: ca-certificates, fuse, openssh-client, tzdata, jq
```

### Kopia

```bash
docker pull kopia/kopia
# Size: ~20 MB
# Base: Alpine
# Includes: ca-certificates, fuse
```

Both require custom images to add database clients (`sqlite3`, `mysql-client`).

## Performance

Both tools are fast and efficient. Benchmarks show similar performance for most use cases:

| Operation          | Restic    | Kopia     |
| ------------------ | --------- | --------- |
| Initial backup     | Fast      | Fast      |
| Incremental backup | Fast      | Fast      |
| Deduplication      | Excellent | Excellent |
| Restore            | Fast      | Fast      |

Kopia claims to be faster in some scenarios, but real-world differences are minimal for our use case.

## Community and Ecosystem

| Aspect           | Restic                             | Kopia   |
| ---------------- | ---------------------------------- | ------- |
| GitHub Stars     | ~27k                               | ~8k     |
| Contributors     | ~200                               | ~100    |
| First Release    | 2015                               | 2019    |
| 3rd Party Tools  | Many (Autorestic, Runrestic, etc.) | Growing |
| Tutorials/Guides | Extensive                          | Good    |
| Stack Overflow   | Many questions/answers             | Fewer   |

## Our Use Case Analysis

For the Torrust Tracker Deployer backup scenario:

| Requirement              | Restic   | Kopia      | Notes                      |
| ------------------------ | -------- | ---------- | -------------------------- |
| Docker support           | ✅       | ✅         | Both work                  |
| SQLite backup (pre-hook) | ✅       | ✅         | Both work                  |
| MySQL backup (pre-hook)  | ✅       | ✅         | Both work                  |
| S3/B2 cloud storage      | ✅       | ✅         | Both native                |
| Encryption               | ✅       | ✅         | Both mandatory             |
| Deduplication            | ✅       | ✅         | Both excellent             |
| Retention policies       | ✅       | ✅         | Both support               |
| Maturity                 | ✅ High  | ⚠️ Medium  | Restic is older            |
| Community resources      | ✅ Large | ⚠️ Smaller | More Restic help available |

## Recommendation

**Restic is the better choice for Torrust Tracker Deployer.**

### Reasons

1. **Maturity**: 4+ years older, proven in production at scale
2. **Community**: More tutorials, answers, and integrations available
3. **Simplicity**: Easier to understand and troubleshoot
4. **Sufficient Features**: Has everything we need without extra complexity
5. **"Boring Technology"**: Stable, predictable, well-understood

### When to Reconsider Kopia

- If we need centralized backup management for many servers
- If Reed-Solomon error correction becomes critical
- If we want a GUI for operators
- If Kopia matures significantly (check again in 2-3 years)

## References

- [Restic Documentation](https://restic.readthedocs.io/)
- [Kopia Documentation](https://kopia.io/docs/)
- [Restic GitHub](https://github.com/restic/restic)
- [Kopia GitHub](https://github.com/kopia/kopia)
