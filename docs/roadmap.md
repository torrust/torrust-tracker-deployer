# Torrust Tracker Deployer - Roadmap

**GitHub Issue**: [#1 - Roadmap](https://github.com/torrust/torrust-tracker-deployer/issues/1)

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

**Epic Issue**: [#2 - Scaffolding for main app](https://github.com/torrust/torrust-tracker-deployer/issues/2)

- [x] **1.1** Setup logging - [Issue #3](https://github.com/torrust/torrust-tracker-deployer/issues/3) ✅ Completed
  - [x] Setup logging for production CLI - [PR #4](https://github.com/torrust/torrust-tracker-deployer/pull/4)
  - [x] Remove ANSI codes from file logging - [Issue #5](https://github.com/torrust/torrust-tracker-deployer/issues/5), [PR #7](https://github.com/torrust/torrust-tracker-deployer/pull/7)
- [x] **1.2** Create command `torrust-tracker-deployer destroy` to destroy an environment ✅ Completed
  - Parent EPIC: [Implement `destroy` Command](https://github.com/torrust/torrust-tracker-deployer/issues/8) - GitHub Issue #8
  - Split into two child EPICs for incremental delivery:
    - [x] **Child EPIC #9**: [App Layer Destroy Command](https://github.com/torrust/torrust-tracker-deployer/issues/9) - Core business logic
    - [x] **Child EPIC #10**: [UI Layer Destroy Command](https://github.com/torrust/torrust-tracker-deployer/issues/10) - CLI interface
- [x] **1.3** Refactor extract shared code between testing and production for app bootstrapping ✅ Completed
- [x] **1.4** Improve command to use better abstraction to handle presentation layer ✅ Completed
  - User output architecture improvements implemented
  - Epic [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) completed
  - Message trait system, sink abstraction, and theme support added
  - Folder module structure with focused submodules
- [x] **1.5** Create command `torrust-tracker-deployer create` to create a new environment ✅ Completed
  - EPIC: [Implement Create Environment Command](https://github.com/torrust/torrust-tracker-deployer/issues/34) - GitHub Issue #34
- [x] **1.6** Create command `torrust-tracker-deployer provision` to provision VM infrastructure (UI layer only) ✅ Completed - [Issue #174](https://github.com/torrust/torrust-tracker-deployer/issues/174)
  - **Note:** The App layer ProvisionCommand is already implemented, this task focuses on the console subcommand interface
  - Implementation should call the existing ProvisionCommand business logic
  - Handle user input, validation, and output presentation
- [x] **1.7** Create command `torrust-tracker-deployer configure` to configure provisioned infrastructure (UI layer only) ✅ Completed - [Issue #180](https://github.com/torrust/torrust-tracker-deployer/issues/180)
  - **Note:** The App layer ConfigureCommand is already implemented, this task focuses on the console subcommand interface
  - Implementation should call the existing ConfigureCommandHandler business logic
  - Handle user input, validation, and output presentation
  - Enables transition from "provisioned" to "configured" state via CLI
- [x] **1.8** Create command `torrust-tracker-deployer test` to verify deployment infrastructure (UI layer only) ✅ Completed - [Issue #188](https://github.com/torrust/torrust-tracker-deployer/issues/188)
  - **Note:** The App layer TestCommandHandler is already implemented, this task focuses on the console subcommand interface
  - Implementation should call the existing TestCommandHandler business logic
  - Handle user input, validation, and output presentation
  - Enables verification of deployment state via CLI (cloud-init, Docker, Docker Compose)

**Note:** See [`docs/research/UX/`](./research/UX/) for detailed UX research that will be useful to implement the features in this section.

**Future Enhancement:** The `torrust-tracker-deployer deploy` porcelain command (intelligent orchestration of plumbing commands) will be implemented after the core plumbing commands are stable. See [`docs/features/hybrid-command-architecture/`](./features/hybrid-command-architecture/) for the complete specification.

### 2. Add new infrastructure provider: Hetzner

**Epic Issue**: [#205 - Add Hetzner Provider Support](https://github.com/torrust/torrust-tracker-deployer/issues/205) ✅ Completed

- [x] **2.1** Add Hetzner provider support (Phase 1: Make LXD Explicit) ✅ Completed
  - [x] **2.1.1** Add Provider enum and ProviderConfig types - [Issue #206](https://github.com/torrust/torrust-tracker-deployer/issues/206) ✅ Completed
  - [x] **2.1.2** Update UserInputs to use ProviderConfig - [Issue #207](https://github.com/torrust/torrust-tracker-deployer/issues/207) ✅ Completed
  - [x] **2.1.3** Update EnvironmentCreationConfig DTO - [Issue #208](https://github.com/torrust/torrust-tracker-deployer/issues/208) ✅ Completed
  - [x] **2.1.4** Parameterize TofuTemplateRenderer by provider - [Issue #212](https://github.com/torrust/torrust-tracker-deployer/issues/212) ✅ Completed
  - [x] **2.1.5** Update environment JSON files and E2E tests ✅ Completed (part of #212)
  - [x] **2.1.6** Update user documentation - [Issue #214](https://github.com/torrust/torrust-tracker-deployer/issues/214) ✅ Completed
- [x] **2.2** Add Hetzner provider support (Phase 2: Add Hetzner) ✅ Completed
  - Hetzner OpenTofu templates implemented
  - Full deployment workflow tested with Hetzner Cloud

### 3. Continue adding more application commands

**Note:** These are internal app layer commands (like ProvisionCommand or ConfigureCommand), not console commands. The approach is to slice by functional services rather than deployment stages - we fully deploy a working stack from the beginning and incrementally add new services.

- [x] **3.1** Finish ConfigureCommand ✅ Completed - [Epic #16](https://github.com/torrust/torrust-tracker-deployer/issues/16)
  - System security configuration added (automatic updates, UFW firewall)
  - Ansible templates refactored to centralized variables pattern
  - Tasks completed: [#17](https://github.com/torrust/torrust-tracker-deployer/issues/17), [#18](https://github.com/torrust/torrust-tracker-deployer/issues/18), [#19](https://github.com/torrust/torrust-tracker-deployer/issues/19)
- [ ] **3.2** Implement ReleaseCommand and RunCommand with vertical slices - [Epic #216](https://github.com/torrust/torrust-tracker-deployer/issues/216)

  **Strategy:** Build incrementally with working deployments at each step. Each slice adds a new service to the docker-compose stack.

  - [x] **3.2.1** Hello World slice (scaffolding) - [Issue #217](https://github.com/torrust/torrust-tracker-deployer/issues/217) ✅ Completed
    - Create `release` and `run` commands with minimal docker-compose template
    - Deploy and run a simple hello-world container to validate the full pipeline
  - [x] **3.2.2** Torrust Tracker slice - [Issue #220](https://github.com/torrust/torrust-tracker-deployer/issues/220) ✅ Completed
    - Replace hello-world with Torrust Tracker service
    - Add tracker configuration template (start with hardcoded defaults, then progressively expose configuration options)
  - [x] **3.2.3** MySQL slice - [Issue #232](https://github.com/torrust/torrust-tracker-deployer/issues/232) ✅ Completed
    - Add MySQL service to docker-compose stack
    - Allow user to choose between SQLite and MySQL in environment config
  - [x] **3.2.4** Prometheus slice - [Issue #238](https://github.com/torrust/torrust-tracker-deployer/issues/238) ✅ Completed
    - Add Prometheus service for metrics collection
  - [x] **3.2.5** Grafana slice - [Issue #246](https://github.com/torrust/torrust-tracker-deployer/issues/246) ✅ Completed
    - Add Grafana service for metrics visualization

  **Notes:**

  - Each slice delivers a working deployment
  - Configuration complexity grows incrementally (hardcoded → environment config → full flexibility)
  - Detailed implementation tasks will be defined in EPIC issues

### 4. Create a docker image for the deployer

- [ ] **4.1** Create docker image for the deployer to use it without needing to install the dependencies (OpenTofu, Ansible, etc) - [Issue #264](https://github.com/torrust/torrust-tracker-deployer/issues/264)
  - Mount data and build dirs when running it.

### 5. Add extra console app commands

- [x] **5.1** `torrust-tracker-deployer show` - Display environment information and current state - [Issue #241](https://github.com/torrust/torrust-tracker-deployer/issues/241) ✅ Completed
- [x] **5.2** `torrust-tracker-deployer test` - Run application tests ✅ Completed
- [x] **5.3** `torrust-tracker-deployer list` - List environments or deployments - [Issue #260](https://github.com/torrust/torrust-tracker-deployer/issues/260) ✅ Completed

**Note:** The test console subcommand is already partially implemented. The `show` command displays stored environment data (read-only, no remote verification). A future `status` command may be added for service health checks.

### 6. Add HTTPS support

- [ ] **6.1** Add HTTPS support for HTTP tracker
- [ ] **6.2** Add HTTPS support for tracker API
- [ ] **6.3** Add HTTPS support for Grafana

### 7. Add backup and disaster recovery

- [ ] **7.1** Implement database backups for MySQL
- [ ] **7.2** Implement configuration backups
- [ ] **7.3** Create recovery procedures documentation

### 8. Add levels of verbosity

- [ ] **8.1** Add levels of verbosity as described in the UX research
  - Implement `-v`, `-vv`, `-vvv` flags for user-facing output
  - See [`docs/research/UX/`](https://github.com/torrust/torrust-tracker-deployer/tree/main/docs/research/UX) for detailed UX research

---

## Notes

- This roadmap will be linked to an EPIC issue on GitHub for tracking progress
- Each major feature should have corresponding documentation in `docs/features/` before implementation begins
