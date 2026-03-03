# Command: provision

> **Status**: ❌ ProvisionFailed — destroy + recreate required before retrying.
> See [problems.md](problems.md) for the full failure analysis and recovery steps.
> See [improvements.md](improvements.md) for recommended deployer improvements found during this phase.

## What `provision` does

The `provision` command:

1. Renders OpenTofu templates (Terraform HCL + cloud-init YAML) into `build/<env>/tofu/hetzner/`.
2. Runs `tofu init` + `tofu apply` to create the Hetzner server.
3. Waits for SSH connectivity (up to 120 seconds, 60 × 2 s intervals).
4. Marks the environment as `Provisioned` once SSH responds.

It does **not** install Docker, configure the tracker, or deploy any software — that is done by
the `configure`, `release`, and `run` commands.

## Command

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  provision torrust-tracker-demo
```

## Provisioned Server Details

The Hetzner server was created successfully by OpenTofu on 2026-03-03 at ~15:30 UTC.

| Property     | Value                                     |
| ------------ | ----------------------------------------- |
| Server name  | `torrust-tracker-vm-torrust-tracker-demo` |
| IPv4         | `46.225.234.201`                          |
| IPv6         | `2a01:4f8:1c19:620b::/64`                 |
| IPv6 address | `2a01:4f8:1c19:620b::1`                   |
| Location     | `nbg1` (Nuremberg, Germany)               |
| Server type  | `ccx23` (4 vCPU, 16 GB RAM)               |
| Image        | `ubuntu-24.04`                            |
| SSH user     | `torrust`                                 |

> **Note**: These are the server's own IPs assigned by Hetzner. The floating IPs
> (`116.202.176.169` IPv4, `2a01:4f8:1c0c:9aae::/64` IPv6) are separate and must be
> assigned to this server manually in the Hetzner Console after successful provisioning.

![Hetzner console showing the provisioned server details](../../media/hetzner-console-provisioned-server-details-attempt-1.png)

> **Note**: This screenshot shows the first successfully created server. A new server will be
> provisioned after destroying this one; the IP address will be different.

## Generated Artifacts

After provisioning the following build artifacts are created:

- `build/torrust-tracker-demo/tofu/hetzner/` — rendered OpenTofu project (HCL + cloud-init)
- `build/torrust-tracker-demo/tofu/hetzner/cloud-init.yml` — cloud-init config with SSH key
- `data/torrust-tracker-demo/environment.json` — state updated to `Provisioned` (or
  `ProvisionFailed` if something went wrong)
- `data/torrust-tracker-demo/traces/` — failure trace files (generated automatically on error)

## Verifying the SSH Key Injection

The rendered `build/torrust-tracker-demo/tofu/hetzner/cloud-init.yml` should contain the
public SSH key in the `ssh_authorized_keys` section:

```yaml
users:
  - name: torrust
    groups: sudo
    shell: /bin/bash
    sudo: ["ALL=(ALL) NOPASSWD:ALL"]
    ssh_authorized_keys:
      - ssh-ed25519 <KEY> torrust-tracker-deployer
```

To verify the key matches your local public key:

```bash
grep -A1 "ssh_authorized_keys" build/torrust-tracker-demo/tofu/hetzner/cloud-init.yml
cat ~/.ssh/torrust_tracker_deployer_ed25519.pub
```

## Manual SSH Verification (After Provisioning)

Once the server is up, verify SSH access directly from the host:

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@<SERVER_IP> "whoami && cloud-init status"
```

Expected output:

```text
torrust
status: done
```
