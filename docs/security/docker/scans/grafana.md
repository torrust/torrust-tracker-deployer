# Grafana - Security Scans

Security scan history for the `grafana/grafana` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status                 | Last Scan   | Support EOL  |
| ------- | ---- | -------- | ---------------------- | ----------- | ------------ |
| 12.3.1  | 24   | 0        | ⚠️ CVE database update | Apr 8, 2026 | Feb 24, 2026 |

## Scan History

### April 8, 2026

**Image**: `grafana/grafana:12.3.1`
**Trivy Version**: 0.68.2
**Status**: ⚠️ **24 vulnerabilities** (24 HIGH, 0 CRITICAL) - Significant increase from Dec scan

#### Summary

Vulnerability count increased dramatically from 0 to 24 HIGH. Breakdown by target:

- Alpine 3.23.0 base: 7 HIGH
- grafana binary: 9 HIGH
- grafana-cli binary: 4 HIGH
- grafana-server binary: 4 HIGH

This sharp increase strongly suggests the Trivy vulnerability database was updated rather than Grafana becoming intrinsically more vulnerable.

#### Changes Since December

- December scan: 0 vulnerabilities
- April scan: 24 HIGH total
- Alpine warnings likely cosmetic (see Dec notes)

**Recommended Action**: Verify findings against official Grafana security advisories: https://github.com/grafana/grafana/security/advisories

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
