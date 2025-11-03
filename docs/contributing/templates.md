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

## 📦 Adding New Ansible Playbooks

When adding new Ansible playbooks to the project, you need to understand the difference between **static playbooks** and **dynamic templates**, and follow the correct registration process.

### Static vs Dynamic Playbooks

#### Static Playbooks (No Tera Variables)

Static playbooks are standard Ansible YAML files that don't require variable substitution:

- **No `.tera` extension** - Just `.yml`
- **No Tera variables** - No `{{ variable }}` syntax needed
- **Direct copy** - Copied as-is from `templates/ansible/` to `build/` directory
- **Examples**: `install-docker.yml`, `wait-cloud-init.yml`, `configure-security-updates.yml`

#### Dynamic Playbooks (With Tera Variables)

Dynamic playbooks need runtime variable substitution:

- **`.tera` extension** - Named like `inventory.ini.tera`
- **Contains Tera variables** - Uses `{{ ansible_host }}`, `{{ username }}`, etc.
- **Rendered during execution** - Variables replaced at runtime
- **Examples**: Ansible inventory files with instance IPs

### Adding a Static Ansible Playbook

Follow these steps when adding a new static playbook:

#### Step 1: Create the Playbook File

Create your playbook in `templates/ansible/`:

```bash
# Example: Adding a new security configuration playbook
templates/ansible/configure-security-updates.yml
```

Write standard Ansible YAML with no Tera variables:

```yaml
---
- name: Configure automatic security updates
  hosts: all
  become: true
  tasks:
    - name: Install unattended-upgrades package
      ansible.builtin.apt:
        name: unattended-upgrades
        state: present
        update_cache: true
```

#### Step 2: Register in Template Copy List ⚠️ CRITICAL

**This is the step that's easy to miss!**

Add your playbook filename to the array in `src/infrastructure/external_tools/ansible/template/renderer/mod.rs`:

```rust
// Find the copy_static_templates method
async fn copy_static_templates(
    &self,
    template_manager: &TemplateManager,
    destination_dir: &Path,
) -> Result<(), ConfigurationTemplateError> {
    // ... existing code ...

    // Copy all playbook files
    for playbook in &[
        "update-apt-cache.yml",
        "install-docker.yml",
        "install-docker-compose.yml",
        "wait-cloud-init.yml",
        "configure-security-updates.yml",  // 👈 ADD YOUR PLAYBOOK HERE
    ] {
        self.copy_static_file(template_manager, playbook, destination_dir)
            .await?;
    }

    tracing::debug!(
        "Successfully copied {} static template files",
        6 // 👈 UPDATE THE COUNT: ansible.cfg + N playbooks
    );

    Ok(())
}
```

**Why This is Required:**

- The template system uses a **two-phase approach** (see `docs/technical/template-system-architecture.md`)
- **Phase 1**: Static file copying - requires explicit registration
- **Phase 2**: Dynamic rendering - automatic for `.tera` files
- Without registration, your playbook **will not be copied** to the build directory
- Ansible will fail with: `[ERROR]: the playbook: your-playbook.yml could not be found`

#### Step 3: Update the File Count

In the same method, update the debug log count:

```rust
tracing::debug!(
    "Successfully copied {} static template files",
    6 // ansible.cfg + 5 playbooks  👈 Update this comment
);
```

#### Step 4: Test Your Changes

Run E2E tests to verify the playbook is copied correctly:

```bash
# Run E2E config tests (faster, tests configuration only)
cargo run --bin e2e-config-tests

# Or run full E2E tests
cargo run --bin e2e-tests-full
```

If you forgot Step 2, you'll see this error:

```text
[ERROR]: the playbook: your-playbook.yml could not be found
```

#### Step 5: Use the Playbook in Your Code

Create a step that executes your playbook:

```rust
// In src/application/steps/system/your_step.rs
pub struct YourStep {
    ansible_client: Arc<dyn AnsibleClient>,
}

impl YourStep {
    pub async fn execute(&self) -> Result<(), YourStepError> {
        self.ansible_client
            .run_playbook("your-playbook.yml")
            .await
            .map_err(YourStepError::AnsibleExecution)?;

        Ok(())
    }
}
```

### Common Mistakes

❌ **Forgetting to register the playbook** in `copy_static_templates`

- Error: Playbook not found during execution
- Fix: Add playbook name to the array

❌ **Forgetting to update the file count** in debug log

- Error: Confusing logs during debugging
- Fix: Update the count comment

❌ **Using `.tera` extension for static playbooks**

- Error: Unnecessary complexity
- Fix: Only use `.tera` if you need variable substitution

❌ **Adding dynamic variables without `.tera` extension**

- Error: Variables not resolved, literal `{{ variable }}` in output
- Fix: Rename to `.yml.tera` and handle in rendering phase

### Quick Checklist

When adding a static Ansible playbook:

- [ ] Create `.yml` file in `templates/ansible/`
- [ ] Write standard Ansible YAML (no Tera variables)
- [ ] Add filename to `copy_static_templates` array in `src/infrastructure/external_tools/ansible/template/renderer/mod.rs`
- [ ] Update file count in debug log
- [ ] Run E2E tests to verify
- [ ] Create application step to execute the playbook
- [ ] Verify playbook appears in `build/` directory during execution

## 🎯 Using Centralized Variables in Ansible Playbooks

When creating new Ansible playbooks that need dynamic variables (ports, paths, etc.), use the **centralized variables pattern** instead of creating new Tera templates.

### DO ✅

**Add variables to `templates/ansible/variables.yml.tera`:**

```yaml
# System Configuration
ssh_port: {{ ssh_port }}
my_service_port: {{ my_service_port }}  # ← Add your new variable
```

**Reference variables in static playbook using `vars_files`:**

```yaml
# templates/ansible/my-new-service.yml (static playbook, no .tera extension)
---
- name: Configure My Service
  hosts: all
  vars_files:
    - variables.yml  # Load centralized variables
  
  tasks:
    - name: Configure service port
      ansible.builtin.lineinfile:
        path: /etc/myservice/config
        line: "port={{ my_service_port }}"
```

**Register playbook in `copy_static_templates()` method:**

```rust
for playbook in &[
    "update-apt-cache.yml",
    "install-docker.yml",
    "my-new-service.yml",  // ← Add here
] {
    // ...
}
```

### DON'T ❌

- ❌ Create a new `.tera` template for the playbook
- ❌ Create a new renderer/wrapper/context for each playbook
- ❌ Add variables directly in `inventory.yml.tera` (unless inventory-specific)

### Benefits

1. **Minimal Code**: No Rust boilerplate (renderer, wrapper, context) needed
2. **Centralized Management**: All variables in one place
3. **Runtime Resolution**: Variables resolved by Ansible, not at template rendering
4. **Easy Maintenance**: Adding new variables requires minimal changes

### When to Create a New Tera Template

Only create a new `.tera` template if:

1. The file **cannot** use Ansible's `vars_files` directive (e.g., inventory files)
2. The file requires **complex logic** that Tera provides but Ansible doesn't
3. The file needs **different variable scopes** than what centralized variables provide

Otherwise, use the centralized variables pattern for simplicity.

### Related Documentation

- **Architecture**: [`docs/technical/template-system-architecture.md`](../technical/template-system-architecture.md) - Understanding the two-phase template system
- **Tera Syntax**: This document (above) - When you DO need dynamic templates with variables
- **Testing**: [`docs/e2e-testing.md`](../e2e-testing.md) - How to run E2E tests to validate your changes
