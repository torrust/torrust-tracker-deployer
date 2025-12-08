# Template System Architecture

Technical documentation for contributors working with the template rendering system.

## 🏗️ System Overview

The template system uses a **double indirection** approach to provide flexible infrastructure deployment while maintaining portability and customizability.

## 📦 Double Indirection Pattern

The system operates through two levels of indirection to balance portability with flexibility:

### Level 1: Embedded → External Extraction

1. **Source**: Templates are compiled into the binary as embedded resources
2. **Extraction**: On first use, templates are extracted to an external directory (e.g., `data/templates`)
3. **Benefit**: Enables single binary deployment while allowing runtime customization

### Level 2: Template → Build Directory Rendering

1. **Source**: Templates are read from the external directory
2. **Processing**: Templates are processed (static copy or dynamic rendering with variables)
3. **Output**: Final configuration files are written to the build directory
4. **Benefit**: Separates template definitions from runtime-generated configurations

## 🔄 Template Flow

```text
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Embedded        │    │ External         │    │ Build           │
│ Templates       │───▶│ Templates        │───▶│ Directory       │
│ (in binary)     │    │ (data/templates) │    │ (build/)        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
       │                        │                        │
   Compile Time            Runtime Extraction       Runtime Rendering
```

## 🎯 Template Types

### Static Templates

- **Processing**: Direct file copy from templates to build directory
- **Examples**: Infrastructure definitions, Ansible playbooks (`install-docker.yml`, `configure-security-updates.yml`)
- **Use Case**: Configuration files that don't need variable substitution
- **Registration**: **Must be explicitly registered** in the template renderer's copy list
- **Guide**: See [`docs/contributing/templates.md`](../contributing/templates.md#-adding-new-ansible-playbooks) for adding new static Ansible playbooks

### Dynamic Templates (Tera)

- **Processing**: Variable substitution using Tera templating engine
- **File Suffix**: `.tera` extension (e.g., `variables.tfvars.tera`, `inventory.ini.tera`)
- **Use Case**: Configuration files requiring runtime parameters (IPs, usernames, paths)
- **Registration**: Automatically discovered by `.tera` extension

## 🎨 Ansible Variables Pattern

For Ansible templates, the system uses a **hybrid approach** combining static playbooks with centralized variables:

### Tera Templates (2 templates)

1. `inventory.yml.tera` - Inventory requires direct variable substitution (Ansible inventories don't support vars_files)
2. `variables.yml.tera` - Centralized variables for all playbooks

### Static Playbooks

- All playbooks are static YAML files (no `.tera` extension)
- Playbooks reference variables via `vars_files: [variables.yml]`
- Variables are resolved at Ansible runtime, not at template rendering time

### Benefits

- **Reduced Rust Boilerplate**: No per-playbook renderer/wrapper/context needed
- **Centralized Variable Management**: All playbook variables in one place
- **Consistency**: Follows the same pattern as OpenTofu's `variables.tfvars.tera`
- **Maintainability**: Adding new playbooks requires minimal code changes

### Example

```yaml
# templates/ansible/configure-firewall.yml (static playbook)
---
- name: Configure UFW firewall
  hosts: all
  vars_files:
    - variables.yml # Load centralized variables

  tasks:
    - name: Allow SSH access
      community.general.ufw:
        port: "{{ ssh_port }}" # Variable from variables.yml
```

```yaml
# templates/ansible/variables.yml.tera (rendered once)
---
ssh_port: { { ssh_port } }
```

## 🔧 Key Components

### Template Manager

- Handles the embedded → external extraction process
- Manages template source selection (embedded vs external directory)
- Coordinates template availability and caching

### Project Generator Pattern (Orchestrator/Worker)

The system uses a **Project Generator** pattern to standardize how different tools (OpenTofu, Ansible, Docker Compose) generate their project files. This pattern separates concerns into three distinct layers:

#### 1. **Wrapper Types** (Template Representation)

Wrappers are domain types that represent templates statically and define the variables needed:

- **Context**: Contains the variables needed by a template (e.g., `InventoryContext`, `EnvContext`)
  - Strongly typed fields that match template variables
  - Serializable for Tera rendering
  - Validated at construction time
- **Template**: Wraps the template file and context together (e.g., `InventoryTemplate`, `EnvTemplate`)
  - Validates template syntax at creation
  - Performs variable substitution
  - Provides rendering to output file

**Example**:

```rust
// Context defines what variables the template needs
pub struct EnvContext {
    tracker_api_admin_token: String,
}

// Template wraps the .tera file content and context
pub struct EnvTemplate {
    context: EnvContext,
    content: String, // Rendered content
}
```

#### 2. **Renderer Types** (Template Processing)

One renderer per `.tera` template file. Renderers are responsible for:

- Loading the specific `.tera` template from the template manager
- Creating the Template wrapper with the provided Context
- Rendering the template to an output file

**Examples**:

- `InventoryRenderer` - Renders `inventory.yml.tera` for Ansible
- `VariablesRenderer` - Renders `variables.yml.tera` for Ansible
- `EnvRenderer` - Renders `env.tera` for Docker Compose

**Example**:

```rust
pub struct EnvRenderer {
    template_manager: Arc<TemplateManager>,
}

impl EnvRenderer {
    pub fn render(&self, env_context: &EnvContext, output_dir: &Path) -> Result<()> {
        // 1. Load env.tera template file
        // 2. Create EnvTemplate with context
        // 3. Render to .env file
    }
}
```

#### 3. **Project Generator** (Orchestration)

One project generator per tool (Ansible, OpenTofu, Docker Compose). Orchestrates all renderers and static file copying:

- **Orchestrator (`ProjectGenerator`)**: Manages the overall generation process
  - `AnsibleProjectGenerator` - Orchestrates Ansible template rendering
  - `OpenTofuProjectGenerator` - Orchestrates OpenTofu template rendering
  - `DockerComposeProjectGenerator` - Orchestrates Docker Compose template rendering
- **Responsibilities**:
  - Create build directory structure
  - Call individual renderers with appropriate contexts
  - Copy static files (files without `.tera` extension)
  - Coordinate the complete template generation workflow

**Example**:

```rust
pub struct DockerComposeProjectGenerator {
    env_renderer: EnvRenderer,
    template_manager: Arc<TemplateManager>,
}

impl DockerComposeProjectGenerator {
    pub async fn render(&self, env_context: &EnvContext) -> Result<PathBuf> {
        // 1. Create build directory
        // 2. Render .env using EnvRenderer
        // 3. Copy static files (docker-compose.yml)
    }
}
```

### Two-Phase Processing

1. **Phase 1 - Dynamic Template Rendering**:

   - Files with `.tera` extension are processed first
   - Each `.tera` file has its own Renderer
   - Renderers use Context and Template wrappers
   - Example: `env.tera` → `.env` (EnvRenderer with EnvContext)

2. **Phase 2 - Static File Copying**:
   - Files without `.tera` extension are copied as-is
   - **Requires explicit registration** in the ProjectGenerator's copy list
   - Example: `docker-compose.yml` must be added to `copy_static_templates` method

⚠️ **Common Pitfalls**:

- Forgetting to register static files in Phase 2 will cause "file not found" errors at runtime
- Creating a `.tera` file without a corresponding Renderer and Wrapper types
- Not following the naming convention: `{template_name}.tera` → `{TemplateName}Renderer`

### Architecture Summary

```text
┌────────────────────────────────────────────────────────┐
│ ProjectGenerator (e.g., DockerComposeProjectGenerator) │
│                                                        │
│  ┌─────────────────────┐  ┌──────────────────────┐     │
│  │ EnvRenderer         │  │ Static File Copying  │     │
│  │                     │  │                      │     │
│  │  ┌──────────────┐   │  │ - docker-compose.yml │     │
│  │  │ EnvTemplate  │   │  │ (registered in code) │     │
│  │  │ EnvContext   │   │  │                      │     │
│  │  └──────────────┘   │  └──────────────────────┘     │
│  │                     │                               │
│  │  env.tera ────→ .env│                               │
│  └─────────────────────┘                               │
└────────────────────────────────────────────────────────┘
```

### Template Engine

- Tera-based templating for dynamic content
- Variable context resolution via Context types
- Template syntax validation and error handling
- Strongly typed wrappers prevent runtime template errors

## ⚠️ Important Behaviors

### Template Persistence

- Once extracted, external templates persist between runs
- Templates are **not** automatically refreshed from embedded sources
- This enables template customization but can cause confusion during development

### E2E Test Isolation

- E2E tests clean the templates directory before each run
- This ensures fresh embedded template extraction for consistent test results
- Production deployments may use persistent template directories

## 🎯 Design Goals

### Portability

- Single binary contains all necessary templates
- No external dependencies for basic deployment

### Flexibility

- External templates can be customized without recompilation
- Support for both static and dynamic template processing
- CLI option to specify custom template directories

### Test Isolation

- Template cleanup ensures consistent test environments
- Separation of template sources from generated configurations

## 📋 Beta Status Notice

This system is currently in beta. The implementation details, APIs, and internal structure may change significantly. This document focuses on the core architectural concept rather than specific implementation details that are likely to evolve.
