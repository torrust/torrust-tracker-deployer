# Tracker Issues Documentation

This directory contains documentation about issues found in the **Torrust Tracker** itself during deployment testing, not issues with this deployer application.

## Purpose

When testing the deployer, we sometimes discover issues, limitations, or unexpected behaviors in the Torrust Tracker container or application. This folder documents those findings for reference and potential upstream fixes in the tracker repository.

## Scope

**IN SCOPE** (belongs here):

- Tracker container behavior issues
- Tracker configuration problems
- Tracker entrypoint script issues
- Tracker application bugs discovered during deployment
- Tracker API inconsistencies
- Tracker database driver issues

**OUT OF SCOPE** (belongs elsewhere):

- Deployer bugs → GitHub Issues in this repository
- Infrastructure provisioning issues → `docs/external-issues/github-actions/` or `docs/contributing/known-issues.md`
- Template rendering problems → Code fixes in `src/infrastructure/templating/`
- Ansible playbook issues → Code fixes in `templates/ansible/`

## Document Format

Each issue should be documented with:

1. **Problem Description**: Clear explanation of the issue
2. **Root Cause**: Analysis of why it happens
3. **Impact**: How it affects deployments
4. **Current Workaround**: How we handle it in the deployer
5. **Recommended Solution**: Proposed fix for the tracker repository
6. **References**: Links to relevant tracker repo files or issues

## Example Issues

- `database-driver-double-specification.md` - Tracker requires database driver in both config file and environment variable

## Related Documentation

- [Torrust Tracker Repository](https://github.com/torrust/torrust-tracker)
- [Torrust Tracker Container Images](https://hub.docker.com/r/torrust/tracker)
- [External Issues Overview](../README.md)
- [Contributing Guidelines](../../contributing/README.md)
