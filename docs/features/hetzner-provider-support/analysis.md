# Technical Analysis: Current LXD Provider Implementation

This document analyzes the current LXD provider implementation to understand how it can be refactored to support multiple providers.

## 📋 Executive Summary

The current implementation has LXD deeply embedded but is relatively well-isolated. The main integration points are:

1. **Environment Configuration** (`UserInputs`): Common fields (`instance_name`) plus provider-specific config
2. **OpenTofu Templates**: Hardcoded path to `tofu/lxd/`
3. **Template Renderer**: Hardcoded provider path constants
4. **LXD Client**: Direct LXD API interaction (can remain LXD-specific)
5. **Domain Types**: `InstanceName` (generic across providers), `ProfileName` (LXD-specific)

The recommended approach is to:

1. Add a `ProviderConfig` enum with provider-specific variants (only extra fields per provider)
2. Keep `instance_name` as a **global field** in `UserInputs` (all providers need a VM name)
3. Move `profile_name` to **LXD-specific config** (Hetzner doesn't have profiles)
4. Parameterize the template renderer by provider
5. Keep provider-specific clients separate (no common interface)
6. Update both Domain (`UserInputs`) and Application (`EnvironmentCreationConfig`) layers

## 1. Current LXD Integration Points

### 1.1 Environment Configuration (`UserInputs`)

**Location**: `src/domain/environment/user_inputs.rs`

```rust
pub struct UserInputs {
    pub name: EnvironmentName,
    pub instance_name: InstanceName,    // Global - all providers need a VM name
    pub profile_name: ProfileName,      // Global - LXD uses it, others ignore it
    pub ssh_credentials: SshCredentials,
    pub ssh_port: u16,
}
```

**Analysis**:

- `instance_name`: **Global field** - all providers need a name for the VM/server
- `profile_name`: **LXD-specific** - move to LXD provider config (Hetzner doesn't have profiles)
- Auto-generation in `generate_instance_name()` works for all providers

**Refactoring Strategy** (based on user feedback):

- Keep `instance_name` as a **global field** in `UserInputs`
- Move `profile_name` to **LXD-specific config** inside `ProviderConfig::Lxd`
- Add `provider_config: ProviderConfig` field for provider-specific data
- `ProviderConfig::Lxd(LxdConfig)` contains `profile_name`
- `ProviderConfig::Hetzner(HetznerConfig)` contains server_type, location, image, and API token

### 1.2 OpenTofu Template Renderer

**Location**: `src/infrastructure/external_tools/tofu/template/renderer/mod.rs`

```rust
impl TofuTemplateRenderer {
    const OPENTOFU_BUILD_PATH: &'static str = "tofu/lxd";
    const OPENTOFU_TEMPLATE_PATH: &'static str = "tofu/lxd";

    // ... uses these constants throughout
}
```

**Analysis**:

- Hardcoded paths to `tofu/lxd/` directory
- Template files: `main.tf`, `cloud-init.yml.tera`, `variables.tfvars.tera`
- Variables template uses LXD-specific context (`InstanceName`, `ProfileName`)

**Refactoring Strategy**:

- Add `provider` field to `TofuTemplateRenderer`
- Create `fn opentofu_path(&self) -> String` that returns provider-specific path
- Create provider-specific context builders for variables template

### 1.3 LXD Client

**Location**: `src/adapters/lxd/client.rs`

```rust
pub struct LxdClient {
    command_executor: CommandExecutor,
}

impl LxdClient {
    pub fn get_instance_ip(&self, instance_name: &InstanceName) -> Result<Option<IpAddr>>
    pub fn wait_for_instance_ip(&self, ...) -> Result<IpAddr>
    pub fn delete_instance(&self, ...) -> Result<()>
    pub fn delete_profile(&self, ...) -> Result<()>
}
```

**Analysis**:

- Direct interaction with `lxc` command-line tool
- Methods are LXD-specific (profiles, etc.)
- Only used for post-provisioning checks and cleanup

**Refactoring Strategy**:

- Keep as-is for LXD
- Create separate `HetznerClient` for Hetzner-specific operations
- No common provider interface needed (OpenTofu handles provisioning)

### 1.4 OpenTofu Templates

**Location**: `templates/tofu/lxd/`

Files:

- `main.tf` - LXD provider configuration and resources
- `cloud-init.yml.tera` - Cloud-init template (mostly provider-agnostic)
- `variables.tfvars.tera` - LXD-specific variables

**Analysis**:

The `main.tf` is completely LXD-specific:

```hcl
terraform {
  required_providers {
    lxd = {
      source  = "terraform-lxd/lxd"
      version = "~> 2.0"
    }
  }
}

resource "lxd_profile" "torrust_profile" { ... }
resource "lxd_instance" "torrust_vm" { ... }
```

**Refactoring Strategy**:

- Create `templates/tofu/hetzner/` with Hetzner-specific templates
- Keep cloud-init template largely shared (copy with minor adjustments)
- Each provider has its own `main.tf` and `variables.tfvars.tera`

### 1.5 Domain Types

**Location**: `src/domain/instance_name.rs`, `src/domain/profile_name.rs`

**Analysis**:

- `InstanceName`: Validates names for LXD compatibility (63 chars, ASCII alphanumeric + dash)

  - **Hetzner compatibility**: Hetzner server names have similar constraints
  - **Global field** - reused for all providers

- `ProfileName`: LXD-specific concept
  - Hetzner doesn't have profiles
  - **LXD-specific field** - moved to `ProviderConfig::Lxd(LxdConfig)` (not a global field)

### 1.6 Environment Context

**Location**: `src/domain/environment/context.rs`

```rust
impl EnvironmentContext {
    pub fn tofu_build_dir(&self) -> PathBuf {
        self.internal_config.build_dir.join(TOFU_DIR_NAME).join(LXD_PROVIDER_NAME)
    }
}
```

**Analysis**:

- `tofu_build_dir()` hardcodes "lxd" in path via `LXD_PROVIDER_NAME` constant
- Other directory methods are provider-agnostic

**Refactoring Strategy**:

- Make `tofu_build_dir()` accept provider parameter
- Or store provider in context and use it dynamically

## 2. Data Flow Analysis

### 2.1 Provision Command Flow

```text
┌──────────────────────────────────────────────────────────────────────────┐
│                     Current Provision Flow                               │
└──────────────────────────────────────────────────────────────────────────┘

User creates environment.json
         │
         ▼
┌─────────────────────┐
│ create environment  │  - Validates user inputs
│ command             │  - Creates Environment<Created>
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ provision command   │  - Loads Environment<Created>
│ controller          │  - Delegates to handler
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ ProvisionCommand    │  - Creates TofuTemplateRenderer
│ Handler             │  - Creates OpenTofuClient
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ TofuTemplateRenderer│  - Renders templates to build/env/tofu/lxd/
│                     │  - Uses hardcoded "tofu/lxd" path
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ OpenTofuClient      │  - Runs tofu init/plan/apply
│                     │  - Retrieves instance info from output
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ LxdClient           │  - Waits for instance IP (via lxc list)
│ (optional)          │  - Could use tofu output instead
└─────────────────────┘
```

### 2.2 Proposed Multi-Provider Flow

```text
┌──────────────────────────────────────────────────────────────────────────┐
│                    Proposed Multi-Provider Flow                          │
└──────────────────────────────────────────────────────────────────────────┘

User creates environment.json with "provider": "lxd" | "hetzner"
         │
         ▼
┌─────────────────────┐
│ create environment  │  - Validates provider and provider_config
│ command             │  - Creates Environment<Created> with provider
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ provision command   │  - Loads Environment<Created>
│ controller          │  - Provider-aware delegation
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ ProvisionCommand    │  - Reads provider from environment
│ Handler             │  - Creates TofuTemplateRenderer with provider
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ TofuTemplateRenderer│  - Selects template dir: tofu/{provider}/
│ (provider-aware)    │  - Uses provider-specific variable context
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│ OpenTofuClient      │  - Same as before
│                     │  - Works with any provider's TF files
└─────────────────────┘
         │
         ▼
┌─────────────────────┐      ┌──────────────────────┐
│ LxdClient           │  OR  │ (No extra client)    │
│ (if provider=lxd)   │      │ (if provider=hetzner)│
└─────────────────────┘      └──────────────────────┘
```

## 3. Proposed Changes by Component

### 3.1 Domain Layer Changes

#### New Types (`src/domain/provider.rs`)

```rust
//! Infrastructure provider types
//!
//! This module defines the supported infrastructure providers and their
//! specific configurations. Global fields (instance_name, profile_name) are
//! kept in UserInputs; only provider-specific extras go in ProviderConfig.

use serde::{Deserialize, Serialize};

/// Provider-specific configuration
///
/// Each variant contains the fields specific to that provider.
/// Only `instance_name` is a global field in UserInputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "provider")]
pub enum ProviderConfig {
    /// LXD provider - requires profile_name
    #[serde(rename = "lxd")]
    Lxd(LxdConfig),

    /// Hetzner provider - requires server type, location, image, and API token
    #[serde(rename = "hetzner")]
    Hetzner(HetznerConfig),
}

/// LXD-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LxdConfig {
    /// LXD profile name for the instance
    pub profile_name: ProfileName,
}

impl ProviderConfig {
    /// Returns the provider name as used in directory paths
    #[must_use]
    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Lxd => "lxd",
            Self::Hetzner(_) => "hetzner",
        }
    }
}

/// Hetzner-specific configuration
///
/// Contains the fields specific to Hetzner Cloud.
/// The server name comes from the global `instance_name` field in UserInputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HetznerConfig {
    /// Hetzner API token (stored in config - entire local config is considered sensitive)
    pub api_token: String,
    /// Hetzner server type (e.g., "cx22", "cx32", "cpx11")
    pub server_type: String,
    /// Hetzner datacenter location (e.g., "fsn1", "nbg1", "hel1")
    pub location: String,
    // Note: image is configured in the OpenTofu template, not here.
}
```

#### Updated `UserInputs` (Domain Layer)

```rust
pub struct UserInputs {
    pub name: EnvironmentName,
    pub instance_name: InstanceName,      // GLOBAL - all providers use this
    pub provider_config: ProviderConfig,  // Provider-specific config (includes profile_name for LXD)
    pub ssh_credentials: SshCredentials,
    pub ssh_port: u16,
}

impl UserInputs {
    /// Returns the provider name as used in directory paths
    pub fn provider_name(&self) -> &'static str {
        self.provider_config.provider_name()
    }
}
```

#### Updated `EnvironmentCreationConfig` (Application Layer DTO)

**Location**: `src/application/command_handlers/create/config/environment_config.rs`

The Application layer DTO must also be updated to include the provider configuration:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentCreationConfig {
    /// Environment-specific settings
    pub environment: EnvironmentSection,

    /// SSH credentials configuration
    pub ssh_credentials: SshCredentialsConfig,

    /// Provider-specific configuration (NEW)
    pub provider: ProviderConfigDto,
}

/// DTO for provider configuration (mirrors domain ProviderConfig)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderConfigDto {
    #[serde(rename = "lxd")]
    Lxd(LxdConfigDto),

    #[serde(rename = "hetzner")]
    Hetzner(HetznerConfigDto),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LxdConfigDto {
    pub profile_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HetznerConfigDto {
    pub api_token: String,
    pub server_type: String,
    pub location: String,
}
```

### 3.2 Infrastructure Layer Changes

#### Updated `TofuTemplateRenderer`

```rust
pub struct TofuTemplateRenderer {
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
    provider: Provider,  // NEW
    ssh_credentials: SshCredentials,
    cloud_init_renderer: CloudInitTemplateRenderer,
    provider_config: ProviderConfig,  // NEW: replaces instance_name, profile_name
}

impl TofuTemplateRenderer {
    /// Returns the OpenTofu build path for the current provider
    fn opentofu_build_path(&self) -> PathBuf {
        PathBuf::from("tofu").join(self.provider.as_str())
    }

    /// Returns the OpenTofu template path for the current provider
    fn opentofu_template_path(&self) -> String {
        format!("tofu/{}", self.provider.as_str())
    }
}
```

### 3.3 New Hetzner Templates

**`templates/tofu/hetzner/main.tf`**:

```hcl
terraform {
  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.45"
    }
  }
  required_version = ">= 1.0"
}

provider "hcloud" {
  # Token is read from HCLOUD_TOKEN environment variable
provider "hcloud" {
  # Token is read from HCLOUD_TOKEN environment variable
}

variable "instance_name" {
  description = "Name of the Hetzner server"
  type        = string
}

variable "server_type" {
  description = "Hetzner server type"
  type        = string
  default     = "cx21"
}

variable "location" {
  description = "Hetzner datacenter location"
  type        = string
  default     = "fsn1"
}

variable "image" {
  description = "OS image to use"
  type        = string
  default     = "ubuntu-24.04"
}

# SSH keys are injected via cloud-init (user_data) for statelessness.
# We do not create hcloud_ssh_key resources to avoid leaving keys in the project.

resource "hcloud_server" "torrust" {
  name        = var.instance_name
  server_type = var.server_type
  location    = var.location
  image       = var.image

  user_data = file("${path.module}/cloud-init.yml")

  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }
}
  description = "Information about the created server"
  value = {
    name       = hcloud_server.torrust.name
    image      = var.image
    status     = hcloud_server.torrust.status
    ip_address = hcloud_server.torrust.ipv4_address
  }
}
```

## 4. Migration Considerations

### 4.1 Environment JSON Migration

**Before (current format):**

```json
{
  "Created": {
    "context": {
      "user_inputs": {
        "name": "e2e-full",
        "instance_name": "torrust-tracker-vm-e2e-full",
        "profile_name": "torrust-profile-e2e-full",
        "ssh_credentials": { ... },
        "ssh_port": 22
      },
      "internal_config": { ... },
      "runtime_outputs": { ... }
    },
    "state": null
  }
}
```

**After (new format for LXD):**

```json
{
  "Created": {
    "context": {
      "user_inputs": {
        "name": "e2e-full",
        "instance_name": "torrust-tracker-vm-e2e-full",
        "provider_config": {
          "provider": "lxd",
          "profile_name": "torrust-profile-e2e-full"
        },
        "ssh_credentials": { ... },
        "ssh_port": 22
      },
      "internal_config": { ... },
      "runtime_outputs": { ... }
    },
    "state": null
  }
}
```

**After (new format for Hetzner):**

````json
{
  "Created": {
    "context": {
      "user_inputs": {
        "name": "production",
        "instance_name": "torrust-tracker-prod",
**After (new format for Hetzner):**

```json
{
  "Created": {
    "context": {
      "user_inputs": {
        "name": "production",
        "instance_name": "torrust-tracker-prod",
        "provider_config": {
          "provider": "hetzner",
          "api_token": "your-hetzner-api-token",
          "server_type": "cx22",
          "location": "nbg1"
        },
        "ssh_credentials": { ... },
        "ssh_port": 22
      },
      "internal_config": { ... },
      "runtime_outputs": { ... }
    },
    "state": null
  }
}
```

**Note**: The `instance_name` is a global field (all providers need a VM name). The `profile_name` is LXD-specific and moved inside `provider_config`. Hetzner API token is stored in `provider_config` (entire local config is considered sensitive).

### 4.2 Backward Compatibility Options

1. **Option A: Breaking change** (recommended for now)

   - Require new format
   - Clear error message with migration instructions
   - Simple implementation

2. **Option B: Auto-migration**
   - Detect old format (no `provider_config` field)
   - Automatically convert to LXD provider
   - More complex, potential bugs

### 4.3 Test Configuration Updates

All test environment files in `data/` will need updating:

- `data/e2e-config/environment.json`
- `data/e2e-config-new/environment.json`
- `data/e2e-full/environment.json`
- `data/e2e-provision/environment.json`

## 5. Risk Assessment

### 5.1 Low Risk

- Adding `Provider` enum and `ProviderConfig` types
- Creating new Hetzner templates
- Adding provider parameter to `TofuTemplateRenderer`

### 5.2 Medium Risk

- Changing `UserInputs` structure (affects serialization)
- Updating all test configurations
- Maintaining backward compatibility

### 5.3 Mitigation Strategies

1. **Comprehensive test coverage**: Ensure all E2E tests pass after changes
2. **Clear error messages**: Guide users through configuration updates
3. **Documentation**: Update all docs before releasing changes
4. **Phased rollout**: Release Phase 1 (LXD explicit) before Phase 2 (Hetzner)

## 6. Reference Implementation: Bash PoC

The Torrust project has an existing Hetzner implementation in the bash-based PoC repository:

- **Repository**: [torrust/torrust-tracker-deploy-bash-poc](https://github.com/torrust/torrust-tracker-deploy-bash-poc)
- **Hetzner Provider**: [infrastructure/terraform/providers/hetzner/](https://github.com/torrust/torrust-tracker-deploy-bash-poc/tree/main/infrastructure/terraform/providers/hetzner)

### 6.1 Key Components from Bash PoC

**File Structure:**

```text
providers/hetzner/
├── main.tf           # Server, firewall resources
├── variables.tf      # Input variables with validation
├── outputs.tf        # Standard output interface
└── versions.tf       # Provider version requirements
```

**Provider Requirements:**

```hcl
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
```

### 6.2 Hetzner-Specific Features from PoC

**Server Types (validated in variables.tf):**

| Type | vCPUs | RAM  | Monthly Cost |
| ---- | ----- | ---- | ------------ |
| cx11 | 1     | 2GB  | ~€4          |
| cx21 | 2     | 4GB  | ~€6          |
| cx31 | 2     | 8GB  | ~€12         |
| cx41 | 4     | 16GB | ~€22         |
| cx51 | 8     | 32GB | ~€42         |

**Datacenter Locations:**

- `nbg1` - Nuremberg, Germany (default)
- `fsn1` - Falkenstein, Germany
- `hel1` - Helsinki, Finland
- `ash` - Ashburn, VA, USA
- `hil` - Hillsboro, OR, USA

### 6.3 Firewall Configuration

The PoC includes a pre-configured firewall with all required Torrust Tracker ports:

| Port | Protocol | Purpose      |
| ---- | -------- | ------------ |
| 22   | TCP      | SSH          |
| 80   | TCP      | HTTP         |
| 443  | TCP      | HTTPS        |
| 6868 | UDP      | Tracker UDP  |
| 6969 | UDP      | Tracker UDP  |
| 7070 | TCP      | Tracker HTTP |
| 1212 | TCP      | API/Metrics  |

### 6.4 Output Interface Compatibility

The PoC uses a standard output interface that matches our `instance_info` output:

```hcl
output "instance_info" {
  value = {
    name       = hcloud_server.torrust_server.name
    image      = var.hetzner_image
    status     = hcloud_server.torrust_server.status
    ip_address = hcloud_server.torrust_server.ipv4_address
  }
}
```

This is compatible with our existing `src/adapters/tofu/json_parser.rs` parser.

### 6.5 SSH Key Handling Difference

**Important**: The PoC handles SSH keys via cloud-init (no separate `hcloud_ssh_key` resource). The specification adopts this approach for statelessness and simplicity.

### 6.6 Lessons Learned from PoC

1. **Time provider needed**: Use `hashicorp/time` provider for `time_sleep` resource to wait for server readiness
2. **Firewall first**: Create firewall resource before server and attach via `firewall_ids`
3. **Labels**: Add `managed_by = "opentofu"` label for resource identification
4. **Data source refresh**: Use `data.hcloud_server` after wait to get accurate IP address
5. **Token via environment**: Hetzner token passed via `HCLOUD_TOKEN` env var, not in tfvars

## 7. Conclusion

The current implementation is reasonably well-structured for adding multi-provider support. The main changes are:

1. **Domain layer**: Add `Provider` enum and `ProviderConfig` types, update `UserInputs`
2. **Infrastructure layer**: Parameterize `TofuTemplateRenderer` by provider
3. **Templates**: Create Hetzner-specific OpenTofu templates
4. **Configuration**: Update environment JSON format

The "refactor first" approach (Phase 1) will validate the architecture before adding Hetzner-specific code (Phase 2), reducing risk and simplifying code review.

---

**Created**: December 1, 2025
````
