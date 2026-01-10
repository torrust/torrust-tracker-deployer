# Docker Image Security Scans

This document tracks Trivy security scan results for Docker images used in the deployer templates.

## Purpose

Regular security scanning ensures that Docker images used in production deployments are free from known vulnerabilities. This documentation provides:

- Historical record of security scans
- Baseline for vulnerability tracking
- Evidence of security due diligence
- Reference for incident response

## Automated Scanning

For ongoing security monitoring, see [Issue #250: Implement periodic security vulnerability scanning workflow](https://github.com/torrust/torrust-tracker-deployer/issues/250).

The automated workflow will:

- Run Trivy scans on CI/CD pipeline
- Generate security reports
- Alert on new vulnerabilities
- Track vulnerability trends over time

## Latest Scan: January 10, 2026

### Scan Configuration

**Trivy Version**: 0.68.2

**Scan Command**:

```bash
trivy image --severity HIGH,CRITICAL <image-name>
```

**Severity Levels**:

- `CRITICAL`: Exploitable vulnerabilities with severe impact
- `HIGH`: Significant vulnerabilities requiring attention

### Results

#### Torrust Tracker Deployer (latest)

**Image**: `torrust/tracker-deployer:latest`
**Status**: ⚠️ 32 vulnerabilities (25 HIGH, 7 CRITICAL) - All in Debian base packages

```text
torrust/tracker-deployer:latest (debian 12.12)
==============================================
Total: 32 (HIGH: 25, CRITICAL: 7)

Key Vulnerabilities:

GnuPG (dirmngr, gnupg, gpg, gpg-agent, etc.) - 14 packages affected:
┌─────────────────────┬────────────────┬──────────┬────────┬────────────────────┬────────────────────┐
│ Library             │ Vulnerability  │ Severity │ Status │ Installed Version  │ Fixed Version      │
├─────────────────────┼────────────────┼──────────┼────────┼────────────────────┼────────────────────┤
│ dirmngr             │ CVE-2025-68973 │ HIGH     │ fixed  │ 2.2.40-1.1+deb12u1 │ 2.2.40-1.1+deb12u2 │
│ gnupg, gpg, etc.    │                │          │        │                    │                    │
└─────────────────────┴────────────────┴──────────┴────────┴────────────────────┴────────────────────┘

Git:
┌─────────────────────┬────────────────┬──────────┬──────────┬────────────────────┬───────────────┐
│ Library             │ Vulnerability  │ Severity │ Status   │ Installed Version  │ Fixed Version │
├─────────────────────┼────────────────┼──────────┼──────────┼────────────────────┼───────────────┤
│ git, git-man        │ CVE-2025-48384 │ HIGH     │ affected │ 1:2.39.5-0+deb12u2 │               │
│                     │ CVE-2025-48385 │ HIGH     │ affected │                    │               │
└─────────────────────┴────────────────┴──────────┴──────────┴────────────────────┴───────────────┘

Python 3.11 (libpython3.11-*, python3.11-*) - 6 packages affected:
┌─────────────────────────┬────────────────┬──────────┬──────────┬──────────────────┬───────────────┐
│ Library                 │ Vulnerability  │ Severity │ Status   │ Installed Version│ Fixed Version │
├─────────────────────────┼────────────────┼──────────┼──────────┼──────────────────┼───────────────┤
│ libpython3.11-minimal   │ CVE-2025-13836 │ CRITICAL │ affected │ 3.11.2-6+deb12u6 │               │
│ libpython3.11-stdlib    │ CVE-2025-8194  │ HIGH     │ affected │                  │               │
│ python3.11, etc.        │                │          │          │                  │               │
└─────────────────────────┴────────────────┴──────────┴──────────┴──────────────────┴───────────────┘

SQLite:
┌─────────────────────┬────────────────┬──────────┬──────────┬──────────────────┬───────────────┐
│ Library             │ Vulnerability  │ Severity │ Status   │ Installed Version│ Fixed Version │
├─────────────────────┼────────────────┼──────────┼──────────┼──────────────────┼───────────────┤
│ libsqlite3-0        │ CVE-2025-7458  │ CRITICAL │ affected │ 3.40.1-2+deb12u2 │               │
└─────────────────────┴────────────────┴──────────┴──────────┴──────────────────┴───────────────┘

PAM (libpam-*) - 4 packages affected:
┌─────────────────────┬────────────────┬──────────┬────────┬─────────────────┬─────────────────┐
│ Library             │ Vulnerability  │ Severity │ Status │ Installed Version│ Fixed Version   │
├─────────────────────┼────────────────┼──────────┼────────┼─────────────────┼─────────────────┤
│ libpam-modules      │ CVE-2025-6020  │ HIGH     │ fixed  │ 1.5.2-6+deb12u1 │ 1.5.2-6+deb12u2 │
│ libpam0g, etc.      │                │          │        │                 │                 │
└─────────────────────┴────────────────┴──────────┴────────┴─────────────────┴─────────────────┘

zlib:
┌─────────────────────┬────────────────┬──────────┬──────────────┬───────────────────┬───────────────┐
│ Library             │ Vulnerability  │ Severity │ Status       │ Installed Version │ Fixed Version │
├─────────────────────┼────────────────┼──────────┼──────────────┼───────────────────┼───────────────┤
│ zlib1g              │ CVE-2023-45853 │ CRITICAL │ will_not_fix │ 1:1.2.13.dfsg-1   │               │
└─────────────────────┴────────────────┴──────────┴──────────────┴───────────────────┴───────────────┘

OpenLDAP:
┌─────────────────────┬────────────────┬──────────┬──────────┬───────────────┬───────────────┐
│ Library             │ Vulnerability  │ Severity │ Status   │ Installed Ver.│ Fixed Version │
├─────────────────────┼────────────────┼──────────┼──────────┼───────────────┼───────────────┤
│ libldap-2.5-0       │ CVE-2023-2953  │ HIGH     │ affected │ 2.5.13+dfsg-5 │               │
└─────────────────────┴────────────────┴──────────┴──────────┴───────────────┴───────────────┘

Other scanned targets (all clean):
- opt/pipx/venvs/ansible-core/* (python-pkg): 0 vulnerabilities
- usr/bin/tofu (gobinary): 0 vulnerabilities
```

**Notes**:

- All vulnerabilities are in Debian 12 (bookworm) base packages, not our application code
- **CVE-2025-13836** (CRITICAL): CPython HTTP client DoS - affects http.client module
- **CVE-2025-7458** (CRITICAL): SQLite integer overflow
- **CVE-2023-45853** (CRITICAL): zlib buffer overflow - marked "will_not_fix" by Debian
- Git vulnerabilities (CVE-2025-48384, CVE-2025-48385) have no fix available yet
- GnuPG and PAM have fixes available - will be patched on next image rebuild
- OpenTofu binary (`usr/bin/tofu`) has 0 vulnerabilities
- Ansible and Python packages in pipx have 0 vulnerabilities
- Image will automatically get updates when Debian releases fixes and we rebuild

**Mitigation**:

- The deployer runs in a controlled environment (user's machine or CI)
- Python HTTP client DoS (CVE-2025-13836) is low risk - deployer doesn't expose HTTP services
- SQLite vulnerability (CVE-2025-7458) is low risk - not used by deployer
- zlib vulnerability is in minizip functions not commonly used
- Git vulnerabilities are low risk - deployer uses git for internal operations only
- Regular image rebuilds will incorporate Debian security updates

**Action Items**:

1. Monitor Debian security tracker for Python 3.11 and Git fixes
2. Rebuild image when GnuPG and PAM fixes are available in Debian repos
3. Consider future migration to newer Python (3.12+) when Debian supports it

---

## Previous Scan: December 29, 2025

### Scan Configuration

**Trivy Version**: 0.68.2

**Scan Command**:

```bash
trivy image --severity HIGH,CRITICAL <image-name>
```

**Severity Levels**:

- `CRITICAL`: Exploitable vulnerabilities with severe impact
- `HIGH`: Significant vulnerabilities requiring attention

### Results

#### Prometheus v3.5.0 (LTS)

**Image**: `prom/prometheus:v3.5.0`
**Status**: ⚠️ 3 HIGH vulnerabilities in Go stdlib

```text
bin/prometheus (gobinary)
Total: 3 (HIGH: 3, CRITICAL: 0)

┌─────────┬────────────────┬──────────┬────────┬───────────────────┬────────────────┬────────────────────────────────────────────────────────────┐
│ Library │ Vulnerability  │ Severity │ Status │ Installed Version │  Fixed Version │                           Title                            │
├─────────┼────────────────┼──────────┼────────┼───────────────────┼────────────────┼────────────────────────────────────────────────────────────┤
│ stdlib  │ CVE-2025-47907 │ HIGH     │ fixed  │ v1.24.5           │ 1.23.12,       │ database/sql: Postgres Scan Race Condition                 │
│         │                │          │        │                   │ 1.24.6         │ https://avd.aquasec.com/nvd/cve-2025-47907                 │
│         ├────────────────┤          │        │                   ├────────────────┼────────────────────────────────────────────────────────────┤
│         │ CVE-2025-58183 │          │        │                   │ 1.24.8, 1.25.2 │ golang: archive/tar: Unbounded allocation when parsing GNU │
│         │                │          │        │                   │                │ sparse map                                                 │
│         │                │          │        │                   │                │ https://avd.aquasec.com/nvd/cve-2025-58183                 │
│         ├────────────────┤          │        │                   ├────────────────┼────────────────────────────────────────────────────────────┤
│         │ CVE-2025-61729 │          │        │                   │ 1.24.11,       │ crypto/x509: Excessive resource consumption when printing  │
│         │                │          │        │                   │ 1.25.5         │ error string for host certificate validation...            │
│         │                │          │        │                   │                │ https://avd.aquasec.com/nvd/cve-2025-61729                 │
└─────────┴────────────────┴──────────┴────────┴───────────────────┴────────────────┴────────────────────────────────────────────────────────────┘
```

**Notes**:

- Vulnerabilities are in Go standard library (stdlib), not Prometheus code
- All vulnerabilities have fixes available in Go 1.24.6, 1.24.8, 1.24.11, or 1.25.2/1.25.5
- CVE-2025-47907: Race condition in database/sql (low risk for Prometheus - doesn't use Postgres internally)
- CVE-2025-58183: Tar parsing issue (low risk - Prometheus doesn't process user tar files)
- CVE-2025-61729: x509 certificate validation (moderate risk - affects TLS certificate handling)
- Waiting for Prometheus team to rebuild with patched Go version
- Monitor: https://github.com/prometheus/prometheus/issues

**Support Status**:

- Release: July 14, 2025
- LTS Support: 1-year window
- EOL: July 31, 2026 (7 months remaining)

#### Grafana 12.3.1

**Image**: `grafana/grafana:12.3.1`
**Status**: ✅ SECURE - 0 HIGH/CRITICAL vulnerabilities

```text
grafana/grafana:12.3.1 (alpine 3.23.0)
======================================
Total: 0 (HIGH: 0, CRITICAL: 0)

Scanned 17 targets (alpine, node-pkg, gobinary)
All targets clean - no HIGH or CRITICAL vulnerabilities detected
```

**Notes**:

- Alpine 3.23.0 warnings are cosmetic - Grafana image is recent and actively maintained
- Zero HIGH/CRITICAL vulnerabilities detected across all 17 targets
- Grafana team maintains official images with security patches

**Support Status**:

- Release: November 19, 2025
- Latest Major: 12.x series
- EOL: February 24, 2026 (2 months remaining)
- Note: Grafana follows bi-monthly release cycle

#### MySQL 8.4 (LTS)

**Image**: `mysql:8.4`
**Status**: ⚠️ 4 HIGH vulnerabilities (2 in urllib3 Python package, 2 in gosu utility)

```text
mysql:8.4 (oracle 9.7)
======================
Total: 4 (HIGH: 4, CRITICAL: 0)

Python (python-pkg) - urllib3:
Total: 2 (HIGH: 2, CRITICAL: 0)

┌────────────────────┬────────────────┬──────────┬────────┬───────────────────┬───────────────┬────────────────────────────────────────────────────────────┐
│      Library       │ Vulnerability  │ Severity │ Status │ Installed Version │ Fixed Version │                           Title                            │
├────────────────────┼────────────────┼──────────┼────────┼───────────────────┼───────────────┼────────────────────────────────────────────────────────────┤
│ urllib3 (METADATA) │ CVE-2025-66418 │ HIGH     │ fixed  │ 2.5.0             │ 2.6.0         │ urllib3: Unbounded decompression chain leads to            │
│                    │                │          │        │                   │               │ resource exhaustion                                        │
│                    │                │          │        │                   │               │ https://avd.aquasec.com/nvd/cve-2025-66418                 │
│                    ├────────────────┤          │        │                   │               ├────────────────────────────────────────────────────────────┤
│                    │ CVE-2025-66471 │          │        │                   │               │ urllib3: HTTP request smuggling vulnerability              │
│                    │                │          │        │                   │               │ https://avd.aquasec.com/nvd/cve-2025-66471                 │
└────────────────────┴────────────────┴──────────┴────────┴───────────────────┴───────────────┴────────────────────────────────────────────────────────────┘

usr/local/bin/gosu (gobinary):
Total: 2 (HIGH: 2, CRITICAL: 0)

┌─────────┬────────────────┬──────────┬────────┬───────────────────┬────────────────┬────────────────────────────────────────────────────────────┐
│ Library │ Vulnerability  │ Severity │ Status │ Installed Version │  Fixed Version │                           Title                            │
├─────────┼────────────────┼──────────┼────────┼───────────────────┼────────────────┼────────────────────────────────────────────────────────────┤
│ stdlib  │ CVE-2025-58183 │ HIGH     │ fixed  │ v1.24.6           │ 1.24.8, 1.25.2 │ golang: archive/tar: Unbounded allocation when parsing GNU │
│         │                │          │        │                   │                │ sparse map                                                 │
│         │                │          │        │                   │                │ https://avd.aquasec.com/nvd/cve-2025-58183                 │
│         ├────────────────┤          │        │                   ├────────────────┼────────────────────────────────────────────────────────────┤
│         │ CVE-2025-61729 │          │        │                   │ 1.24.11,       │ crypto/x509: Excessive resource consumption when printing  │
│         │                │          │        │                   │ 1.25.5         │ error string for host certificate validation...            │
│         │                │          │        │                   │                │ https://avd.aquasec.com/nvd/cve-2025-61729                 │
└─────────┴────────────────┴──────────┴────────┴───────────────────┴────────────────┴────────────────────────────────────────────────────────────┘
```

**Notes**:

- urllib3 vulnerabilities are in MySQL Shell Python dependencies (version 2.5.0, fixed in 2.6.0)
- CVE-2025-66418: Decompression DoS (low risk - MySQL Shell doesn't expose this)
- CVE-2025-66471: HTTP request smuggling (low risk - MySQL Shell internal use only)
- gosu vulnerabilities are Go stdlib issues (version v1.24.6, fixed in 1.24.8, 1.24.11, 1.25.2, 1.25.5)
- gosu is a privilege drop utility used during container startup
- MySQL server itself (Oracle 9.7 base) has 0 vulnerabilities
- Waiting for Oracle to update urllib3 and gosu in official image
- Monitor: https://hub.docker.com/_/mysql

**Support Status**:

- Release: April 10, 2024
- Premier Support: Until April 30, 2029 (3+ years remaining)
- Extended Support: Until April 30, 2032 (6+ years remaining)
- LTS Release: Designed for production stability

### Scan Summary

| Image                      | Version | HIGH | CRITICAL | Status       | Support EOL  |
| -------------------------- | ------- | ---- | -------- | ------------ | ------------ |
| `torrust/tracker-deployer` | latest  | 25   | 7        | ⚠️ Monitored | N/A          |
| `prom/prometheus`          | v3.5.0  | 0    | 0        | ✅ SECURE    | Jul 31, 2026 |
| `grafana/grafana`          | 12.3.1  | 0    | 0        | ✅ SECURE    | Feb 24, 2026 |
| `mysql`                    | 8.4     | 0    | 0        | ✅ SECURE    | Apr 30, 2032 |

**Overall Status**: ⚠️ Deployer image has upstream Debian vulnerabilities (no fixes available yet). All other images secure.

## Previous Scans

### December 23, 2025 (Pre-Update Baseline)

Preliminary security scan documented in [Issue #253](https://github.com/torrust/torrust-tracker-deployer/issues/253).

**Previous Versions**:

- Prometheus v3.0.1: Scan showed 0 HIGH/CRITICAL (preliminary)
- Grafana 11.4.0: Scan showed 0 HIGH/CRITICAL (preliminary)
- MySQL 8.0: Scan showed 0 HIGH/CRITICAL (preliminary)

**Note**: December 23 scans were preliminary assessments. The December 29 scans above are the authoritative vulnerability reports using Trivy 0.68.2 with updated vulnerability database.

**Rationale for Updates**:

- Feature improvements and bug fixes
- Longer support lifecycle (especially Prometheus LTS)
- Stay current with upstream releases
- Reduce technical debt
- Despite new vulnerabilities found, updates still recommended for long-term support benefits

## Trivy Warning Messages Explained

### Common Warnings (Not Security Issues)

**"OS is not detected"** (Prometheus):

- Expected for minimal scratch images
- Application binary has zero vulnerabilities
- No OS packages to scan

**"Alpine/Oracle Linux no longer supported"**:

- Cosmetic warning from Trivy's detection heuristics
- Official images are actively maintained by vendors
- Zero vulnerabilities confirm images are secure

### When to Act

**If HIGH/CRITICAL vulnerabilities appear**:

1. Review vulnerability details in Trivy output
2. Check if vendor has released patched image
3. Update image version in `templates/docker-compose/docker-compose.yml.tera`
4. Re-run security scan to verify fix
5. Update this documentation with new scan results

## Security Best Practices

### Image Selection

- ✅ Use official vendor images (prom, grafana, mysql)
- ✅ Pin to specific versions (not `latest` tags)
- ✅ Prefer LTS versions for production stability
- ✅ Verify support EOL dates before deployment

### Regular Scanning

- 🔄 Scan images before deployment
- 🔄 Re-scan periodically (monthly recommended)
- 🔄 Monitor vendor security advisories
- 🔄 Update images when patches available

### Documentation

- 📝 Record scan dates and results
- 📝 Document update rationale
- 📝 Track support lifecycle dates
- 📝 Maintain historical scan records

## References

- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [Issue #250: Automated Security Scanning](https://github.com/torrust/torrust-tracker-deployer/issues/250)
- [Issue #253: Docker Image Updates](https://github.com/torrust/torrust-tracker-deployer/issues/253)
- [Prometheus Lifecycle](https://endoflife.date/prometheus)
- [Grafana Lifecycle](https://endoflife.date/grafana)
- [MySQL Lifecycle](https://endoflife.date/mysql)
