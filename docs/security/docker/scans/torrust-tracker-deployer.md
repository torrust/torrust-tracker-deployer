# Torrust Tracker Deployer - Security Scans

Security scan history for the `torrust/tracker-deployer` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status       | Last Scan    |
| ------- | ---- | -------- | ------------ | ------------ |
| latest  | 25   | 7        | ⚠️ Monitored | Jan 10, 2026 |

## Scan History

### January 10, 2026

**Image**: `torrust/tracker-deployer:latest`
**Trivy Version**: 0.68.2
**Base OS**: Debian 12.12 (bookworm)
**Status**: ⚠️ 32 vulnerabilities (25 HIGH, 7 CRITICAL) - All in Debian base packages

#### Summary

All vulnerabilities are in Debian 12 (bookworm) base packages, not our application code:

- **OpenTofu binary** (`usr/bin/tofu`): 0 vulnerabilities
- **Ansible packages** (pipx venvs): 0 vulnerabilities
- **Deployer binary**: 0 vulnerabilities

#### Detailed Results

**GnuPG (14 packages affected)**:

| Package  | CVE            | Severity | Status | Installed          | Fixed              |
| -------- | -------------- | -------- | ------ | ------------------ | ------------------ |
| dirmngr  | CVE-2025-68973 | HIGH     | fixed  | 2.2.40-1.1+deb12u1 | 2.2.40-1.1+deb12u2 |
| gnupg    | CVE-2025-68973 | HIGH     | fixed  | 2.2.40-1.1+deb12u1 | 2.2.40-1.1+deb12u2 |
| gpg      | CVE-2025-68973 | HIGH     | fixed  | 2.2.40-1.1+deb12u1 | 2.2.40-1.1+deb12u2 |
| gpg-agent| CVE-2025-68973 | HIGH     | fixed  | 2.2.40-1.1+deb12u1 | 2.2.40-1.1+deb12u2 |
| ...      | ...            | ...      | ...    | ...                | ...                |

**Git (2 packages affected)**:

| Package  | CVE            | Severity | Status   | Installed          | Fixed |
| -------- | -------------- | -------- | -------- | ------------------ | ----- |
| git      | CVE-2025-48384 | HIGH     | affected | 1:2.39.5-0+deb12u2 | -     |
| git      | CVE-2025-48385 | HIGH     | affected | 1:2.39.5-0+deb12u2 | -     |
| git-man  | CVE-2025-48384 | HIGH     | affected | 1:2.39.5-0+deb12u2 | -     |
| git-man  | CVE-2025-48385 | HIGH     | affected | 1:2.39.5-0+deb12u2 | -     |

**Python 3.11 (6 packages affected)**:

| Package                 | CVE            | Severity | Status   | Installed        | Fixed |
| ----------------------- | -------------- | -------- | -------- | ---------------- | ----- |
| libpython3.11-minimal   | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| libpython3.11-minimal   | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |
| libpython3.11-stdlib    | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| libpython3.11-stdlib    | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |
| python3.11              | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| python3.11              | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |
| python3.11-minimal      | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| python3.11-minimal      | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |
| python3.11-venv         | CVE-2025-13836 | CRITICAL | affected | 3.11.2-6+deb12u6 | -     |
| python3.11-venv         | CVE-2025-8194  | HIGH     | affected | 3.11.2-6+deb12u6 | -     |

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

| Package | CVE            | Severity | Status       | Installed         | Fixed |
| ------- | -------------- | -------- | ------------ | ----------------- | ----- |
| zlib1g  | CVE-2023-45853 | CRITICAL | will_not_fix | 1:1.2.13.dfsg-1   | -     |

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
