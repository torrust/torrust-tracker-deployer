# Decision: Exit Code Zero for Docker Security Scanning

## Status

Accepted

## Date

2025-12-23

## Context

When implementing automated Docker vulnerability scanning with Trivy in GitHub Actions, we faced a critical decision about how the CI/CD pipeline should respond to discovered vulnerabilities.

Traditional approaches make CI fail when vulnerabilities are found, blocking all development until issues are resolved. However, this creates several problems:

1. **False Positives**: Security scanners can report issues that don't apply to our context or are accepted risks
2. **Third-Party Dependencies**: We cannot immediately fix vulnerabilities in upstream images (mysql, prometheus, grafana)
3. **Scanner Quirks**: Trivy occasionally exits with code 1 even when no vulnerabilities are found
4. **Development Flow**: Security findings should not block unrelated development work
5. **Policy Enforcement**: Security decisions should be made by security teams, not automated tooling
6. **Partial Data Loss**: If CI fails early, later scans never run and we lose visibility into other images

The initial implementation used `exit-code: "1"` which caused the workflow to fail on any HIGH or CRITICAL vulnerability, including when scanning third-party production images with known CVEs that we cannot immediately fix.

## Decision

Implement a **security-first philosophy** where:

1. **Exit Code Zero Everywhere**: All Trivy scan steps use `exit-code: "0"` - the scanner never fails the CI pipeline
2. **Dual Output Strategy**:
   - Human-readable table format in workflow logs for immediate visibility
   - SARIF format uploaded to GitHub Security tab for tracking and alerting
3. **Separation of Concerns**:
   - Trivy's role: **Detect** vulnerabilities and provide data
   - GitHub Security's role: **Decide** enforcement policies and alert routing
   - CI's role: **Stay green** and maintain development velocity
4. **Always Run Policy**: Upload job uses `if: always()` to ensure partial results are never lost
5. **Unique Categories**: Each image gets a unique SARIF category for proper alert tracking and deduplication
6. **Scheduled Scanning**: Daily cron ensures continuous monitoring without blocking code changes

This philosophy is summarized as: **"Trivy detects, GitHub Security decides, CI stays green"**

## Consequences

### Positive

- **No False Failures**: Development work never blocked by scanner quirks or edge cases
- **Continuous Visibility**: All scans complete even if one fails, providing complete security picture
- **Flexible Enforcement**: Security team can configure GitHub Security policies without changing code
- **Third-Party Tolerance**: Known vulnerabilities in upstream images don't block development
- **Developer Experience**: Green builds maintain team velocity while security team reviews findings
- **Policy Separation**: Security enforcement decoupled from CI/CD implementation
- **Audit Trail**: All findings recorded in GitHub Security tab for compliance and tracking
- **Incremental Improvement**: Can address vulnerabilities based on priority without CI pressure

### Negative

- **Potential Complacency**: Green CI might lead to ignoring security findings (mitigated by GitHub Security alerts)
- **Requires Monitoring**: Security team must actively monitor GitHub Security tab
- **Policy Configuration**: Requires additional GitHub Security policy setup for enforcement
- **Learning Curve**: Non-traditional approach may confuse developers expecting red builds for vulnerabilities

### Risks Introduced

- **Missed Critical Issues**: If GitHub Security is not properly configured or monitored, critical vulnerabilities might go unaddressed
  - **Mitigation**: Daily scheduled scans ensure consistent monitoring; GitHub Security sends email notifications
- **Organizational Resistance**: Some organizations mandate CI failure on security issues
  - **Mitigation**: GitHub Security can be configured to block PRs or deployments if needed

## Alternatives Considered

### 1. Exit Code 1 (Fail on Vulnerabilities)

**Approach**: Use `exit-code: "1"` to fail CI when HIGH/CRITICAL vulnerabilities are found.

**Rejected Because**:

- Blocks development on third-party image vulnerabilities we cannot fix immediately
- Scanner quirks cause false CI failures even with zero vulnerabilities
- No flexibility for security team to make risk-based decisions
- Partial data loss when early scans fail

### 2. Mixed Exit Codes (Project vs Third-Party)

**Approach**: Use `exit-code: "1"` for project images but `exit-code: "0"` for third-party images.

**Rejected Because**:

- Inconsistent philosophy creates confusion
- Project images can have legitimate accepted risks
- Still susceptible to scanner quirks on project images
- Doesn't solve the fundamental policy enforcement problem

### 3. Continue-on-Error Pattern

**Approach**: Use `exit-code: "1"` but add `continue-on-error: true` to allow workflow to proceed.

**Rejected Because**:

- Shows misleading "failed" status even though workflow continues
- Scanner errors appear as failures in UI, creating noise
- Doesn't fundamentally change the enforcement model
- Confusing to developers seeing "failed" steps that don't actually fail

### 4. CodeQL Action with Single Category

**Approach**: Upload all SARIF files using github/codeql-action/upload-sarif with same category.

**Rejected Because**:

- CodeQL Action rejects multiple SARIF uploads with identical categories (as of July 2025)
- Results in "multiple SARIF runs with same category" error
- Cannot distinguish alerts between different images

## Related Decisions

- [GitHub Actions Workflow Structure](https://github.com/torrust/torrust-tracker-deployer/pull/256) - How the three-job structure enables this philosophy
- Future: Security Policy Configuration (to be documented when GitHub Security policies are configured)

## References

- [Issue #251: Implement basic Trivy scanning workflow](https://github.com/torrust/torrust-tracker-deployer/issues/251)
- [Pull Request #256: Implement Basic Trivy Scanning Workflow](https://github.com/torrust/torrust-tracker-deployer/pull/256)
- [Trivy Action Documentation](https://github.com/aquasecurity/trivy-action)
- [GitHub Code Scanning Documentation](https://docs.github.com/en/code-security/code-scanning)
- [GitHub Security Policy Enforcement](https://docs.github.com/en/code-security/code-scanning/managing-code-scanning-alerts)
- [Security-First Philosophy Discussion](https://github.com/torrust/torrust-tracker-deployer/pull/256#discussion) - External review recommending exit-code 0 approach
