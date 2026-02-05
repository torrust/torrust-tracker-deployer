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

### Process: One Image at a Time

This issue will be implemented systematically, updating and testing one Docker image at a time.

### Latest Versions (Verified Feb 5, 2026)

Sources: https://hub.docker.com/_/rust, https://hub.docker.com/_/debian, https://hub.docker.com/_/alpine, https://hub.docker.com/_/ubuntu

| Image                | Current            | Latest             | Update |
| -------------------- | ------------------ | ------------------ | ------ |
| deployer             | rust:bookworm      | rust:trixie        | YES    |
| backup               | debian:trixie-slim | debian:trixie-slim | No     |
| ssh-server           | alpine:3.23.3      | alpine:3.23.3      | No     |
| provisioned-instance | ubuntu:24.04       | ubuntu:24.04       | No     |

### Update Progress

#### ① Deployer: `rust:bookworm` → `rust:trixie` (Priority)

**File**: `docker/deployer/Dockerfile` line 31  
**Status**: ✅ **COMPLETED**

**Tasks completed**:

- [x] Update FROM line: Changed `rust:bookworm` to `rust:trixie`
- [x] Build image locally: Successfully built `docker build --tag deployer:test docker/deployer/`
- [x] Run security scan: Trivy scan completed (1 HIGH - existing Ansible private key, no new vulnerabilities)
- [x] Run linter: All linters passed (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- [x] Run tests: All 416 unit and integration tests passed
- [x] Commit: Committed with message `build: [#317] update deployer docker base image from rust:bookworm to rust:trixie`

#### ② Backup, SSH Server, Provisioned Instance

**Status**: ✅ Already on latest versions (no updates needed)  
**No action required** - confirmed current with official sources

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Base Image Updates**:

- [x] `docker/deployer/Dockerfile`: Updated from `rust:bookworm` to `rust:trixie` (line 31) ✅ **COMPLETED**
- [x] All other Dockerfiles verified as current (backup, ssh-server, provisioned-instance)

**Security Scanning**:

- [x] Trivy security scan run for updated deployer image
- [x] Scan results document no new HIGH/CRITICAL vulnerabilities
- [x] Scan comparison (before deployer change/after) documented

**Testing**:

- [x] Deployer Docker image builds successfully locally
- [x] Unit and integration tests pass with updated base image
- [x] No regressions in dependent services

**Documentation**:

- [x] Commit message follows conventional format
- [x] Links to Torrust Tracker PR (#1629) included in commit

## Related Documentation

- [Docker Security Scanning Guide](../security/docker/README.md)
- [Docker Security Scan Results](../security/docker/scans/README.md)
- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [Torrust Tracker PR #1629](https://github.com/torrust/torrust-tracker/pull/1629) - Motivation for this task
- [Commit Process](./commit-process.md)

## Notes

### Timeline

- **Debian trixie**: Current stable since June 2024, expected 10-year support until 2034
- **Rust**: Latest stable versions track Debian releases; trixie variant includes Rust 1.93.0
- **Ubuntu 24.04**: LTS release with 5-year support until April 2029
- **Alpine 3.23**: Released November 2024, community support expected until May 2025

### Related Issues

- [Issue #250: Implement periodic security vulnerability scanning workflow](https://github.com/torrust/torrust-tracker-deployer/issues/250) - Future automated scanning
- [Torrust Tracker PR #1629](https://github.com/torrust/torrust-tracker/pull/1629) - Motivation for this task

### Maintenance Pattern

This is the first in a series of periodic image update reviews. Future updates should:

1. Verify latest versions from official Docker Hub sources
2. Update only the images that have newer versions available
3. Process one image at a time
4. Document scan results for historical reference
5. Consider automating the security scanning (see Issue #250)
