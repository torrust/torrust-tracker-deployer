# Architectural Decision Records (ADRs)

This directory contains architectural decision records for the Torrust Tracker Deployer project. Each ADR documents an important architectural decision, the context that led to it, and the consequences of the decision.

## Decision Index

| Status      | Date       | Decision                                                                                            | Summary                                                                                   |
| ----------- | ---------- | --------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| ✅ Accepted | 2025-11-03 | [Environment Variable Prefix](./environment-variable-prefix.md)                                     | Use `TORRUST_TD_` prefix for all environment variables                                    |
| ✅ Accepted | 2025-10-15 | [External Tool Adapters Organization](./external-tool-adapters-organization.md)                     | Consolidate external tool wrappers in `src/adapters/` for better discoverability          |
| ✅ Accepted | 2025-10-10 | [Repository Rename to Deployer](./repository-rename-to-deployer.md)                                 | Rename from "Torrust Tracker Deploy" to "Torrust Tracker Deployer" for production use     |
| ✅ Accepted | 2025-10-03 | [Error Context Strategy](./error-context-strategy.md)                                               | Use structured error context with trace files for complete error information              |
| ✅ Accepted | 2025-10-03 | [Command State Return Pattern](./command-state-return-pattern.md)                                   | Commands return typed states (Environment<S> → Environment<T>) for compile-time safety    |
| ✅ Accepted | 2025-10-03 | [Actionable Error Messages](./actionable-error-messages.md)                                         | Use tiered help system with brief tips + .help() method for detailed troubleshooting      |
| ✅ Accepted | 2025-10-01 | [Type Erasure for Environment States](./type-erasure-for-environment-states.md)                     | Use enum-based type erasure to enable runtime handling and serialization of typed states  |
| ✅ Accepted | 2025-09-29 | [Test Context vs Deployment Environment Naming](./test-context-vs-deployment-environment-naming.md) | Rename TestEnvironment to TestContext to avoid conflicts with multi-environment feature   |
| ✅ Accepted | 2025-09-10 | [LXD VMs over Containers](./lxd-vm-over-containers.md)                                              | Use LXD virtual machines instead of containers for production alignment                   |
| ✅ Accepted | 2025-09-09 | [Tera Minimal Templating Strategy](./tera-minimal-templating-strategy.md)                           | Use Tera with minimal variables and templates to avoid complexity and delimiter conflicts |
| ✅ Accepted | -          | [LXD over Multipass](./lxd-over-multipass.md)                                                       | Choose LXD containers over Multipass VMs for deployment testing                           |
| ✅ Resolved | -          | [Docker Testing Evolution](./docker-testing-evolution.md)                                           | Evolution from Docker rejection to hybrid approach for split E2E testing                  |
| ✅ Accepted | -          | [Meson Removal](./meson-removal.md)                                                                 | Remove Meson build system from the project                                                |

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
