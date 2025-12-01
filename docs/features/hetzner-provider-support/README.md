# Hetzner Provider Support

Add Hetzner Cloud as the first production infrastructure provider for the Torrust Tracker Deployer.

## 📄 Documents

### [specification.md](./specification.md)

The main feature specification including:

- Overview and problem statement
- Feature goals
- Proposed solution
- Implementation details
- Definition of done
- Testing strategy

### [questions.md](./questions.md)

Clarifying questions that need to be answered before implementation:

- Scope and requirements
- Technical approach
- Priority and timeline
- Success criteria
- Risk assessment

### [analysis.md](./analysis.md)

Technical analysis of the current implementation and proposed changes:

- Current LXD provider implementation analysis
- Provider abstraction strategy
- Implementation approach for multi-provider support

## 📋 Status

**Current Phase**: Planning Complete - Ready for Implementation

**Completed**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ✅ Create analysis document
4. ✅ Answer clarifying questions (all 21 questions answered)
5. ✅ Update specification based on answers
6. ⏳ Begin implementation

**Next Steps**:

1. Create issues for Phase 1 (LXD explicit provider refactoring)
2. Implement Phase 1 incrementally
3. Manual E2E testing with Hetzner (token provided by product owner)

## 🎯 Quick Summary

**Problem**: The deployer currently only supports LXD as a hardcoded provider. LXD was chosen for development and testing, but for production deployments (like the Torrust Tracker Demo), we need cloud provider support.

**Solution**: Implement a multi-provider architecture starting with Hetzner Cloud as the first production provider. The approach is to:

1. **Phase 1**: Refactor to make LXD an explicit, selectable provider
2. **Phase 2**: Add Hetzner provider alongside LXD

**Key Design Decisions**:

- **No provider abstraction layer**: Each provider uses its own native configuration (Terraform/OpenTofu variables) without a common abstraction
- **Provider selection via environment config**: User specifies provider in the environment JSON file (no default - always explicit)
- **OpenTofu templates per provider**: Separate template directories for each provider (`tofu/lxd/`, `tofu/hetzner/`)
- **Rust-based validation (MVP)**: Serde tagged enums handle provider-specific validation. JSON Schema generation is a future enhancement, not MVP.
- **Single-use tool philosophy**: This is a deployment tool, not a management platform. All config (including API tokens) stored in environment JSON. Users responsible for securing local data.
- **OS-level firewall only**: Use UFW instead of provider firewalls (Hetzner Cloud Firewall) for portability across providers
- **`InstanceName` generic, `ProfileName` LXD-specific**: Instance name uses strictest validation across all providers; profile name moves to LXD-specific config
- **Local Terraform state only**: No remote state backends - not a team/multi-user tool

## 🔗 Related Documentation

- [Development Principles](../../development-principles.md)
- [VM Providers Comparison](../../vm-providers.md)
- [LXD VM Decision Record](../../decisions/lxd-vm-over-containers.md)
- [Roadmap Item 2.1](../../roadmap.md)

## 📚 Reference Implementation

A working Hetzner Terraform configuration exists in the bash Proof of Concept:

- **Repository**: [torrust/torrust-tracker-deploy-bash-poc](https://github.com/torrust/torrust-tracker-deploy-bash-poc)
- **Hetzner Provider**: [infrastructure/terraform/providers/hetzner/](https://github.com/torrust/torrust-tracker-deploy-bash-poc/tree/main/infrastructure/terraform/providers/hetzner)

This PoC implementation provides:

- `main.tf` - Hetzner firewall and server resources with cloud-init
- `variables.tf` - Server type/location/image definitions with validation
- `outputs.tf` - Standard output interface compatible with our parser
- `versions.tf` - Required provider versions (hcloud ~> 1.47)
