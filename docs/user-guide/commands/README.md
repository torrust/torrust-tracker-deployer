# Commands Guide

This directory contains detailed guides for all Torrust Tracker Deployer commands.

## Available Commands

### Environment Creation

- **[create](create.md)** - Create environments and generate configuration templates
  - `create template` - Generate environment configuration template
  - `create environment` - Create deployment environment from configuration

### Infrastructure Management

- **[provision](provision.md)** - Provision VM infrastructure
- **[configure](configure.md)** - Configure provisioned infrastructure
- **[test](test.md)** - Verify deployment infrastructure

### Environment Cleanup

- **[destroy](destroy.md)** - Destroy deployment environment

## Command Workflow

The typical command sequence for a complete deployment:

```text
1. create template    → Generate configuration template
2. (edit template)    → Customize your settings
3. create environment → Create environment from config
4. provision          → Provision VM infrastructure
5. configure          → Install Docker, Docker Compose
6. test               → Verify infrastructure readiness
7. (deploy app)       → Deploy Torrust Tracker (coming soon)
8. destroy            → Clean up when done
```

## Command Categories

### Plumbing Commands (Low-Level)

These commands provide fine-grained control over each deployment step:

- `create template` / `create environment`
- `provision`
- `configure`
- `test`
- `destroy`

**Best for**: CI/CD pipelines, automation, advanced users, debugging

### Porcelain Commands (High-Level) - Coming Soon

Simplified commands that orchestrate multiple plumbing commands:

- `deploy` - Intelligent orchestration from current state to running

**Best for**: Quick deployments, beginners, interactive use

## Quick Reference

| Command              | State Transition         | Description              |
| -------------------- | ------------------------ | ------------------------ |
| `create template`    | N/A → Template           | Generate config template |
| `create environment` | Template → Created       | Create environment       |
| `provision`          | Created → Provisioned    | Provision infrastructure |
| `configure`          | Provisioned → Configured | Install software         |
| `test`               | (validation only)        | Verify infrastructure    |
| `destroy`            | Any → Destroyed          | Clean up resources       |

## Getting Started

If you're new to the Torrust Tracker Deployer, we recommend:

1. Start with the **[Quick Start Guide](../quick-start.md)** for a complete walkthrough
2. Use the individual command guides for detailed information
3. Check the **[Console Commands Reference](../../console-commands.md)** for technical details

## See Also

- [Quick Start Guide](../quick-start.md) - Complete deployment walkthrough
- [Console Commands Reference](../../console-commands.md) - Technical command reference
- [Codebase Architecture](../../codebase-architecture.md) - Understanding the internals
