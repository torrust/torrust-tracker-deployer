# Torrust Tracker Deployer - Security Scans

Security scan history for the `torrust/tracker-deployer` Docker image.

## Current Status

| Version | HIGH | CRITICAL | Status                                             | Last Scan    |
| ------- | ---- | -------- | -------------------------------------------------- | ------------ |
| trixie  | 46   | 1        | ⚠️ CRITICAL blocked on OpenTofu upstream (grpc-go) | Apr 15, 2026 |

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

### April 15, 2026 - Remediation Pass 2 (Issue #429)

**Image**: `torrust/tracker-deployer:local`
**OpenTofu version**: v1.11.6 (latest, released 2026-04-08)
**Trivy Version**: 0.69.3
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Base OS**: Debian 13.4 (trixie)
**Status**: ⚠️ **1 CRITICAL remains** (blocked on OpenTofu upstream) — 46 HIGH, 1 CRITICAL

#### Summary

Image rebuilt from scratch with `--no-cache`. OpenTofu v1.11.6 (latest) was installed.
CRITICAL in `usr/bin/tofu` (CVE-2026-33186, grpc-go) **remains unresolved** — needs
OpenTofu to upgrade `google.golang.org/grpc` to v1.79.3+.

#### Target breakdown

| Target                                         | HIGH   | CRITICAL |
| ---------------------------------------------- | ------ | -------- |
| `torrust/tracker-deployer:local` (debian 13.4) | 42     | 0        |
| `usr/bin/tofu`                                 | 4      | 1        |
| **Total**                                      | **46** | **1**    |

#### Comparison vs pass 1 (Apr 8)

| Target         | Apr 8 (HIGH / CRITICAL) | Apr 15 (HIGH / CRITICAL) | Delta                                      |
| -------------- | ----------------------- | ------------------------ | ------------------------------------------ |
| Debian OS      | 42 / 0                  | 42 / 0                   | no change (same Debian state)              |
| `usr/bin/tofu` | 2 / 1                   | 4 / 1                    | +2 HIGH (Trivy DB update)                  |
| **Total**      | **44 / 1**              | **46 / 1**               | **+2 HIGH (Trivy DB), CRITICAL unchanged** |

#### `usr/bin/tofu` CVE details (OpenTofu v1.11.6)

| CVE            | Library                        | Severity | Status | Installed | Fixed  | Title                                           |
| -------------- | ------------------------------ | -------- | ------ | --------- | ------ | ----------------------------------------------- |
| CVE-2026-33186 | google.golang.org/grpc         | CRITICAL | fixed  | v1.76.0   | 1.79.3 | gRPC-Go: Authorization bypass via HTTP/2 path   |
| CVE-2026-34986 | github.com/go-jose/go-jose/v4  | HIGH     | fixed  | v4.1.2    | 4.1.4  | JOSE: DoS via crafted JSON Web Encryption       |
| CVE-2026-4660  | github.com/hashicorp/go-getter | HIGH     | fixed  | v1.8.2    | 1.8.6  | go-getter: Arbitrary file reads via crafted URL |
| CVE-2026-24051 | go.opentelemetry.io/otel/sdk   | HIGH     | fixed  | v1.38.0   | 1.40.0 | OTel Go SDK: Arbitrary code execution via PATH  |
| CVE-2026-39883 | go.opentelemetry.io/otel/sdk   | HIGH     | fixed  | v1.38.0   | 1.43.0 | OTel Go SDK: BSD kenv PATH hijacking            |

All `usr/bin/tofu` CVEs have fixes available in their respective upstream libraries but
require OpenTofu to update its Go module dependencies and ship a new release.

#### Notable Debian OS CVEs (selected new or notable HIGH, all `affected` / no fix in trixie)

| CVE            | Package        | Title                                                       |
| -------------- | -------------- | ----------------------------------------------------------- |
| CVE-2025-13836 | python3.13     | cpython: Excessive read buffering DoS in http.client        |
| CVE-2025-15366 | python3.13     | cpython: IMAP command injection (`will_not_fix`)            |
| CVE-2025-15367 | python3.13     | cpython: POP3 command injection (`will_not_fix`)            |
| CVE-2026-25210 | libexpat1      | libexpat: Integer overflow — data integrity issues          |
| CVE-2026-29111 | libsystemd0    | systemd: Assert/freeze via spurious IPC (`<no-dsa>`)        |
| CVE-2026-35385 | openssh-client | OpenSSH: Priv escalation via scp legacy protocol            |
| CVE-2026-35414 | openssh-client | OpenSSH: Security bypass via authorized_keys principals     |
| CVE-2026-35535 | sudo           | Sudo: Privilege escalation via failed privilege drop        |
| CVE-2025-69720 | ncurses        | ncurses: Buffer overflow in `infocmp` CLI tool (`<no-dsa>`) |

#### Decision

**Leave issue #429 open — CRITICAL unresolved.**

- CRITICAL CVE-2026-33186 (grpc-go, gRPC authorization bypass) remains in `usr/bin/tofu` v1.11.6
- Fix requires OpenTofu to bump `google.golang.org/grpc` to v1.79.3+ and ship a new release
- Debian OS CVEs are all `affected`/`will_not_fix`/`<no-dsa>` with no trixie backports available

**Revisit**: When OpenTofu releases v1.11.7+ or v1.12.x with updated `grpc-go` dependency.

---

### April 8, 2026 - Remediation Pass 1 (Issue #428)

**Image**: `torrust/tracker-deployer:local`
**Trivy Version**: 0.68.2
**Scan Mode**: `--scanners vuln --severity HIGH,CRITICAL`
**Base OS**: Debian 13.4 (trixie)
**Status**: ⚠️ **Partial improvement** - 44 HIGH, 1 CRITICAL

#### Summary

After the first remediation pass in issue #428:

- Runtime GnuPG footprint was reduced (install only for OpenTofu setup, then purge)
- Package upgrade was applied during image build
- Image remained functional in smoke test (`docker run --rm ... --help`)

#### Comparison vs previous April scan

| Target                                | Previous                | Current                 | Delta       |
| ------------------------------------- | ----------------------- | ----------------------- | ----------- |
| `torrust/tracker-deployer:local` (OS) | 49 HIGH                 | 42 HIGH                 | -7 HIGH     |
| `usr/bin/tofu`                        | 2 HIGH, 1 CRITICAL      | 2 HIGH, 1 CRITICAL      | no change   |
| **Total**                             | **51 HIGH, 1 CRITICAL** | **44 HIGH, 1 CRITICAL** | **-7 HIGH** |

#### Remaining concerns

- OpenTofu binary still reports 1 CRITICAL and 2 HIGH findings
- Debian base packages still contain unresolved HIGH findings
- Additional remediation/follow-up required

### April 8, 2026 - Regression Alert - New CVEs Discovered

**Image**: `torrust/tracker-deployer:local`
**Trivy Version**: 0.68.2
**Base OS**: Debian 13.4 (trixie)
**Status**: ⚠️ **Significant regression** - 49 HIGH vulnerabilities (up from 1 in Feb 5 scan)

#### Summary

The April 8, 2026 scan reveals major vulnerabilities that were not detected in the February 5 scan. The Trivy vulnerability database appears to have been updated with new CVE entries, or Debian 13.4 contains additional security issues not present in earlier 13.x releases.

**Alert**: Before deploying, manually verify whether these represent:

1. New Debian 13.4 package vulnerabilities that need investigation
2. Updated Trivy database with previously unknown CVEs
3. False positives from secret scanning (test fixtures)

#### Detailed Results

**Debian Base Packages (49 HIGH)**:

The majority of vulnerabilities are in Debian 13.4 base packages:

| Package Category | Count | CVEs                                                                 | Status                         |
| ---------------- | ----- | -------------------------------------------------------------------- | ------------------------------ |
| GnuPG packages   | ~32   | CVE-2026-24882                                                       | Affected - needs investigation |
| System libraries | ~8    | CVE-2025-69720, CVE-2026-27135, CVE-2025-13836, CVE-2025-15366, etc. | Affected                       |
| Test artifacts   | ~9    | SSH keys, AWS credentials in test fixtures                           | Low risk - test only           |

**Binary Vulnerabilities (3 total: 2 HIGH, 1 CRITICAL)**:

| Binary                            | CVEs                     | Severity           |
| --------------------------------- | ------------------------ | ------------------ |
| `/usr/bin/tofu` (OpenTofu binary) | CVE-2026-34986 (go-jose) | 1 HIGH, 1 CRITICAL |

#### Risk Assessment

**High Priority Action Needed**:

1. **GnuPG Regression** (CVE-2026-24882): Stack-based buffer overflow in tpm2daemon
   - Status: Marked as "affected" with no fixed version in Debian repos
   - Risk: Could allow arbitrary code execution
   - Action: Contact Debian maintainers or consider alternative base image

2. **Binary Dependencies**: OpenTofu go-jose library needs update
   - Status: Fixed version available (go-jose v4.1.4)
   - Action: Rebuild with updated OpenTofu

3. **Test Artifacts**: Private keys and AWS credentials in test fixtures
   - Status: Expected in test code
   - Risk: Negligible (test-only, not deployed)

#### Investigation Required

This is a significant change from the February scan result (1 HIGH) to April (49 HIGH). Before updating deployment systems, determine:

1. Check Debian 13.4 security advisories: https://security-tracker.debian.org/
2. Compare Trivy database versions between Feb and Apr scans
3. Verify if base `debian:trixie` image has the same issues
4. Check if Dockerfile changes inadvertently added new packages

**Pending**: Need manual investigation before marking as clear for deployment.

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
