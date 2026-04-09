# Prometheus - Security Scans

Security scan history for the `prom/prometheus` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status                               | Last Scan   | Support EOL  |
| ------- | ---- | -------- | ------------------------------------ | ----------- | ------------ |
| v3.5.1  | 6    | 4        | ⚠️ Partial improvement after upgrade | Apr 8, 2026 | Jul 31, 2026 |

## Scan History

### April 8, 2026 - Remediation Pass 1 (Issue #428)

**Image**: `prom/prometheus:v3.5.1`
**Trivy Version**: 0.68.2
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Status**: ⚠️ **10 vulnerabilities** (6 HIGH, 4 CRITICAL)

#### Summary

Easy remediation applied by upgrading Prometheus from `v3.5.0` to `v3.5.1`.

Vulnerability comparison:

- Previous (`v3.5.0`): 16 HIGH, 4 CRITICAL
- Current (`v3.5.1`): 6 HIGH, 4 CRITICAL

Improvement: -10 HIGH, 0 CRITICAL

#### Target Breakdown (`v3.5.1`)

| Target           | Type     | HIGH | CRITICAL |
| ---------------- | -------- | ---- | -------- |
| `bin/prometheus` | gobinary | 3    | 2        |
| `bin/promtool`   | gobinary | 3    | 2        |

Remaining vulnerabilities are in upstream Prometheus binary dependencies.

### April 8, 2026

**Image**: `prom/prometheus:v3.5.0`
**Trivy Version**: 0.68.2
**Status**: ⚠️ **20 vulnerabilities** (20 HIGH, 0 CRITICAL) - Significant increase from Dec scan

#### Summary

Vulnerability count increased dramatically from 0 to 20 HIGH. This represents a significant change, strongly suggesting the Trivy vulnerability database was updated with new CVE entries rather than Prometheus actually becoming more vulnerable.

#### Changes Since December

- December scan: 0 vulnerabilities
- April scan: 10 HIGH per binary (prometheus, promtool) = 20 total
- Most likely cause: Trivy database updated with newly-discovered Go stdlib CVEs

**Recommended Action**: Verify that Prometheus binary and dependencies haven't actually been compromised. Check official Prometheus security advisories: https://github.com/prometheus/prometheus/security/advisories

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
