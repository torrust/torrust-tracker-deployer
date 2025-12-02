# Parameterize TofuTemplateRenderer by Provider

**Issue**: #212
**Parent Epic**: #205 - Epic: Add Hetzner Provider Support
**Related**: [#206](https://github.com/torrust/torrust-tracker-deployer/issues/206), [#207](https://github.com/torrust/torrust-tracker-deployer/issues/207), [#208](https://github.com/torrust/torrust-tracker-deployer/issues/208)

## Overview

This task makes the `TofuTemplateRenderer` provider-aware so it can render templates for different infrastructure providers (LXD, Hetzner). Currently, the renderer has hardcoded paths to `tofu/lxd/` templates. After this change, it will dynamically select the template directory based on the provider specified in the environment configuration.

This is Task 4 in the Phase 1 ("Make LXD Explicit") of the Hetzner Provider Support epic.

## Goals

- [ ] Create working Hetzner OpenTofu templates and validate them manually before any Rust code changes
- [ ] Make `TofuTemplateRenderer` select template directory based on provider
- [ ] Add Hetzner-specific template wrapper for variables rendering
- [ ] Ensure backward compatibility with existing LXD workflow

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure
**Module Path**: `src/infrastructure/external_tools/tofu/template/`
**Pattern**: Template Renderer with Strategy Pattern for provider-specific rendering

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Infrastructure layer depends on domain types (`Provider`, `ProviderConfig`)
- [ ] Provider-specific template wrappers follow existing LXD pattern
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Hardcoding provider-specific paths in generic code
- ❌ Complex conditionals that grow with each new provider
- ❌ Mixing template rendering concerns across different abstractions
- ❌ Sharing templates between providers (each provider must be self-contained)
- ❌ Placing provider-specific Rust code in shared modules (use `wrappers/{provider}/` structure)

## Specifications

### Current State

The `TofuTemplateRenderer` currently has hardcoded paths:

```rust
impl TofuTemplateRenderer {
    const OPENTOFU_BUILD_PATH: &'static str = "tofu/lxd";
    const OPENTOFU_TEMPLATE_PATH: &'static str = "tofu/lxd";
    // ...
}
```

### Target State

The renderer will accept a `Provider` and dynamically determine paths:

```rust
impl TofuTemplateRenderer {
    fn opentofu_build_path(&self) -> PathBuf {
        PathBuf::from("tofu").join(self.provider.as_str())
    }

    fn opentofu_template_path(&self) -> String {
        format!("tofu/{}", self.provider.as_str())
    }
}
```

### OpenTofu Output Contract

**CRITICAL**: All provider implementations must produce the same output structure that the Rust parser expects. The `src/adapters/tofu/json_parser.rs` parses `tofu output -json` and requires:

```hcl
output "instance_info" {
  value = {
    name       = <instance_name>
    image      = <image_name>
    status     = <instance_status>
    ip_address = <ipv4_address>
  }
}
```

### Hetzner Template Structure

Based on the reference implementation in the bash PoC, create:

```text
templates/tofu/hetzner/
├── versions.tf           # Provider version requirements (hcloud ~> 1.47, time ~> 0.11)
├── main.tf               # Server resource, outputs, and variable definitions
├── variables.tfvars.tera # Tera template for variable values (generated at runtime)
└── cloud-init.yml.tera   # Cloud-init template (independent from LXD - no sharing)
```

> **Note**: The `variables.tf` file is embedded in `main.tf` or generated at runtime. Only the `variables.tfvars.tera` template is needed for the Tera rendering pipeline - it generates the final `variables.tfvars` with actual values.

### Key Design Decisions

1. **No template sharing between providers**: Each provider has its own independent templates. This makes providers easier to add, maintain, and debug. Template sharing can be analyzed in the future if beneficial, but for now isolation is preferred.
2. **No provider firewall**: Use OS-level firewall (UFW) for portability, not Hetzner Cloud Firewall
3. **SSH via cloud-init**: SSH keys are injected via cloud-init, no `hcloud_ssh_key` resource
4. **Time provider for waiting**: Use `time_sleep` resource to wait for server readiness
5. **API token in tfvars**: Token is written to `variables.tfvars` (build directory is sensitive)
6. **Provider-specific code isolation**: All Rust code specific to a provider must be in its own module (e.g., `wrappers/hetzner/`, `wrappers/lxd/`). Shared code can be in common modules, but provider-specific logic stays isolated for easier navigation and scalability.

## Implementation Plan

### Phase 1: Create and Validate Hetzner OpenTofu Templates (estimated 2-3 hours)

**Goal**: Create static OpenTofu files and test them manually with real Hetzner infrastructure before any Rust code changes. This isolates infrastructure issues from code issues.

#### Task 1.1: Create static Hetzner OpenTofu configuration

Create a test directory with static `.tf` files (no Tera templates yet):

```bash
mkdir -p build/hetzner-manual-test/tofu/hetzner
```

Create the following files based on the bash PoC reference:

**`versions.tf`**:

```hcl
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

**`main.tf`**:

```hcl
provider "hcloud" {
  token = var.hcloud_token
}

resource "hcloud_server" "torrust_server" {
  name         = var.instance_name
  image        = var.hetzner_image
  server_type  = var.hetzner_server_type
  location     = var.hetzner_location

  user_data = file("${path.module}/cloud-init.yml")

  labels = {
    environment = "test"
    purpose     = "torrust-tracker"
    managed_by  = "opentofu"
  }

  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }

  lifecycle {
    prevent_destroy = false
  }
}

resource "time_sleep" "wait_for_server" {
  depends_on      = [hcloud_server.torrust_server]
  create_duration = "30s"
}

data "hcloud_server" "torrust_server" {
  depends_on = [time_sleep.wait_for_server]
  id         = hcloud_server.torrust_server.id
}

# IMPORTANT: This output is parsed by src/adapters/tofu/json_parser.rs
output "instance_info" {
  description = "Information about the created server"
  value = {
    name       = hcloud_server.torrust_server.name
    image      = var.hetzner_image
    status     = try(data.hcloud_server.torrust_server.status, hcloud_server.torrust_server.status, "unknown")
    ip_address = try(data.hcloud_server.torrust_server.ipv4_address, hcloud_server.torrust_server.ipv4_address, "pending")
  }
}

output "server_id" {
  description = "Hetzner server ID"
  value       = hcloud_server.torrust_server.id
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

**`variables.tf`**:

```hcl
variable "hcloud_token" {
  description = "Hetzner Cloud API token"
  type        = string
  sensitive   = true
}

variable "instance_name" {
  description = "Name of the Hetzner server"
  type        = string
}

variable "hetzner_server_type" {
  description = "Hetzner server type"
  type        = string
  default     = "cx22"

  validation {
    condition = contains([
      "cx22", "cx32", "cx42", "cx52",
      "cpx11", "cpx21", "cpx31", "cpx41", "cpx51",
      "cax11", "cax21", "cax31", "cax41"
    ], var.hetzner_server_type)
    error_message = "Server type must be a valid Hetzner server type."
  }
}

variable "hetzner_location" {
  description = "Hetzner datacenter location"
  type        = string
  default     = "nbg1"

  validation {
    condition = contains([
      "nbg1", "fsn1", "hel1", "ash", "hil"
    ], var.hetzner_location)
    error_message = "Location must be a valid Hetzner datacenter."
  }
}

variable "hetzner_image" {
  description = "Hetzner server image"
  type        = string
  default     = "ubuntu-24.04"

  validation {
    condition     = var.hetzner_image == "ubuntu-24.04"
    error_message = "Only ubuntu-24.04 is supported."
  }
}
```

**`variables.tfvars`** (test values):

```hcl
hcloud_token        = "YOUR_HETZNER_API_TOKEN_HERE"
instance_name       = "torrust-test-manual"
hetzner_server_type = "cx22"
hetzner_location    = "nbg1"
hetzner_image       = "ubuntu-24.04"
```

**`cloud-init.yml`** (copy from LXD and adapt if needed):

```yaml
#cloud-config
users:
  - name: torrust
    groups: sudo, docker
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
      - YOUR_SSH_PUBLIC_KEY_HERE

package_update: true
package_upgrade: true

packages:
  - curl
  - git
  - htop

runcmd:
  - echo "Cloud-init completed at $(date)" >> /var/log/cloud-init-complete.log
```

#### Task 1.2: Test OpenTofu configuration manually

Run the following commands to validate the configuration works:

```bash
cd build/hetzner-manual-test/tofu/hetzner

# Initialize OpenTofu
tofu init

# Validate configuration
tofu validate

# Plan deployment (review what will be created)
tofu plan -var-file=variables.tfvars

# Apply configuration (creates real Hetzner server - costs money!)
tofu apply -var-file=variables.tfvars

# Verify output structure matches what Rust parser expects
tofu output -json

# Test SSH access
ssh torrust@<ip_address>

# Destroy when done
tofu destroy -var-file=variables.tfvars
```

#### Task 1.3: Document any issues and fixes

- Record any errors encountered during manual testing
- Note any changes needed to match the Rust parser expectations
- Update the static files as needed until they work correctly

### Phase 2: Integrate Hetzner Templates with Rust Code (estimated 3-4 hours)

**Goal**: Convert working static files to Tera templates and integrate with the Rust codebase.

#### Task 2.1: Create Hetzner Tera templates

Convert the validated static files to Tera templates in `templates/tofu/hetzner/`:

- [ ] Create `templates/tofu/hetzner/versions.tf` (static, no templating needed)
- [ ] Create `templates/tofu/hetzner/main.tf` (static, includes variable definitions)
- [ ] Create `templates/tofu/hetzner/variables.tfvars.tera` (Tera template for variable values)
- [ ] Create `templates/tofu/hetzner/cloud-init.yml.tera` (independent template, no sharing with LXD)

> **Important**: Do NOT share templates between providers. Each provider should have its own complete, independent template set. This ensures providers are self-contained and easier to maintain.

#### Task 2.2: Create Hetzner variables template wrapper

Create `src/infrastructure/external_tools/tofu/template/wrappers/hetzner/` module:

- [ ] Create `variables.rs` with `HetznerVariablesTemplate` and `HetznerVariablesContextBuilder`
- [ ] Create `cloud_init.rs` with `HetznerCloudInitTemplate` and `HetznerCloudInitContextBuilder`
- [ ] Create `mod.rs` to export the module
- [ ] Update parent `wrappers/mod.rs` to include `hetzner` module

> **Module Isolation Principle**: All Hetzner-specific code must live in the `wrappers/hetzner/` module. Do not mix provider-specific logic in shared modules. This makes it easy to:
>
> - Find all code related to a specific provider
> - Add new providers without modifying existing provider code
> - Maintain and debug provider-specific issues

#### Task 2.3: Refactor TofuTemplateRenderer

Update `src/infrastructure/external_tools/tofu/template/renderer/mod.rs`:

- [ ] Add `provider: Provider` field to `TofuTemplateRenderer`
- [ ] Replace hardcoded `OPENTOFU_BUILD_PATH` constant with dynamic method
- [ ] Replace hardcoded `OPENTOFU_TEMPLATE_PATH` constant with dynamic method
- [ ] Update constructor to accept `Provider` parameter
- [ ] Update `render_variables_template()` to use provider-specific wrapper

#### Task 2.4: Update callers of TofuTemplateRenderer

- [ ] Update `ProvisionCommandHandler` to pass provider to renderer
- [ ] Update any tests that create `TofuTemplateRenderer` instances

#### Task 2.5: Register static Hetzner templates

Per `docs/contributing/templates.md`, static templates must be registered:

- [ ] Update `src/infrastructure/external_tools/ansible/template/renderer/mod.rs` `copy_static_templates` method to include Hetzner static files if applicable
- [ ] Or ensure OpenTofu templates are handled by the Tofu renderer's copy mechanism

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Phase 1 Criteria (Manual Testing)**:

- [ ] Static Hetzner OpenTofu files created in test directory
- [ ] `tofu init` succeeds with Hetzner provider
- [ ] `tofu validate` passes
- [ ] `tofu plan` shows expected resources
- [ ] `tofu apply` creates Hetzner server successfully
- [ ] `tofu output -json` produces `instance_info` structure matching Rust parser expectations
- [ ] SSH access to created server works
- [ ] `tofu destroy` cleans up resources
- [ ] Any issues found are documented and resolved

**Phase 2 Criteria (Rust Integration)**:

- [ ] Hetzner Tera templates created in `templates/tofu/hetzner/`
- [ ] `HetznerVariablesTemplate` wrapper created
- [ ] `TofuTemplateRenderer` accepts `Provider` parameter
- [ ] Renderer selects correct template directory based on provider
- [ ] All existing LXD E2E tests still pass
- [ ] Unit tests cover provider selection logic
- [ ] No compiler warnings
- [ ] Code follows project conventions (DDD layers, error handling)

## Related Documentation

- [Hetzner Provider Feature Specification](../features/hetzner-provider-support/specification.md)
- [Hetzner Provider Technical Analysis](../features/hetzner-provider-support/analysis.md)
- [Epic: Add Hetzner Provider Support](./205-epic-hetzner-provider-support.md)
- [Template Contribution Guidelines](../contributing/templates.md)
- [Codebase Architecture](../codebase-architecture.md)
- [Development Principles](../development-principles.md)

## Reference Implementation

The Hetzner OpenTofu configuration in the bash PoC provides a working reference:

- **Repository**: [torrust/torrust-tracker-deploy-bash-poc](https://github.com/torrust/torrust-tracker-deploy-bash-poc)
- **Hetzner Provider**: [infrastructure/terraform/providers/hetzner/](https://github.com/torrust/torrust-tracker-deploy-bash-poc/tree/main/infrastructure/terraform/providers/hetzner)

Key files to reference:

- `main.tf` - Server and firewall resources
- `variables.tf` - Input variables with validation
- `outputs.tf` - Standard output interface (matches our parser)
- `versions.tf` - Provider version requirements

## Notes

- **Manual testing requires Hetzner API token**: Coordinate with product owner to get access
- **Cost awareness**: Hetzner servers cost money, destroy after testing
- **Sensitive files**: The `build/` directory and `variables.tfvars` contain API tokens - never commit
- **Two-phase approach**: Phase 1 validates OpenTofu works before Phase 2 adds Rust complexity
- **No template sharing**: Each provider has independent templates. While cloud-init content may be similar across providers, maintain separate files for easier provider-specific customization and maintenance

---

**Created**: December 2, 2025
**Status**: Ready for Implementation
