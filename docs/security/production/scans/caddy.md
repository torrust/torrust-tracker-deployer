# Caddy Security Scan History

**Image**: `caddy:2.11.2`
**Purpose**: TLS termination proxy for HTTPS support
**Documentation**: [Caddy TLS Proxy Evaluation](../../research/caddy-tls-proxy-evaluation/README.md)

## Current Status

| Version | HIGH | CRITICAL | Status                               | Scan Date    |
| ------- | ---- | -------- | ------------------------------------ | ------------ |
| 2.11.2  | 10   | 2        | ⚠️ Partial improvement after upgrade | Apr 15, 2026 |

**Deployment Status**: ⚠️ Requires follow-up — 2 CRITICAL CVEs remain in upstream Caddy binary dependencies (smallstep/certificates, grpc-go). Fixes require upstream Caddy releases.

## Vulnerability Summary

The Caddy 2.11.2 image has:

- **Alpine base image**: 3 HIGH, 0 CRITICAL (libcrypto3/libssl3, zlib — fixed versions available)
- **Caddy binary (Go)**: 7 HIGH, 2 CRITICAL in dependencies (not Caddy core)

The 2 CRITICAL CVEs are in upstream Caddy binary dependencies and require Caddy to update its vendored modules.

## Scan History

### April 15, 2026 - Remediation Pass 2 (Issue #432)

**Scanner**: Trivy v0.69.3
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Image**: `caddy:2.11.2`
**Status**: ⚠️ **12 vulnerabilities** (10 HIGH, 2 CRITICAL)

#### Summary

Upgraded Caddy from `2.10.2` to `2.11.2` (latest as of 2026-04-14). Meaningful reduction in findings but 2 CRITICAL CVEs remain in upstream binary dependencies.

Vulnerability comparison:

| Version  | HIGH | CRITICAL |
| -------- | ---- | -------- |
| `2.10`   | 18   | 6        |
| `2.10.2` | 14   | 4        |
| `2.11.2` | 10   | 2        |

Issue left open — CRITICALs not fully cleared.

#### Target Breakdown (`2.11.2`)

| Target         | Type     | HIGH | CRITICAL |
| -------------- | -------- | ---- | -------- |
| caddy (alpine) | alpine   | 3    | 0        |
| usr/bin/caddy  | gobinary | 7    | 2        |

#### CVE Details

**Alpine OS layer:**

| CVE            | Library             | Severity | Fixed In | Notes                      |
| -------------- | ------------------- | -------- | -------- | -------------------------- |
| CVE-2026-28390 | libcrypto3, libssl3 | HIGH     | 3.5.6-r0 | OpenSSL DoS via NULL deref |
| CVE-2026-22184 | zlib                | HIGH     | 1.3.2-r0 | Buffer overflow in untgz   |

**Caddy binary (Go):**

| CVE            | Library                | Severity | Fixed In      | Notes                                       |
| -------------- | ---------------------- | -------- | ------------- | ------------------------------------------- |
| CVE-2026-34986 | go-jose/go-jose v3+v4  | HIGH     | 3.0.5 / 4.1.4 | DoS via crafted JWE                         |
| CVE-2026-30836 | smallstep/certificates | CRITICAL | 0.30.0        | Unauthenticated SCEP cert issuance          |
| CVE-2026-39883 | otel/sdk               | HIGH     | 1.43.0        | Local PATH hijack (no remote path)          |
| CVE-2026-33186 | google.golang.org/grpc | CRITICAL | 1.79.3        | Authorization bypass via HTTP/2 path        |
| CVE-2026-25679 | stdlib                 | HIGH     | 1.26.1        | Incorrect IPv6 parsing in net/url           |
| CVE-2026-27137 | stdlib                 | HIGH     | 1.26.1        | Email constraint enforcement in crypto/x509 |
| CVE-2026-32280 | stdlib                 | HIGH     | 1.26.2        | Excessive work during chain building        |
| CVE-2026-32282 | stdlib                 | HIGH     | 1.26.2        | Root.Chmod follows symlinks out of root     |

**Overall risk**: The 2 CRITICAL CVEs (CVE-2026-30836, CVE-2026-33186) are in upstream
Caddy binary dependencies and require a new Caddy release to fix. CVE-2026-33186
(gRPC authorization bypass) has a network-accessible attack path. Revisit when
Caddy ships the updated grpc-go and smallstep dependencies.

---

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
trivy image --severity HIGH,CRITICAL caddy:2.11.2
```

## Security Advisories

- **Caddy**: <https://github.com/caddyserver/caddy/security/advisories>
- **Alpine Linux**: <https://secdb.alpinelinux.org/>
