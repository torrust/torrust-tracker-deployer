# Torrust Tracker Deployer - Security Scans

Security scan history for the `torrust/tracker-deployer` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status                      | Last Scan   |
| ------- | ---- | -------- | --------------------------- | ----------- |
| trixie  | 1    | 0        | ✅ Improved (Trixie Update) | Feb 5, 2026 |

## Build & Scan Commands

**Build the image**:

```bash
docker build --target release --tag torrust/tracker-deployer:local --file docker/deployer/Dockerfile .
```

**Run Trivy security scan**:

```bash
trivy image --severity HIGH,CRITICAL torrust/tracker-deployer:local
```

## Scan History

### February 5, 2026 - UPDATE TO DEBIAN 13 (TRIXIE)

**Image**: `torrust/tracker-deployer:local`
**Trivy Version**: 0.68.2
**Base OS**: Debian 13.3 (trixie)
**Rust Version**: 1.93.0
**Status**: ✅ **1 vulnerability** (1 HIGH, 0 CRITICAL) - **Significant improvement from bookworm**

#### Summary

The upgrade from `rust:bookworm` to `rust:trixie` (Debian 12 → Debian 13) resulted in a **dramatic security improvement**, reducing vulnerabilities from 32 (bookworm) to just 1 (trixie):

✅ **Resolved 31 vulnerabilities**:

- 24 HIGH vulnerabilities in Debian 12 packages
- 7 CRITICAL vulnerabilities (Python 3.11, SQLite, zlib)

**Current Status by Component**:

- **OpenTofu binary** (`usr/bin/tofu`): 0 vulnerabilities
- **Ansible packages** (pipx venvs): 0 vulnerabilities
- **Deployer binary**: 0 vulnerabilities
- **Debian 13 base packages**: 0 known HIGH/CRITICAL vulnerabilities
- **Test artifacts**: 1 HIGH (private key in Ansible test collections - expected)

#### Detailed Results

The only vulnerability detected is in test/example code within Ansible collections:

| Library             | CVE    | Severity | Type        | File                                                                                                                                         | Status                |
| ------------------- | ------ | -------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------- | --------------------- |
| ansible-collections | (none) | HIGH     | private-key | `/opt/pipx/venvs/ansible-core/lib/python3.11/site-packages/ansible_collections/netapp/storagegrid/plugins/modules/na_sg_grid_certificate.py` | Expected in test code |

This is an example private key embedded in Ansible collection documentation for the NetApp StorageGRID module - not a real security issue.

#### Security Improvement Analysis

**Before (Debian 12 Bookworm - Jan 10, 2026)**:

- 32 total vulnerabilities
- 25 HIGH severity
- 7 CRITICAL severity
- Multiple unpatched CVEs in Python 3.11, SQLite, zlib, Git

**After (Debian 13 Trixie - Feb 5, 2026)**:

- 1 total vulnerability
- 1 HIGH (test artifact, not in deployed code)
- 0 CRITICAL severity
- **97% reduction in vulnerability count**

#### Why Trixie Improved Security

Debian 13 (Trixie, current stable) includes:

- ✅ Updated Python 3.11 with security patches for CVE-2025-13836 and CVE-2025-8194
- ✅ SQLite 3.46.x with fix for CVE-2025-7458
- ✅ zlib with updates for CVE-2023-45853
- ✅ Git 2.43.x with patches for CVE-2025-48384, CVE-2025-48385
- ✅ GnuPG fixes for CVE-2025-68973
- ✅ PAM security updates

#### Risk Assessment

| Item                      | Risk       | Rationale                                                      |
| ------------------------- | ---------- | -------------------------------------------------------------- |
| Private key in test code  | NEGLIGIBLE | Example key in Ansible docs, not deployed, Ansible not exposed |
| OpenTofu binary           | NONE       | No known vulnerabilities                                       |
| Deployer application code | NONE       | 0 vulnerabilities in Rust application                          |
| Ansible runtime           | NONE       | Base packages are current, collection examples not used        |

#### Alignment with Ecosystem

This update aligns with [Torrust Tracker PR #1629](https://github.com/torrust/torrust-tracker/pull/1629), which also updated to Debian 13 (trixie) for consistent security posture across the ecosystem.

#### Action Items

1. ✅ BASE IMAGE UPDATED - No further action needed for base OS vulnerabilities
2. ✅ SECURITY SIGNIFICANTLY IMPROVED - Vulnerability count reduced by 97%
3. Monitor Debian security tracker for any new trixie releases
4. Continue regular image rebuilds to capture future Debian security patches

### January 10, 2026

**Image**: `torrust/tracker-deployer:latest`  
**Trivy Version**: 0.68.2  
**Base OS**: Debian 12.12 (bookworm)  
**Status**: ⚠️ 32 vulnerabilities (25 HIGH, 7 CRITICAL) - Superseded by trixie update

#### Summary

All vulnerabilities are in Debian 12 (bookworm) base packages, not our application code:

- **OpenTofu binary** (`usr/bin/tofu`): 0 vulnerabilities
- **Ansible packages** (pipx venvs): 0 vulnerabilities
- **Deployer binary**: 0 vulnerabilities

#### Detailed Results

**GnuPG (14 packages affected)**:

| Package   | CVE            | Severity | Status | Installed          | Fixed              |
| --------- | -------------- | -------- | ------ | ------------------ | ------------------ |
| dirmngr   | CVE-2025-68973 | HIGH     | fixed  | 2.2.40-1.1+deb12u1 | 2.2.40-1.1+deb12u2 |
| gnupg     | CVE-2025-68973 | HIGH     | fixed  | 2.2.40-1.1+deb12u1 | 2.2.40-1.1+deb12u2 |
| gpg       | CVE-2025-68973 | HIGH     | fixed  | 2.2.40-1.1+deb12u1 | 2.2.40-1.1+deb12u2 |
| gpg-agent | CVE-2025-68973 | HIGH     | fixed  | 2.2.40-1.1+deb12u1 | 2.2.40-1.1+deb12u2 |
| ...       | ...            | ...      | ...    | ...                | ...                |

**Git (2 packages affected)**:

| Package | CVE            | Severity | Status   | Installed          | Fixed |
| ------- | -------------- | -------- | -------- | ------------------ | ----- |
| git     | CVE-2025-48384 | HIGH     | affected | 1:2.39.5-0+deb12u2 | -     |
| git     | CVE-2025-48385 | HIGH     | affected | 1:2.39.5-0+deb12u2 | -     |
| git-man | CVE-2025-48384 | HIGH     | affected | 1:2.39.5-0+deb12u2 | -     |
| git-man | CVE-2025-48385 | HIGH     | affected | 1:2.39.5-0+deb12u2 | -     |

**Python 3.11 (6 packages affected)**:

| Package               | CVE            | Severity | Status   | Installed        | Fixed |
| --------------------- | -------------- | -------- | -------- | ---------------- | ----- |
| libpython3.11-minimal | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| libpython3.11-minimal | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |
| libpython3.11-stdlib  | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| libpython3.11-stdlib  | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |
| python3.11            | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| python3.11            | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |
| python3.11-minimal    | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| python3.11-minimal    | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |
| python3.11-venv       | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| python3.11-venv       | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |

**SQLite**:

| Package      | CVE           | Severity | Status   | Installed        | Fixed |
| ------------ | ------------- | -------- | -------- | ---------------- | ----- |
| libsqlite3-0 | CVE-2025-7458 | CRITICAL | affected | 3.40.1-2+deb12u2 | -     |

**PAM (4 packages affected)**:

| Package            | CVE           | Severity | Status | Installed       | Fixed           |
| ------------------ | ------------- | -------- | ------ | --------------- | --------------- |
| libpam-modules     | CVE-2025-6020 | HIGH     | fixed  | 1.5.2-6+deb12u1 | 1.5.2-6+deb12u2 |
| libpam-modules-bin | CVE-2025-6020 | HIGH     | fixed  | 1.5.2-6+deb12u1 | 1.5.2-6+deb12u2 |
| libpam-runtime     | CVE-2025-6020 | HIGH     | fixed  | 1.5.2-6+deb12u1 | 1.5.2-6+deb12u2 |
| libpam0g           | CVE-2025-6020 | HIGH     | fixed  | 1.5.2-6+deb12u1 | 1.5.2-6+deb12u2 |

**zlib**:

| Package | CVE            | Severity | Status       | Installed       | Fixed |
| ------- | -------------- | -------- | ------------ | --------------- | ----- |
| zlib1g  | CVE-2023-45853 | CRITICAL | will_not_fix | 1:1.2.13.dfsg-1 | -     |

**OpenLDAP**:

| Package       | CVE           | Severity | Status   | Installed     | Fixed |
| ------------- | ------------- | -------- | -------- | ------------- | ----- |
| libldap-2.5-0 | CVE-2023-2953 | HIGH     | affected | 2.5.13+dfsg-5 | -     |

#### Risk Assessment

| CVE            | Component | Risk for Deployer | Rationale                                      |
| -------------- | --------- | ----------------- | ---------------------------------------------- |
| CVE-2025-13836 | Python    | LOW               | HTTP client DoS - deployer doesn't expose HTTP |
| CVE-2025-7458  | SQLite    | LOW               | Integer overflow - SQLite not used by deployer |
| CVE-2023-45853 | zlib      | LOW               | minizip functions not commonly used            |
| CVE-2025-48384 | Git       | LOW               | Internal operations only                       |
| CVE-2025-68973 | GnuPG     | LOW               | Key management not exposed                     |

#### Mitigation

- The deployer runs in a controlled environment (user's machine or CI)
- No network services are exposed from the container
- Regular image rebuilds will incorporate Debian security updates
- Application code has 0 vulnerabilities

#### Action Items

1. Monitor Debian security tracker for Python 3.11 and Git fixes
2. Rebuild image when GnuPG and PAM fixes are available in Debian repos
3. Consider future migration to newer Python (3.12+) when Debian supports it
