# Tracker Backup Container - Security Scans

Security scan history for the `torrust/tracker-backup` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status                        | Last Scan   |
| ------- | ---- | -------- | ----------------------------- | ----------- |
| local   | 7    | 3        | ⚠️ Vulnerabilities in base OS | Feb 2, 2026 |

## Scan History

### February 2, 2026

**Image**: `torrust/tracker-backup:local`
**Trivy Version**: 0.68.2
**Base OS**: Debian 12.13 (bookworm-slim)
**Status**: ⚠️ **10 vulnerabilities found** (7 HIGH, 3 CRITICAL)

#### Summary

The tracker-backup container is based on `debian:bookworm-slim` and contains vulnerabilities in system libraries that are part of the base OS. All vulnerabilities are in upstream Debian packages with status "affected" (no fix available yet) or "will_not_fix".

**Installed Tools**:

- `bash` - Backup script execution
- `default-mysql-client` (MySQL 8) - Database dumps
- `sqlite3` - SQLite backups
- `gzip` - Compression
- `tar` - Archive creation
- `bats` - Unit testing (only in test stage, not in production image)

#### Detailed Results

```text
torrust/tracker-backup:local (debian 12.13)

Total: 10 (HIGH: 7, CRITICAL: 3)

┌─────────────────────┬────────────────┬──────────┬──────────────┬──────────────────────┬───────────────┬──────────────────────────────────────────────────────────────┐
│       Library       │ Vulnerability  │ Severity │    Status    │  Installed Version   │ Fixed Version │                            Title                             │
├─────────────────────┼────────────────┼──────────┼──────────────┼──────────────────────┼───────────────┼──────────────────────────────────────────────────────────────┤
│ gpgv                │ CVE-2026-24882 │ HIGH     │ affected     │ 2.2.40-1.1+deb12u2   │               │ GnuPG: Stack-based buffer overflow in tpm2daemon            │
│                     │                │          │              │                      │               │ allows arbitrary code execution                              │
├─────────────────────┼────────────────┤          │              ├──────────────────────┼───────────────┼──────────────────────────────────────────────────────────────┤
│ libc-bin            │ CVE-2026-0861  │          │              │ 2.36-9+deb12u13      │               │ glibc: Integer overflow in memalign leads to heap corruption │
├─────────────────────┤                │          │              │                      ├───────────────┤                                                              │
│ libc6               │                │          │              │                      │               │                                                              │
├─────────────────────┼────────────────┤          │              ├──────────────────────┼───────────────┼──────────────────────────────────────────────────────────────┤
│ libmariadb3         │ CVE-2025-13699 │          │              │ 1:10.11.14-0+deb12u2 │               │ mariadb: mariadb-dump utility vulnerable to remote           │
│                     │                │          │              │                      │               │ code execution via improper path                             │
├─────────────────────┼────────────────┼──────────┤              ├──────────────────────┼───────────────┼──────────────────────────────────────────────────────────────┤
│ libsqlite3-0        │ CVE-2025-7458  │ CRITICAL │              │ 3.40.1-2+deb12u2     │               │ sqlite: SQLite integer overflow                              │
├─────────────────────┼────────────────┼──────────┤              ├──────────────────────┼───────────────┼──────────────────────────────────────────────────────────────┤
│ mariadb-client      │ CVE-2025-13699 │ HIGH     │              │ 1:10.11.14-0+deb12u2 │               │ mariadb: mariadb-dump utility vulnerable to remote           │
│                     │                │          │              │                      │               │ code execution via improper path                             │
├─────────────────────┤                │          │              │                      ├───────────────┤                                                              │
│ mariadb-client-core │                │          │              │                      │               │                                                              │
├─────────────────────┤                │          │              │                      ├───────────────┤                                                              │
│ mariadb-common      │                │          │              │                      │               │                                                              │
├─────────────────────┼────────────────┼──────────┤              ├──────────────────────┼───────────────┼──────────────────────────────────────────────────────────────┤
│ sqlite3             │ CVE-2025-7458  │ CRITICAL │              │ 3.40.1-2+deb12u2     │               │ sqlite: SQLite integer overflow                              │
├─────────────────────┼────────────────┤          ├──────────────┼──────────────────────┼───────────────┼──────────────────────────────────────────────────────────────┤
│ zlib1g              │ CVE-2023-45853 │          │ will_not_fix │ 1:1.2.13.dfsg-1      │               │ zlib: integer overflow and resultant heap-based buffer       │
│                     │                │          │              │                      │               │ overflow in zipOpenNewFileInZip4_6                           │
└─────────────────────┴────────────────┴──────────┴──────────────┴──────────────────────┴───────────────┴──────────────────────────────────────────────────────────────┘
```

#### Risk Assessment

**Current Risk Level**: ⚠️ **MEDIUM-HIGH**

All vulnerabilities are in upstream Debian packages. The container itself:

- ✅ Minimal package footprint (reduces attack surface)
- ✅ Non-root user execution (UID 1000)
- ✅ Read-only configuration mounts
- ✅ Comprehensive unit test coverage (44 tests)
- ⚠️ Contains vulnerabilities in base OS libraries

**Vulnerability Analysis**:

1. **CVE-2025-7458** (CRITICAL) - SQLite integer overflow
   - Impact: Affects `libsqlite3-0` and `sqlite3` binary
   - Risk: Required for SQLite database backups
   - Mitigation: No fix available yet in Debian 12
   - Status: Monitor for Debian security updates

2. **CVE-2025-13699** (HIGH) - MariaDB dump RCE
   - Impact: Affects `mariadb-client` and related packages
   - Risk: Used for MySQL database backups via `mysqldump`
   - Mitigation: No fix available yet in Debian 12
   - Status: Monitor for Debian security updates

3. **CVE-2026-0861** (HIGH) - glibc integer overflow
   - Impact: Core system library (`libc-bin`, `libc6`)
   - Risk: Fundamental to all operations
   - Mitigation: No fix available yet in Debian 12
   - Status: Monitor for Debian security updates

4. **CVE-2026-24882** (HIGH) - GnuPG buffer overflow
   - Impact: Affects `gpgv` package
   - Risk: Not directly used by backup operations
   - Mitigation: No fix available yet in Debian 12

5. **CVE-2023-45853** (CRITICAL) - zlib buffer overflow
   - Impact: Affects `zlib1g` compression library
   - Risk: Used for gzip compression of backups
   - Mitigation: Marked as "will_not_fix" by Debian
   - Status: Known issue, considered low practical risk

**Recommended Actions**:

1. **Monitor**: Watch Debian security advisories for patches
2. **Update regularly**: Rebuild with `--no-cache` when base image updates
3. **Review**: Re-scan monthly or when new Debian releases appear
4. **Consider alternatives**: If risk tolerance is low, consider:
   - Alpine-based image (different vulnerability profile)
   - Wait for Debian security updates before deployment
   - Use network isolation to limit attack surface

**Operational Context**:

The backup container:

- Runs with read-only access to data being backed up
- Executes in isolated Docker network
- Runs non-interactively (batch mode)
- Has limited network exposure (only MySQL connection if needed)
- Exits immediately after backup completion (not long-running)

#### Security Features

| Feature            | Implementation                      | Benefit                      |
| ------------------ | ----------------------------------- | ---------------------------- |
| Minimal base image | `debian:bookworm-slim`              | Reduced attack surface       |
| Non-root execution | User `torrust` (UID 1000)           | Limited privilege escalation |
| Read-only configs  | Mounted as `:ro`                    | Prevents tampering           |
| Explicit packages  | Only required tools installed       | Minimizes vulnerabilities    |
| Unit-tested code   | 44 BATS tests during Docker build   | Catches errors early         |
| Multi-stage build  | Test stage separate from production | Production image is clean    |

## Monitoring

The `tracker-backup` image is included in the automated security scanning workflow (`.github/workflows/docker-security-scan.yml`). Scans run:

- On every push to main/develop branches
- Weekly on schedule
- Results uploaded to GitHub Security tab

## Update Policy

**When to update**:

- When Debian releases security patches for bookworm-slim
- When MySQL client or SQLite have security advisories
- On quarterly review cycle (minimum)

**Update process**:

1. Rebuild container with latest base image: `docker build --no-cache`
2. Run security scan: `trivy image --severity HIGH,CRITICAL torrust/tracker-backup:local`
3. Verify 44 unit tests pass during build
4. Update this document with scan results
5. Push to Docker Hub via GitHub Actions workflow

## References

- [Backup Container Workflow](../../../../.github/workflows/backup-container.yaml)
- [Security Scan Workflow](../../../../.github/workflows/docker-security-scan.yml)
- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
