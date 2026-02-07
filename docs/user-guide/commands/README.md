# Commands Guide

This directory contains detailed guides for all Torrust Tracker Deployer commands.

## Available Commands

### Environment Creation

- **[create](create.md)** - Create environments and generate configuration templates
  - `create template` - Generate environment configuration template
  - `create environment` - Create deployment environment from configuration

### Configuration Validation

- **[validate](validate.md)** - Validate environment configuration files without deployment

### Environment Information

- **[show](show.md)** - Display environment information with state-aware details

### Infrastructure Management

- **[provision](provision.md)** - Provision VM infrastructure
- **[register](register.md)** - Register existing infrastructure (alternative to provision)
- **[configure](configure.md)** - Configure provisioned infrastructure
- **[test](test.md)** - Verify deployment infrastructure

### Application Deployment

- **[release](release.md)** - Deploy application configuration and files
- **[run](run.md)** - Start Torrust Tracker services

### Environment Cleanup

- **[destroy](destroy.md)** - Destroy deployment infrastructure
- **[purge](purge.md)** - Remove local environment data and free up names

## Command Workflow

The typical command sequence for a complete deployment:

```text
1. create template    → Generate configuration template
2. (edit template)    → Customize your settings
3. validate           → Verify configuration is correct (optional but recommended)
4. create environment → Create environment from config
5. show               → View environment details
6. provision          → Provision VM infrastructure
7. configure          → Install Docker, Docker Compose, configure firewall
8. test               → Verify infrastructure readiness
9. release            → Deploy application configuration and files
10. run               → Start Torrust Tracker services
11. destroy           → Tear down infrastructure
12. purge             → Remove local data (optional, for cleanup)
```

## Command Categories

### Plumbing Commands (Low-Level)

These commands provide fine-grained control over each deployment step:

- `create template` / `create environment`
- `show`
- `provision` / `register`
- `configure`
- `test`
- `release`
- `run`
- `purge`
- `destroy`

**Best for**: CI/CD pipelines, automation, advanced users, debugging

### Porcelain Commands (High-Level) - Coming Soon

Simplified commands that orchestrate multiple plumbing commands:

- `deploy` - Intelligent orchestration from current state to running

**Best for**: Quick deployments, beginners, interactive use

## State Transitions

| Command              | State Transition         | Description                |
| -------------------- | ------------------------ | -------------------------- |
| `create template`    | N/A → Template           | Generate config template   |
| `create environment` | Template → Created       | Create environment         |
| `show`               | (read-only)              | Display environment info   |
| `provision`          | Created → Provisioned    | Provision infrastructure   |
| `register`           | Created → Provisioned    | Register existing infra    |
| `configure`          | Provisioned → Configured | Install software, firewall |
| `test`               | (validation only)        | Verify infrastructure      |
| `release`            | Configured → Released    | Deploy application files   |
| `run`                | Released → Running       | Tear down infrastructure   |
| `purge`              | Any → (removed)          | Remove local data ices     |
| `destroy`            | Any → Destroyed          | Clean up resources         |

## Getting Started

If you're new to the Torrust Tracker Deployer, we recommend:

1. Start with the **[Quick Start Guides](../quick-start/README.md)** for Docker or native installation
2. Use the individual command guides for detailed information
3. Check the **[Console Commands Reference](../../console-commands.md)** for technical details

## See Also

- [Quick Start Guides](../quick-start/README.md) - Docker and native installation guides
- [Console Commands Reference](../../console-commands.md) - Technical command reference
- [Codebase Architecture](../../codebase-architecture.md) - Understanding the internals
