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

## 🔧 Key Components

### Template Manager

- Handles the embedded → external extraction process
- Manages template source selection (embedded vs external directory)
- Coordinates template availability and caching

### Template Renderers

- **OpenTofu Renderer**: Processes infrastructure templates
- **Ansible Renderer**: Processes configuration management templates
- Handle the template → build directory rendering process

**Two-Phase Processing:**

1. **Phase 1 - Static File Copying**:

   - Files without `.tera` extension are copied as-is
   - **Requires explicit registration** in the renderer's copy list
   - Example: `install-docker.yml` must be added to `copy_static_templates` array

2. **Phase 2 - Dynamic Template Rendering**:
   - Files with `.tera` extension are processed for variable substitution
   - Automatically discovered, no manual registration needed
   - Example: `inventory.ini.tera` → `inventory.ini` with resolved variables

⚠️ **Common Pitfall**: Forgetting to register static files in Phase 1 will cause "file not found" errors at runtime.

### Template Engine

- Tera-based templating for dynamic content
- Variable context resolution
- Template syntax validation and error handling

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
