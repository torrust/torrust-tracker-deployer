---
name: work-with-tera-templates
description: Guide for working with Tera template files (.tera extension) in this project. Covers correct variable syntax (double curly braces without spaces inside), avoiding Prettier IDE interference, the difference between dynamic .tera templates and static templates, and template file registration. Use when creating or editing .tera files, adding template variables, or troubleshooting template rendering. Triggers on "tera template", ".tera file", "template variable", "template syntax", "template rendering", "Tera", or "template file".
metadata:
  author: torrust
  version: "1.0"
---

# Working with Tera Templates

## Correct Variable Syntax

Tera template files use `.tera` extension. Variables use **double curly braces**:

```yaml
# ✅ CORRECT
{{ variable_name }}
{{ username }}
{{ ssh_public_key }}
{{ instance_name }}
```

```yaml
# ❌ WRONG — do NOT add spaces inside braces
{ { variable_name } }

# ❌ WRONG — single braces
{ variable_name }
```

## Prettier IDE Problem

VS Code's Prettier extension **will corrupt `.tera` files** by adding spaces inside braces.

**Fix**: Add to `.prettierignore`:

```gitignore
# Prettier breaks Tera template syntax
*.tera
```

Or in `.vscode/settings.json`:

```json
{
  "[tera]": {
    "editor.formatOnSave": false
  }
}
```

After applying the fix, manually remove any `{ { ... } }` patterns already introduced.

## Static vs Dynamic Templates

| Type        | Extension   | Processing              | Registration       |
| ----------- | ----------- | ----------------------- | ------------------ |
| **Dynamic** | `.tera`     | Tera variable rendering | Auto-discovered    |
| **Static**  | `.yml` etc. | Direct file copy        | Must be registered |

Tera files are automatically discovered. Static files must be explicitly registered.

## Registering Static Templates

Static templates (e.g., Ansible playbooks) must be added to `copy_static_templates()` in the project generator:

```text
src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs
```

Without this, the file will not be copied into the build directory.

## Examples

```yaml
# templates/ansible/inventory.yml.tera
torrust_servers:
  hosts:
    torrust_vm:
      ansible_host: { { ansible_host } }
```

```hcl
# templates/tofu/variables.tfvars.tera
instance_name = "{{ instance_name }}"
ssh_public_key = "{{ ssh_public_key }}"
```

## Reference

Syntax guide: [`docs/contributing/templates/tera.md`](../../docs/contributing/templates/tera.md)
Architecture: [`docs/contributing/templates/template-system-architecture.md`](../../docs/contributing/templates/template-system-architecture.md)
