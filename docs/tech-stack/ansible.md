# Ansible Setup and Usage Guide

Ansible is an open-source automation platform that enables infrastructure as code, configuration management, application deployment, and orchestration.

## 📋 Prerequisites

### Installation Verification

Check if Ansible is installed:

```bash
ansible --version
ansible-playbook --version
```

Expected output format:

```text
ansible [core 2.x.x]
  config file = /etc/ansible/ansible.cfg
  configured module search path = [...]
  ansible python module location = [...]
  ansible collection location = [...]
  executable location = /usr/bin/ansible
  python version = 3.x.x
```

### Installation

If Ansible is not installed, you can install it using various methods:

#### Ubuntu/Debian (APT)

```bash
# Update package index
sudo apt update

# Install Ansible
sudo apt install ansible

# Verify installation
ansible --version
```

#### macOS (Homebrew)

```bash
# Install Ansible
brew install ansible

# Verify installation
ansible --version
```

#### Python pip (Universal)

```bash
# Install Ansible via pip
pip install ansible

# Or install with additional collections
pip install ansible-core ansible

# Verify installation
ansible --version
```

#### RHEL/CentOS/Fedora

```bash
# RHEL/CentOS (Enable EPEL first)
sudo yum install epel-release
sudo yum install ansible

# Fedora
sudo dnf install ansible
```

## 🚀 Common Commands

### Basic Operations

```bash
# Ping all hosts (test connectivity)
ansible all -m ping

# Run a command on all hosts
ansible all -m shell -a "uptime"

# Run command on specific group
ansible webservers -m shell -a "systemctl status nginx"

# Copy files to remote hosts
ansible all -m copy -a "src=/local/file dest=/remote/file"
```

### Playbook Execution

```bash
# Run a playbook
ansible-playbook playbook.yml

# Run with specific inventory
ansible-playbook -i inventory.yml playbook.yml

# Run with verbose output
ansible-playbook -v playbook.yml

# Check what changes would be made (dry run)
ansible-playbook --check playbook.yml

# Run specific tags only
ansible-playbook --tags "setup,config" playbook.yml
```

### Inventory Management

```bash
# List all hosts
ansible-inventory --list

# List hosts in specific group
ansible-inventory --list --group webservers

# Show host variables
ansible-inventory --host hostname

# Verify inventory syntax
ansible-inventory --inventory inventory.yml --list
```

### Information Gathering

```bash
# Gather facts about hosts
ansible all -m setup

# Get specific facts
ansible all -m setup -a "filter=ansible_os_family"

# Check connectivity to all hosts
ansible all -m ping
```

## 🔧 Configuration Structure

### Basic File Structure

```text
├── ansible.cfg          # Ansible configuration
├── inventory/           # Host inventory files
│   ├── hosts.yml       # Main inventory
│   └── group_vars/     # Group-specific variables
├── playbooks/          # Ansible playbooks
│   ├── site.yml        # Main playbook
│   └── roles/          # Custom roles
├── host_vars/          # Host-specific variables
└── requirements.yml    # External roles/collections
```

### Inventory Configuration (YAML)

```yaml
all:
  children:
    webservers:
      hosts:
        web1:
          ansible_host: 192.168.1.10
          ansible_user: ubuntu
        web2:
          ansible_host: 192.168.1.11
          ansible_user: ubuntu
    databases:
      hosts:
        db1:
          ansible_host: 192.168.1.20
          ansible_user: admin
  vars:
    ansible_ssh_private_key_file: ~/.ssh/id_rsa
    ansible_ssh_common_args: "-o StrictHostKeyChecking=no"
```

### Basic Playbook Structure

```yaml
---
- name: Configure web servers
  hosts: webservers
  become: yes
  vars:
    package_name: nginx

  tasks:
    - name: Install web server
      package:
        name: "{{ package_name }}"
        state: present

    - name: Start and enable service
      service:
        name: "{{ package_name }}"
        state: started
        enabled: yes
```

### Ansible Configuration (ansible.cfg)

```ini
[defaults]
inventory = inventory/hosts.yml
remote_user = ubuntu
private_key_file = ~/.ssh/id_rsa
host_key_checking = False
timeout = 30
gathering = smart
fact_caching = jsonfile
fact_caching_connection = ~/.ansible/facts

[ssh_connection]
ssh_args = -o ControlMaster=auto -o ControlPersist=60s
pipelining = True
```

## 🎯 Best Practices

### Inventory Organization

- Use YAML format for better readability
- Group hosts logically (by function, environment, etc.)
- Use group variables for common settings
- Keep sensitive data in encrypted files (ansible-vault)

### Playbook Design

- Use descriptive task names
- Implement idempotency (tasks can run multiple times safely)
- Use handlers for service restarts
- Tag tasks for selective execution
- Use roles for reusable components

### Security

- Use Ansible Vault for sensitive data
- Implement proper SSH key management
- Use become (sudo) only when necessary
- Validate inputs and use proper escaping

### Performance

- Enable SSH pipelining
- Use fact caching
- Limit fact gathering when not needed
- Use async tasks for long-running operations

## 🔐 SSH Key Management

### Generate SSH Key Pair

```bash
# Generate SSH key pair for Ansible
ssh-keygen -t rsa -b 4096 -f ~/.ssh/ansible_rsa -N ""

# Copy public key to remote hosts
ssh-copy-id -i ~/.ssh/ansible_rsa.pub user@remote-host
```

### Using SSH Agent

```bash
# Start SSH agent
eval $(ssh-agent)

# Add private key to agent
ssh-add ~/.ssh/ansible_rsa

# List loaded keys
ssh-add -l
```

## 🔒 Ansible Vault

### Creating and Managing Encrypted Files

```bash
# Create encrypted file
ansible-vault create secrets.yml

# Edit encrypted file
ansible-vault edit secrets.yml

# View encrypted file
ansible-vault view secrets.yml

# Encrypt existing file
ansible-vault encrypt vars.yml

# Decrypt file
ansible-vault decrypt vars.yml
```

### Using Vault in Playbooks

```bash
# Run playbook with vault password prompt
ansible-playbook --ask-vault-pass playbook.yml

# Use password file
ansible-playbook --vault-password-file vault_pass.txt playbook.yml

# Use environment variable
export ANSIBLE_VAULT_PASSWORD_FILE=vault_pass.txt
ansible-playbook playbook.yml
```

## 🐛 Troubleshooting

### Common Issues

#### Connection Problems

```bash
# Test SSH connectivity
ssh -i ~/.ssh/ansible_rsa user@host

# Debug Ansible connection
ansible host -m ping -vvv

# Check SSH configuration
ansible host -m setup -a "filter=ansible_ssh*"
```

#### Permission Issues

```bash
# Check sudo access
ansible host -m shell -a "sudo whoami" --become

# Test privilege escalation
ansible host -m ping --become --ask-become-pass
```

#### Syntax and Logic Errors

```bash
# Check playbook syntax
ansible-playbook --syntax-check playbook.yml

# Validate inventory
ansible-inventory --list

# Run in check mode (dry run)
ansible-playbook --check playbook.yml
```

### Debugging

```bash
# Verbose output levels
ansible-playbook -v playbook.yml    # Level 1
ansible-playbook -vv playbook.yml   # Level 2
ansible-playbook -vvv playbook.yml  # Level 3 (connection debug)
ansible-playbook -vvvv playbook.yml # Level 4 (all debug)

# Use debug module in playbooks
- name: Debug variable
  debug:
    var: my_variable
    verbosity: 2
```

## 📚 Additional Resources

- [Ansible Documentation](https://docs.ansible.com/)
- [Ansible Galaxy](https://galaxy.ansible.com/) (community roles and collections)
- [Ansible GitHub Repository](https://github.com/ansible/ansible)
- [Best Practices Guide](https://docs.ansible.com/ansible/latest/user_guide/playbooks_best_practices.html)
- [Module Index](https://docs.ansible.com/ansible/latest/collections/index_module.html)
- [Cloud-init Documentation](https://cloudinit.readthedocs.io/)
