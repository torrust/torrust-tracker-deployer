# SSH Keys

SSH key pairs are used to securely authenticate with provisioned VMs without passwords.

## Overview

The deployer uses SSH keys for:

- Secure access to provisioned instances
- Running Ansible playbooks for configuration
- Executing remote commands via the test command

## Generate SSH Keys

If you don't already have SSH keys:

```bash
# Generate a new SSH key pair (Ed25519 recommended)
ssh-keygen -t ed25519 -C "torrust-deployer" -f ~/.ssh/torrust_deployer

# Set proper permissions
chmod 600 ~/.ssh/torrust_deployer
chmod 644 ~/.ssh/torrust_deployer.pub
```

For RSA keys (if Ed25519 is not supported):

```bash
ssh-keygen -t rsa -b 4096 -C "torrust-deployer" -f ~/.ssh/torrust_deployer
```

## Key Permissions

SSH requires strict file permissions:

```bash
# Private key: owner read/write only
chmod 600 ~/.ssh/your_private_key

# Public key: owner read/write, others read
chmod 644 ~/.ssh/your_private_key.pub

# SSH directory
chmod 700 ~/.ssh
```

## Configuration

Reference your keys in the environment configuration:

```json
{
  "ssh_credentials": {
    "private_key_path": "/home/youruser/.ssh/torrust_deployer",
    "public_key_path": "/home/youruser/.ssh/torrust_deployer.pub",
    "username": "torrust",
    "port": 22
  }
}
```

## Development Keys

For local development and testing, the repository includes test keys in `fixtures/`:

```bash
fixtures/testing_rsa      # Private key
fixtures/testing_rsa.pub  # Public key
```

> ⚠️ **Warning**: Never use test keys for production deployments.

## Best Practices

1. **Use unique keys per project** - Don't reuse keys across different projects
2. **Never commit private keys** - Keep private keys out of version control
3. **Use passphrases for production** - Add passphrase protection for production keys
4. **Ed25519 over RSA** - Prefer Ed25519 keys for better security and performance

## Troubleshooting

### Permission denied (publickey)

```bash
# Check key permissions
ls -la ~/.ssh/your_private_key

# Should show: -rw------- (600)
chmod 600 ~/.ssh/your_private_key
```

### Agent forwarding

If you need SSH agent forwarding:

```bash
# Add key to SSH agent
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/your_private_key
```
