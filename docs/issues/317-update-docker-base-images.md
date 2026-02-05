# Update Docker Base Images

**Issue**: #317
**Roadmap**: [#1 - Project Roadmap](https://github.com/torrust/torrust-tracker-deployer/issues/1)
**Related**: [PR #1629 - Torrust Tracker](https://github.com/torrust/torrust-tracker/pull/1629), [Docs: Docker Security Scanning](../security/docker/README.md)

## Overview

Update all Docker base images to the latest stable releases to ensure security, stability, and consistency across the Torrust Tracker Deployer environment. This is a periodic maintenance task to keep the application secure and aligned with current best practices for container base image selection.

This task was motivated by the Torrust Tracker project's recent update to use `trixie` as the stable Debian version (see PR #1629), ensuring consistency across the ecosystem.

## Goals

- [ ] Ensure all Docker container base images are on the latest stable releases
- [ ] Achieve consistency across Debian-based images (standardize on `trixie` where applicable)
- [ ] Verify security vulnerability status for all base images
- [ ] Document scan results for future reference
- [ ] Maintain backward compatibility and functionality

## Current State

### Base Images in Use

1. **docker/backup/Dockerfile**
   - Current: `debian:trixie-slim` ✅ (Already updated)
   - Status: Current stable

2. **docker/deployer/Dockerfile**
   - Current: `rust:bookworm`
   - Needs: Update to `rust:trixie` for consistency

3. **docker/provisioned-instance/Dockerfile**
   - Current: `ubuntu:24.04` (LTS)
   - Status: Check for newer LTS or validate 24.04 is current

4. **docker/ssh-server/Dockerfile**
   - Current: `alpine:3.23.3` (Fixed version)
   - Status: Check for newer Alpine releases

## 🏗️ Architecture Requirements

**Type**: Infrastructure & Maintenance  
**Layer**: Infrastructure (Docker configuration)  
**Impact**: Zero runtime code changes; updates only base images

### No Architectural Changes Required

- This is infrastructure configuration maintenance only
- No Rust code changes
- No DDD layer placement needed
- Changes are isolated to Dockerfile base image declarations

## Specifications

### 1. Debian Base Images (Backup and Deployer)

**Consistency Goal**: Both Debian-based images should use the same stable Debian version.

**Current Situation**:

- `debian:trixie-slim` is Debian 13 (current stable)
- `rust:trixie` is built on Debian 13
- Both use the same underlying OS - they are aligned

**Decision Rationale**:

- Trixie is the current stable Debian release
- Using stable vs. unstable ensures security updates
- Aligns with Torrust Tracker's recent update (PR #1629)

### 2. Ubuntu Base Image (Provisioned Instance)

**Purpose**: Simulates a provisioned VM ready for Ansible configuration  
**Current**: Ubuntu 24.04 LTS

**Options**:

- Keep: `ubuntu:24.04` (LTS provides 5 years support until April 2029)
- Update: Check if `ubuntu:24.10` or newer LTS is available

**Decision**: Verify current status and choose based on:

- Support lifecycle
- Stability vs. recency
- Use case fit (E2E testing VM simulation)

### 3. Alpine Base Image (SSH Server)

**Purpose**: Minimal SSH test server for integration testing  
**Current**: `alpine:3.23.3` (Fixed version - unusual for Alpine)

**Note**: Alpine typically uses floating tags (`alpine:3.23`, `alpine:latest`). Fixed versions are uncommon.

**Verification Steps**:

- Check Alpine release schedule
- Determine if 3.23.3 is the latest or if newer patch/minor versions exist
- Consider moving to floating tag (`alpine:3.23`) for automatic security updates
- Verify this works with SSH and test requirements

## Implementation Plan

### Phase 1: Review and Plan (Security Scanning)

**For Each Docker Image**:

1. **Check Latest Release**
   - [ ] Review official image repositories
   - [ ] Identify available stable versions
   - [ ] Check release dates and support timelines

2. **Security Scan Baseline**
   - [ ] Run current Trivy scan (before update)
   - [ ] Document baseline vulnerabilities in `docs/security/docker/scans/`
   - [ ] Record findings

3. **Review Changes**
   - [ ] List breaking changes in release notes
   - [ ] Identify dependency incompatibilities
   - [ ] Plan for testing requirements

**Specific Images to Review**:

#### Image 1: docker/deployer/Dockerfile

- Current: `rust:bookworm`
- Action: Update to `rust:trixie`
- Reason: Consistency with backup image and Torrust Tracker
- Scan: Run Trivy scan with updated image
- Test: Ensure build completes and dependencies resolve

#### Image 2: docker/backup/Dockerfile

- Current: `debian:trixie-slim` ✅
- Action: Verify this is correct
- Status: No changes needed (already current)

#### Image 3: docker/provisioned-instance/Dockerfile

- Current: `ubuntu:24.04`
- Action: Verify if this is the latest or if update needed
- Options: Stay on 24.04 LTS or update
- Scan: Run Trivy scan with current/updated image
- Test: Verify Ansible connectivity and package installation

#### Image 4: docker/ssh-server/Dockerfile

- Current: `alpine:3.23.3`
- Action: Review Alpine release schedule
- Consider: Move to floating tag or update to latest patch
- Scan: Run Trivy scan with updated image
- Test: Verify SSH functionality in integration tests

### Phase 2: Update Dockerfiles

- [ ] Update `docker/deployer/Dockerfile`: Change `rust:bookworm` to `rust:trixie`
- [ ] Update `docker/provisioned-instance/Dockerfile`: Update Ubuntu version if newer stable available
- [ ] Update `docker/ssh-server/Dockerfile`: Update Alpine version if needed
- [ ] Verify `docker/backup/Dockerfile`: Already using `trixie-slim` ✅

### Phase 3: Security Scanning

**For each updated image**:

1. **Build the image locally**

   ```bash
   docker build --tag {image-name}:test docker/{image}/
   ```

2. **Run Trivy scan**

   ```bash
   trivy image --severity HIGH,CRITICAL {image-name}:test
   ```

3. **Compare to baseline**
   - Check if vulnerabilities improved
   - Document any new HIGH/CRITICAL findings
   - If vulnerabilities exist, evaluate if acceptable

4. **Document results**
   - Update scan documentation in `docs/security/docker/scans/`
   - Record date, image version, vulnerability count
   - Note any changes from previous scan

### Phase 4: Testing

- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`
- [ ] Build all Dockerfiles locally
- [ ] Verify tests pass (unit and integration tests)
- [ ] Run E2E deployment workflow if applicable
- [ ] Check no regressions in dependent services

### Phase 5: Documentation and Commit

- [ ] Update `docs/security/docker/scans/` with new results
- [ ] Commit with clear message: `build: update docker base images to trixie/latest stable`
- [ ] Create draft PR for team review

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Base Image Updates**:

- [ ] All Dockerfiles reviewed for outdated base images
- [ ] `docker/deployer/Dockerfile`: Updated from `rust:bookworm` to `rust:trixie`
- [ ] `docker/provisioned-instance/Dockerfile`: Verified current or updated appropriately
- [ ] `docker/ssh-server/Dockerfile`: Verified current or updated appropriately
- [ ] `docker/backup/Dockerfile`: Verified already on `trixie-slim`

**Security Scanning**:

- [ ] Trivy security scan run for each updated image
- [ ] Scan results documented in `docs/security/docker/scans/`
- [ ] HIGH/CRITICAL vulnerabilities reviewed and evaluated
- [ ] Scan comparison (before/after) documented

**Testing**:

- [ ] All Docker images build successfully locally
- [ ] Unit tests pass with new base images
- [ ] Integration tests pass (SSH connectivity, etc.)
- [ ] E2E workflow tests pass (if applicable)
- [ ] No regressions in dependent services

**Documentation**:

- [ ] Security scan results added to `docs/security/docker/scans/`
- [ ] Commit message follows conventional format
- [ ] Links to relevant Torrust Tracker PR (#1629) in commit message

## Related Documentation

- [Docker Security Scanning Guide](../security/docker/README.md)
- [Security Scan Results](../security/docker/scans/README.md)
- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [Torrust Tracker PR #1629](https://github.com/torrust/torrust-tracker/pull/1629) - Similar update in Tracker project
- [Contributing Guide](./README.md)
- [Commit Process](./commit-process.md)

## Notes

### Timeline

- **Debian trixie**: Current stable since June 2024, expected 10-year support until 2034
- **Ubuntu 24.04**: LTS release with 5-year support until April 2029
- **Alpine 3.23**: Released November 2024, community support expected until May 2025

### Related Issues

- [Issue #250: Implement periodic security vulnerability scanning workflow](https://github.com/torrust/torrust-tracker-deployer/issues/250) - Future automated scanning
- [Torrust Tracker PR #1629](https://github.com/torrust/torrust-tracker/pull/1629) - Motivation for this task

### Maintenance Pattern

This is the first in a series of periodic image update reviews. Future updates should:

1. Follow the same process documented here
2. Update this specification with new findings
3. Maintain historical scan results in `docs/security/docker/scans/`
4. Consider automating the security scanning (see Issue #250)
