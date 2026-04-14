[![Linting](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/linting.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/linting.yml)
[![Testing](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/testing.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/testing.yml)
[![E2E Infrastructure Tests](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-infrastructure.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-infrastructure.yml)
[![E2E Deployment Tests](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-deployment.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-deployment.yml)
[![Coverage](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/coverage.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/coverage.yml)
[![SDK Examples Tests](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-sdk-examples.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-sdk-examples.yml)
[![Test LXD Container Provisioning](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-lxd-provision.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-lxd-provision.yml)
[![Test Dependency Installer](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-dependency-installer.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-dependency-installer.yml)
[![Cargo Security Audit](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/cargo-security-audit.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/cargo-security-audit.yml)
[![Docker Security Scan](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/docker-security-scan.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/docker-security-scan.yml)
[![Container](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/container.yaml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/container.yaml)
[![Publish Crate](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/publish-crate.yaml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/publish-crate.yaml)
[![Backup Container](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/backup-container.yaml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/backup-container.yaml)
[![Code Statistics](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/code-statistics.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/code-statistics.yml)

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/torrust/torrust-tracker-deployer?quickstart=1)

# Torrust Tracker Deployer

Deployment automation for [Torrust Tracker](https://github.com/torrust/torrust-tracker) environments using [OpenTofu](https://opentofu.org), [Ansible](https://www.ansible.com), and [Rust](https://www.rust-lang.org).

## Release Status

Version 0.1.0 is the first fully functional release line of the deployer.

Current status:

- End-to-end workflow is implemented: create, provision, configure, test, release, run, destroy
- Multi-provider architecture is implemented
- Providers currently supported: LXD (local development) and Hetzner Cloud (cloud deployments)
- CI includes linting, unit/integration tests, and split E2E workflows

## What This Project Does

The deployer provisions and configures VM infrastructure, then deploys and runs the Torrust Tracker stack.

Workflow:

1. OpenTofu provisions infrastructure and cloud-init setup.
2. Ansible configures the provisioned host.
3. The deployer releases tracker artifacts and starts services.
4. Built-in commands support verification and teardown.

## Quick Start

### 1. Install dependencies

Recommended (automatic dependency installer):

```bash
cargo run --bin dependency-installer install
cargo run --bin dependency-installer check
```

### 2. Build and run the CLI

```bash
cargo run
```

### 3. Create and deploy an environment

```bash
# Generate environment config template
cargo run -- create template my-env.json

# Edit config values, then create the environment from the file
cargo run -- create environment --env-file my-env.json

# Provision and configure
cargo run -- provision my-environment
cargo run -- configure my-environment

# Verify and deploy application
cargo run -- test my-environment
cargo run -- release my-environment
cargo run -- run my-environment

# Tear down when done
cargo run -- destroy my-environment
```

Important:

- Keep your environment JSON files in [`envs/`](envs/).
- The [`data/`](data/) directory is application-managed deployment state and should not be edited manually.

## Docker Usage

For cloud-provider deployments, you can run the deployer via container image:

```bash
docker pull torrust/tracker-deployer:latest

docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  --help
```

Note: Docker workflow supports cloud providers. For local LXD usage, run the deployer natively on the host.

## Development Commands

```bash
# Comprehensive linting
cargo run --bin linter all

# Run test suite
cargo test

# E2E suites
cargo run --bin e2e-infrastructure-lifecycle-tests
cargo run --bin e2e-deployment-workflow-tests

# Full E2E workflow (local only)
cargo run --bin e2e-complete-workflow-tests
```

## Documentation Map

For detailed guides, use the docs index and user guide:

- [Documentation Index](docs/README.md)
- [User Guide](docs/user-guide/README.md)
- [Quick Start Guides](docs/user-guide/quick-start/README.md)
- [Commands](docs/user-guide/commands/)
- [Providers](docs/user-guide/providers/README.md)
- [E2E Testing](docs/e2e-testing/README.md)
- [Contributing](docs/contributing/README.md)
- [Architecture Overview](docs/codebase-architecture.md)
- [Roadmap](docs/roadmap.md)

## Related Repository

The deployer focuses on provisioning, configuring, and deploying Torrust Tracker.
For example application operations and maintenance after deployment, see:

- [Torrust Tracker Demo](https://github.com/torrust/torrust-tracker-demo)

## Repository Layout

Top-level directories:

- [`src/`](src/): Rust codebase using DDD layers (domain, application, infrastructure, presentation)
- [`templates/`](templates/): OpenTofu and Ansible templates
- [`docs/`](docs/README.md): user and contributor documentation
- [`envs/`](envs/): user environment configuration files (git-ignored)
- `build/`: generated runtime files (git-ignored)
- [`data/`](data/): application-managed deployment state

## Roadmap After 0.1.0

The 0.1.0 line establishes the functional baseline. Upcoming improvements are tracked in the roadmap, including broader provider support and deployment UX refinements.

See: [Roadmap](docs/roadmap.md)

## License

This project is licensed under the [MIT License](LICENSE).
