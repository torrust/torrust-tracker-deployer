---
name: add-new-template
description: Guide for adding new templates to this project. Covers the Project Generator pattern, the difference between dynamic templates (.tera, automatically discovered) and static templates (must be explicitly registered in copy_static_templates()), and the two-level indirection design (embedded binary → external extraction → build directory rendering). Use when adding infrastructure templates, Ansible playbooks, OpenTofu files, or configuration templates. Triggers on "add template", "new template", "static template", "register template", "copy_static_templates", "project generator", or "template system".
metadata:
  author: torrust
  version: "1.0"
---

# Adding New Templates

## Two-Level Template Indirection

```text
Embedded in binary  →  data/templates/  →  build/{env}/
  (compile time)       (extracted once)    (rendered per-run)
```

Templates live in `templates/` in the repo. They are compiled into the binary, extracted to `data/templates/` at first use, then rendered into `build/` with variables.

## Template Types

| Type        | Extension | Processing           | Discovery              |
| ----------- | --------- | -------------------- | ---------------------- |
| **Dynamic** | `.tera`   | Tera variable subst. | Auto by extension      |
| **Static**  | Any       | File copy only       | Manual — must register |

## Adding a Dynamic Template (`.tera`)

1. Create the file in `templates/` with a `.tera` extension
2. Use `{{ variable_name }}` syntax for runtime values
3. **No additional registration needed** — automatically discovered

```text
templates/ansible/new-config.yml.tera
```

## Adding a Static Template

Static templates (Ansible playbooks, fixed YAML, etc.) require explicit registration.

### Step 1: Create the file

```text
templates/ansible/my-new-playbook.yml
```

### Step 2: Register in copy_static_templates()

Find the appropriate `ProjectGenerator` for your template type:

```text
src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs
```

Add your file to the `copy_static_templates()` function:

```rust
fn copy_static_templates(&self, ...) -> Result<(), Error> {
    // existing registrations...
    self.copy_template("ansible/my-new-playbook.yml", ...)?;
    Ok(())
}
```

**Without this step, the file will never appear in the build directory** and won't be available at runtime.

## Ansible-Specific: Variables Pattern

For Ansible playbooks, add runtime variables to `variables.yml.tera`, not the playbook. The playbook loads them via `vars_files: [variables.yml]`.

## Checklist

- [ ] File placed in `templates/` directory
- [ ] If `.tera`: uses correct `{{ variable }}` syntax
- [ ] If static: registered in `copy_static_templates()` in the project generator
- [ ] New variables added to `variables.yml.tera` (for Ansible) or equivalent context

## Reference

Architecture: [`docs/contributing/templates/template-system-architecture.md`](../../docs/contributing/templates/template-system-architecture.md)
Tera syntax: [`docs/contributing/templates/tera.md`](../../docs/contributing/templates/tera.md)
Ansible guide: [`docs/contributing/templates/ansible.md`](../../docs/contributing/templates/ansible.md)
