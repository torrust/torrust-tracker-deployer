# GitHub Copilot Agent Configuration

This directory contains documentation for configuring and working with GitHub Copilot coding agent in the Torrust Tracker Deployer project.

## Documents

### [Firewall Configuration](./firewall.md)

Describes the firewall configuration for GitHub Copilot coding agent, including:

- Custom allowlist domains (e.g., `opentofu.org`)
- Recommended allowlist coverage
- Setup instructions for repository administrators

### [Pre-commit Configuration](./pre-commit-config.md)

Explains how to configure environment variables to skip slow tests during Copilot agent pre-commit checks:

- Why this is needed (timeout issues)
- How to set up `TORRUST_TD_SKIP_SLOW_TESTS` environment variable
- Pre-commit timing breakdown (individual tasks and unit test breakdown)
- What gets skipped (E2E tests and coverage checks)
- Testing and verification instructions

## Related Issues

- [GitHub Issue #121](https://github.com/torrust/torrust-tracker-deployer/issues/121) - Install Git Pre-Commit Hooks for Copilot Agent
- [Community Discussion](https://github.com/orgs/community/discussions/178998) - Copilot timeout during long-running pre-commit checks

## References

- [GitHub Copilot Coding Agent Documentation](https://docs.github.com/en/copilot/using-github-copilot/using-copilot-coding-agent-to-work-on-tasks)
- [Customizing Copilot Agent Environment](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-environment)
