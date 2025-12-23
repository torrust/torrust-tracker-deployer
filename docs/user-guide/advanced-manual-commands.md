# Advanced: Manual OpenTofu and Ansible Commands

> **⚠️ Advanced Users Only - Not Recommended for Production**
>
> This guide is for advanced users who want to execute OpenTofu and Ansible commands directly for debugging, learning, or development purposes. **For normal deployments, always use the CLI commands** (`create`, `provision`, `configure`, `test`, `release`, `run`, `destroy`).
>
> **Critical Warning**: If you manually modify infrastructure using OpenTofu or Ansible commands, the deployer will **not track these changes**. This can lead to state inconsistencies and deployment failures. Manual commands should only be used for:
>
> - Learning how the system works
> - Debugging template generation issues
> - Developing and testing new features for the deployer itself

## Overview

The Torrust Tracker Deployer provides high-level CLI commands that manage the complete deployment lifecycle. Behind the scenes, these commands generate and execute OpenTofu and Ansible templates. You may want to inspect or manually execute these templates for:

- **Debugging** - Inspecting the generated templates to understand failures
- **Learning** - Understanding the underlying infrastructure-as-code
- **Development** - Testing template changes before integrating them into the deployer
- **Troubleshooting** - Running individual commands to isolate problems

**Why This Is Not Recommended for Production:**

1. **State Management** - The deployer won't track manual changes, leading to state inconsistencies
2. **Complexity** - Requires deep knowledge of OpenTofu, Ansible, and the template system
3. **Error-Prone** - Easy to make mistakes that break the deployment
4. **No Rollback** - Manual changes are harder to undo or debug

## Create and Provision an Environment

Use the deployer CLI commands to create an environment and provision infrastructure. This generates the resolved templates you can then inspect or execute manually:

```bash
# Step 1: Generate a configuration template (if you don't have one)
torrust-tracker-deployer create template my-test-env.json

# Step 2: Edit my-test-env.json with your settings
# (Set environment name, SSH credentials, provider config, etc.)

# Step 3: Create the environment from your configuration
torrust-tracker-deployer create environment --env-file my-test-env.json

# Step 4: Provision infrastructure (generates OpenTofu templates and executes them)
torrust-tracker-deployer provision my-test-env

# Optional: Configure the infrastructure (generates Ansible templates and executes them)
torrust-tracker-deployer configure my-test-env
```

**What this creates:**

- `data/my-test-env/environment.json` - Environment state (tracked by deployer)
- `data/my-test-env/runtime-outputs.json` - Infrastructure outputs (IP addresses, etc.)
- `build/my-test-env/tofu/<provider>/` - Fully-resolved OpenTofu templates
- `build/my-test-env/ansible/` - Fully-resolved Ansible playbooks and inventory
- `build/my-test-env/docker-compose/` - Docker Compose configurations
- `build/my-test-env/tracker/` - Tracker configuration files
- `build/my-test-env/prometheus/` - Prometheus configuration (if enabled)
- `build/my-test-env/grafana/` - Grafana configuration (if enabled)
- Provisioned infrastructure (VM, networking, etc.)

### Alternative: Use E2E Tests for Experimentation

If you want to experiment without affecting your production environments, you can use E2E tests with the `--keep` flag:

```bash
# Run E2E tests but keep the infrastructure for manual experimentation
cargo run --bin e2e-complete-workflow-tests -- --keep
```

After running the E2E tests, you can find the resolved templates in the `build/` directory using the E2E environment name (`e2e-complete`, `e2e-deployment`, or `e2e-infrastructure`):

```bash
# Navigate to the E2E test's resolved OpenTofu templates
cd build/e2e-complete/tofu/lxd/          # For LXD provider
# or
cd build/e2e-complete/tofu/hetzner/      # For Hetzner provider (if using Hetzner)

# Navigate to resolved Ansible templates
cd build/e2e-complete/ansible/
```

**Directory structure for a typical environment:**

```text
build/
└── my-test-env/              # Your environment name
    ├── tofu/
    │   ├── lxd/              # LXD provider templates
    │   │   ├── main.tf       # Infrastructure definition
    │   │   ├── variables.tfvars  # Resolved variables
    │   │   ├── cloud-init.yml    # Cloud-init configuration
    │   │   └── ...
    │   └── hetzner/          # Hetzner provider templates (if using Hetzner)
    │       └── ...
    ├── ansible/              # Ansible playbooks
    │   ├── inventory.yml     # Inventory with instance details
    │   ├── wait-cloud-init.yml
    │   ├── install-docker.yml
    │   ├── install-docker-compose.yml
    │   └── ...
    ├── docker-compose/       # Docker Compose configurations
    │   ├── docker-compose.yml
    │   └── ...
    ├── tracker/              # Tracker configuration files
    │   ├── config.toml
    │   └── ...
    ├── prometheus/           # Prometheus configuration (if enabled)
    │   ├── prometheus.yml
    │   └── ...
    └── grafana/              # Grafana configuration (if enabled)
        ├── grafana.ini
        └── ...

data/
└── my-test-env/              # Environment state (managed by deployer)
    ├── environment.json      # Current state and configuration
    └── runtime-outputs.json  # Infrastructure outputs (IP addresses, etc.)
```

### Inspect and Execute OpenTofu Commands Manually

From the OpenTofu template directory (e.g., `build/my-test-env/tofu/lxd/`):

```bash
# Inspect the generated templates
cat main.tf              # Infrastructure definition
cat variables.tfvars     # Resolved variables
cat cloud-init.yml       # Cloud-init configuration

# Validate the configuration
tofu validate

# Preview what OpenTofu would do (without making changes)
tofu plan -var-file=variables.tfvars

# View current state and outputs (after infrastructure is provisioned)
tofu show
tofu output -json

# Apply changes (⚠️ Not recommended - use deployer CLI instead)
# tofu apply -var-file=variables.tfvars -auto-approve
```

> **⚠️ Warning**: Manually applying changes with `tofu apply` will create infrastructure changes that the deployer doesn't track. This can cause issues when you later use deployer commands like `destroy`. Only use this for learning or debugging.

### Inspect and Execute Ansible Commands Manually

From the Ansible template directory (e.g., `build/my-test-env/ansible/`):

```bash
# Inspect the generated templates
cat inventory.yml        # Inventory with instance IP and SSH details
ls -la *.yml            # List all available playbooks

# Test inventory and connectivity
ansible-playbook --list-hosts -i inventory.yml wait-cloud-init.yml
ansible all -i inventory.yml -m ping    # Test SSH connectivity

# Run individual playbooks (these are the same ones the deployer uses)
ansible-playbook -i inventory.yml wait-cloud-init.yml              # Wait for cloud-init
ansible-playbook -i inventory.yml update-apt-cache.yml             # Update APT cache
ansible-playbook -i inventory.yml install-docker.yml               # Install Docker
ansible-playbook -i inventory.yml install-docker-compose.yml       # Install Docker Compose
ansible-playbook -i inventory.yml configure-firewall.yml           # Configure UFW firewall
ansible-playbook -i inventory.yml configure-security-updates.yml   # Configure automatic security updates

# Run with verbose output for debugging
ansible-playbook -i inventory.yml -v install-docker.yml           # Verbose
ansible-playbook -i inventory.yml -vvv install-docker.yml         # Very verbose (includes SSH details)

# Dry run (check mode) - see what would change without making changes
ansible-playbook -i inventory.yml --check install-docker.yml
```

> **Note**: The playbooks in `build/` are the exact same ones the deployer executes. You can run them manually to debug issues or understand what the deployer does.

### Connect to the Provisioned Instance

#### Method 1: Connect via LXD (for LXD provider only)

```bash
lxc exec torrust-tracker-vm-my-test-env -- /bin/bash
```

#### Method 2: Connect via SSH using deployer's SSH key

First, get the instance IP from OpenTofu outputs:

```bash
cd build/my-test-env/tofu/lxd/
tofu output -json | jq -r '.instance_info.value.ip_address'
```

Then connect (replace `<IP>` and `<your-key-path>` with actual values):

```bash
ssh -i <your-private-key-path> torrust@<instance-ip>
```

#### Method 3: One-liner using jq to extract IP

```bash
ssh -i ~/.ssh/id_rsa torrust@$(cd build/my-test-env/tofu/lxd && tofu output -json | jq -r '.instance_info.value.ip_address')
```

#### Method 4: Use the deployer's stored instance IP

```bash
cat data/my-test-env/runtime-outputs.json | jq -r '.instance_ip'
ssh -i ~/.ssh/id_rsa torrust@$(cat data/my-test-env/runtime-outputs.json | jq -r '.instance_ip')
```

## Destroy Infrastructure

When you're done with an environment, **always use the deployer CLI** to destroy infrastructure properly:

### Option 1: Using the Deployer CLI (Strongly Recommended)

```bash
# This is the correct way to destroy infrastructure
torrust-tracker-deployer destroy my-test-env
```

**Why this is recommended:**

- Updates the environment state in `data/my-test-env/`
- Ensures proper cleanup of all resources
- Maintains state consistency for potential future operations
- Logs all actions with trace IDs for debugging

#### Option 2: Using OpenTofu Directly (Only for Learning/Debugging)

```bash
# Navigate to the OpenTofu directory
cd build/my-test-env/tofu/lxd/

# Preview what will be destroyed
tofu plan -destroy -var-file=variables.tfvars

# Destroy the infrastructure
tofu destroy -var-file=variables.tfvars -auto-approve
```

> **⚠️ Warning**: If you destroy infrastructure manually with OpenTofu, the deployer's state in `data/my-test-env/` will become inconsistent. You may need to manually delete the environment directory or run `deployer destroy` which might fail due to missing infrastructure.

#### Option 3: Using LXD Commands Directly (Emergency Only)

```bash
# Delete the VM instance
lxc delete torrust-tracker-vm-my-test-env --force

# Delete the LXD profile
lxc profile delete torrust-profile-my-test-env

# Manually clean up deployer state
rm -rf data/my-test-env/
rm -rf build/my-test-env/
```

> **⚠️ Critical**: This completely bypasses the deployer. Only use this if both the deployer and OpenTofu commands fail.

## Tips and Best Practices

### Use Deployer Commands for Environment Creation

Always use the deployer CLI commands to create and provision environments:

```bash
# Create environment from configuration file
torrust-tracker-deployer create environment --env-file my-config.json

# Provision infrastructure
torrust-tracker-deployer provision my-env

# Then you can inspect the generated templates for learning or debugging
cd build/my-env/tofu/lxd/
```

### Keep E2E Tests for Development

E2E tests with the `--keep` flag are only for testing the deployer itself:

```bash
# For deployer development/testing only
cargo run --bin e2e-complete-workflow-tests -- --keep
```

Don't use this for creating production environments.

## Common Use Cases

### Use Case 1: Debugging Infrastructure Issues

When something goes wrong with provisioning:

```bash
# Step 1: Create and provision an environment
torrust-tracker-deployer create environment --env-file test-config.json
torrust-tracker-deployer provision test-env

# Step 2: Inspect generated templates
cd build/test-env/tofu/lxd/
cat main.tf
cat variables.tfvars

# Step 3: Validate syntax
tofu validate

# Step 4: If there are template issues, fix them in templates/ directory
#       then destroy and recreate to test the fixes
torrust-tracker-deployer destroy test-env
torrust-tracker-deployer create environment --env-file test-config.json
torrust-tracker-deployer provision test-env

# Step 5: View the runtime outputs
cat data/test-env/runtime-outputs.json
```

### Use Case 2: Testing Ansible Playbooks

When developing new Ansible playbooks:

```bash
# Step 1: Create and provision infrastructure first
torrust-tracker-deployer create environment --env-file test-config.json
torrust-tracker-deployer provision test-env

# Step 2: Run your custom playbook manually
cd build/test-env/ansible/
ansible-playbook -i inventory.yml your-custom-playbook.yml -vv

# Step 3: Once tested, integrate into deployer
#         by adding it to templates/ and running configure command
torrust-tracker-deployer configure test-env
```

### Use Case 3: Developing New Features

When adding new functionality to the deployer:

```bash
# Step 1: Create a test environment
torrust-tracker-deployer create environment --env-file test-config.json
torrust-tracker-deployer provision test-env

# Step 2: Modify templates in templates/ directory
vim templates/tofu/lxd/main.tf.tera

# Step 3: Destroy and recreate to regenerate templates with changes
# (The state machine doesn't allow re-running provision on an already provisioned environment)
torrust-tracker-deployer destroy test-env
torrust-tracker-deployer create environment --env-file test-config.json
torrust-tracker-deployer provision test-env

# Step 4: Test the changes
cd build/test-env/tofu/lxd/
tofu validate
tofu plan -var-file=variables.tfvars

# Step 5: Once working, integrate into deployer
```

## Additional Notes

### Keep E2E Environments Separate from Production

When using E2E tests for experimentation, use dedicated test configuration files:

```bash
# Use test-specific configuration (e.g., test-env.json, dev-test.json)
torrust-tracker-deployer create environment --env-file test-env.json

# Never use production configuration files for manual experiments
```

### State Management is Critical

The deployer tracks infrastructure state in `data/<env-name>/`. If you manually modify infrastructure:

- The deployer won't know about the changes
- Future deployer commands may fail or produce unexpected results
- You may need to manually sync or recreate state

Always prefer using deployer commands over manual OpenTofu/Ansible execution.

## Troubleshooting

### OpenTofu State Issues

If you encounter state-related errors:

```bash
# View the current state
tofu state list

# Remove a specific resource from state (if needed)
tofu state rm <resource_address>

# Import existing resources (if needed)
tofu import <resource_address> <resource_id>
```

### Ansible Connection Issues

If Ansible can't connect to the instance:

```bash
# Test SSH connectivity
ssh -i ~/.ssh/id_rsa torrust@<instance_ip>

# Check if cloud-init has finished
lxc exec <instance_name> -- cloud-init status

# View Ansible connection details
cat build/my-test-env/ansible/inventory.yml
```

### Resource Cleanup Issues

If resources aren't properly cleaned up:

```bash
# List all LXD instances
lxc list

# List all LXD profiles
lxc profile list

# Force delete stuck instance
lxc delete <instance_name> --force

# Delete profile
lxc profile delete <profile_name>
```

## Additional Resources

- **[OpenTofu Documentation](https://opentofu.org/docs/)** - Official OpenTofu documentation
- **[Ansible Documentation](https://docs.ansible.com/)** - Official Ansible documentation
- **[LXD Documentation](https://documentation.ubuntu.com/lxd/)** - LXD reference
- **[Project Template System](../contributing/templates/)** - Internal template documentation
- **[LXD Technology Guide](../tech-stack/lxd.md)** - Comprehensive LXD commands and examples

## See Also

- [`docs/user-guide/commands/`](commands/) - CLI command reference
- [`docs/e2e-testing/`](../e2e-testing/) - E2E testing documentation
- [`docs/tech-stack/opentofu.md`](../tech-stack/opentofu.md) - OpenTofu setup guide
- [`docs/tech-stack/ansible.md`](../tech-stack/ansible.md) - Ansible setup guide
