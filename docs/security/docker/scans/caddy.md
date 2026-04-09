# Caddy Security Scan History

**Image**: `caddy:2.10.2`
**Purpose**: TLS termination proxy for HTTPS support
**Documentation**: [Caddy TLS Proxy Evaluation](../../research/caddy-tls-proxy-evaluation/README.md)

## Current Status

| Version | HIGH | CRITICAL | Status                               | Scan Date   |
| ------- | ---- | -------- | ------------------------------------ | ----------- |
| 2.10.2  | 14   | 4        | ⚠️ Partial improvement after upgrade | Apr 8, 2026 |

**Deployment Status**: ⚠️ Requires follow-up - upgrading from `2.10` to `2.10.2` reduced findings, but HIGH/CRITICAL issues remain in Caddy binary dependencies

## Vulnerability Summary

The Caddy 2.10 image has:

- **Alpine base image**: Clean (0 vulnerabilities)
- **Caddy binary (Go)**: 4 vulnerabilities in dependencies (not Caddy core)

All vulnerabilities have fixed versions available upstream and are expected to be resolved in the next Caddy release.

## Scan History

### April 8, 2026 - Remediation Pass 1 (Issue #428)

**Scanner**: Trivy v0.68.2
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Image**: `caddy:2.10.2`
**Status**: ⚠️ **18 vulnerabilities** (14 HIGH, 4 CRITICAL)

#### Summary

Easy remediation applied by upgrading Caddy image tag from `2.10` to `2.10.2`.

Vulnerability comparison:

- Previous (`2.10`): 18 HIGH, 6 CRITICAL
- Current (`2.10.2`): 14 HIGH, 4 CRITICAL

Improvement: -4 HIGH, -2 CRITICAL

#### Target Breakdown (`2.10.2`)

| Target        | Type     | HIGH | CRITICAL |
| ------------- | -------- | ---- | -------- |
| usr/bin/caddy | gobinary | 14   | 4        |

Remaining issues are in upstream Caddy binary dependencies and require vendor/upstream updates.

### January 13, 2026 - caddy:2.10

**Scanner**: Trivy v0.68

| Target                     | Type     | HIGH | CRITICAL |
| -------------------------- | -------- | ---- | -------- |
| caddy:2.10 (alpine 3.22.2) | alpine   | 0    | 0        |
| usr/bin/caddy              | gobinary | 3    | 1        |

**Vulnerabilities Found**:

| CVE            | Severity | Component                         | Fixed Version   |
| -------------- | -------- | --------------------------------- | --------------- |
| CVE-2025-44005 | CRITICAL | github.com/smallstep/certificates | 0.29.0          |
| CVE-2025-59530 | HIGH     | github.com/quic-go/quic-go        | 0.49.1, 0.54.1  |
| CVE-2025-58183 | HIGH     | stdlib (archive/tar)              | 1.24.8, 1.25.2  |
| CVE-2025-61729 | HIGH     | stdlib (crypto/x509)              | 1.24.11, 1.25.5 |

**Risk Assessment**:

1. **CVE-2025-44005**: Authorization bypass in certificate creation (smallstep library)
2. **CVE-2025-59530**: QUIC protocol crash (affects HTTP/3 only)
3. **CVE-2025-58183**: Unbounded allocation in tar parsing
4. **CVE-2025-61729**: Resource consumption in x509 certificate validation

**Recommendation**: Deploy with monitoring. Update to patched version when Caddy v2.11 releases.

## Related Documentation

- [Full Security Analysis](../../../research/caddy-tls-proxy-evaluation/security-scan.md)
- [Caddy Evaluation Summary](../../../research/caddy-tls-proxy-evaluation/README.md)
- [HTTPS Implementation](../../../issues/272-add-https-support-with-caddy.md)

## How to Rescan

```bash
trivy image --severity HIGH,CRITICAL caddy:2.10.2
```

## Security Advisories

- **Caddy**: <https://github.com/caddyserver/caddy/security/advisories>
- **Alpine Linux**: <https://secdb.alpinelinux.org/>
