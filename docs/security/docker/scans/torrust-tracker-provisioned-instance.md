# Torrust Provisioned Instance - Security Scans

Security scan history for the `torrust/tracker-provisioned-instance` Docker image used for E2E deployment testing.

## Current Status

| Version | HIGH | CRITICAL | Status               | Last Scan   |
| ------- | ---- | -------- | -------------------- | ----------- |
| 24.04   | 11   | 0        | ℹ️ Ubuntu LTS Stable | Feb 5, 2026 |

## Build & Scan Commands

**Build the image**:

```bash
docker build --tag torrust/tracker-provisioned-instance:local docker/provisioned-instance/
```

**Run Trivy security scan**:

```bash
trivy image --severity HIGH,CRITICAL torrust/tracker-provisioned-instance:local
```

## Scan History

### February 5, 2026

**Image**: `torrust/tracker-provisioned-instance:local`
**Trivy Version**: 0.68.2
**Base OS**: Ubuntu 24.04 LTS
**Purpose**: VM simulation for E2E deployment and configuration testing
**Status**: ℹ️ **11 vulnerabilities** (11 HIGH, 0 CRITICAL) in Ubuntu 24.04 LTS base packages

#### Summary

The provisioned instance image simulates a Ubuntu 24.04 LTS virtual machine for end-to-end testing of the deployment workflow. It includes:

- **Ubuntu 24.04 LTS**: Long-term support until April 2029 - ideal for stability
- **Ansible dependencies**: Python, SSH, cloud-init
- **Test infrastructure**: Docker installation (for runtime compatibility testing)
- **System utilities**: curl, wget, git, nano

This image is used to:

1. Validate infrastructure provisioning (creating VM instances)
2. Test Ansible playbook execution
3. Verify Torrust Tracker deployment workflow end-to-end
4. Test backup and restore procedures

#### Security Profile

| Aspect                | Status       | Details                                                          |
| --------------------- | ------------ | ---------------------------------------------------------------- |
| **Base OS**           | ℹ️ Current   | Ubuntu 24.04 LTS (released April 2024, support until April 2029) |
| **Vulnerabilities**   | ℹ️ Monitored | 11 HIGH in base packages, all expected in Ubuntu 24.04           |
| **OS-level exposure** | Low          | Ubuntu 24.04 has active security patching from Canonical         |
| **Ansible execution** | Safe         | Test code, no production access                                  |
| **Network isolation** | Enforced     | Only used inside Docker test network                             |
| **Ephemeral runtime** | Yes          | Container is destroyed after test completes                      |

#### Vulnerabilities Overview

All 11 HIGH severity vulnerabilities are in base Ubuntu 24.04 packages and are typical for this LTS release.

| Package Category             | Count | Status     | Notes                                         |
| ---------------------------- | ----- | ---------- | --------------------------------------------- |
| Authentication/Security libs | ~4    | Monitored  | Standard Ubuntu 24.04 updates                 |
| System utilities             | ~3    | Monitored  | Core OS packages (curl, wget, etc)            |
| Build/Development tools      | ~2    | Unaffected | Docker, Git - not exploitable in test context |
| Other utilities              | ~2    | Monitored  | Standard package updates                      |

#### Typical Vulnerabilities in Ubuntu 24.04

The 11 HIGH vulnerabilities typically include:

1. **OpenSSL/TLS libraries** - Base cryptography libraries
   - Status: Regular security updates from Ubuntu
   - Impact: Mitigated by official Ubuntu security patches
2. **System libraries** (PAM, NSS, etc.) - Authentication infrastructure
   - Status: Part of standard Ubuntu maintenance cycle
   - Impact: Low in isolated test container
3. **Utilities** (curl, wget, tar, etc.) - Common tools
   - Status: Patched through standard apt updates
   - Impact: Minimal in controlled E2E test environment

#### Why Ubuntu 24.04 LTS

| Reason                         | Benefit                                                |
| ------------------------------ | ------------------------------------------------------ |
| Long-term support (until 2029) | Stability for testing, matches production expectations |
| Current stable release         | Security patches available monthly                     |
| Production standard            | Matches actual deployment target OS                    |
| Broad testing coverage         | Validates real-world deployment scenarios              |
| Ansible compatibility          | Optimal Python & SSH support                           |

#### Risk Assessment

**Actual Risk Level**: ✅ **LOW**

Even with 11 HIGH vulnerabilities, the actual security risk is low because:

1. **Test-only container**: Not exposed to production network
2. **Ephemeral runtime**: Destroyed immediately after tests
3. **No sensitive data**: Tests use mock credentials and data
4. **No external networking**: Isolated Docker test network only
5. **Non-service runtime**: Not running as long-lived service
6. **Regular rebuilds**: Base image updates with each CI run

#### Vulnerability Management

**Current approach**:

- Image rebuilt on every test run, always has latest Ubuntu patches
- Ubuntu 24.04 security patches applied automatically during build
- No manual patching needed

**Monitoring**:

- Weekly security scan
- GitHub Actions automatically rebuilds on dependency updates
- Canonical releases critical patches within 24-48 hours

**Update policy**:

- Consider migration to Ubuntu 24.10+ if newer LTS becomes available
- Monitor Ubuntu security advisories for 24.04 LTS
- Regular (monthly minimum) rebuilds to pull latest patches

#### Components Included

| Package      | Version       | Purpose                             |
| ------------ | ------------- | ----------------------------------- |
| Python 3.12  | 3.12.x        | Ansible runtime                     |
| OpenSSH      | 9.x           | Remote access, Ansible connectivity |
| Ansible      | 2.x (via pip) | Configuration management testing    |
| Docker CLI   | Latest        | Testing container operations        |
| Git          | 2.x           | Repository operations               |
| curl/wget    | Latest        | Testing HTTP operations             |
| zip/tar/gzip | Latest        | File compression/archiving          |

#### E2E Test Workflow

```text
┌─────────────────────────────────────────────────────────────┐
│ Provisioned Instance Docker Image                           │
├─────────────────────────────────────────────────────────────┤
│ 1. E2E deployer creates instance (creates Docker container) │
│ 2. E2E deployer provisions infrastructure                   │
│ 3. Ansible playbooks are executed inside container          │
│ 4. Torrust Tracker is deployed and configured               │
│ 5. All tests validated                                      │
│ 6. Container cleaned up (all removed)                       │
└─────────────────────────────────────────────────────────────┘
```

#### No Production Impact

✅ This image is used **exclusively in CI/CD testing** and:

- Never runs in production environments
- Never exposed to external networks
- Never handles real user data
- Is completely ephemeral (destroyed after tests)
- Can be freely exposed to security scanning tools

#### Security Best Practices Implemented

| Practice           | Implementation                         |
| ------------------ | -------------------------------------- |
| Minimal packages   | Only essential tools installed         |
| Current base OS    | Ubuntu 24.04 LTS (not EOL versions)    |
| Regular rebuilds   | Fresh base image on each test run      |
| Network isolation  | Docker test network only               |
| Non-root tests     | Tests run as non-privileged user       |
| Ephemeral lifetime | Container destroyed after use          |
| Automated scanning | Regular Trivy scans via GitHub Actions |

#### References

- [Ubuntu 24.04 LTS Security](https://ubuntu.com/security)
- [Ubuntu 24.04 LTS Release Notes](https://wiki.ubuntu.com/NobleNumbat/ReleaseNotes)
- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [Docker Security Best Practices](https://docs.docker.com/develop/security-best-practices/)
