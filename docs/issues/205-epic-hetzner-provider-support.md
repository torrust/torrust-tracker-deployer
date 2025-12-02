# Epic: Add Hetzner Provider Support

**Issue**: #205
**Parent**: #1 - Project Roadmap
**Roadmap Section**: 2.1 - Add Hetzner provider support
**Feature Documentation**: [docs/features/hetzner-provider-support/](../features/hetzner-provider-support/)

## Overview

This epic tracks the implementation of Hetzner Cloud as the first production infrastructure provider for the Torrust Tracker Deployer.

### Background

The Torrust Tracker Deployer currently uses LXD as the sole infrastructure provider. LXD was chosen for development and local testing because:

- Fast VM creation (good for E2E tests)
- No cloud costs for development
- Works well in CI environments (GitHub Actions)
- Full VM isolation matching production behavior

However, for actual production deployments (like the Torrust Tracker Demo), we need cloud provider support. Hetzner Cloud was selected as the first production provider because:

- Cost-effective for the Torrust project
- Good European presence (relevant for torrent infrastructure)
- Simple API with good Terraform/OpenTofu support
- Established provider with proven reliability

### Problem Statement

1. **Provider is hardcoded**: The current implementation assumes LXD everywhere, with no way for users to select a different provider.
2. **Production deployments are blocked**: Cannot deploy Torrust Tracker to cloud infrastructure using this tool.
3. **Provider-specific configuration is implicit**: LXD-specific parameters (instance name, profile name) are embedded in the domain layer without clear provider context.

## Approach: "Refactor First, Then Extend"

The implementation follows a two-phase approach:

1. **Phase 1 - Make LXD Explicit**: Refactor the current code to treat LXD as one of potentially many providers, even though it's currently the only one. This makes the codebase ready for extension.

2. **Phase 2 - Add Hetzner**: Implement Hetzner support by adding new templates and minimal code changes.

This approach is preferred because:

- Separates architectural changes from new functionality
- Allows validating the architecture before adding complexity
- Makes code reviews easier (smaller, focused PRs)
- Reduces risk of introducing bugs

## Key Design Decisions

- **No provider abstraction layer**: Each provider uses its own native configuration (Terraform/OpenTofu variables) without a common abstraction
- **Provider selection via environment config**: User specifies provider in the environment JSON file (no default - always explicit)
- **OpenTofu templates per provider**: Separate template directories for each provider (`tofu/lxd/`, `tofu/hetzner/`)
- **Rust-based validation (MVP)**: Serde tagged enums handle provider-specific validation
- **Single-use tool philosophy**: All config (including API tokens) stored in environment JSON
- **OS-level firewall only**: Use UFW instead of provider firewalls for portability
- **`InstanceName` generic, `ProfileName` LXD-specific**: Instance name uses strictest validation across all providers
- **Local Terraform state only**: No remote state backends

## Tasks

### Phase 1: Make LXD Explicit

This phase refactors the codebase to make LXD an explicit, selectable provider. Tasks are ordered by dependency.

#### Task 1: Add Provider enum and ProviderConfig types (#206)

**Issue**: [#206](https://github.com/torrust/torrust-tracker-deployer/issues/206)
**Dependencies**: None (foundational task)

Create the foundational types:

- `Provider` enum in domain layer (`src/domain/provider/`)
- `ProviderConfig`, `LxdConfig`, `HetznerConfig` (domain types with validated fields)
- `ProviderSection`, `LxdProviderSection`, `HetznerProviderSection` (application layer config types for JSON parsing)

#### Task 2: Update UserInputs to use ProviderConfig

**Issue**: [#207](https://github.com/torrust/torrust-tracker-deployer/issues/207)
**Dependencies**: Task 1

Refactor the domain `UserInputs` struct:

- Add `provider_config: ProviderConfig` field
- Move `profile_name` from global field to `LxdConfig`
- Keep `instance_name` as global (all providers need it)
- Update all code that accesses `profile_name`

#### Task 3: Update EnvironmentCreationConfig DTO

**Issue**: [#208](https://github.com/torrust/torrust-tracker-deployer/issues/208)
**Dependencies**: Task 1

Update the application layer DTO to include provider configuration:

- Add `provider` section to `EnvironmentCreationConfig`
- Update `to_environment_params()` to handle provider config
- Update JSON schema/examples in documentation

#### Task 4: Parameterize TofuTemplateRenderer by provider

**Issue**: [#212](https://github.com/torrust/torrust-tracker-deployer/issues/212)
**Dependencies**: Task 2

Make the template renderer provider-aware:

- Add `provider` field to `TofuTemplateRenderer`
- Change hardcoded `tofu/lxd` path to dynamic `tofu/{provider}`
- Update `EnvironmentContext::tofu_build_dir()` to use provider
- Create and validate Hetzner OpenTofu templates manually before Rust integration
- Add Hetzner-specific template wrappers

#### Task 5: Update environment JSON files and E2E tests

**Issue**: TBD
**Dependencies**: Tasks 2, 3

Update all configuration files to the new format:

- Update `data/e2e-*/environment.json` files
- Update `data/my-environment/environment.json`
- Ensure all E2E tests pass with new config format
- Add clear error message when `provider` field is missing

#### Task 6: Update user documentation

**Issue**: TBD
**Dependencies**: Tasks 1-5

Document the provider selection feature:

- Update user guide with provider configuration
- Update example configurations
- Document migration from old format (if needed)

#### Phase 1 Dependency Graph

```text
Task 1: Provider types (foundation)
   │
   ├──► Task 2: Update UserInputs (domain)
   │       │
   │       └──► Task 4: Parameterize TofuTemplateRenderer
   │               │
   └──► Task 3: Update EnvironmentCreationConfig (application)
           │
           └──► Task 5: Update JSON files & E2E tests
                   │
                   └──► Task 6: Update documentation
```

### Phase 2: Add Hetzner Provider

| Task                                     | Description                              | Status      |
| ---------------------------------------- | ---------------------------------------- | ----------- |
| Create static Hetzner OpenTofu config    | Test manually before Tera templating     | Not Started |
| Add HetznerConfig to ProviderConfig enum | Domain type for Hetzner-specific config  | Not Started |
| Convert OpenTofu files to Tera templates | Create templates/tofu/hetzner/ directory | Not Started |
| Add Hetzner variables template wrapper   | Infrastructure layer template handling   | Not Started |
| Create example Hetzner environment       | Example configuration for users          | Not Started |
| Add Hetzner-specific documentation       | Setup guide and troubleshooting          | Not Started |
| Manual testing with Hetzner              | Full deployment workflow testing         | Not Started |
| Add unit tests                           | Config parsing and validation tests      | Not Started |

### Phase 3: Documentation and Polish

| Task                              | Description                       | Status      |
| --------------------------------- | --------------------------------- | ----------- |
| User guide for Hetzner setup      | Step-by-step guide                | Not Started |
| Provider comparison documentation | LXD vs Hetzner comparison         | Not Started |
| Cost estimation guidance          | Help users estimate Hetzner costs | Not Started |
| Troubleshooting guide             | Common issues and solutions       | Not Started |

## Definition of Done

### Phase 1 Requirements

- [ ] Environment JSON requires `provider: "lxd"` field
- [ ] All existing E2E tests pass with updated configuration
- [ ] Clear error message if provider is missing
- [ ] Documentation updated to show provider selection
- [ ] Code follows project conventions (DDD layers, error handling)

### Phase 2 Requirements

- [ ] Can provision Hetzner VM using `provision` command
- [ ] Can configure Hetzner VM (Docker, security) using `configure` command
- [ ] Can destroy Hetzner infrastructure using `destroy` command
- [ ] Can test Hetzner deployment using `test` command
- [ ] Documentation for Hetzner setup and configuration
- [ ] Unit tests for Hetzner configuration handling

### Technical Requirements

- [ ] Code follows project conventions and style guidelines
- [ ] All linters pass (`cargo run --bin linter all`)
- [ ] No compiler warnings
- [ ] Pre-commit checks pass (`./scripts/pre-commit.sh`)

### Testing Requirements

- [ ] Unit tests cover provider selection logic
- [ ] E2E tests for LXD still pass
- [ ] Manual testing completed for Hetzner workflow

## Reference Implementation

A working Hetzner Terraform configuration exists in the bash Proof of Concept:

- **Repository**: [torrust/torrust-tracker-deploy-bash-poc](https://github.com/torrust/torrust-tracker-deploy-bash-poc)
- **Hetzner Provider**: [infrastructure/terraform/providers/hetzner/](https://github.com/torrust/torrust-tracker-deploy-bash-poc/tree/main/infrastructure/terraform/providers/hetzner)

This PoC implementation provides:

- `main.tf` - Hetzner firewall and server resources with cloud-init
- `variables.tf` - Server type/location/image definitions with validation
- `outputs.tf` - Standard output interface compatible with our parser
- `versions.tf` - Required provider versions (hcloud ~> 1.47)

## Related Documentation

- [Feature Specification](../features/hetzner-provider-support/specification.md)
- [Technical Analysis](../features/hetzner-provider-support/analysis.md)
- [Questions & Answers](../features/hetzner-provider-support/questions.md)
- [Development Principles](../development-principles.md)
- [VM Providers Comparison](../vm-providers.md)
- [LXD VM Decision Record](../decisions/lxd-vm-over-containers.md)

## Notes

- **Important Context**: This is a **single-use deployment tool**, not a management platform or sysadmin replacement. The tool automates initial Torrust Tracker deployment but is not intended for ongoing infrastructure management.
- The entire local configuration (`data/`, `envs/`, `build/` directories) is considered sensitive.
- Hetzner API token is provided by the product owner for manual testing.

---

**Created**: December 1, 2025
**Status**: Planning Complete - Ready for Implementation
