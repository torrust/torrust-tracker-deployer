# Tera Template Variable Syntax

This document explains the correct syntax for defining variables in Tera templates used in the Torrust Tracker Deployer project.

## 📝 Correct Variable Syntax

All Tera template variables must use **double curly braces** with **no spaces** inside the braces:

```yaml
# ✅ CORRECT
{{ variable_name }}
{{ username }}
{{ ssh_public_key }}
{{ instance_name }}
```

## ❌ Incorrect Syntax to Avoid

```yaml
# ❌ WRONG - Spaces inside braces
{ { variable_name } }
{ { username } }

# ❌ WRONG - Single braces
{ variable_name }

# ❌ WRONG - Mixed spacing
{{ variable_name}}
{{variable_name }}
```

## 📖 Examples in Practice

### Cloud-init Template

```yaml
users:
  - name: { { username } }
    ssh_authorized_keys:
      - { { ssh_public_key } }
```

### Ansible Inventory Template

```yaml
torrust_servers:
  hosts:
    torrust_vm:
      ansible_host: { { ansible_host } }
```

### OpenTofu Variables Template

```hcl
instance_name = "{{ instance_name }}"
```

## 🎯 Key Rules

1. Always use double curly braces: `{{` and `}}`
2. No spaces between braces and variable name: `{{variable}}` not `{ { variable } }`
3. Variable names are case-sensitive
4. Works in any file format (YAML, HCL, etc.)

## 🔧 Troubleshooting

### VS Code Prettier Extension Adding Spaces in Variables

**Problem**: When using VS Code with the Prettier extension, saving `.tera` files automatically adds unwanted spaces inside Tera variables:

- **Before saving**: `{{ username }}` ✅
- **After saving**: `{ { username } }` ❌

**Cause**: Prettier doesn't understand Tera template syntax and tries to format `.tera` files incorrectly.

**Solution**: Create a `.prettierignore` file in your project root to exclude Tera template files:

```gitignore
# Ignore Tera template files - they have specific syntax that Prettier doesn't understand
*.tera
```

**Alternative Solution**: Disable formatting for `.tera` files in your VS Code settings:

```json
{
  "[tera]": {
    "editor.formatOnSave": false,
    "editor.defaultFormatter": null
  }
}
```

After applying the fix, manually correct any existing formatting issues in your `.tera` files by removing the spaces inside the curly braces.
