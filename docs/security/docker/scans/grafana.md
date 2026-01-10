# Grafana - Security Scans

Security scan history for the `grafana/grafana` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status    | Last Scan    | Support EOL  |
| ------- | ---- | -------- | --------- | ------------ | ------------ |
| 12.3.1  | 0    | 0        | ✅ SECURE | Dec 29, 2025 | Feb 24, 2026 |

## Scan History

### December 29, 2025

**Image**: `grafana/grafana:12.3.1`
**Trivy Version**: 0.68.2
**Status**: ✅ SECURE - 0 HIGH/CRITICAL vulnerabilities

#### Results

```text
grafana/grafana:12.3.1 (alpine 3.23.0)
======================================
Total: 0 (HIGH: 0, CRITICAL: 0)

Scanned 17 targets (alpine, node-pkg, gobinary)
All targets clean - no HIGH or CRITICAL vulnerabilities detected
```

#### Notes

- Alpine 3.23.0 warnings are cosmetic - Grafana image is recent and actively maintained
- Zero HIGH/CRITICAL vulnerabilities detected across all 17 targets
- Grafana team maintains official images with security patches

#### Support Status

- Release: November 19, 2025
- Latest Major: 12.x series
- EOL: February 24, 2026 (2 months remaining as of Dec 2025)
- Note: Grafana follows bi-monthly release cycle

---

### Previous Scans

#### December 23, 2025 (Pre-Update Baseline)

**Image**: `grafana/grafana:11.4.0`
**Status**: Preliminary scan - 0 HIGH/CRITICAL (informal assessment)

**Note**: December 23 scan was a preliminary assessment before formal documentation was established.
