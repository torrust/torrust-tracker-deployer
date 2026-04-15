# MySQL - Security Scans

Security scan history for the `mysql` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status                   | Last Scan    | Support EOL  |
| ------- | ---- | -------- | ------------------------ | ------------ | ------------ |
| 8.4     | 9    | 1        | ⚠️ Accepted risk (gosu)  | Apr 15, 2026 | Apr 30, 2032 |

## Scan History

### April 15, 2026 - Remediation Pass 2 / Accepted Risk (Issue #435)

**Image**: `mysql:8.4` (resolves to `8.4.8`)
**Trivy Version**: 0.69.3
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Status**: ⚠️ **10 vulnerabilities** (9 HIGH, 1 CRITICAL)

#### Summary

Floating tag still resolves to `8.4.8` (unchanged from Apr 8 baseline). Vulnerability count
increased from 7 HIGH + 1 CRITICAL to 9 HIGH + 1 CRITICAL due to Trivy DB updates only;
no new MySQL release shipped.

A comparison scan of `mysql:9.6` (latest Innovation Release, shipped 2026-04-14) shows an
**identical CVE profile** — same `gosu v1.24.6` Go binary and same Python packages:

| Version   | HIGH | CRITICAL | Notes                              |
| --------- | ---- | -------- | ---------------------------------- |
| `8.4.8`   | 9    | 1        | LTS, support EOL Apr 2032          |
| `9.6`     | 9    | 1        | Innovation Release, shorter lifecycle |

All CVEs are in helper components only:

| Target                           | HIGH | CRITICAL |
| -------------------------------- | ---- | -------- |
| `mysql:8.4` (oracle 9.7)         | 0    | 0        |
| Python packages (mysqlsh)        | 2    | 0        |
| `usr/local/bin/gosu`             | 7    | 1        |
| **Total**                        | **9** | **1**   |

**CVE details — Python packages (`cryptography 45.0.7`, `pyOpenSSL 25.1.0`):**

| CVE            | Library      | Severity | Status | Fixed Version | Title                                         |
| -------------- | ------------ | -------- | ------ | ------------- | --------------------------------------------- |
| CVE-2026-26007 | cryptography | HIGH     | fixed  | 46.0.5        | Subgroup attack due to missing SECT validation |
| CVE-2026-27459 | pyOpenSSL    | HIGH     | fixed  | 26.0.0        | DTLS cookie callback buffer overflow           |

**CVE details — `gosu` (`stdlib v1.24.6`):**

| CVE            | Severity | Status | Fixed Version        | Title                                                        |
| -------------- | -------- | ------ | -------------------- | ------------------------------------------------------------ |
| CVE-2025-68121 | CRITICAL | fixed  | 1.24.13, 1.25.7      | crypto/tls: Incorrect certificate validation (TLS resumption) |
| CVE-2025-58183 | HIGH     | fixed  | 1.24.8, 1.25.2       | archive/tar: Unbounded allocation in GNU sparse map          |
| CVE-2025-61726 | HIGH     | fixed  | 1.24.12, 1.25.6      | net/url: Memory exhaustion in query parameter parsing        |
| CVE-2025-61728 | HIGH     | fixed  | 1.24.12, 1.25.6      | archive/zip: Excessive CPU - building archive index          |
| CVE-2025-61729 | HIGH     | fixed  | 1.24.11, 1.25.5      | crypto/x509: DoS via excessive resource consumption          |
| CVE-2026-25679 | HIGH     | fixed  | 1.25.8, 1.26.1       | net/url: Incorrect parsing of IPv6 host literals             |
| CVE-2026-32280 | HIGH     | fixed  | 1.25.9, 1.26.2       | chain building: unbounded work amount                        |
| CVE-2026-32282 | HIGH     | fixed  | 1.25.9, 1.26.2       | internal/syscall/unix: Root.Chmod can follow symlinks        |

#### Decision

**Accepted risk — close issue #435.**

- No viable upgrade path: `mysql:9.6` (latest) has an identical CVE profile
- All CVEs are in `gosu` (process privilege helper) and MySQL Shell Python packages —
  **not MySQL Server itself**
- The CRITICAL (CVE-2025-68121, crypto/tls cert validation) is in `gosu`, not in any
  MySQL network-facing code path
- `mysql:8.4` remains the correct choice: LTS with support until Apr 30, 2032
- Fix requires MySQL upstream to release a new image with `gosu` rebuilt on Go ≥ 1.24.13

**Revisit**: When MySQL upstream ships `8.4.9` or later with updated `gosu`.

---

### April 8, 2026 - Remediation Pass 1 (Issue #428)

**Image**: `mysql:8.4`
**Trivy Version**: 0.68.2
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Status**: ⚠️ **8 vulnerabilities** (7 HIGH, 1 CRITICAL)

#### Summary

Findings are concentrated in helper components, not MySQL server core:

- Python packages: 2 HIGH
- `gosu` Go stdlib dependencies: 5 HIGH, 1 CRITICAL

Tag comparison for easy remediation was performed (`8.4.1`, `8.4.2`, `8.4.3`, `9.0`, `9.1`, `latest`).
No safer drop-in tag with lower overall risk profile was identified for immediate adoption in this pass.

#### Decision

- Keep `mysql:8.4` for now (validated runtime and LTS alignment)
- Track unresolved CVEs in follow-up issue for deeper investigation

### April 8, 2026

**Image**: `mysql:8.4`
**Trivy Version**: 0.68.2
**Status**: ⚠️ **8 vulnerabilities** (8 HIGH, 0 CRITICAL) - Increase from Dec scan

#### Summary

Vulnerability count increased from 0 to 8 HIGH. Breakdown:

- Python libraries: 2 HIGH
- `/usr/local/bin/gosu`: 6 HIGH

This increase suggests Trivy database update rather than actual MySQL regression.

#### Changes Since December

- December scan: 0 vulnerabilities
- April scan: 8 HIGH
- MySQL server binary itself appears unaffected

**Recommended Action**: Most concerns are in helper binaries (gosu) and Python tools, not MySQL core. Verify with MySQL security advisories: https://www.mysql.com/support/security/

### December 29, 2025

**Image**: `mysql:8.4`
**Trivy Version**: 0.68.2
**Status**: ✅ SECURE - 0 HIGH/CRITICAL vulnerabilities

#### Results

```text
mysql:8.4 (oracle 9.7)
======================
Total: 0 (HIGH: 0, CRITICAL: 0)

MySQL server core: 0 vulnerabilities
```

#### Notes

- MySQL 8.4 is an LTS release with extended support
- Oracle Linux 9.7 base has no HIGH/CRITICAL vulnerabilities
- MySQL server itself has 0 vulnerabilities
- LTS release designed for production stability

#### Support Status

- Release: April 10, 2024
- Premier Support: Until April 30, 2029 (3+ years remaining)
- Extended Support: Until April 30, 2032 (6+ years remaining)
- LTS Release: Designed for production stability

---

### Previous Scans

#### December 23, 2025 (Pre-Update Baseline)

**Image**: `mysql:8.0`
**Status**: Preliminary scan - 0 HIGH/CRITICAL (informal assessment)

**Note**: December 23 scan was a preliminary assessment before formal documentation was established.
