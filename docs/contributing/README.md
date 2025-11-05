# Contributing Guide

This guide will help you understand our development practices and contribution workflow.

## 📋 Quick Reference

| Topic                                | File                                                         |
| ------------------------------------ | ------------------------------------------------------------ |
| DDD layer placement (architecture)   | [ddd-layer-placement.md](./ddd-layer-placement.md)           |
| PR review guide for reviewers        | [pr-review-guide.md](./pr-review-guide.md)                   |
| Creating roadmap issues              | [roadmap-issues.md](./roadmap-issues.md)                     |
| Branching conventions                | [branching.md](./branching.md)                               |
| Commit process and pre-commit checks | [commit-process.md](./commit-process.md)                     |
| Code quality and linting             | [linting.md](./linting.md)                                   |
| Module organization and imports      | [module-organization.md](./module-organization.md)           |
| Error handling principles            | [error-handling.md](./error-handling.md)                     |
| Working with Tera templates          | [templates.md](./templates.md)                               |
| Debugging techniques                 | [debugging.md](./debugging.md)                               |
| Spell checking and dictionaries      | [spelling.md](./spelling.md)                                 |
| Known issues and expected behaviors  | [known-issues.md](./known-issues.md)                         |
| Logging best practices               | [logging-guide.md](./logging-guide.md)                       |
| GitHub Markdown pitfalls             | [github-markdown-pitfalls.md](./github-markdown-pitfalls.md) |
| Testing conventions and practices    | [testing/](./testing/)                                       |

## 🚀 Getting Started

1. **Fork and clone** the repository
2. **Install dependencies** using the automated installer:

   ```bash
   cargo run --bin dependency-installer install
   ```

   See [Dependency Installer](../../packages/dependency-installer/README.md) for details.

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
