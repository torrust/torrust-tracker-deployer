# Prometheus - Security Scans

Security scan history for the `prom/prometheus` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status    | Last Scan    | Support EOL  |
| ------- | ---- | -------- | --------- | ------------ | ------------ |
| v3.5.0  | 0    | 0        | ✅ SECURE | Dec 29, 2025 | Jul 31, 2026 |

## Scan History

### December 29, 2025

**Image**: `prom/prometheus:v3.5.0`
**Trivy Version**: 0.68.2
**Status**: ✅ SECURE - 0 HIGH/CRITICAL vulnerabilities

#### Results

```text
bin/prometheus (gobinary)
Total: 0 (HIGH: 0, CRITICAL: 0)
```

#### Notes

- Prometheus v3.5.0 LTS release
- Go stdlib vulnerabilities from earlier scans have been patched
- Minimal scratch-based image reduces attack surface
- LTS support until July 31, 2026

#### Support Status

- Release: July 14, 2025
- LTS Support: 1-year window
- EOL: July 31, 2026 (7 months remaining as of Dec 2025)

---

### Previous Scans

#### December 23, 2025 (Pre-Update Baseline)

**Image**: `prom/prometheus:v3.0.1`
**Status**: Preliminary scan - 0 HIGH/CRITICAL (informal assessment)

**Note**: December 23 scan was a preliminary assessment before formal documentation was established.
