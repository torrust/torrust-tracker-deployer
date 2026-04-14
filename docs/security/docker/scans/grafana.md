# Grafana - Security Scans

Security scan history for the `grafana/grafana` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status                                | Last Scan    | Support EOL |
| ------- | ---- | -------- | ------------------------------------- | ------------ | ----------- |
| 13.0.0  | 10   | 0        | ⚠️ Accepted risk (no remote exposure) | Apr 14, 2026 | Unknown     |
| 12.4.2  | 13   | 0        | ✅ Replaced by 13.0.0                 | Apr 14, 2026 | Unknown     |

## Scan History

### April 14, 2026 - CVE-2026-34986 remediation (Issue #434)

**Image**: `grafana/grafana:13.0.0`
**Trivy Version**: 0.68.2
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Status**: ⚠️ **10 HIGH, 0 CRITICAL** — CVE-2026-34986 (remote DoS) eliminated

#### Summary

Full re-scan revealed 13 HIGH in `grafana/grafana:12.4.2` including CVE-2026-34986,
an unauthenticated remote DoS via a crafted JWE bearer token (CVSS 7.5,
AV:N/AC:L/PR:N/UI:N). The fix (bumping `go-jose/v4` to `4.1.4`) was merged in
[grafana/grafana#121830](https://github.com/grafana/grafana/pull/121830) with label
`no-backport` — no 12.x patch will be issued. Upgraded to `13.0.0`.

Vulnerability comparison:

- `12.4.2`: 13 HIGH, 0 CRITICAL (CVE-2026-34986 present)
- `13.0.0`: 10 HIGH, 0 CRITICAL (CVE-2026-34986 **absent**)

Improvement: -3 HIGH; remote DoS eliminated.

Detail by target in 13.0.0:

- Alpine 3.23.3 base: 3 HIGH (openssl + zlib — same as 12.4.2, blocked on Alpine rebuild)
- grafana binary: 2 HIGH (moby/moby CVE-2026-34040, otel/sdk CVE-2026-39883)
- grafana-cli binary: 0 HIGH ✅
- grafana-server binary: 0 HIGH ✅
- elasticsearch plugin (new bundled binary): 5 HIGH (otel + stdlib, all local-only)

### April 14, 2026 - Full scan (Issue #434)

**Image**: `grafana/grafana:12.4.2`
**Trivy Version**: 0.68.2 (updated DB)
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Status**: ⚠️ **13 HIGH, 0 CRITICAL** — includes remote-exploitable CVE-2026-34986

#### Summary

Re-scan with updated Trivy DB (April 14) revealed 13 HIGH in `12.4.2`, significantly
more than the 4 HIGH found in the April 8 scan due to new CVE entries added to the
vulnerability database. CVE-2026-34986 (`go-jose/v4`, CVSS 7.5) is the only
finding with a remote attack path.

Breakdown:

- Alpine 3.23.3 base: 3 HIGH (openssl + zlib)
- grafana binary: 6 HIGH (go-jose, moby, otel × 2, stdlib × 2)
- grafana-cli binary: 2 HIGH (moby + otel)
- grafana-server binary: 2 HIGH (moby + otel)

### April 8, 2026 — Remediation Pass 1 (Issue #428)

**Image**: `grafana/grafana:12.4.2`
**Trivy Version**: 0.68.2
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Status**: ⚠️ **4 vulnerabilities** (4 HIGH, 0 CRITICAL)

#### Summary

Easy remediation applied by upgrading Grafana from `12.3.1` to `12.4.2`.

Vulnerability comparison:

- Previous (`12.3.1`): 18 HIGH, 6 CRITICAL
- Current (`12.4.2`): 4 HIGH, 0 CRITICAL

Improvement: -14 HIGH, -6 CRITICAL. All CRITICAL findings cleared.

### April 8, 2026 — Prior scan pre-upgrade (12.3.1)

**Image**: `grafana/grafana:12.3.1`
**Trivy Version**: 0.68.2
**Status**: ⚠️ **24 vulnerabilities** (24 HIGH, 0 CRITICAL) — significant increase from Dec scan

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
