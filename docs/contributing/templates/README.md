# Template Documentation

This directory contains documentation about the template system used in the Torrust Tracker Deployer project.

## 📚 Documentation Overview

The template system uses a **double indirection** approach with embedded templates that are extracted and then rendered to build directories. Understanding this system is essential when working with infrastructure configurations.

### Core Documentation

- **[Template System Architecture](template-system-architecture.md)** - **CRITICAL**: Technical architecture overview

  - Double indirection pattern (embedded → external → build)
  - Project Generator pattern (Orchestrator/Worker)
  - Two-phase processing (dynamic rendering + static copying)
  - Wrapper types, Renderer types, and Project Generators
  - Read this first to understand the overall system design

- **[Tera Template Syntax](tera.md)** - **CRITICAL**: Working with Tera templates

  - Correct Tera variable syntax: `{{ variable }}` not `{ { variable } }`
  - Static vs dynamic playbooks
  - Adding new Ansible playbooks (registration required)
  - Using centralized variables pattern
  - Common mistakes and troubleshooting
  - Read this when creating or modifying `.tera` files

- **[Ansible Templates](ansible.md)** - Ansible-specific documentation
  - Available playbooks and their purpose
  - Usage order for typical deployments
  - CI/Testing considerations (firewall, container limitations)
  - Variables pattern and template processing
  - Read this when working with Ansible infrastructure

## 🎯 Quick Reference

### When to Read What

| Task                                    | Read This                                                                                                     |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| Understanding template architecture     | [Template System Architecture](template-system-architecture.md)                                               |
| Creating/modifying `.tera` files        | [Tera Template Syntax](tera.md)                                                                               |
| Adding new Ansible playbooks            | [Tera Template Syntax](tera.md#-adding-new-ansible-playbooks)                                                 |
| Working with Ansible infrastructure     | [Ansible Templates](ansible.md)                                                                               |
| Understanding Project Generator pattern | [Template System Architecture](template-system-architecture.md#-project-generator-pattern-orchestratorworker) |

## ⚠️ Critical Rules

### 1. Tera Variable Syntax

Always use correct Tera syntax:

```yaml
# ✅ CORRECT
{{ variable_name }}

# ❌ WRONG - Spaces inside braces
{ { variable_name } }
```

### 2. Static Template Registration

Static templates (without `.tera` extension) **MUST be registered** in the Project Generator:

```rust
// In copy_static_templates() method
for playbook in &[
    "install-docker.yml",
    "your-new-playbook.yml",  // ← ADD HERE
] {
    // ...
}
```

**Without registration**: Ansible will fail with "playbook not found" error.

### 3. Centralized Variables Pattern

For Ansible playbooks, prefer the centralized variables pattern:

- Add variables to `templates/ansible/variables.yml.tera`
- Reference via `vars_files: [variables.yml]` in static playbooks
- Do NOT create new `.tera` templates unless necessary

## 🏗️ Template Structure

```text
templates/
├── ansible/           Ansible playbooks and configuration
├── docker-compose/    Docker Compose configurations
├── grafana/          Grafana monitoring dashboards
├── prometheus/       Prometheus monitoring configuration
├── tofu/             OpenTofu (Terraform) infrastructure
└── tracker/          Torrust Tracker configuration
```

## 📦 Template Types

### Dynamic Templates (.tera)

- **Extension**: `.tera`
- **Processing**: Variable substitution using Tera engine
- **Examples**: `inventory.yml.tera`, `variables.yml.tera`, `env.tera`
- **Registration**: Automatic (discovered by extension)

### Static Templates

- **Extension**: `.yml`, `.cfg`, etc. (no `.tera`)
- **Processing**: Direct file copy
- **Examples**: `install-docker.yml`, `ansible.cfg`, `docker-compose.yml`
- **Registration**: **Required** - must be added to Project Generator's copy list

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

## 📝 Related Documentation

- **[DDD Layer Placement](../ddd-layer-placement.md)** - Where template-related code belongs
- **[Module Organization](../module-organization.md)** - How to organize template code
- **[E2E Testing](../../e2e-testing/README.md)** - Testing template generation
- **[AGENTS.md](../../../AGENTS.md)** - AI assistant instructions for templates

## 🎓 Learning Path

For new contributors working with templates:

1. **Start**: Read [Template System Architecture](template-system-architecture.md) for the big picture
2. **Practice**: Read [Tera Template Syntax](tera.md) and try modifying existing templates
3. **Apply**: Read [Ansible Templates](ansible.md) when working with infrastructure
4. **Extend**: Follow the guides to add new templates and playbooks

## 🐛 Common Issues

### Problem: Prettier adds spaces in Tera variables

**Solution**: Add `*.tera` to `.prettierignore` or disable formatting for `.tera` files.

### Problem: Ansible playbook not found error

**Solution**: Static playbooks must be registered in `copy_static_templates()` method.

### Problem: Variables not resolved in output

**Solution**: Use `.tera` extension for files needing variable substitution.

## 📄 Beta Status Notice

The template system is currently in beta. Implementation details, APIs, and internal structure may change. These docs focus on core concepts and best practices rather than specific implementation details that may evolve.
