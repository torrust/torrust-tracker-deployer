# GitHub Actions Runner Issues

This directory documents recurring issues encountered when running our CI/CD workflows on GitHub Actions runners, particularly those related to networking, package management, and containerization within VMs or containers.

## 📋 Issue Categories

### Package Management Issues

- [Docker Installation APT Cache Issues](./docker-apt-cache-issue.md) - Package not available errors due to stale apt cache

## 📄 Templates

- [Issue Template](./issue-template.md) - Standard template for documenting new GitHub Actions issues

## 🔍 Troubleshooting Guide

When encountering issues in GitHub Actions runners:

1. **Check package availability**: Ensure required packages are available in the runner's repositories
2. **Force cache updates**: GitHub Actions containers may have stale package caches
3. **Add debugging output**: Use ansible debug tasks or shell commands to investigate the environment
4. **Test locally first**: Try to reproduce the issue in a local container or VM

## 📚 Contributing

When documenting a new issue:

1. Create a new markdown file with a descriptive name
2. Include the complete error output
3. Describe the root cause analysis
4. Document the solution with code changes
5. Include the commit hash where the fix was applied
6. Update this README to link to the new issue

## 🏷️ Issue Template

Use the [issue template](./issue-template.md) as a starting point when documenting new GitHub Actions issues. Copy the template content to create a new issue documentation file.
