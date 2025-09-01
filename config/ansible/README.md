# Ansible Configuration for Torrust Testing Infrastructure

This directory contains Ansible configurations for executing tasks on VMs provisioned by OpenTofu/Terraform in the Torrust testing infrastructure.

## 📋 Prerequisites

### Ansible Installation

Check if Ansible is installed:

```bash
ansible --version
ansible-playbook --version
```

If not installed, install Ansible:

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install ansible

# macOS
brew install ansible

# Python pip
pip install ansible
```

### SSH Key Setup

Ensure you have the testing SSH key available:

- Private key: `~/.ssh/testing_rsa`
- Public key: `~/.ssh/testing_rsa.pub`

The public key should match the one configured in the cloud-init configuration.

## 🗂️ Files Structure

```text
config/ansible/
├── ansible.cfg          # Ansible configuration
├── inventory.yml        # Host inventory
├── wait-cloud-init.yml  # Playbook to wait for cloud-init completion
└── README.md           # This documentation
```

## ⚙️ Configuration Files

### `ansible.cfg`

Basic Ansible configuration with:

- Default inventory file
- SSH connection optimizations
- Disabled host key checking for lab environment

### `inventory.yml`

Defines the target hosts with:

- VM connection details (IP, user, SSH key)
- Python interpreter path
- SSH connection parameters

### `wait-cloud-init.yml`

A playbook that:

- Waits for cloud-init to complete
- Verifies cloud-init status
- Gathers basic system information
- Displays system details

## 🚀 Usage

### Verify Connectivity

Test Ansible connection to all hosts:

```bash
cd config/ansible
ansible all -m ping
```

Expected output:

```text
torrust-vm | SUCCESS => {
    "changed": false,
    "ping": "pong"
}
```

### Run Cloud-Init Wait Playbook

Execute the cloud-init completion check:

```bash
cd config/ansible
ansible-playbook wait-cloud-init.yml
```

This playbook will:

1. Wait for the `/var/lib/cloud/instance/boot-finished` file
2. Check cloud-init status with `cloud-init status --wait`
3. Gather and display system information

### Useful Ansible Commands

```bash
# Check all hosts status
ansible all -m ping

# Run a simple command on all hosts
ansible all -m command -a "uptime"

# Check disk space
ansible all -m command -a "df -h"

# Gather facts about the system
ansible all -m setup

# Run a specific playbook
ansible-playbook wait-cloud-init.yml

# Run with verbose output
ansible-playbook -v wait-cloud-init.yml

# Dry run (check mode)
ansible-playbook --check wait-cloud-init.yml

# List all hosts in inventory
ansible-inventory --list

# Test connectivity to specific host
ansible torrust-vm -m ping
```

## � Machine-Readable Output for Automation

Ansible provides excellent support for machine-readable output formats, perfect for integration with other tools (like Rust applications) that need to parse execution results programmatically.

### JSON Callback Plugin (Recommended)

The most reliable way to get JSON output is using the JSON callback plugin:

```bash
# Get JSON output from ansible commands
ANSIBLE_STDOUT_CALLBACK=json ansible all -m ping

# Get JSON output from playbook execution
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml

# Example JSON output from ping module:
# {
#     "torrust-vm": {
#         "changed": false,
#         "ping": "pong"
#     }
# }
```

### Structured Playbook Results

For detailed playbook execution results in JSON format:

```bash
# Full JSON output with comprehensive task details
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml

# Combine with other useful flags
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml --check  # Dry run with JSON
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml -v       # Verbose JSON output

# Save to file for processing
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml > playbook_results.json
```

### Example JSON Output Structure

When running `ansible-playbook wait-cloud-init.yml --output json`, you'll get structured output like:

```json
{
  "torrust-vm": {
    "Wait for cloud-init to finish": {
      "changed": false,
      "path": "/var/lib/cloud/instance/boot-finished",
      "elapsed": 0,
      "stat": {
        "exists": true
      }
    },
    "Check cloud-init status": {
      "changed": false,
      "stdout": "status: done",
      "cmd": ["cloud-init", "status", "--wait"],
      "rc": 0
    }
  }
}
```

### Integration with External Tools

Perfect for Rust applications or other automation tools:

```bash
# Save results to file for processing
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml > playbook_results.json

# Parse specific values (using jq)
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml | jq '.stats."torrust-vm".failures'

# Check for any failures programmatically
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml | jq '.stats | to_entries[] | select(.value.failures > 0) | .key'

# Get cloud-init status from task results
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml | jq '.plays[0].tasks[] | select(.task.name == "Check cloud-init status") | .hosts."torrust-vm".stdout'

# Extract all task results for a specific host
ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml | jq '.plays[0].tasks[].hosts."torrust-vm"'
```

### Alternative Output Formats

You can also use other callback plugins for different output formats:

```bash
# Use minimal output for cleaner parsing
ANSIBLE_STDOUT_CALLBACK=minimal ansible-playbook wait-cloud-init.yml

# Use yaml output format
ANSIBLE_STDOUT_CALLBACK=yaml ansible-playbook wait-cloud-init.yml

# Use oneline for compact output
ANSIBLE_STDOUT_CALLBACK=oneline ansible-playbook wait-cloud-init.yml
```

### Exit Codes for Automation

Ansible provides meaningful exit codes for scripting:

- `0`: Success, no changes
- `1`: Error occurred
- `2`: Success with changes
- `4`: Unreachable hosts
- `5`: Failed/unreachable hosts

```bash
# Example usage in scripts
if ANSIBLE_STDOUT_CALLBACK=json ansible-playbook wait-cloud-init.yml > results.json; then
    echo "Playbook executed successfully"
    # Parse results.json with your Rust application
    failures=$(jq '.stats."torrust-vm".failures' results.json)
    if [ "$failures" -eq 0 ]; then
        echo "All tasks completed successfully"
    else
        echo "Some tasks failed: $failures"
    fi
else
    exit_code=$?
    echo "Playbook failed with exit code: $exit_code"
    # Handle different failure scenarios
fi
```

### Rust Integration Example

For your Rust application, you can parse the JSON output structure:

```rust
// Example JSON parsing structure for Rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct AnsibleOutput {
    stats: HashMap<String, HostStats>,
    plays: Vec<Play>,
}

#[derive(Deserialize)]
struct HostStats {
    changed: u32,
    failures: u32,
    ok: u32,
    unreachable: u32,
}

#[derive(Deserialize)]
struct Play {
    tasks: Vec<Task>,
}

#[derive(Deserialize)]
struct Task {
    task: TaskInfo,
    hosts: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct TaskInfo {
    name: String,
}

// Usage example
fn check_ansible_results(json_output: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output: AnsibleOutput = serde_json::from_str(json_output)?;

    for (host, stats) in output.stats {
        if stats.failures > 0 || stats.unreachable > 0 {
            eprintln!("Host {} has failures: {} or unreachable: {}", host, stats.failures, stats.unreachable);
            return Ok(false);
        }
    }

    Ok(true)
}
```

## �🔧 Troubleshooting

### Connection Issues

1. **Permission denied (publickey)**

   - Verify SSH key path in `inventory.yml`
   - Ensure private key permissions: `chmod 600 ~/.ssh/testing_rsa`
   - Check if public key is in VM's authorized_keys

2. **Host unreachable**

   - Verify VM is running: `lxc list`
   - Check IP address matches inventory
   - Test direct SSH: `ssh -i ~/.ssh/testing_rsa torrust@10.140.190.177`

3. **Cloud-init timeout**
   - Check cloud-init logs in VM: `lxc exec torrust-vm -- cloud-init status --long`
   - View cloud-init logs: `lxc exec torrust-vm -- journalctl -u cloud-init`

### VM Management

```bash
# Check VM status
lxc list

# Connect directly to VM
lxc exec torrust-vm -- bash

# Check cloud-init status in VM
lxc exec torrust-vm -- cloud-init status --wait

# View cloud-init logs
lxc exec torrust-vm -- cat /var/log/cloud-init-output.log
```

## 🎯 Next Steps

1. **Extend Playbooks**: Create additional playbooks for:

   - Docker installation and configuration
   - Torrust application deployment
   - System monitoring setup
   - Security hardening

2. **Inventory Management**:

   - Add more hosts as needed
   - Use dynamic inventory for scaling
   - Group hosts by environment (dev, test, prod)

3. **Role Development**:
   - Create Ansible roles for reusable tasks
   - Implement proper variable management
   - Add handlers for service management

## 📚 Resources

- [Ansible Documentation](https://docs.ansible.com/)
- [Ansible Best Practices](https://docs.ansible.com/ansible/latest/user_guide/playbooks_best_practices.html)
- [Cloud-init Documentation](https://cloudinit.readthedocs.io/)
