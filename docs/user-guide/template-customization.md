# Template System Overview

This document explains how the template system works in the Torrust Tracker Deploy tool.

## 📋 Overview

The deployment tool uses templates to generate configuration files for infrastructure provisioning (OpenTofu) and software configuration (Ansible). Currently, templates are embedded in the binary for production use.

## 🗂️ Template Structure

Templates are organized in the following structure:

```text
templates/
├── ansible/           # Ansible playbooks and inventory templates
│   ├── inventory.yml.tera
│   ├── install-docker.yml
│   └── ...
└── tofu/             # OpenTofu infrastructure templates
    └── lxd/          # LXD provider templates
        ├── main.tf
        ├── variables.tfvars.tera
        └── cloud-init.yml.tera
```

## 🚀 Current Template System

### Embedded Templates (Production)

- Templates are embedded in the binary during compilation
- No external files required for deployment
- Provides consistent, tested deployment configurations
- Templates are automatically extracted to build directory during deployment

## 🎛️ Template Types

### Static Templates

- Files copied as-is to the build directory
- Examples: `main.tf`, `install-docker.yml`
- No variable substitution

### Dynamic Templates (`.tera` extension)

- Use Tera templating engine for variable substitution
- Examples: `variables.tfvars.tera`, `inventory.yml.tera`
- Support runtime variables like `{{ instance_name }}`

## � Template Variables

Common variables available in templates:

| Variable              | Description       | Example              |
| --------------------- | ----------------- | -------------------- |
| `instance_name`       | VM/container name | `torrust-tracker-vm` |
| `ansible_host`        | Target server IP  | `10.140.190.11`      |
| `ssh_pub_key_content` | SSH public key    | `ssh-rsa AAAA...`    |

## 🔄 Template Processing Flow

1. **Template Loading**: Load embedded templates from binary
2. **Extraction**: Copy templates to build directory
3. **Variable Resolution**: Render `.tera` templates with runtime values
4. **Deployment**: Use generated files for infrastructure provisioning

## ⚠️ Important Notes

- Templates are currently embedded and cannot be customized in production deployments
- All template modifications require recompilation of the binary
- Templates are tested as part of the CI/CD pipeline to ensure reliability

## 🛠️ Development & Testing

For development and testing purposes only, the system supports external template directories through the `--templates-dir` CLI argument. This is used internally for:

- E2E testing with fresh template states
- Development and debugging of template rendering
- Template validation during CI/CD

**Note**: This functionality is not available in production builds and is reserved for development and testing workflows.
