# Implement ReleaseCommand and RunCommand with vertical slices

**Issue**: #216
**Parent**: #1 (Project Roadmap)

## Overview

This epic tracks the implementation of roadmap task **3.2**: Implement ReleaseCommand and RunCommand with vertical slices.

The goal is to add two new application commands:

- **`release`**: Copy docker-compose configuration to the provisioned VM
- **`run`**: Execute `docker compose up -d` to start the services

## Strategy

Build incrementally with working deployments at each step. Each slice adds a new service to the docker-compose stack, delivering value progressively.

## Roadmap Reference

From [docs/roadmap.md](../roadmap.md):

> **3.2** Implement ReleaseCommand and RunCommand with vertical slices
>
> **Strategy:** Build incrementally with working deployments at each step. Each slice adds a new service to the docker-compose stack.
>
> - **3.2.1** Hello World slice (scaffolding)
> - **3.2.2** Torrust Tracker slice
> - **3.2.3** MySQL slice
> - **3.2.4** Prometheus slice
> - **3.2.5** Grafana slice

## Tasks

Slices will be implemented sequentially, each delivering a working deployment:

- [x] **3.2.1** Hello World slice (scaffolding) - ✅ #217

  - Create `release` and `run` commands structure (App + UI layers)
  - Add minimal docker-compose template with nginx demo container
  - Validate full pipeline: release → run → verify container running
  - **Specification**: [docs/issues/217-demo-slice-release-run-commands.md](217-demo-slice-release-run-commands.md)

- [x] **3.2.2** Torrust Tracker slice - ✅ #220

  - Replace nginx demo with Torrust Tracker service
  - Add tracker configuration template (start with hardcoded defaults)
  - Progressively expose configuration options to environment config
  - **Specification**: [docs/issues/220-tracker-slice-release-run-commands.md](220-tracker-slice-release-run-commands.md)

- [ ] **3.2.3** MySQL slice - #232

  - Add MySQL service to docker-compose stack
  - Allow user to choose between SQLite and MySQL in environment config
  - **Specification**: [docs/issues/232-mysql-slice-release-run-commands.md](232-mysql-slice-release-run-commands.md)

- [ ] **3.2.4** Prometheus slice

  - Add Prometheus service for metrics collection
  - Configure tracker metrics endpoint

- [ ] **3.2.5** Grafana slice
  - Add Grafana service for metrics visualization
  - Include basic dashboard configuration

(Individual task issues will be created and linked as work progresses)

## Implementation Approach

Each slice follows this pattern:

1. **Hardcoded first**: Start with fixed configuration to validate the pipeline
2. **Add environment config**: Progressively expose configuration options
3. **Full flexibility**: Add advanced options as needed

This ensures we always have a working deployment while incrementally adding complexity.

## Related

- Parent: #1 (Project Roadmap)
- Depends on: ConfigureCommand completion (✅ Done - Epic #16)

## Reference Implementation

The [torrust-demo](https://github.com/torrust/torrust-demo) repository contains the current Torrust Tracker Demo configuration and serves as a reference for this implementation:

- [compose.yaml](https://github.com/torrust/torrust-demo/blob/main/compose.yaml) - Docker Compose configuration for the full stack
- [share/](https://github.com/torrust/torrust-demo/tree/main/share) - Service configurations (tracker, MySQL, Prometheus, Grafana, etc.)

These will be consulted when creating subissues for each slice.

## Notes

- Each slice requires new templates, template Rust wrappers, and potentially new environment options
- Configuration complexity grows incrementally across slices
- Detailed implementation tasks will be defined in subissues for each slice
