# Architectural Decision Records (ADRs)

This directory contains architectural decision records for the Torrust Tracker Deployer project. Each ADR documents an important architectural decision, the context that led to it, and the consequences of the decision.

## Decision Index

| Status        | Date       | Decision                                                                                                  | Summary                                                                                    |
| ------------- | ---------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| ✅ Accepted   | 2026-01-20 | [Grafana Default Port 3000](./grafana-default-port-3000.md)                                               | Use Grafana's default port 3000 instead of 3100 for dedicated VM deployments               |
| ✅ Accepted   | 2026-01-20 | [Caddy for TLS Termination](./caddy-for-tls-termination.md)                                               | Use Caddy v2.10 as TLS proxy for automatic HTTPS with WebSocket support                    |
| ✅ Accepted   | 2026-01-20 | [Per-Service TLS Configuration](./per-service-tls-configuration.md)                                       | Use domain + use_tls_proxy fields instead of nested tls section for explicit TLS opt-in    |
| ✅ Accepted   | 2026-01-20 | [Uniform HTTP Tracker TLS Requirement](./uniform-http-tracker-tls-requirement.md)                         | All HTTP trackers must use same TLS setting due to tracker's global on_reverse_proxy       |
| ✅ Accepted   | 2026-01-10 | [Hetzner SSH Key Dual Injection Pattern](./hetzner-ssh-key-dual-injection.md)                             | Use both OpenTofu SSH key and cloud-init for debugging capability with manual hardening    |
| ✅ Accepted   | 2026-01-10 | [Configuration and Data Directories as Secrets](./configuration-directories-as-secrets.md)                | Treat envs/, data/, build/ as secrets; no env var injection; users secure via permissions  |
| ✅ Accepted   | 2026-01-07 | [Configuration DTO Layer Placement](./configuration-dto-layer-placement.md)                               | Keep configuration DTOs in application layer, not domain; defer package extraction         |
| ✅ Accepted   | 2025-12-23 | [Docker Security Scan Exit Code Zero](./docker-security-scan-exit-code-zero.md)                           | Use exit-code 0 for security scanning - Trivy detects, GitHub Security decides, CI green   |
| ✅ Accepted   | 2025-12-20 | [Grafana Integration Pattern](./grafana-integration-pattern.md)                                           | Enable Grafana by default with hard Prometheus dependency and environment variable config  |
| ✅ Accepted   | 2025-12-17 | [Secrecy Crate for Sensitive Data Handling](./secrecy-crate-for-sensitive-data.md)                        | Use secrecy crate for type-safe secret handling with memory zeroing                        |
| ✅ Accepted   | 2025-12-14 | [Database Configuration Structure in Templates](./database-configuration-structure-in-templates.md)       | Expose structured database fields in templates rather than pre-resolved connection strings |
| ✅ Accepted   | 2025-12-13 | [Environment Variable Injection in Docker Compose](./environment-variable-injection-in-docker-compose.md) | Use .env file injection instead of hardcoded values for runtime configuration changes      |
| ✅ Accepted   | 2025-12-11 | [Cloud-Init SSH Port Configuration with Reboot](./cloud-init-ssh-port-reboot.md)                          | Use cloud-init with reboot pattern to configure custom SSH ports during VM provisioning    |
| ✅ Accepted   | 2025-12-10 | [Single Docker Image for Sequential E2E Command Testing](./single-docker-image-sequential-testing.md)     | Use single Docker image with sequential command execution instead of multi-image phases    |
| ✅ Accepted   | 2025-12-09 | [Register Command SSH Port Override](./register-ssh-port-override.md)                                     | Add optional --ssh-port argument to register command for non-standard SSH ports            |
| ✅ Accepted   | 2025-11-19 | [Disable MD060 Table Formatting Rule](./md060-table-formatting-disabled.md)                               | Disable MD060 to allow flexible table formatting and emoji usage                           |
| ✅ Accepted   | 2025-11-19 | [Test Command as Smoke Test](./test-command-as-smoke-test.md)                                             | Test command validates running services, not infrastructure components                     |
| ✅ Accepted   | 2025-11-13 | [Migration to AGENTS.md Standard](./agents-md-migration.md)                                               | Adopt open AGENTS.md standard for multi-agent compatibility while keeping GitHub redirect  |
| ✅ Accepted   | 2025-11-11 | [Use ReentrantMutex Pattern for UserOutput Reentrancy](./reentrant-mutex-useroutput-pattern.md)           | Use Arc<ReentrantMutex<RefCell<UserOutput>>> to fix same-thread deadlock in issue #164     |
| ❌ Superseded | 2025-11-11 | [Remove UserOutput Mutex](./user-output-mutex-removal.md)                                                 | Remove Arc<Mutex<UserOutput>> pattern for simplified, deadlock-free architecture           |
| ✅ Accepted   | 2025-11-07 | [ExecutionContext Wrapper Pattern](./execution-context-wrapper.md)                                        | Use ExecutionContext wrapper around Container for future-proof command signatures          |
| ✅ Accepted   | 2025-11-03 | [Environment Variable Prefix](./environment-variable-prefix.md)                                           | Use `TORRUST_TD_` prefix for all environment variables                                     |
| ✅ Accepted   | 2025-10-15 | [External Tool Adapters Organization](./external-tool-adapters-organization.md)                           | Consolidate external tool wrappers in `src/adapters/` for better discoverability           |
| ✅ Accepted   | 2025-10-10 | [Repository Rename to Deployer](./repository-rename-to-deployer.md)                                       | Rename from "Torrust Tracker Deploy" to "Torrust Tracker Deployer" for production use      |
| ✅ Accepted   | 2025-10-03 | [Error Context Strategy](./error-context-strategy.md)                                                     | Use structured error context with trace files for complete error information               |
| ✅ Accepted   | 2025-10-03 | [Command State Return Pattern](./command-state-return-pattern.md)                                         | Commands return typed states (Environment<S> → Environment<T>) for compile-time safety     |
| ✅ Accepted   | 2025-10-03 | [Actionable Error Messages](./actionable-error-messages.md)                                               | Use tiered help system with brief tips + .help() method for detailed troubleshooting       |
| ✅ Accepted   | 2025-10-01 | [Type Erasure for Environment States](./type-erasure-for-environment-states.md)                           | Use enum-based type erasure to enable runtime handling and serialization of typed states   |
| ✅ Accepted   | 2025-09-29 | [Test Context vs Deployment Environment Naming](./test-context-vs-deployment-environment-naming.md)       | Rename TestEnvironment to TestContext to avoid conflicts with multi-environment feature    |
| ✅ Accepted   | 2025-09-10 | [LXD VMs over Containers](./lxd-vm-over-containers.md)                                                    | Use LXD virtual machines instead of containers for production alignment                    |
| ✅ Accepted   | 2025-09-09 | [Tera Minimal Templating Strategy](./tera-minimal-templating-strategy.md)                                 | Use Tera with minimal variables and templates to avoid complexity and delimiter conflicts  |
| ✅ Accepted   | -          | [LXD over Multipass](./lxd-over-multipass.md)                                                             | Choose LXD containers over Multipass VMs for deployment testing                            |
| ✅ Resolved   | -          | [Docker Testing Evolution](./docker-testing-evolution.md)                                                 | Evolution from Docker rejection to hybrid approach for split E2E testing                   |
| ✅ Accepted   | -          | [Meson Removal](./meson-removal.md)                                                                       | Remove Meson build system from the project                                                 |

## ADR Template

When creating new ADRs, use the structure defined in [TEMPLATE.md](./TEMPLATE.md).

## Guidelines

- **One decision per file**: Each ADR should focus on a single architectural decision
- **Immutable**: Once accepted, ADRs should not be modified. Create new ADRs to supersede old ones
- **Context-rich**: Include enough background for future readers to understand why the decision was made
- **Consequence-aware**: Document both positive and negative consequences
- **Linked**: Reference related decisions and external resources

## Status Definitions

- **Proposed**: Decision is under discussion
- **Accepted**: Decision has been approved and is being implemented
- **Rejected**: Decision was considered but not approved
- **Superseded**: Decision has been replaced by a newer ADR
