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

- [x] **4.1** Create docker image for the deployer to use it without needing to install the dependencies (OpenTofu, Ansible, etc) - [Issue #264](https://github.com/torrust/torrust-tracker-deployer/issues/264) ✅ Completed
  - Docker image published to Docker Hub
  - CI/CD workflow for automated builds
  - Security scanning with Trivy

### 5. Add extra console app commands

- [x] **5.1** `torrust-tracker-deployer show` - Display environment information and current state - [Issue #241](https://github.com/torrust/torrust-tracker-deployer/issues/241) ✅ Completed
- [x] **5.2** `torrust-tracker-deployer test` - Run application tests ✅ Completed
- [x] **5.3** `torrust-tracker-deployer list` - List environments or deployments - [Issue #260](https://github.com/torrust/torrust-tracker-deployer/issues/260) ✅ Completed

**Note:** The test console subcommand is already partially implemented. The `show` command displays stored environment data (read-only, no remote verification). A future `status` command may be added for service health checks.

### 6. Add HTTPS support ✅ COMPLETED

- [x] **6.1** Add HTTPS support with Caddy for all HTTP services - [Issue #272](https://github.com/torrust/torrust-tracker-deployer/issues/272) ✅ Completed
  - Implemented Caddy TLS termination proxy
  - Added HTTPS support for HTTP tracker
  - Added HTTPS support for tracker API
  - Added HTTPS support for Grafana
  - **Research Complete**: [Issue #270](https://github.com/torrust/torrust-tracker-deployer/issues/270) - Caddy evaluation successful, production deployment verified

### 7. Add backup support ✅ COMPLETED

**Epic Issue**: [#309 - Add backup support](https://github.com/torrust/torrust-tracker-deployer/issues/309)

- [x] **7.1** Research database backup strategies - [Issue #310](https://github.com/torrust/torrust-tracker-deployer/issues/310) ✅ Completed
  - Investigated SQLite and MySQL backup approaches
  - Recommended **maintenance-window hybrid approach** (container + crontab)
  - Built and tested POC with 58 unit tests
  - Documented findings in `docs/research/backup-strategies/`
- [x] **7.2** Implement backup support - [Issue #315](https://github.com/torrust/torrust-tracker-deployer/issues/315) ✅ Completed
  - Added backup container templates (Dockerfile, backup.sh) - Published to Docker Hub
  - Added backup service to Docker Compose template with profile-based enablement
  - Extended environment configuration schema with backup settings
  - Deployed backup artifacts via Ansible playbooks
  - Installed crontab for scheduled maintenance-window backups (3 AM daily)
  - Supports: MySQL dumps, SQLite file copy, config archives
  - Backup retention cleanup (configurable days, default 7)
  - **Note**: Volume management is out of scope - user provides a mounted location
  - **Implementation Details**: Phase 1-4 completed (container, service integration, crontab scheduling, documentation)

### 8. Add levels of verbosity

- [ ] **8.1** Add levels of verbosity as described in the UX research
  - Implement `-v`, `-vv`, `-vvv` flags for user-facing output
  - See [`docs/research/UX/`](https://github.com/torrust/torrust-tracker-deployer/tree/main/docs/research/UX) for detailed UX research

### 9. Extend deployer usability

Add new commands to allow users to take advantage of the deployer even if they do not want to use all functionalities. This enables partial adoption of the tool.

These commands complete a trilogy of "lightweight" entry points:

- `register` - For users with pre-provisioned instances
- `validate` - For users who only want to validate a deployment configuration
- `render` - For users who only want to build artifacts and handle deployment manually

This makes the deployer more versatile for different scenarios and more AI-agent friendly (dry-run commands provide feedback without side effects).

- [x] **9.1** Implement `validate` command (✅ Completed in [272847e3](https://github.com/torrust/torrust-tracker-deployer/commit/272847e3))
  - Validate deployment configuration without executing any deployment steps
  - See feature specification: [`docs/features/config-validation-command/`](./features/config-validation-command/)
  - User documentation: [`docs/user-guide /commands/validate.md`](./user-guide/commands/validate.md)
- [x] **9.2** Implement artifact generation command (✅ Completed in [37cbe240](https://github.com/torrust/torrust-tracker-deployer/commit/37cbe240)) - [Issue #326](https://github.com/torrust/torrust-tracker-deployer/issues/326)
  - **Command name**: `render` - Generates deployment artifacts without provisioning infrastructure
  - Dual input modes: `--env-name` (from Created state environment) or `--env-file` (from config file)
  - Requires `--instance-ip` parameter for Ansible inventory generation
  - Generates all 8 service artifacts: OpenTofu, Ansible, Docker Compose, Tracker, Prometheus, Grafana, Caddy, Backup
  - Output to user-specified directory via `--output-dir <PATH>` parameter (prevents conflicts with provision artifacts)
  - No remote operations - purely local artifact generation
  - Use cases: Preview before provisioning, manual deployment workflows, configuration inspection
  - User documentation: [`docs/user-guide/commands/render.md`](./user-guide/commands/render.md)
  - Manual testing guide: [`docs/e2e-testing/manual/render-verification.md`](./e2e-testing/manual/render-verification.md)
  - All templates always rendered (no conditional logic)
  - Specification: [`docs/issues/326-implement-artifact-generation-command.md`](./issues/326-implement-artifact-generation-command.md)

### 10. Improve usability (UX)

Minor changes to improve the output of some commands and overall user experience.

- [ ] **10.1** Add DNS setup reminder in `provision` command output
  - Display reminder when any service has a domain configured
  - Issue: [#332](https://github.com/torrust/torrust-tracker-deployer/issues/332)
  - Specification: [`docs/issues/332-dns-setup-reminder-in-provision-command.md`](./issues/332-dns-setup-reminder-in-provision-command.md)
- [ ] **10.2** Improve `run` command output with service URLs
  - Show service URLs immediately after services start
  - Include hint about `show` command for full details
  - See draft: [`docs/issues/drafts/improve-run-command-output-with-service-urls.md`](./issues/drafts/improve-run-command-output-with-service-urls.md)
- [ ] **10.3** Add DNS resolution check to `test` command
  - Verify configured domains resolve to the expected instance IP
  - Advisory warning only (doesn't fail tests) - DNS is decoupled from service tests
  - See draft: [`docs/issues/drafts/add-dns-resolution-check-to-test-command.md`](./issues/drafts/add-dns-resolution-check-to-test-command.md)
- [x] **10.4** Add `purge` command to remove local environment data - [Issue #322](https://github.com/torrust/torrust-tracker-deployer/issues/322) ✅ Completed
  - Removes `data/{env}/` and `build/{env}/` for destroyed environments
  - Allows reusing environment names after destruction
  - Users don't need to know internal storage details
  - Added confirmation prompt with `--force` flag
  - Added comprehensive user documentation

### 11. Improve AI agent experience

Add features and documentation that make the use of AI agents to operate the deployer easier, more efficient, more reliable, and less prone to hallucinations.

**Context**: We assume users will increasingly interact with the deployer indirectly via AI agents (GitHub Copilot, Cursor, etc.) rather than running commands directly. This section ensures AI agents have the best possible experience when working with the deployer.

- [ ] **11.1** Consider using [agentskills.io](https://agentskills.io) for AI agent capabilities
  - Agent Skills is an open format for extending AI agent capabilities with specialized knowledge and workflows
  - Developed by Anthropic, adopted by Claude Code, OpenAI Codex, Amp, and others
  - Provides progressive disclosure: metadata at startup, instructions on activation, resources on demand
  - Skills can bundle scripts, templates, and reference materials
  - Evaluate compatibility with current `AGENTS.md` approach
  - See issue: [#274](https://github.com/torrust/torrust-tracker-deployer/issues/274)
  - See spec: [`docs/issues/274-consider-using-agentskills-io.md`](./issues/274-consider-using-agentskills-io.md)

- [x] **11.2** Add AI-discoverable documentation headers to template files ✅ Completed
  - Templates generate production config files (docker-compose, tracker.toml, Caddyfile, etc.)
  - Documentation is moving from templates to Rust wrapper types (published on docs.rs)
  - Problem: AI agents in production only see rendered output, not the source repo
  - Solution: Add standardized header to templates with links to repo, wrapper path, and docs.rs
  - Enables AI agents to find documentation even when working with deployed configs
  - See draft: [`docs/issues/drafts/add-ai-discoverable-documentation-headers-to-templates.md`](./issues/drafts/add-ai-discoverable-documentation-headers-to-templates.md)

- [ ] **11.3** Provide configuration examples and questionnaire for AI agent guidance
  - Problem: AI agents struggle with the many valid configuration combinations
  - Questionnaire template: structured decision tree to gather all required user information
  - Example dataset: real-world scenarios mapping requirements to validated configs
  - Covers: provider selection, database type, tracker protocols, HTTPS, monitoring, etc.
  - Benefits: few-shot learning for agents, reduced hallucination, training/RAG dataset
  - Can integrate with `create-environment-config` skill from task 11.1
  - See draft: [`docs/issues/drafts/provide-config-examples-and-questionnaire-for-ai-agents.md`](./issues/drafts/provide-config-examples-and-questionnaire-for-ai-agents.md)

- [ ] **11.4** Add dry-run mode for all commands
  - Allow AI agents (and users) to preview what will happen before executing operations
  - Particularly valuable for destructive commands (`destroy`, `stop`)
  - Flag: `--dry-run` shows planned actions without executing
  - Reduces risk when AI agents operate autonomously

---

## Deferred Features

Features considered valuable but **out of scope for v1**. We want to release the first version and wait for user acceptance before investing more time. These can be revisited based on user feedback.

| Feature                                      | Rationale                                                  | Notes                                                     |
| -------------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------------- |
| Machine-readable JSON output (`--json` flag) | Easy to add thanks to MVC pattern, but not critical for v1 | Structured output helps AI agents parse results reliably  |
| MCP (Model Context Protocol) server          | Native AI integration without shell commands               | Would let AI agents call deployer as MCP tools directly   |
| Structured error format for AI agents        | Already improving errors in section 10                     | Could formalize with error codes, fix suggestions in JSON |

---

## Notes

- This roadmap will be linked to an EPIC issue on GitHub for tracking progress
- Each major feature should have corresponding documentation in `docs/features/` before implementation begins
