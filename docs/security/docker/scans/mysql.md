# MySQL - Security Scans

Security scan history for the `mysql` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status    | Last Scan    | Support EOL  |
| ------- | ---- | -------- | --------- | ------------ | ------------ |
| 8.4     | 0    | 0        | ✅ SECURE | Dec 29, 2025 | Apr 30, 2032 |

## Scan History

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
