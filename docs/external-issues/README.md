# External Issues Documentation

This directory contains documentation about issues found in **external tools and services** during development and testing of the Torrust Tracker Deployer, not issues with the deployer application itself.

## Purpose

When developing and testing the deployer, we sometimes discover issues, limitations, or unexpected behaviors in external tools and services we depend on. This folder documents those findings for reference, workarounds, and potential upstream fixes.

## Subdirectories

### [`github-actions/`](github-actions/)

Issues related to GitHub Actions CI/CD environment:

- Runner limitations
- Network connectivity issues
- Resource constraints
- CI-specific environment problems

### [`tracker/`](tracker/)

Issues related to the Torrust Tracker application and container:

- Tracker container behavior issues
- Tracker configuration problems
- Tracker entrypoint script issues
- Tracker application bugs discovered during deployment
- Tracker API inconsistencies
- Tracker database driver issues

## Scope

**IN SCOPE** (belongs here):

- External tool bugs or limitations
- Third-party service issues
- Dependency behavior that affects the deployer
- Upstream issues requiring workarounds
- External API inconsistencies

**OUT OF SCOPE** (belongs elsewhere):

- Deployer bugs → GitHub Issues in this repository
- Infrastructure code issues → Code fixes in `src/`
- Template rendering problems → Code fixes in `src/infrastructure/templating/`
- Ansible playbook issues → Code fixes in `templates/ansible/`
- Development workflow issues → `docs/contributing/known-issues.md`

## Document Format

Each issue should be documented with:

1. **Problem Description**: Clear explanation of the issue
2. **Root Cause**: Analysis of why it happens
3. **Impact**: How it affects the deployer
4. **Current Workaround**: How we handle it in the deployer
5. **Recommended Solution**: Proposed fix for the upstream project
6. **References**: Links to relevant external documentation or issues

## Related Documentation

- [Contributing Guidelines](../contributing/README.md)
- [Known Issues](../contributing/known-issues.md) (deployer-specific issues)
- [Documentation Index](../README.md)
