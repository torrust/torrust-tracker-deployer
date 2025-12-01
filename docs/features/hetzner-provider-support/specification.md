# Hetzner Provider Support Specification

## 📋 Overview

### Context

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

## 🎯 Goals

### Primary Goals

- **Multi-provider support**: Allow users to select between LXD (local) and Hetzner (cloud) providers
- **Production deployment capability**: Enable deploying Torrust Tracker to Hetzner Cloud
- **Maintain simplicity**: Avoid over-engineering with complex provider abstractions

### Secondary Goals (Nice-to-Have)

- Clean separation for adding future providers (AWS, DigitalOcean, etc.)
- Provider-specific documentation and examples
- Cost estimation for cloud deployments

### Non-Goals

What this feature explicitly does NOT aim to do:

- **No provider abstraction layer**: We will NOT create a common interface that normalizes provider parameters (e.g., no `small/medium/large` instance sizes)
- **No multi-provider environments**: One environment = one provider
- **No provider migration**: No automated way to move from LXD to Hetzner
- **No other cloud providers**: Only LXD and Hetzner in this feature

## 💡 Proposed Solution

### Approach: "Refactor First, Then Extend"

The implementation follows a two-phase approach:

1. **Phase 1 - Make LXD Explicit**: Refactor the current code to treat LXD as one of potentially many providers, even though it's currently the only one. This makes the codebase ready for extension.

2. **Phase 2 - Add Hetzner**: Implement Hetzner support by adding new templates and minimal code changes.

This approach is preferred because:

- Separates architectural changes from new functionality
- Allows validating the architecture before adding complexity
- Makes code reviews easier (smaller, focused PRs)
- Reduces risk of introducing bugs

### Design Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                     Environment JSON                            │
├─────────────────────────────────────────────────────────────────┤
│  {                                                              │
│    "provider": "lxd" | "hetzner",                               │
│    "provider_config": {                                         │
│      // Provider-specific configuration                         │
│      // LXD: instance_name, profile_name                        │
│      // Hetzner: server_type, location, image                   │
│    },                                                           │
│    "ssh_credentials": { ... },                                  │
│    ...                                                          │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  ProvisionCommandHandler                        │
├─────────────────────────────────────────────────────────────────┤
│  - Reads provider from environment                              │
│  - Selects appropriate template directory                       │
│  - Delegates to TofuTemplateRenderer                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  TofuTemplateRenderer                           │
├─────────────────────────────────────────────────────────────────┤
│  - Renders templates from provider-specific directory           │
│  - tofu/lxd/ OR tofu/hetzner/                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                ┌─────────────┴─────────────┐
                ▼                           ▼
┌─────────────────────────┐   ┌─────────────────────────┐
│    templates/tofu/lxd/  │   │  templates/tofu/hetzner/│
├─────────────────────────┤   ├─────────────────────────┤
│  main.tf                │   │  main.tf                │
│  cloud-init.yml.tera    │   │  cloud-init.yml.tera    │
│  variables.tfvars.tera  │   │  variables.tfvars.tera  │
└─────────────────────────┘   └─────────────────────────┘
```

### Key Design Decisions

1. **Provider selection in environment JSON**: Users specify `provider: "lxd"` or `provider: "hetzner"` in their environment configuration file.

2. **No abstraction layer**: Each provider uses its native configuration format. Hetzner uses `server_type: "cx11"`, not a generic `size: "small"`.

3. **Separate template directories**: Each provider has its own OpenTofu template directory, allowing full customization without complex conditionals.

4. **Shared cloud-init**: The cloud-init configuration is largely shared between providers (SSH setup, user creation), with provider-specific adjustments as needed.

5. **Instance name validation uses strictest common subset**: The existing `InstanceName` domain type is reused for both LXD and Hetzner server names. While providers have slightly different naming rules (e.g., Hetzner allows periods in server names per RFC 1123, LXD does not), we use the strictest validation that works for all providers:

   | Provider | Naming Standard    | Periods Allowed |
   | -------- | ------------------ | --------------- |
   | LXD      | RFC 1123 (strict)  | ❌ No           |
   | Hetzner  | RFC 1123 (relaxed) | ✅ Yes          |

   **Decision**: Keep the current `InstanceName` validation (1-63 chars, ASCII alphanumeric + dashes, no leading digit/dash, no trailing dash). This is a subset of what Hetzner allows, so names valid for our tool will work on all providers.

   **Rationale**:

   - Periods in server names are rarely needed in practice
   - One validation rule is simpler to understand and maintain
   - Forward-compatible: adding providers later won't require tightening rules
   - Users can still use descriptive names like `torrust-tracker-prod-01`

6. **Rust-based validation (MVP)**: For the initial implementation, validation is handled entirely by Rust's serde deserialization. The `ProviderConfig` enum uses Serde's tagged enum pattern which provides clear error messages when the JSON structure doesn't match expected types.

7. **JSON Schema generation (Future Enhancement)**: After the MVP is complete, we may add JSON Schema generation using the [`schemars`](https://crates.io/crates/schemars) crate. This would provide:

   - IDE autocomplete and validation in VS Code
   - Pre-validation before running the application
   - Schema always in sync with Rust types

   This is **not required for the MVP** - Rust validation is sufficient for the initial release. Schema versioning could also be added at that point if needed, but since this is a single-use deployment tool (installations take hours, not months), version compatibility is not critical.

## 🔧 Implementation Details

### Phase 1: Make LXD Explicit

#### Changes to Environment Configuration

**Current `user_inputs.rs`:**

```rust
pub struct UserInputs {
    pub name: EnvironmentName,
    pub instance_name: InstanceName,    // Generic - used by all providers
    pub profile_name: ProfileName,      // LXD-specific - will move to provider config
    pub ssh_credentials: SshCredentials,
    pub ssh_port: u16,
}
```

**Proposed change:**

```rust
pub struct UserInputs {
    pub name: EnvironmentName,
    pub instance_name: InstanceName,     // Generic - all providers need a VM name
    pub provider_config: ProviderConfig, // Provider-specific config
    pub ssh_credentials: SshCredentials,
    pub ssh_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderConfig {
    #[serde(rename = "lxd")]
    Lxd(LxdConfig),
    #[serde(rename = "hetzner")]
    Hetzner(HetznerConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LxdConfig {
    pub profile_name: ProfileName,  // LXD-specific
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HetznerConfig {
    pub api_token: String,    // Stored in config (entire config is sensitive)
    pub server_type: String,  // e.g., "cx22"
    pub location: String,     // e.g., "nbg1"
    // Note: image is configured in the OpenTofu template (variables.tfvars.tera), not here.
    // This matches the LXD provider pattern.
}
```

#### Changes to Environment JSON

**Current format:**

```json
{
  "name": "my-env",
  "instance_name": "torrust-tracker-vm-my-env",
  "profile_name": "torrust-profile-my-env",
  "ssh_credentials": { ... },
  "ssh_port": 22
}
```

**Proposed format (Phase 1 - LXD):**

```json
{
  "name": "my-env",
  "instance_name": "torrust-tracker-vm-my-env",
  "provider_config": {
    "provider": "lxd",
    "profile_name": "torrust-profile-my-env"
  },
  "ssh_credentials": { ... },
  "ssh_port": 22
}
```

**Note**: No default provider - users must always specify `provider` explicitly. This is clearer and maintains flexibility for future testing scenarios.

#### Changes to TofuTemplateRenderer

The `TofuTemplateRenderer` needs to be parameterized by provider:

```rust
impl TofuTemplateRenderer {
    // Current: hardcoded "tofu/lxd"
    const OPENTOFU_BUILD_PATH: &'static str = "tofu/lxd";

    // Proposed: dynamic based on provider
    fn opentofu_build_path(&self) -> String {
        format!("tofu/{}", self.provider.as_str())
    }
}
```

### Phase 2: Add Hetzner Provider

#### OpenTofu Output Contract

**CRITICAL**: There is a contract between Rust code and OpenTofu outputs that ALL provider implementations must follow.

The Rust code in `src/adapters/tofu/json_parser.rs` parses the output of `tofu output -json` and expects a specific structure. Every provider's OpenTofu configuration **must** include an `instance_info` output with these exact fields:

| Field        | Type   | Description                     | Example                |
| ------------ | ------ | ------------------------------- | ---------------------- |
| `name`       | string | Instance/server name            | `"torrust-tracker-vm"` |
| `image`      | string | OS image used                   | `"ubuntu:24.04"`       |
| `status`     | string | Current instance status         | `"Running"`            |
| `ip_address` | string | IPv4 address (must be valid IP) | `"10.140.190.68"`      |

**Required OpenTofu output structure:**

```hcl
output "instance_info" {
  description = "Information about the created instance"
  value = {
    name       = <instance_name>
    image      = <image_name>
    status     = <instance_status>
    ip_address = <ipv4_address>
  }
}
```

**JSON output format** (from `tofu output -json`):

```json
{
  "instance_info": {
    "value": {
      "name": "torrust-tracker-vm",
      "image": "ubuntu:24.04",
      "ip_address": "10.140.190.68",
      "status": "Running"
    }
  }
}
```

See the LXD implementation in `templates/tofu/lxd/main.tf` for reference. The Hetzner implementation must follow the same contract.

#### Discovering Valid Hetzner Values

Before implementing typed validation for Hetzner configuration, use the Hetzner API or CLI (`hcloud`) to get current valid values. These values change over time as Hetzner adds/removes server types, locations, and images.

```bash
# Install hcloud CLI (if not installed)
# See: https://github.com/hetznercloud/cli

# List available server types
hcloud server-type list

# List available locations (datacenters)
hcloud location list

# List available images
hcloud image list --type system
```

**API alternatives** (no authentication needed for public endpoints):

```bash
# Server types
curl -s https://api.hetzner.cloud/v1/server_types | jq '.server_types[].name'

# Locations
curl -s https://api.hetzner.cloud/v1/locations | jq '.locations[].name'

# Images (requires auth)
curl -s -H "Authorization: Bearer $HCLOUD_TOKEN" \
  https://api.hetzner.cloud/v1/images?type=system | jq '.images[].name'
```

This approach ensures we use accurate, up-to-date values in our OpenTofu validation rules and documentation.

#### Hetzner Provider Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HetznerConfig {
    pub api_token: String,          // Hetzner API token (stored in config)
    pub server_type: String,        // e.g., "cx11", "cx21"
    pub location: String,           // e.g., "fsn1", "nbg1"
    // Note: image is configured in the OpenTofu template, not here.
}
```

**Environment JSON for Hetzner:**

```json
{
  "name": "production",
  "instance_name": "torrust-tracker-demo",
  "provider_config": {
    "provider": "hetzner",
    "api_token": "your-hetzner-api-token-here",
    "server_type": "cx21",
    "location": "fsn1"
  },
  "ssh_credentials": { ... },
  "ssh_port": 22
}
```

**Note**: The API token is stored in the environment JSON. The entire local configuration is considered sensitive. Users should never share their `data/`, `envs/`, or `build/` directories.

#### Hetzner OpenTofu Templates

**`templates/tofu/hetzner/versions.tf`:**

```hcl
# Hetzner Provider Version Requirements
# Based on: https://github.com/torrust/torrust-tracker-deploy-bash-poc/tree/main/infrastructure/terraform/providers/hetzner

terraform {
  required_version = ">= 1.0"

  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.47"
    }
    time = {
      source  = "hashicorp/time"
      version = "~> 0.11"
    }
  }
}
```

**`templates/tofu/hetzner/main.tf`:**

```hcl
# Hetzner Cloud Provider Implementation
# Note: We use OS-level firewall (UFW) instead of Hetzner Cloud Firewall for portability.
# This allows the firewall configuration to move with the VM if migrating to another provider.

provider "hcloud" {
  token = var.hcloud_token
}

# Server Resource
resource "hcloud_server" "torrust_server" {
  name         = var.instance_name
  image        = var.hetzner_image
  server_type  = var.hetzner_server_type
  location     = var.hetzner_location

  # SSH keys are injected via cloud-init (user_data) for statelessness.
  # We do not create hcloud_ssh_key resources to avoid leaving keys in the project.
  user_data = file("${path.module}/cloud-init.yml")

  labels = {
    environment = "deployment"
    purpose     = "torrust-tracker"
    managed_by  = "opentofu"
  }

  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }

  lifecycle {
    prevent_destroy = false  # Set to true for production
  }
}

# Wait for server to be ready
resource "time_sleep" "wait_for_server" {
  depends_on = [hcloud_server.torrust_server]
  create_duration = "30s"
}

# Data source to get server info after creation
data "hcloud_server" "torrust_server" {
  depends_on = [time_sleep.wait_for_server]
  id         = hcloud_server.torrust_server.id
}

# IMPORTANT: This output is parsed by src/adapters/tofu/json_parser.rs
# The output name "instance_info" and all fields must remain present with these exact names.
output "instance_info" {
  description = "Information about the created server"
  value = {
    name       = hcloud_server.torrust_server.name
    image      = var.hetzner_image
    status     = try(data.hcloud_server.torrust_server.status, hcloud_server.torrust_server.status, "unknown")
    ip_address = try(data.hcloud_server.torrust_server.ipv4_address, hcloud_server.torrust_server.ipv4_address, "pending")
  }
}

# Additional debugging outputs
output "server_id" {
  description = "Hetzner server ID"
  value       = hcloud_server.torrust_server.id
}

output "ipv6_address" {
  description = "IPv6 address of the server"
  value       = try(data.hcloud_server.torrust_server.ipv6_address, hcloud_server.torrust_server.ipv6_address, "No IPv6 assigned")
}

output "connection_info" {
  description = "SSH connection command"
  value = try(
    data.hcloud_server.torrust_server.ipv4_address != "" ?
      "SSH: ssh torrust@${data.hcloud_server.torrust_server.ipv4_address}" :
      "VM created, waiting for IP address...",
    "VM created, waiting for IP address..."
  )
}
```

**`templates/tofu/hetzner/variables.tf`:**

```hcl
# Hetzner Provider Variables

variable "hcloud_token" {
  description = "Hetzner Cloud API token"
  type        = string
  sensitive   = true
}

variable "instance_name" {
  description = "Name of the Hetzner server (matches Domain InstanceName)"
  type        = string
}

variable "hetzner_server_type" {
  description = "Hetzner server type (e.g. cx22, cpx11, etc.)"
  type        = string
  default     = "cx22"
}

variable "hetzner_location" {
  description = "Hetzner datacenter location (e.g. nbg1, fsn1, ash, etc.)"
  type        = string
  default     = "nbg1"
}

variable "hetzner_image" {
  description = "Hetzner server image"
  type        = string
  default     = "ubuntu-24.04"

  validation {
    condition     = var.hetzner_image == "ubuntu-24.04"
    error_message = "Only ubuntu-24.04 is supported. This matches the LXD provider configuration."
  }
}
```

**`templates/tofu/hetzner/variables.tfvars.tera`:**

```hcl
# Generated by Torrust Tracker Deployer
# Hetzner Cloud provider variables

hcloud_token        = "{{ api_token }}"
instance_name       = "{{ instance_name }}"
hetzner_server_type = "{{ server_type }}"
hetzner_location    = "{{ location }}"
hetzner_image       = "ubuntu-24.04"
```

**Note**: The API token is written to the tfvars file. This file is in the build directory which should be treated as sensitive.

### Component Changes Summary

| Component                 | Phase 1 Changes                             | Phase 2 Changes                          |
| ------------------------- | ------------------------------------------- | ---------------------------------------- |
| `UserInputs`              | Add `provider` and `provider_config` fields | Add `HetznerConfig` variant              |
| `Environment`             | Update accessors for provider config        | Add Hetzner-specific accessors           |
| `TofuTemplateRenderer`    | Parameterize by provider                    | No additional changes                    |
| Templates                 | Move LXD templates, update paths            | Add Hetzner templates                    |
| `ProvisionCommandHandler` | Pass provider to renderer                   | No additional changes                    |
| Tests                     | Update E2E test configs                     | Add Hetzner integration tests (optional) |

## 📊 Impact Analysis

### Files to Modify

| File Path                                                         | Changes Required                  | Effort |
| ----------------------------------------------------------------- | --------------------------------- | ------ |
| `src/domain/environment/user_inputs.rs`                           | Add provider enum and config      | Medium |
| `src/domain/environment/mod.rs`                                   | Add provider accessors            | Low    |
| `src/infrastructure/external_tools/tofu/template/renderer/mod.rs` | Parameterize by provider          | Medium |
| `templates/tofu/lxd/*`                                            | No changes (path already correct) | None   |
| `templates/tofu/hetzner/*`                                        | Create new directory and files    | Medium |
| `data/*/environment.json`                                         | Update to new format              | Low    |
| `docs/user-guide/*`                                               | Document provider selection       | Low    |

### Breaking Changes

- **Environment JSON format changes**: Existing environment files will need to be updated to include the `provider_config` field
- **No backward compatibility needed**: The deployer is not in production use yet - there are no external users. We can make breaking changes freely.
- **Mitigation**: Clear error message when provider config is missing, with instructions to update

### Performance Impact

None expected. Provider selection is a one-time decision at environment creation.

### Security Considerations

- **Hetzner API token**: Stored in environment JSON file (plaintext). The entire local configuration is considered sensitive and should not be shared. Users are responsible for securing their local deployment data.
- **SSH keys**: Same handling as LXD (user provides paths)
- **Single-use philosophy**: This tool is for initial deployment only, not ongoing management. Treat all generated files as sensitive.

## 🗓️ Implementation Plan

### Phase 1: Make LXD Explicit

**Estimated Duration**: 2-3 days

- [ ] Add `Provider` enum to domain layer
- [ ] Add `ProviderConfig` enum with `LxdConfig` variant
- [ ] Update `UserInputs` to use new structure
- [ ] Update `TofuTemplateRenderer` to use provider path
- [ ] Update environment JSON files in `data/`
- [ ] Update E2E tests to use new config format
- [ ] Update documentation

### Phase 2: Add Hetzner Provider

**Estimated Duration**: 3-4 days

**Recommended Workflow - OpenTofu First**:

To avoid mixing two types of problems (bad Rust code vs bad OpenTofu config), we validate the OpenTofu configuration in isolation before integrating with Rust:

1. **Create concrete OpenTofu config**: Write static `.tf` files directly in a test build directory (e.g., `build/hetzner-test/tofu/hetzner/`) - no Tera templates yet
2. **Test with raw OpenTofu commands**: Run `tofu init`, `tofu plan`, `tofu apply`, `tofu destroy` manually to verify the configuration works with real Hetzner infrastructure
3. **Once validated**: Convert the working `.tf` files to Tera templates and integrate with Rust code

This approach:

- Isolates infrastructure issues from code issues
- Makes debugging much easier
- Provides confidence that templates are correct before adding complexity
- Creates a reference implementation to compare against

**Implementation Tasks**:

- [ ] Create static Hetzner OpenTofu config and test manually
- [ ] Add `HetznerConfig` to `ProviderConfig` enum
- [ ] Convert working OpenTofu files to Tera templates
- [ ] Add Hetzner variables template wrapper
- [ ] Create example Hetzner environment configuration
- [ ] Add Hetzner-specific documentation
- [ ] Manual testing with real Hetzner account
- [ ] Add mock/unit tests for Hetzner config handling

### Phase 3: Documentation and Polish

**Estimated Duration**: 1-2 days

- [ ] User guide for Hetzner setup
- [ ] Provider comparison documentation
- [ ] Cost estimation guidance
- [ ] Troubleshooting guide

## ✅ Definition of Done

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

## 🧪 Testing Strategy

### Approach

- **Automated tests**: Use LXD provider in E2E tests (fast, free, works in CI)
- **Manual testing**: Hetzner requires user-provided API token - cannot run in CI
- **Unit tests**: Cover provider configuration parsing and validation

### Unit Tests

Test provider configuration parsing and validation:

```rust
#[test]
fn it_should_parse_lxd_provider_config() {
    let json = r#"{
        "name": "test-env",
        "instance_name": "test-vm",
        "provider_config": {
            "provider": "lxd",
            "profile_name": "test-profile"
        },
        "ssh_credentials": { ... }
    }"#;

    let inputs: UserInputs = serde_json::from_str(json).unwrap();
    assert!(matches!(inputs.provider_config, ProviderConfig::Lxd(_)));
}

#[test]
fn it_should_parse_hetzner_provider_config() {
    let json = r#"{
        "name": "test-env",
        "instance_name": "test-server",
        "provider_config": {
            "provider": "hetzner",
            "api_token": "test-token",
            "server_type": "cx11",
            "location": "fsn1"
        },
        "ssh_credentials": { ... }
    }"#;

    let inputs: UserInputs = serde_json::from_str(json).unwrap();
    assert!(matches!(inputs.provider_config, ProviderConfig::Hetzner(_)));
}
```

### Integration Tests

- Template rendering produces valid OpenTofu configuration for each provider
- Provider-specific template variables are correctly substituted

### Manual Testing (Hetzner)

1. Create Hetzner account and API token
2. Create environment JSON with `provider: "hetzner"` and your API token
3. Run `create environment --env-file <path>` command
4. Verify VM is created in Hetzner console
5. Verify Docker is installed and Tracker is running
6. Run `destroy` command
7. Verify VM is deleted in Hetzner console

## 📚 Related Documentation

- [Development Principles](../../development-principles.md)
- [DDD Layer Placement](../../contributing/ddd-layer-placement.md)
- [Error Handling Guidelines](../../contributing/error-handling.md)
- [VM Providers Comparison](../../vm-providers.md)
- [LXD VM Decision Record](../../decisions/lxd-vm-over-containers.md)

## 🔗 References

- [Hetzner Cloud Documentation](https://docs.hetzner.com/cloud/)
- [Hetzner Terraform Provider](https://registry.terraform.io/providers/hetznercloud/hcloud/latest/docs)
- [OpenTofu Documentation](https://opentofu.org/docs/)
- [Cloud-init Documentation](https://cloudinit.readthedocs.io/)

---

**Created**: December 1, 2025
**Last Updated**: December 1, 2025
**Status**: Planning Complete - Ready for Implementation

**Important Context**: This is a **single-use deployment tool**, not a management platform or sysadmin replacement. The tool automates initial Torrust Tracker deployment but is not intended for ongoing infrastructure management. Once deployment is complete, users are responsible for managing their infrastructure. The entire local configuration (`data/`, `envs/`, `build/` directories) is considered sensitive.
