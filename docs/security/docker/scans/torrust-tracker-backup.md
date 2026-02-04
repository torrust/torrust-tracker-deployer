# Tracker Backup Container - Security Scans

Security scan history for the `torrust/tracker-backup` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status                        | Last Scan   |
| ------- | ---- | -------- | ----------------------------- | ----------- |
| local   | 9    | 2        | ⚠️ Vulnerabilities in base OS | Feb 2, 2026 |

## Scan History

### February 2, 2026

**Image**: `torrust/tracker-backup:local`
**Trivy Version**: 0.68.2
**Base OS**: Debian 13.3 (trixie-slim)
**Status**: ⚠️ **11 vulnerabilities found** (9 HIGH, 2 CRITICAL)

#### Summary

The tracker-backup container is based on `debian:trixie-slim` (Debian 13, current stable). After upgrading from Debian 12 (bookworm) to Debian 13 (trixie), vulnerabilities remain in system libraries. **However, the OpenSSL vulnerabilities have fixes available in Debian 13.**

**Installed Tools**:

- `bash` - Backup script execution
- `default-mysql-client` (MySQL 8) - Database dumps
- `sqlite3` - SQLite backups
- `gzip` - Compression
- `tar` - Archive creation
- `bats` - Unit testing (only in test stage, not in production image)

#### Detailed Results

```text
torrust/tracker-backup:local (debian 13.3)

Total: 11 (HIGH: 9, CRITICAL: 2)

┌─────────────────────────┬────────────────┬──────────┬──────────┬───────────────────┬─────────────────┬──────────────────────────────────────────────────────────────┐
│         Library         │ Vulnerability  │ Severity │  Status  │ Installed Version │  Fixed Version  │                            Title                             │
├─────────────────────────┼────────────────┼──────────┼──────────┼───────────────────┼─────────────────┼──────────────────────────────────────────────────────────────┤
│ libc-bin                │ CVE-2026-0861  │ HIGH     │ affected │ 2.41-12+deb13u1   │                 │ glibc: Integer overflow in memalign leads to heap corruption │
├─────────────────────────┤                │          │          │                   ├─────────────────┤                                                              │
│ libc6                   │                │          │          │                   │                 │                                                              │
├─────────────────────────┼────────────────┤          │          ├───────────────────┼─────────────────┼──────────────────────────────────────────────────────────────┤
│ libmariadb3             │ CVE-2025-13699 │          │          │ 1:11.8.3-0+deb13u1│                 │ mariadb: mariadb-dump utility vulnerable to remote           │
│                         │                │          │          │                   │                 │ code execution via improper path                             │
├─────────────────────────┼────────────────┼──────────┼──────────┼───────────────────┼─────────────────┼──────────────────────────────────────────────────────────────┤
│ libssl3t64              │ CVE-2025-15467 │ CRITICAL │ fixed    │ 3.5.4-1~deb13u1   │ 3.5.4-1~deb13u2 │ openssl: Remote code execution or Denial of Service          │
│                         │                │          │          │                   │                 │ via oversized Initialization                                 │
│                         ├────────────────┼──────────┤          │                   │                 ├──────────────────────────────────────────────────────────────┤
│                         │ CVE-2025-69419 │ HIGH     │          │                   │                 │ openssl: Arbitrary code execution due to                     │
│                         │                │          │          │                   │                 │ out-of-bounds write in PKCS#12 processing                    │
├─────────────────────────┼────────────────┤          ├──────────┼───────────────────┼─────────────────┼──────────────────────────────────────────────────────────────┤
│ mariadb-client          │ CVE-2025-13699 │          │ affected │ 1:11.8.3-0+deb13u1│                 │ mariadb: mariadb-dump utility vulnerable to remote           │
│                         │                │          │          │                   │                 │ code execution via improper path                             │
├─────────────────────────┤                │          │          │                   ├─────────────────┤                                                              │
│ mariadb-client-compat   │                │          │          │                   │                 │                                                              │
├─────────────────────────┤                │          │          │                   ├─────────────────┤                                                              │
│ mariadb-client-core     │                │          │          │                   │                 │                                                              │
├─────────────────────────┤                │          │          │                   ├─────────────────┤                                                              │
│ mariadb-common          │                │          │          │                   │                 │                                                              │
├─────────────────────────┼────────────────┼──────────┼──────────┼───────────────────┼─────────────────┼──────────────────────────────────────────────────────────────┤
│ openssl-provider-legacy │ CVE-2025-15467 │ CRITICAL │ fixed    │ 3.5.4-1~deb13u1   │ 3.5.4-1~deb13u2 │ openssl: Remote code execution or Denial of Service          │
│                         │                │          │          │                   │                 │ via oversized Initialization                                 │
│                         ├────────────────┼──────────┤          │                   │                 ├──────────────────────────────────────────────────────────────┤
│                         │ CVE-2025-69419 │ HIGH     │          │                   │                 │ openssl: Arbitrary code execution due to                     │
│                         │                │          │          │                   │                 │ out-of-bounds write in PKCS#12 processing                    │
└─────────────────────────┴────────────────┴──────────┴──────────┴───────────────────┴─────────────────┴──────────────────────────────────────────────────────────────┘
```

#### Risk Assessment

**Current Risk Level**: ⚠️ **MEDIUM**

**Security Improvement with Debian 13**: The upgrade from Debian 12 (bookworm) to Debian 13 (trixie) resolved 3 critical vulnerabilities that had no fixes available in the previous version:

- ✅ CVE-2025-7458 (CRITICAL) - SQLite integer overflow - **RESOLVED**
- ✅ CVE-2023-45853 (CRITICAL) - zlib buffer overflow - **RESOLVED**
- ✅ CVE-2026-24882 (HIGH) - GnuPG buffer overflow - **RESOLVED**

All remaining vulnerabilities are in upstream Debian packages. The container itself:

- ✅ Minimal package footprint (reduces attack surface)
- ✅ Non-root user execution (UID 1000)
- ✅ Read-only configuration mounts
- ✅ Comprehensive unit test coverage (44 tests)
- ✅ Using current Debian stable (trixie - released Aug 9, 2025)
- ⚠️ Contains fixable OpenSSL vulnerabilities (patches available)
- ⚠️ Contains unfixable MariaDB/glibc vulnerabilities (monitoring required)

**Vulnerability Analysis**:

1. **CVE-2025-15467** (CRITICAL) - OpenSSL RCE/DoS
   - Impact: Affects `libssl3t64` and `openssl-provider-legacy`
   - Risk: Potential remote code execution via oversized initialization
   - Mitigation: **FIX AVAILABLE** - Upgrade to 3.5.4-1~deb13u2
   - Status: Can be resolved with `apt-get update && apt-get upgrade -y`

2. **CVE-2025-69419** (HIGH) - OpenSSL arbitrary code execution
   - Impact: Affects `libssl3t64` and `openssl-provider-legacy`
   - Risk: Out-of-bounds write in PKCS#12 processing
   - Mitigation: **FIX AVAILABLE** - Upgrade to 3.5.4-1~deb13u2
   - Status: Can be resolved with `apt-get update && apt-get upgrade -y`

3. **CVE-2025-13699** (HIGH) - MariaDB dump RCE
   - Impact: Affects `mariadb-client` and related packages (5 total)
   - Risk: Used for MySQL database backups via `mysqldump`
   - Mitigation: No fix available yet in Debian 13
   - Status: Monitor for Debian security updates

4. **CVE-2026-0861** (HIGH) - glibc integer overflow
   - Impact: Core system library (`libc-bin`, `libc6`)
   - Risk: Fundamental to all operations (memalign function)
   - Mitigation: No fix available yet in Debian 13
   - Status: Monitor for Debian security updates

**Recommended Actions**:

1. **Immediate**: Add `RUN apt-get update && apt-get upgrade -y` to Dockerfile to fix OpenSSL vulnerabilities (reduces critical count to 0)
2. **Monitor**: Watch Debian security advisories for MariaDB and glibc patches
3. **Update regularly**: Rebuild with `--no-cache` when base image updates
4. **Review**: Re-scan monthly or when new Debian releases appear

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
