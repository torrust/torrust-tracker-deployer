# SSH Key Handling

This page covers everything you need to know about SSH keys for the Torrust Tracker Deployer:
why they are required, what format they must be in for automated deployment, and how to
generate or adjust them.

## Why the Deployer Needs SSH Keys

The deployer uses SSH for every remote operation after infrastructure is provisioned:

- **`provision`** — Waits for the VM to accept SSH connections; verifies cloud-init ran
  successfully.
- **`configure`** — Uploads configuration files and runs Ansible playbooks over SSH.
- **`release`** — Pushes Docker images and starts Docker Compose over SSH.
- **`run`** — Triggers service restarts and smoke tests over SSH.

All steps use the key pair configured in `ssh_credentials.private_key_path` /
`ssh_credentials.public_key_path`.

## Key Requirements for Unattended Automation

When running in an automated environment (Docker container, CI/CD pipeline, GitHub
Actions), there is **no interactive terminal and no SSH agent**. This means:

| Scenario                           | Works? |
| ---------------------------------- | ------ |
| Unencrypted (passphrase-free) key  | ✅ Yes |
| Passphrase-protected + SSH agent   | ✅ Yes |
| Passphrase-protected, no SSH agent | ❌ No  |

A passphrase-protected key without an accessible SSH agent will cause every `provision`
(and later) step to fail with:

```text
Permission denied (publickey,password)
```

This error is indistinguishable from a wrong key or an unconfigured `authorized_keys`
file. The deployer will emit a warning during `create environment` if it detects a
passphrase-protected key so you can resolve this before reaching the `provision` step.

## Supported Workflows

### Workflow 1 — Passphrase-free dedicated deployment key (recommended)

Generate a dedicated key with no passphrase and use it only for deploying this
environment. This is the simplest and most portable approach.

```bash
ssh-keygen -t ed25519 -C "torrust-tracker-deployer" \
    -f ~/.ssh/torrust_tracker_deployer_ed25519
# Leave the passphrase empty (press Enter twice)
```

Configure it in your environment JSON:

```json
"ssh_credentials": {
  "private_key_path": "/home/you/.ssh/torrust_tracker_deployer_ed25519",
  "public_key_path":  "/home/you/.ssh/torrust_tracker_deployer_ed25519.pub"
}
```

### Workflow 2 — Passphrase-protected key with SSH agent forwarding into Docker

If you need to keep the passphrase on the key, you can forward your local SSH agent
socket into the deployer Docker container:

```bash
# Make sure your key is loaded into the agent
ssh-add ~/.ssh/torrust_tracker_deployer_ed25519

# Pass the agent socket when running the container
docker run --rm \
  -v "$SSH_AUTH_SOCK:/tmp/ssh-agent.sock" \
  -e SSH_AUTH_SOCK=/tmp/ssh-agent.sock \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  torrust/tracker-deployer:latest \
  provision my-environment
```

The deployer will use the agent socket to authenticate without needing the passphrase
in plaintext.

### Workflow 3 — Native (non-Docker) execution with an SSH agent on the host

When running the deployer binary directly (not in Docker), your desktop SSH agent is
typically already running and holds the unlocked key. The deployer inherits the
`SSH_AUTH_SOCK` environment variable automatically.

```bash
# Load your key once per session
ssh-add ~/.ssh/torrust_tracker_deployer_ed25519

# Run the deployer natively — the agent socket is inherited
torrust-tracker-deployer provision my-environment
```

## Removing an Existing Passphrase

If you already created a key with a passphrase and want to remove it:

```bash
ssh-keygen -p -f ~/.ssh/torrust_tracker_deployer_ed25519
# Enter old passphrase, then press Enter twice for the new (empty) passphrase
```

## Security Notes

- **Dedicated deployment keys** — Use a separate key pair for each deployer environment;
  never reuse your personal SSH key for automated deployments.
- **Key rotation** — Replace deployment keys after the deployment is complete or the
  environment is destroyed.
- **Filesystem permissions** — Private key files must be readable only by the owner:

  ```bash
  chmod 600 ~/.ssh/torrust_tracker_deployer_ed25519
  ```

- **Never commit private keys** — Add key paths to `.gitignore`; store them outside the
  repository.

## Configuration Reference

The SSH key paths are specified in the `ssh_credentials` section of the environment
configuration file:

```json
"ssh_credentials": {
  "private_key_path": "/absolute/path/to/private_key",
  "public_key_path":  "/absolute/path/to/private_key.pub",
  "username": "torrust",
  "port": 22
}
```

See the [environment config JSON schema](../../schemas/environment-config.json) for the
full `ssh_credentials` field documentation.

## Related Documentation

- [Create Environment Command](commands/create.md) — passphrase warning details
- [Hetzner Provider Guide](providers/hetzner.md) — SSH key requirements for cloud deployments
- [Security Guide](security.md) — broader security considerations
- [ADR: SSH Key Passphrase Detection](../../docs/decisions/ssh-key-passphrase-detection.md)
