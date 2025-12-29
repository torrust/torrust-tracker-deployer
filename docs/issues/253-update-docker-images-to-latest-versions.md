# Update Docker Images to Latest Stable Versions

**Issue**: #253
**Parent Epic**: N/A (Standalone maintenance task)
**Related**: [#250](https://github.com/torrust/torrust-tracker-deployer/issues/250) - Implement periodic security vulnerability scanning workflow

## Overview

Update Docker images in the docker-compose template to their latest stable versions to benefit from security patches, bug fixes, and new features. This task prioritizes images with **long-term support** and **active maintenance windows** to ensure production stability and minimize upgrade frequency.

**Strategy**: Select images that balance current support status with vulnerability security:

- **Security-first**: No HIGH/CRITICAL vulnerabilities (all current images are secure)
- **Support lifecycle**: Prefer versions with extended support windows (LTS > short-term releases)
- **Active maintenance**: Choose versions actively receiving updates and security patches

## References

**End of Life Information**:

- [MySQL End of Life](https://endoflife.date/mysql)
- [Prometheus End of Life](https://endoflife.date/prometheus)
- [Grafana End of Life](https://endoflife.date/grafana)

**Docker Hub Official Images**:

- [Prometheus Docker Images](https://hub.docker.com/r/prom/prometheus/tags)
- [Grafana Docker Images](https://hub.docker.com/r/grafana/grafana/tags)
- [MySQL Docker Images](https://hub.docker.com/_/mysql/tags)

## Goals

- [ ] Update Prometheus from v3.0.1 to v3.5 LTS (Long-Term Support with 7 months remaining)
- [ ] Update Grafana from 11.4.0 to 12.3.1 (latest major version)
- [ ] Update MySQL from generic 8.0 to explicit 8.4 LTS (6+ years support remaining)
- [ ] Add comment in docker-compose template about pinning Tracker to v4.0.0 (separate issue to be created)
- [ ] Run Trivy security scans and document results
- [ ] Update documentation with latest security scan reports

## Current Status

### Docker Images Analysis (December 23, 2025)

| Image             | Current Version | Recommended Version | Support EOL  | Status                              | Security           |
| ----------------- | --------------- | ------------------- | ------------ | ----------------------------------- | ------------------ |
| `prom/prometheus` | v3.0.1          | v3.5.0 (LTS)        | Jul 31, 2026 | ⚠️ Update to LTS for 1-year support | ✅ 0 HIGH/CRITICAL |
| `grafana/grafana` | 11.4.0          | 12.3.1              | Feb 24, 2026 | ⚠️ Update to latest major version   | ✅ 0 HIGH/CRITICAL |
| `mysql`           | 8.0 (generic)   | 8.4 (LTS)           | Apr 30, 2032 | ⚠️ Update to explicit LTS version   | ✅ 0 HIGH/CRITICAL |

**Support Lifecycle Notes**:

- **Prometheus**: Follows 6-week release cycle with each minor version supported ~6 weeks. Latest v3.8.1 support ends Jan 9, 2026. LTS versions (like 3.5) get 1-year support.
- **Grafana**: Bi-monthly release schedule (Feb, Apr, Jun, Aug, Oct, Dec). Previous minor version and last minor of previous major get security fixes. 11.5.0 actively maintained.
- **MySQL**: LTS versions (8.4) get 5 years premier + 3 years extended support. Innovation releases (9.x) have 3-4 month lifecycles and are EOL quickly.

### Trivy Security Scan Results (December 23, 2025)

All current images show **zero HIGH or CRITICAL vulnerabilities**:

**Prometheus v3.0.1**:

```text
2025-12-23T13:45:04.639Z        WARN    OS is not detected and vulnerabilities in OS packages are not detected.
2025-12-23T13:45:04.639Z        INFO    Trivy skips scanning programming language libraries because no supported file was detected
Total: 0 (HIGH: 0, CRITICAL: 0)
```

**Prometheus v3.5.0** (LTS):

```text
2025-12-23T13:45:26.983Z        WARN    OS is not detected and vulnerabilities in OS packages are not detected.
2025-12-23T13:45:26.983Z        INFO    Trivy skips scanning programming language libraries because no supported file was detected
Total: 0 (HIGH: 0, CRITICAL: 0)
```

**Grafana 11.4.0**:

```text
2025-12-23T13:45:32.367Z        WARN    This OS version is not on the EOL list: alpine 3.20
2025-12-23T13:45:32.367Z        INFO    Detecting Alpine vulnerabilities...
2025-12-23T13:45:32.367Z        INFO    Trivy skips scanning programming language libraries because no supported file was detected
2025-12-23T13:45:32.367Z        WARN    This OS version is no longer supported by the distribution: alpine 3.20.3
2025-12-23T13:45:32.367Z        WARN    The vulnerability detection may be insufficient because security updates are not provided

grafana/grafana:11.4.0 (alpine 3.20.3)
======================================
Total: 0 (HIGH: 0, CRITICAL: 0)
```

**Grafana 12.3.1** (latest major):

```text
2025-12-23T13:45:39.635Z        WARN    This OS version is not on the EOL list: alpine 3.20
2025-12-23T13:45:39.635Z        INFO    Detecting Alpine vulnerabilities...
2025-12-23T13:45:39.635Z        INFO    Trivy skips scanning programming language libraries because no supported file was detected
2025-12-23T13:45:39.635Z        WARN    This OS version is no longer supported by the distribution: alpine 3.20.3
2025-12-23T13:45:39.635Z        WARN    The vulnerability detection may be insufficient because security updates are not provided

grafana/grafana:12.3.1 (alpine 3.20.3)
======================================
Total: 0 (HIGH: 0, CRITICAL: 0)
```

**MySQL 8.0**:

```text
2025-12-23T13:45:47.135Z        WARN    No OS package is detected. Make sure you haven't deleted any files that contain information about the installed packages.
2025-12-23T13:45:47.135Z        WARN    e.g. files under "/lib/apk/db/", "/var/lib/dpkg/" and "/var/lib/rpm"
2025-12-23T13:45:47.135Z        WARN    This OS version is not on the EOL list: oracle 9
2025-12-23T13:45:47.135Z        INFO    Detecting Oracle Linux vulnerabilities...
2025-12-23T13:45:47.135Z        INFO    Trivy skips scanning programming language libraries because no supported file was detected
2025-12-23T13:45:47.135Z        WARN    This OS version is no longer supported by the distribution: oracle 9.5
2025-12-23T13:45:47.135Z        WARN    The vulnerability detection may be insufficient because security updates are not provided

mysql:8.0 (oracle 9.5)
======================
Total: 0 (HIGH: 0, CRITICAL: 0)
```

**MySQL 9.1** (latest):

```text
2025-12-23T13:45:56.301Z        WARN    No OS package is detected. Make sure you haven't deleted any files that contain information about the installed packages.
2025-12-23T13:45:56.301Z        WARN    e.g. files under "/lib/apk/db/", "/var/lib/dpkg/" and "/var/lib/rpm"
2025-12-23T13:45:56.302Z        WARN    This OS version is not on the EOL list: oracle 9
2025-12-23T13:45:56.302Z        INFO    Detecting Oracle Linux vulnerabilities...
2025-12-23T13:45:56.302Z        INFO    Trivy skips scanning programming language libraries because no supported file was detected
2025-12-23T13:45:56.302Z        WARN    This OS version is no longer supported by the distribution: oracle 9.5
2025-12-23T13:45:56.302Z        WARN    The vulnerability detection may be insufficient because security updates are not provided

mysql:9.1 (oracle 9.5)
======================
Total: 0 (HIGH: 0, CRITICAL: 0)
```

### Security Status Summary

✅ **All current images are secure** - No HIGH or CRITICAL vulnerabilities detected in any version (current or latest).

**Lifecycle-Aware Recommendations**:

1. **Prometheus v3.5.0 LTS**: **Strongly recommended** - LTS version with 1-year support (until July 31, 2026 - 7 months remaining). Avoid non-LTS versions like v3.8.1 with only 6-week support windows.
2. **Grafana 12.3.1**: **Recommended** - Latest major version (12.x series) with active development. Supported until Feb 24, 2026 (2 months). Grafana follows bi-monthly release cycle.
3. **MySQL 8.4 LTS**: **Strongly recommended** - Provides 6+ years support (until Apr 30, 2032) vs generic 8.0 tag approaching EOL (Apr 2026). Avoid MySQL 9.x innovation releases (short 3-4 month lifecycles).

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (template files)
**Module Path**: `templates/docker-compose/`
**Pattern**: Static template files (no code changes required)

### Module Structure Requirements

- [ ] Template files follow Tera syntax conventions
- [ ] Changes maintain environment variable injection pattern
- [ ] Network segmentation configuration remains unchanged

### Architectural Constraints

- [ ] No changes to environment variable injection pattern (see ADR: [docs/decisions/environment-variable-injection-in-docker-compose.md](../decisions/environment-variable-injection-in-docker-compose.md))
- [ ] Preserve three-network segmentation security architecture
- [ ] Maintain Docker port binding and health check configurations

### Anti-Patterns to Avoid

- ❌ Hardcoding configuration values instead of using environment variables
- ❌ Removing network segmentation or health checks
- ❌ Changing image tags to `latest` (always use specific versions)

## Specifications

### Image Version Updates

**File**: `templates/docker-compose/docker-compose.yml.tera`

#### Prometheus Update

**Current**:

```yaml
prometheus:
  image: prom/prometheus:v3.0.1
```

**Updated**:

```yaml
prometheus:
  image: prom/prometheus:v3.5.0
```

**Release Information**:

- Released: July 14, 2025
- **LTS Version**: Long-Term Support (1-year support window)
- Support EOL: July 31, 2026 (7 months remaining)
- Security: Zero HIGH/CRITICAL vulnerabilities

**Why v3.5 LTS instead of v3.8.1**:

- **v3.5 LTS**: 1-year support until July 31, 2026 (7 months remaining)
- **v3.8.1** (latest): Only 6-week support until January 9, 2026 (2.5 weeks remaining)
- **Rationale**: LTS version provides longer stability and fewer required updates

**References**:

- [Prometheus v3.5.0 on Docker Hub](https://hub.docker.com/r/prom/prometheus/tags?name=v3.5.0)
- [Prometheus Lifecycle Information](https://endoflife.date/prometheus)

#### Grafana Update

**Current**:

```yaml
grafana:
  image: grafana/grafana:11.4.0
```

**Updated**:

```yaml
grafana:
  image: grafana/grafana:12.3.1
```

**Release Information**:

- Released: November 19, 2025 (1 month ago)
- **Latest Major Version**: 12.x series (latest patch: 12.3.1)
- Support EOL: February 24, 2026 (2 months remaining)
- Security: Zero HIGH/CRITICAL vulnerabilities

**Why 12.3.1 instead of 11.5.0**:

- **12.3.1**: Latest major version (12.x) with newest features and improvements
- **11.5.0**: Previous major version, will lose support sooner as 12.x matures
- **Rationale**: Staying on latest major version aligns with Grafana's support model

**Release Cycle**: Grafana follows bi-monthly releases (Feb, Apr, Jun, Aug, Oct, Dec). Previous minor version and last minor of previous major get security/critical bug fixes.

**References**:

- [Grafana 12.3.1 on Docker Hub](https://hub.docker.com/r/grafana/grafana/tags?name=12.3.1)
- [Grafana Lifecycle Information](https://endoflife.date/grafana)

#### MySQL Analysis

**Current**:

```yaml
mysql:
  image: mysql:8.0
```

**Recommendation**: Update to MySQL 8.4 (explicit LTS version)

**MySQL Version Lifecycle Analysis**:

| Version | Release Type | Release Date   | Premier EOL    | Extended EOL   | Status                  | Notes                                                  |
| ------- | ------------ | -------------- | -------------- | -------------- | ----------------------- | ------------------------------------------------------ |
| 8.0     | LTS          | April 8, 2018  | April 30, 2025 | April 30, 2026 | ⚠️ 4 months remaining   | **Premier support ended April 2025**, Extended only    |
| 8.4     | LTS          | April 10, 2024 | April 30, 2029 | April 30, 2032 | ✅ 6+ years remaining   | **Recommended - Long-term stability**                  |
| 9.0     | Innovation   | June 7, 2024   | Oct 15, 2024   | Oct 15, 2024   | ❌ EOL Oct 2024         | Innovation release, already unsupported                |
| 9.1     | Innovation   | Sept 24, 2024  | Jan 21, 2025   | Jan 21, 2025   | ❌ EOL Jan 2025         | Innovation release, already unsupported                |
| 9.5     | Innovation   | Oct 21, 2025   | ~Feb 2026      | ~Feb 2026      | ⚠️ 2-3 months remaining | Current innovation release, short lifecycle (3 months) |

**Rationale for MySQL 8.4**:

- **Explicit versioning**: `mysql:8.0` is too generic - could be 8.0.x with approaching EOL
- **Long-term support**: MySQL 8.4 LTS with Premier Support until April 30, 2029 (3+ years) and Extended Support until April 30, 2032 (6+ years)
- **Stable and mature**: LTS release track designed for production, not innovation/experimental
- **Zero vulnerabilities**: Same clean security posture as other versions
- **Torrust compatibility**: Torrust Tracker supports MySQL 8.x (see [README](https://github.com/torrust/torrust-tracker#key-features): "Persistent SQLite3 or MySQL Databases")
- **Docker best practice**: Pin to specific LTS version for predictable lifecycle
- **Support level**: MySQL 8.0 Premier Support ended April 2025 (only Extended Support remains until April 2026). MySQL 8.4 has full Premier Support for 3+ years.

**Why NOT MySQL 8.0**:

- Premier Support ended April 30, 2025 (only Extended Support until April 2026)
- Only 4 months of Extended Support remaining
- Generic `mysql:8.0` tag may point to minor versions approaching EOL
- Updates and fixes limited to critical patches only

**Why NOT MySQL 9.x Innovation Releases**:

- Innovation releases have **extremely short** support windows (3-4 months per release)
- MySQL 9.0 EOL: October 15, 2024 (already unsupported)
- MySQL 9.1 EOL: January 21, 2025 (already unsupported)
- MySQL 9.5 EOL: ~February 2026 (only 2-3 months remaining)
- Not suitable for production long-term deployments
- May have breaking changes requiring extensive testing
- Frequent version churn increases operational overhead

**Action**:

- Update to `mysql:8.4` for long-term stability and active Premier Support
- Avoid MySQL 8.0 (Extended Support only, approaching EOL)
- Avoid MySQL 9.x innovation releases (short lifecycles, rapid EOL)

**References**:

- [MySQL 8.4 on Docker Hub](https://hub.docker.com/_/mysql/tags?name=8.4)
- [MySQL Lifecycle Information](https://endoflife.date/mysql)

### Security Documentation

Create or update documentation file to track security scan history:

**File**: `docs/security/docker-image-security-scans.md`

Structure:

```markdown
# Docker Image Security Scans

This document tracks Trivy security scan results for Docker images used in the deployer templates.

## Latest Scan: December 23, 2025

### Scan Command

\`\`\`bash
trivy image --severity HIGH,CRITICAL <image-name>
\`\`\`

### Results

#### Prometheus v3.8.1

[Scan output]

#### Grafana 11.5.0

[Scan output]

#### MySQL 8.0

[Scan output]

## Previous Scans

### [Date]

[Previous scan results]

## Implementation Plan

### Phase 0: Add Tracker Version Pinning Comment (estimated: 5 minutes)

**Note**: The actual version update will be done in a separate issue after Tracker v4.0.0 is released. This phase only adds documentation.

- [ ] Add comment in `templates/docker-compose/docker-compose.yml.tera` above Tracker image line
- [ ] Comment should note: "TODO: Pin to stable v4.0.0 when released (currently using develop tag - see issue #TBD)"
- [ ] Create separate follow-up issue: "Pin Tracker image to v4.0.0 in docker-compose template"

### Phase 1: Update Prometheus (estimated: 30 minutes)

- [ ] Update `templates/docker-compose/docker-compose.yml.tera` - Change Prometheus image from `v3.0.1` to `v3.5.0`
- [ ] Regenerate docker-compose template for testing environment
- [ ] Run E2E tests to verify Prometheus functionality
- [ ] Verify Prometheus health checks pass
- [ ] Verify Grafana can query Prometheus data source

### Phase 2: Update Grafana (estimated: 30 minutes)

- [ ] Update `templates/docker-compose/docker-compose.yml.tera` - Change Grafana image from `11.4.0` to `12.3.1`
- [ ] Regenerate docker-compose template for testing environment
- [ ] Run E2E tests to verify Grafana functionality
- [ ] Verify Grafana health checks pass
- [ ] Verify Grafana dashboards load correctly
- [ ] Verify Grafana can query Prometheus metrics

### Phase 3: MySQL Version Update (estimated: 45 minutes)

- [ ] Update `templates/docker-compose/docker-compose.yml.tera` - Change MySQL image from `mysql:8.0` to `mysql:8.4`
- [ ] Document MySQL version lifecycle rationale (8.0 → 8.4 LTS, avoid 9.x innovation releases)
- [ ] Verify MySQL 8.4 compatibility with Torrust Tracker (likely compatible, same major version)
- [ ] Run Trivy security scan on `mysql:8.4`
- [ ] Regenerate docker-compose template for testing environment
- [ ] Run E2E tests to verify MySQL 8.4 functionality
- [ ] Verify MySQL health checks pass with 8.4

### Phase 4: Security Documentation (estimated: 30 minutes)

- [ ] Create `docs/security/` directory (if not exists)
- [ ] Create `docs/security/docker-image-security-scans.md` with scan template structure
- [ ] Document Trivy scan results for all updated images (Prometheus v3.5.0, Grafana 12.3.1, MySQL 8.4)
- [ ] Run Trivy scans with updated images and capture output
- [ ] Add scan date, command used, and full output for each image
- [ ] Update README or contributing guide to reference security scan documentation
- [ ] Add note about issue [#250](https://github.com/torrust/torrust-tracker-deployer/issues/250) for automated periodic scanning

### Phase 5: Testing and Validation (estimated: 1 hour)

- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`
- [ ] Run unit tests: `cargo test`
- [ ] Run E2E infrastructure lifecycle tests: `cargo run --bin e2e-infrastructure-lifecycle-tests`
- [ ] Run E2E deployment workflow tests: `cargo run --bin e2e-deployment-workflow-tests`
- [ ] Verify all services start correctly with updated images
- [ ] Verify network segmentation still works correctly
- [ ] Verify health checks pass for all services
- [ ] Manual verification: Check Grafana dashboards display metrics from Prometheus

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Image Update Criteria**:

- [ ] Comment added in docker-compose template about pinning Tracker to v4.0.0
- [ ] Separate follow-up issue created for Tracker version update
- [ ] Prometheus image updated to v3.5.0 in `templates/docker-compose/docker-compose.yml.tera`
- [ ] Grafana image updated to 12.3.1 in `templates/docker-compose/docker-compose.yml.tera`
- [ ] MySQL updated to explicit LTS version 8.4 (not generic 8.0, not innovation 9.x)
- [ ] All E2E tests pass with updated images
- [ ] Health checks pass for all services (Prometheus, Grafana, MySQL)
- [ ] Network segmentation configuration remains unchanged
- [ ] Environment variable injection pattern preserved

**Security Documentation Criteria**:

- [ ] `docs/security/docker-image-security-scans.md` created with scan results
- [ ] Trivy scan output documented for Prometheus v3.5.0
- [ ] Trivy scan output documented for Grafana 12.3.1
- [ ] Trivy scan output documented for MySQL 8.4
- [ ] Scan date and Trivy version recorded
- [ ] Documentation includes reference to issue [#250](https://github.com/torrust/torrust-tracker-deployer/issues/250)

**MySQL Version Criteria**:

- [ ] MySQL version lifecycle analysis documented (8.0 vs 8.4 vs 9.x)
- [ ] MySQL updated to 8.4 LTS (explicit version, not generic 8.0)
- [ ] Rationale documented: 8.4 LTS supported until 2032, avoid 9.x innovation releases
- [ ] Trivy security scan completed for mysql:8.4

**Functional Verification Criteria**:

- [ ] Prometheus starts successfully and health check passes
- [ ] Grafana starts successfully and health check passes
- [ ] MySQL starts successfully and health check passes
- [ ] Grafana can query Prometheus data source successfully
- [ ] Tracker can connect to MySQL database successfully
- [ ] Prometheus can scrape metrics from Tracker
- [ ] All Docker networks function correctly (database_network, metrics_network, visualization_network)

## Related Documentation

- [Docker Compose Template](../../templates/docker-compose/docker-compose.yml.tera)
- [ADR: Environment Variable Injection in Docker Compose](../decisions/environment-variable-injection-in-docker-compose.md)
- [ADR: Docker UFW Firewall Security Strategy](../decisions/docker-ufw-firewall-security-strategy.md)
- [Analysis: Docker Network Segmentation](../analysis/security/docker-network-segmentation-analysis.md)
- [Issue #250: Implement periodic security vulnerability scanning workflow](https://github.com/torrust/torrust-tracker-deployer/issues/250)
- [Torrust Tracker README - MySQL Support](https://github.com/torrust/torrust-tracker#key-features)

## Notes

### Tracker Version Pinning Strategy

**Current Issue**: Using `develop` tag for Tracker image

**Problem**:

- The `develop` tag is mutable and may introduce unexpected breaking changes
- Makes deployments non-reproducible (different pulls may get different versions)
- Violates principle of immutable infrastructure
- Harder to rollback if issues occur

**Solution**: Pin to stable release version (tracked in separate issue)

**This Issue's Scope**:

- Add a TODO comment in the docker-compose template noting this needs to be changed
- Create a separate follow-up issue to track the actual version update

**Rationale for Separate Issue**:

- Tracker v4.0.0 release date is unknown
- This image update issue should not be blocked waiting for Tracker release
- Separate issue allows tracking and prioritization independently
- Comment serves as in-code reminder for future maintenance

**Future Issue Will Include**:

- Wait for Torrust Tracker v4.0.0 release announcement
- Update template to use `torrust/tracker:v4.0.0`
- Run Trivy security scan on v4.0.0
- Update E2E tests if needed
- Benefits: Predictable deployments, controlled upgrade path, easy rollback

### Security Posture

All current images (v3.0.1 Prometheus, 11.4.0 Grafana, 8.0 MySQL) are **secure** with zero HIGH or CRITICAL vulnerabilities. This update is primarily for:

- Feature improvements (7 minor Prometheus versions include significant enhancements)
- Bug fixes
- Staying current with upstream releases
- Reducing technical debt

### MySQL Version Strategy

**Conservative Approach Recommended**:

- MySQL 8.0 is enterprise-stable and widely deployed
- MySQL 9.1 is an innovation release (may have breaking changes)
- Torrust Tracker MySQL driver compatibility unknown for 9.x
- Both versions show zero security vulnerabilities
- Risk/reward favors staying on 8.0 until:
  - MySQL 9.x matures
  - Torrust Tracker team confirms compatibility
  - Breaking changes are well-documented

**Future Consideration**: Create separate issue for MySQL 9.x migration after thorough compatibility testing.

### Periodic Security Scanning

This manual security scan validates current security posture. For ongoing security monitoring, see:

- [Issue #250: Implement periodic security vulnerability scanning workflow](https://github.com/torrust/torrust-tracker-deployer/issues/250)

The automated workflow will:

- Run Trivy scans on CI/CD pipeline
- Generate security reports
- Alert on new vulnerabilities
- Track vulnerability trends over time

### Trivy Scan Notes

**Warning Messages Explained**:

- **"OS is not detected"** (Prometheus): Prometheus uses a minimal scratch image - expected behavior
- **"Alpine 3.20.3 no longer supported"** (Grafana): Cosmetic warning - Alpine maintainers have moved to newer versions, but Grafana 11.5.0 is recent (Dec 18, 2025) and has zero vulnerabilities
- **"Oracle 9.5 no longer supported"** (MySQL): Similar cosmetic warning - MySQL official images are maintained by Oracle and are secure

These warnings do not indicate security issues - they reflect Trivy's conservative detection heuristics.
