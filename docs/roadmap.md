# Torrust Tracker Deployer - Roadmap

This document outlines the development roadmap for the Torrust Tracker Deployer project. Each task is marked with:

- `[ ]` - Not completed
- `[x]` - Completed

## Development Process

When starting work on a new feature:

1. Create the feature documentation in the `docs/features/` folder and commit it
2. Open an issue on GitHub linking to the feature folder in the repository
3. Add the new issue as a child issue of the main EPIC issue

> **Note:** See [`docs/features/README.md`](./features/README.md) for detailed conventions and process guide for creating new features.

---

## Roadmap

### 1. Add scaffolding for main app

- [ ] **1.1** Setup logging
- [ ] **1.2** Create command `torrust-tracker-deployer destroy` to destroy an environment
  - It will depend on the environment status. For example, if it has not been provisioned we only have to delete the persistent data.
  - For manual testing we can create envs with the E2E full test (there is an option to keep the infra after testing)
- [ ] **1.3** Refactor extract shared code between testing and production for app bootstrapping
- [ ] **1.4** Improve command to use better abstraction to handle presentation layer
  - types to handle user's views, etc
- [ ] **1.5** Create command `torrust-tracker-deployer create` to create a new environment
  - We need to decide how the user will provide config values.
  - output using println and eprintln
- [ ] **1.6** Create command `torrust-tracker-deployer deploy` to run the Provision and Configuration commands (full deployment)
- [ ] **1.7** Add levels of verbosity as described in the UX research

**Note:** See [`docs/research/UX/`](./research/UX/) for detailed UX research that will be useful to implement the features in this section.

### 2. Add new infrastructure provider: Hetzner

- [ ] **2.1** Add Hetzner provider support

### 3. Continue adding more application commands

**Note:** These are internal app layer commands (like ProvisionCommand or ConfigureCommand), not console commands. The approach is to slice by functional services rather than deployment stages - we fully deploy a working stack from the beginning and incrementally add new services.

- [ ] **3.1** Finish ConfigureCommand
  - Add firewall base configuration
- [ ] **3.2** Add ReleaseCommand and RunCommand with slices

  - [ ] **3.2.1** Run only a docker compose configuration with hello-world docker image
  - [ ] **3.2.2** Add MySQL to docker compose stack
  - [ ] **3.2.3** Add Torrust Tracker to docker compose stack
  - [ ] **3.2.4** Add Prometheus to docker compose stack
  - [ ] **3.2.5** Add Grafana to docker compose stack

  **Notes:**

  - Each service will require new templates, template Rust wrappers, environment options, etc.

### 4. Create a docker image for the deployer

- [ ] **4.1** Create docker image for the deployer to use it without needing to install the dependencies (OpenTofu, Ansible, etc)
  - Mount data and build dirs when running it.

### 5. Add extra console app commands

- [ ] **5.1** `torrust-tracker-deployer status` - Check environment and service status
- [ ] **5.2** `torrust-tracker-deployer test` - Run application tests

**Note:** The test console subcommand is already partially implemented.

### 6. Add HTTPS support

- [ ] **6.1** Add HTTPS support for HTTP tracker
- [ ] **6.2** Add HTTPS support for tracker API
- [ ] **6.3** Add HTTPS support for Grafana

### 7. Add backup and disaster recovery

- [ ] **7.1** Implement database backups for MySQL
- [ ] **7.2** Implement configuration backups
- [ ] **7.3** Create recovery procedures documentation

---

## Notes

- This roadmap will be linked to an EPIC issue on GitHub for tracking progress
- Each major feature should have corresponding documentation in `docs/features/` before implementation begins
