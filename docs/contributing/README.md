# Contributing Guide

This guide will help you understand our development practices and contribution workflow.

## 📋 Quick Reference

| Topic                                | File                                     |
| ------------------------------------ | ---------------------------------------- |
| Branching conventions                | [branching.md](./branching.md)           |
| Commit process and pre-commit checks | [commit-process.md](./commit-process.md) |
| Code quality and linting             | [linting.md](./linting.md)               |

## 🚀 Getting Started

1. **Fork and clone** the repository
2. **Set up your development environment** following the main [README](../../README.md)
3. **Read the branching** guidelines in [branching.md](./branching.md)
4. **Install and run linters** as described in [linting.md](./linting.md)
5. **Follow the commit process** outlined in [commit-process.md](./commit-process.md)

## 🔧 Development Workflow Summary

```bash
# 1. Create a feature branch (use issue number)
git checkout -b 42-add-your-feature-name

# 2. Make your changes
# ... edit files ...

# 3. Run pre-commit verification script
./scripts/pre-commit.sh

# 4. Commit with conventional format (include issue number)
git add .
git commit -m "feat: [#42] add new testing feature"

# 5. Push and create PR
git push origin 42-add-your-feature-name
```

## 📖 Additional Resources

- [Main Documentation](../documentation.md) - Project documentation organization
- [E2E Testing Guide](../e2e-testing.md) - End-to-end testing setup and usage
- [Linting Guide](../linting.md) - Detailed linting setup and usage
- [Tech Stack](../tech-stack/) - Technology-specific documentation
- [Architecture Decisions](../decisions/) - Decision records and rationale

## 🤝 Getting Help

- **Issues**: Check existing [GitHub issues](https://github.com/torrust/torrust-tracker-deployer/issues)
- **Discussions**: Start a [GitHub discussion](https://github.com/torrust/torrust-tracker-deployer/discussions)
- **Documentation**: Review the [docs folder](../) for detailed information

Thank you for contributing! 🎉
