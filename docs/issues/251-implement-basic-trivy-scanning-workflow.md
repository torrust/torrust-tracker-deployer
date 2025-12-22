# Implement Basic Trivy Scanning Workflow with Hardcoded Images

**Issue**: #251
**Parent Epic**: #250 - Implement Automated Docker Image Vulnerability Scanning
**Related**:

- Epic specification: `docs/issues/250-epic-docker-image-vulnerability-scanning.md`
- Sibling issue: #252 - Implement Dynamic Image Detection for Scanning
- Trivy documentation: https://github.com/aquasecurity/trivy

## Overview

Implement a GitHub Actions workflow that uses Trivy to scan Docker images for vulnerabilities. This initial implementation uses a hardcoded list of images and provides immediate security coverage while Phase 2 makes it dynamic.

## Goals

- [ ] Create GitHub Actions workflow for Trivy scanning
- [ ] Scan all project Docker images for HIGH/CRITICAL vulnerabilities
- [ ] Run scans on push, PR, and periodically
- [ ] Fail builds when vulnerabilities detected
- [ ] Generate actionable vulnerability reports

## 🏗️ Architecture Requirements

**Layer**: Infrastructure (CI/CD)
**Module Path**: `.github/workflows/`
**Pattern**: GitHub Actions workflow

### Module Structure Requirements

- [ ] Create new workflow file: `.github/workflows/docker-security-scan.yml`
- [ ] Follow existing GitHub Actions patterns in the repository
- [ ] Use official Trivy action from aquasecurity

### Architectural Constraints

- [ ] Workflow must be idempotent and stateless
- [ ] Fail fast on HIGH/CRITICAL vulnerabilities
- [ ] Clear error messages for developers
- [ ] Minimal dependencies (use official actions only)

### Anti-Patterns to Avoid

- ❌ Scanning images that don't exist or aren't built yet
- ❌ Ignoring scan failures (must exit with code 1)
- ❌ Complex vulnerability filtering (keep it simple: HIGH/CRITICAL only)
- ❌ Scanning third-party images we don't control

## Specifications

### Docker Images to Scan

**Project-Built Images** (require building):

1. `torrust-tracker-deployer/provisioned-instance` (from `docker/provisioned-instance/Dockerfile`)
2. `torrust-tracker-deployer/ssh-server` (from `docker/ssh-server/Dockerfile`)

**Third-Party Images** (from Docker Compose template `templates/docker-compose/docker-compose.yml.tera`):

1. `torrust/tracker:develop`
2. `mysql:8.0`
3. `grafana/grafana:11.4.0`
4. `prom/prometheus:v3.0.1`

### Trivy Command Template

```bash
trivy image \
  --severity HIGH,CRITICAL \
  --exit-code 1 \
  <image-name>
```

**Parameters**:

- `--severity HIGH,CRITICAL` - Only report serious vulnerabilities
- `--exit-code 1` - Fail the workflow if vulnerabilities found
- No `--ignore-unfixed` - Report all vulnerabilities (can be added later if too noisy)

### Workflow Triggers

**On Push**:

```yaml
on:
  push:
    branches:
      - main
      - develop
    paths:
      - "docker/**"
      - "templates/docker-compose/**"
      - ".github/workflows/docker-security-scan.yml"
```

**On Pull Request**:

```yaml
on:
  pull_request:
    paths:
      - "docker/**"
      - "templates/docker-compose/**"
      - ".github/workflows/docker-security-scan.yml"
```

**Periodic Schedule** (daily at 6 AM UTC):

```yaml
on:
  schedule:
    - cron: "0 6 * * *"
```

### Workflow Structure

```yaml
name: Docker Security Scan

on:
  push:
    branches: [main, develop]
    paths:
      - "docker/**"
      - "templates/docker-compose/**"
      - ".github/workflows/docker-security-scan.yml"
  pull_request:
    paths:
      - "docker/**"
      - "templates/docker-compose/**"
      - ".github/workflows/docker-security-scan.yml"
  schedule:
    - cron: "0 6 * * *" # Daily at 6 AM UTC

jobs:
  scan-project-images:
    name: Scan Project-Built Docker Images
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        image:
          - dockerfile: docker/provisioned-instance/Dockerfile
            context: docker/provisioned-instance
            name: provisioned-instance
          - dockerfile: docker/ssh-server/Dockerfile
            context: docker/ssh-server
            name: ssh-server
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Build Docker image
        run: |
          docker build -t torrust-tracker-deployer/${{ matrix.image.name }}:latest \
            -f ${{ matrix.image.dockerfile }} \
            ${{ matrix.image.context }}

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: torrust-tracker-deployer/${{ matrix.image.name }}:latest
          format: "sarif"
          output: "trivy-results-${{ matrix.image.name }}.sarif"
          severity: "HIGH,CRITICAL"
          exit-code: "1"

      - name: Upload Trivy results to GitHub Security
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: "trivy-results-${{ matrix.image.name }}.sarif"

  scan-third-party-images:
    name: Scan Third-Party Docker Images
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        image:
          - torrust/tracker:develop
          - mysql:8.0
          - grafana/grafana:11.4.0
          - prom/prometheus:v3.0.1
    steps:
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ matrix.image }}
          format: "sarif"
          output: "trivy-results-${{ matrix.image }}.sarif"
          severity: "HIGH,CRITICAL"
          exit-code: "1"

      - name: Upload Trivy results to GitHub Security
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: "trivy-results-${{ matrix.image }}.sarif"
```

### Output Examples

**Success (no vulnerabilities)**:

```text
✓ Scanning image: torrust/tracker:develop
✓ No HIGH or CRITICAL vulnerabilities found
```

**Failure (vulnerabilities found)**:

```text
✗ Scanning image: mysql:8.0
✗ Found 3 HIGH and 1 CRITICAL vulnerabilities:

CVE-2024-XXXXX (CRITICAL)
Package: libssl1.1
Installed Version: 1.1.1f-1ubuntu2.20
Fixed Version: 1.1.1f-1ubuntu2.21
```

## Implementation Plan

### Phase 1: Create Workflow File (1 hour)

- [ ] Create `.github/workflows/docker-security-scan.yml`
- [ ] Define workflow triggers (push, PR, schedule)
- [ ] Set up matrix strategy for images
- [ ] Add Trivy action configuration

### Phase 2: Configure Project-Built Images (30 minutes)

- [ ] Add job for building and scanning provisioned-instance image
- [ ] Add job for building and scanning ssh-server image
- [ ] Configure build contexts and Dockerfiles
- [ ] Test local builds work correctly

### Phase 3: Configure Third-Party Images (30 minutes)

- [ ] Add job for scanning torrust/tracker:develop
- [ ] Add job for scanning mysql:8.0
- [ ] Add job for scanning grafana/grafana:11.4.0
- [ ] Add job for scanning prom/prometheus:v3.0.1

### Phase 4: Add Result Upload (30 minutes)

- [ ] Configure SARIF output format
- [ ] Add GitHub Security upload step
- [ ] Configure `if: always()` to upload even on failure
- [ ] Test results appear in GitHub Security tab

### Phase 5: Testing and Documentation (1 hour)

- [ ] Test workflow on feature branch
- [ ] Verify failures block PR merges
- [ ] Verify periodic scans execute
- [ ] Add workflow badge to README
- [ ] Document in README or docs/
- [ ] Add troubleshooting guide

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Workflow file passes YAML linting

**Functional Requirements**:

- [ ] Workflow scans all 6 images (2 project-built + 4 third-party)
- [ ] Workflow triggers on push to main/develop
- [ ] Workflow triggers on PR with relevant file changes
- [ ] Workflow runs daily at 6 AM UTC
- [ ] Build fails when HIGH/CRITICAL vulnerabilities found
- [ ] Results uploaded to GitHub Security tab

**Testing**:

- [ ] Workflow tested on feature branch
- [ ] Verified workflow detects known vulnerabilities (if any exist)
- [ ] Verified workflow passes when no vulnerabilities present
- [ ] Periodic schedule configured correctly (cron syntax)

**Documentation**:

- [ ] Workflow badge added to README (e.g., `[![Docker Security Scan](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/docker-security-scan.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/docker-security-scan.yml)`)
- [ ] README or docs updated with security scanning information
- [ ] Workflow comments explain each step
- [ ] Troubleshooting guide for common issues

## Related Documentation

- GitHub Actions: https://docs.github.com/en/actions
- Trivy Action: https://github.com/aquasecurity/trivy-action
- GitHub Security: https://docs.github.com/en/code-security
- SARIF format: https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning
- Existing workflows: `.github/workflows/`

## Notes

### Known Limitations

1. **Hardcoded Image List**: Images must be manually updated in workflow (Phase 2 addresses this)
2. **Version Pinning**: Using `:latest` and `:develop` tags means scans can vary (intentional for now)
3. **Third-Party Control**: We can't fix vulnerabilities in third-party images (only upgrade versions)

### Future Enhancements (Phase 2)

- Dynamic image detection from environment configuration
- Integration with `show` command to list images
- Automatic workflow updates when images change

### Troubleshooting

**Q: What if a third-party image has unfixable vulnerabilities?**
A: Options:

1. Add `--ignore-unfixed` flag to skip vulnerabilities without patches
2. Use `.trivyignore` file to suppress specific CVEs with justification
3. Find alternative image with better security posture

**Q: Should we block merges on third-party vulnerabilities?**
A: Initially yes (Phase 1). In Phase 2, we can make this configurable or only block on project-built images.

**Q: How often should periodic scans run?**
A: Daily is reasonable. Weekly may be sufficient, but daily ensures faster detection.
