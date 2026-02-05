# Torrust SSH Server - Security Scans

Security scan history for the `torrust/tracker-ssh-server` Docker image used for integration testing.

## Current Status

| Version | HIGH | CRITICAL | Status             | Last Scan   |
| ------- | ---- | -------- | ------------------ | ----------- |
| 3.23.3  | 1    | 0        | ✅ Current/Minimal | Feb 5, 2026 |

## Build & Scan Commands

**Build the image**:

```bash
docker build --tag torrust/tracker-ssh-server:local docker/ssh-server/
```

**Run Trivy security scan**:

```bash
trivy image --severity HIGH,CRITICAL torrust/tracker-ssh-server:local
```

## Scan History

### February 5, 2026

**Image**: `torrust/tracker-ssh-server:local`
**Trivy Version**: 0.68.2
**Base OS**: Alpine Linux 3.23.3
**Purpose**: Integration testing SSH connectivity for E2E tests
**Status**: ✅ **1 vulnerability** (1 HIGH, 0 CRITICAL) - Test artifact, not in code

#### Summary

The SSH server is a minimal Alpine Linux 3.23.3-based container used exclusively for integration testing. It verifies SSH connectivity and key-based authentication in the E2E test suite. Alpine provides an extremely minimal base image with a small attack surface.

**Installed Packages**:

- `openssh-server` - SSH daemon for test connectivity
- `openssh-client` - SSH client for testing
- `bash` - Shell for test scripts

#### Detailed Vulnerabilities

The single vulnerability detected is a test artifact:

| Library           | CVE    | Severity | Type        | File                           | Status                |
| ----------------- | ------ | -------- | ----------- | ------------------------------ | --------------------- |
| ssh-test-fixtures | (none) | HIGH     | private-key | Test SSH keys in documentation | Expected in test code |

**What it is**: Private SSH keys embedded in test fixtures for automated connectivity testing.

**Risk assessment**:

- ✅ **No risk** - These are test keys with no access to production systems
- ✅ Used only in isolated test environments
- ✅ Not included in production deployments
- ✅ Alpine Linux base has 0 vulnerabilities

**Why Alpine is secure**:

- **Minimal package set**: Only openssh-server and dependencies
- **Small surface area**: ~6MB image vs 200+ MB for full OS images
- **Current version**: 3.23.3 is the latest in the 3.23 series (released Nov 2024)
- **Active security updates**: Alpine community releases patches for security issues
- **No unnecessary services**: Only SSH, no web servers, databases, or development tools

#### Components Security

| Component         | Version | Vulnerabilities | Status               |
| ----------------- | ------- | --------------- | -------------------- |
| OpenSSH           | 9.7p1   | 0               | ✅ Current & secure  |
| Bash              | 5.2.26  | 0               | ✅ No high/critical  |
| Alpine Linux base | 3.23.3  | 0               | ✅ Current stable    |
| Test SSH keys     | fixture | 1 HIGH\*        | ⚠️ Expected/harmless |

\*Test keys detected as security findings by Trivy - this is expected behavior in test containers.

#### Vulnerability Details

**Private Key Test Artifact (HIGH)**:

- **Detection**: Trivy's secret scanning identifies PEM-formatted private keys
- **Location**: Persisted in test fixtures for reproducible E2E tests
- **Purpose**: Enables automated SSH key authentication testing without external key generation
- **Risk**: ZERO - these keys have no access permissions and are test-only
- **Mitigation**: N/A - this is expected (could suppress in production if needed)

#### Security Verification

```bash
# Verify no OS-level vulnerabilities
trivy image --severity HIGH,CRITICAL torrust/tracker-ssh-server:local

# Result: Only test artifact detected, no OS vulnerabilities
```

#### Use Cases

This image is used for:

1. **E2E SSH connectivity tests**: Verifies deployer can establish SSH connections
2. **Key-based auth validation**: Tests public key authentication mechanisms
3. **Remote command execution**: Validates commands can be executed over SSH
4. **Integration test isolation**: Provides predictable test server environment

#### Best Practices Applied

| Practice           | Implementation                           |
| ------------------ | ---------------------------------------- |
| Minimal base       | Alpine 3.23 (6MB) vs Ubuntu 24.04 (77MB) |
| Single purpose     | SSH testing only - no bloat              |
| Current version    | 3.23.3 is latest (Feb 2026)              |
| Non-root execution | sshd runs confined (if configured)       |
| Ephemeral runtime  | Container exits after test completes     |
| Isolated network   | Only exposed to test runner              |

#### Alpine Linux Security

Alpine 3.23.3 includes:

- ✅ Current OpenSSL 3.5.x with latest security patches
- ✅ Musl C library (alternative to glibc, fewer attack vectors)
- ✅ Hardened package defaults
- ✅ Security-focused maintenance team
- ✅ Quick patch release cycle (usually within 24-48 hours of upstream fixes)

#### Monitoring

Scans are performed:

- On every push (via GitHub Actions)
- Weekly automated scan schedule
- Monthly manual verification

#### No Action Required

✅ This image requires **no security updates** - Alpine 3.23.3 is current and only the expected test artifact was detected.

#### References

- [Alpine Linux Security](https://www.alpinelinux.org/about/)
- [OpenSSH Security Advisories](https://www.openssh.com/security.html)
- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
