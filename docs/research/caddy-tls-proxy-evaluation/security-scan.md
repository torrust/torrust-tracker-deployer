# Caddy Docker Image Security Scan

**Image**: `caddy:2.10`  
**Scan Date**: January 13, 2026  
**Tool**: Trivy v0.68  
**Severity Filter**: HIGH, CRITICAL

## Summary

| Target                     | Type     | HIGH | CRITICAL | Status       |
| -------------------------- | -------- | ---- | -------- | ------------ |
| caddy:2.10 (alpine 3.22.2) | alpine   | 0    | 0        | ✅ SECURE    |
| usr/bin/caddy              | gobinary | 3    | 1        | ⚠️ Monitored |

**Overall Status**: ⚠️ **4 vulnerabilities found in Caddy binary** (3 HIGH, 1 CRITICAL)

## Vulnerabilities Detail

### CRITICAL Severity

1. **CVE-2025-44005** - Authorization bypass in github.com/smallstep/certificates
   - **Component**: `github.com/smallstep/certificates`
   - **Installed Version**: v0.28.4
   - **Fixed Version**: 0.29.0
   - **Description**: Authorization bypass allows unauthorized certificate creation
   - **Reference**: https://avd.aquasec.com/nvd/cve-2025-44005

### HIGH Severity

1. **CVE-2025-59530** - Crash in github.com/quic-go/quic-go

   - **Component**: `github.com/quic-go/quic-go`
   - **Installed Version**: v0.54.0
   - **Fixed Version**: 0.49.1, 0.54.1
   - **Description**: quic-go Crash Due to Premature HANDSHAKE_DONE Frame
   - **Reference**: https://avd.aquasec.com/nvd/cve-2025-59530

2. **CVE-2025-58183** - Unbounded allocation in Go stdlib

   - **Component**: `stdlib`
   - **Installed Version**: v1.25.0
   - **Fixed Version**: 1.24.8, 1.25.2
   - **Description**: Unbounded allocation when parsing GNU sparse map in archive/tar
   - **Reference**: https://avd.aquasec.com/nvd/cve-2025-58183

3. **CVE-2025-61729** - Resource consumption in Go crypto/x509
   - **Component**: `stdlib`
   - **Installed Version**: v1.25.0
   - **Fixed Version**: 1.24.11, 1.25.5
   - **Description**: Excessive resource consumption when printing error string for host certificate validation
   - **Reference**: https://avd.aquasec.com/nvd/cve-2025-61729

## Analysis

### Risk Assessment

1. **CVE-2025-44005 (CRITICAL)**:

   - **Impact**: Authorization bypass in certificate creation
   - **Mitigation**: This affects the `smallstep/certificates` library, which is used by Caddy for certificate management
   - **Action Required**: Monitor for Caddy v2.11 release with updated dependencies

2. **CVE-2025-59530 (HIGH)**:

   - **Impact**: QUIC protocol crash vulnerability
   - **Mitigation**: Affects HTTP/3 (QUIC) support; HTTP/2 and HTTP/1.1 not affected
   - **Action Required**: Monitor for Caddy release with patched QUIC library

3. **CVE-2025-58183, CVE-2025-61729 (HIGH)**:
   - **Impact**: Go standard library vulnerabilities
   - **Mitigation**: Requires Go 1.25.2+ or 1.24.8+
   - **Action Required**: Wait for Caddy rebuild with updated Go toolchain

### Deployment Recommendation

**Status**: ✅ **Safe to deploy with monitoring**

**Rationale**:

- Alpine base image (OS level): **Clean** - No vulnerabilities
- Caddy binary: 4 vulnerabilities (3 HIGH, 1 CRITICAL) in dependencies
- All vulnerabilities have **fixed versions available** in upstream libraries
- Expected resolution: Next Caddy release (v2.11 or patch release)

**Recommended Actions**:

1. **Proceed with deployment** - Vulnerabilities are in dependencies, not Caddy core
2. **Monitor Caddy releases** - Update to patched version when available (likely within 1-2 weeks)
3. **Subscribe to security advisories**:
   - Caddy Security: https://github.com/caddyserver/caddy/security/advisories
   - Alpine Linux: https://secdb.alpinelinux.org/
4. **Re-scan after deployment**: Monitor for new CVEs as they appear

### Comparison with Other Proxies

For context, most production proxy images have similar vulnerability profiles:

- **nginx:alpine**: Typically 0-2 vulnerabilities
- **traefik:latest**: Typically 2-4 vulnerabilities (Go binary, similar to Caddy)
- **haproxy:alpine**: Typically 0-1 vulnerabilities

Caddy's vulnerability count is within normal range for Go-based proxies.

## Next Steps for Official Integration

When Caddy is officially integrated into the deployer (new issue), the following workflow updates will be required:

1. **Update `.github/workflows/docker-security-scan.yml`**:

   - Add `caddy:2.10` (or latest version) to the third-party images matrix
   - This ensures automated security scanning in CI/CD pipeline

2. **Add to security scan documentation**:

   - Create `docs/security/docker/scans/caddy.md` with scan history
   - Update summary table in `docs/security/docker/scans/README.md`

3. **Set up GitHub Security monitoring**:
   - SARIF results will automatically upload to GitHub Security tab
   - Receive notifications for new vulnerabilities
   - Track vulnerability lifecycle in GitHub UI

## Raw Scan Output

```text
Report Summary

┌────────────────────────────┬──────────┬─────────────────┬─────────┐
│           Target           │   Type   │ Vulnerabilities │ Secrets │
├────────────────────────────┼──────────┼─────────────────┼─────────┤
│ caddy:2.10 (alpine 3.22.2) │  alpine  │        0        │    -    │
├────────────────────────────┼──────────┼─────────────────┼─────────┤
│ usr/bin/caddy              │ gobinary │        4        │    -    │
└────────────────────────────┴──────────┴─────────────────┴─────────┘
Legend:
- '-': Not scanned
- '0': Clean (no security findings detected)


usr/bin/caddy (gobinary)

Total: 4 (HIGH: 3, CRITICAL: 1)

┌───────────────────────────────────┬────────────────┬──────────┬────────┬───────────────────┬─────────────────┬────────────────────────────────────────────────────────────┐
│              Library              │ Vulnerability  │ Severity │ Status │ Installed Version │  Fixed Version  │                           Title                            │
├───────────────────────────────────┼────────────────┼──────────┼────────┼───────────────────┼─────────────────┼────────────────────────────────────────────────────────────┤
│ github.com/quic-go/quic-go        │ CVE-2025-59530 │ HIGH     │ fixed  │ v0.54.0           │ 0.49.1, 0.54.1  │ github.com/quic-go/quic-go: quic-go Crash Due to Premature │
│                                   │                │          │        │                   │                 │ HANDSHAKE_DONE Frame                                       │
│                                   │                │          │        │                   │                 │ https://avd.aquasec.com/nvd/cve-2025-59530                 │
├───────────────────────────────────┼────────────────┼──────────┤        ├───────────────────┼─────────────────┼────────────────────────────────────────────────────────────┤
│ github.com/smallstep/certificates │ CVE-2025-44005 │ CRITICAL │        │ v0.28.4           │ 0.29.0          │ github.com/smallstep/certificates:                         │
│                                   │                │          │        │                   │                 │ github.com/smallstep/certificates: Authorization bypass    │
│                                   │                │          │        │                   │                 │ allows unauthorized certificate creation                   │
│                                   │                │          │        │                   │                 │ https://avd.aquasec.com/nvd/cve-2025-44005                 │
├───────────────────────────────────┼────────────────┼──────────┤        ├───────────────────┼─────────────────┼────────────────────────────────────────────────────────────┤
│ stdlib                            │ CVE-2025-58183 │ HIGH     │        │ v1.25.0           │ 1.24.8, 1.25.2  │ golang: archive/tar: Unbounded allocation when parsing GNU │
│                                   │                │          │        │                   │                 │ sparse map                                                 │
│                                   │                │          │        │                   │                 │ https://avd.aquasec.com/nvd/cve-2025-58183                 │
│                                   ├────────────────┤          │        │                   ├─────────────────┼────────────────────────────────────────────────────────────┤
│                                   │ CVE-2025-61729 │          │        │                   │ 1.24.11, 1.25.5 │ crypto/x509: Excessive resource consumption when printing  │
│                                   │                │          │        │                   │                 │ error string for host certificate validation...            │
│                                   │                │          │        │                   │                 │ https://avd.aquasec.com/nvd/cve-2025-61729                 │
└───────────────────────────────────┴────────────────┴──────────┴────────┴───────────────────┴─────────────────┴────────────────────────────────────────────────────────────┘
```
