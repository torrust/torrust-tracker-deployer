# Post-Provision Steps

These are manual steps performed **after** `provision` succeeds and **before** running `configure`.

They are not automated by the deployer CLI — they require actions in the Hetzner Console and
on the server via SSH.

## Steps

| Step               | Guide                                    | Status  |
| ------------------ | ---------------------------------------- | ------- |
| 1. DNS Setup       | [dns-setup.md](dns-setup.md)             | ✅ Done |
| 2. Volume Setup    | [volume-setup.md](volume-setup.md)       | ✅ Done |
| 3. Hetzner Backups | [hetzner-backups.md](hetzner-backups.md) | ✅ Done |

## Post-Deployment Steps

Steps performed after the tracker is running and during ongoing operations:

| Step                        | Guide                                                      | Status         |
| --------------------------- | ---------------------------------------------------------- | -------------- |
| 4. newTrackon Prerequisites | [newtrackon-prerequisites.md](newtrackon-prerequisites.md) | 🔄 In Progress |
| 5. IPv6 UDP Tracker Issue   | [ipv6-udp-tracker-issue.md](ipv6-udp-tracker-issue.md)     | 🔴 Unresolved  |

## Why Before `configure`?

- **DNS**: The `configure` command installs Caddy as a TLS reverse proxy. Caddy uses
  Let's Encrypt to obtain TLS certificates for the domains in the environment config. This
  requires that all DNS records already resolve to the server's IP **before** `configure` runs,
  otherwise the ACME challenge will fail and TLS setup will break.

- **Volume**: The `configure` command creates `/opt/torrust/` and its subdirectories on the
  server. If the external volume is attached and mounted at `/opt/torrust/storage` **before**
  `configure` runs, Ansible writes all persistent data directly onto the volume — so nothing
  needs to be migrated afterwards.

## Design Notes: Tradeoffs in Setup Sequencing

### Deployer state recovery is intentionally absent

The deployer has no failure-recovery mechanism — if a command fails (e.g. `provision`), the
environment is left in a failed state (e.g. `ProvisioningFailed`) and the only path forward is
to clean up and start from scratch. This was a deliberate design decision:

1. **Complexity**: Implementing robust recovery logic for partial infrastructure states
   (half-created servers, partial Ansible runs, etc.) is significantly complex.
2. **Speed**: Recreating a server from scratch takes less than 5 minutes.

This tradeoff works well for the core deployment flow (`provision` → `configure` → `release`
→ `run`). However, it interacts poorly with the extra setup steps introduced by this
deployment's use of floating IPs and an external storage volume.

### Volume attachment timing: a sequencing dilemma

For this demo deployment we chose to add two infrastructure extras:

- **Floating IPs** — so DNS records stay stable if the server is ever recreated.
- **External storage volume** — so tracker data survives server recreation and can be
  backed up independently.

These extras must currently be set up **before** `configure` (see "Why Before `configure`?")
because Ansible writes data directly to `/opt/torrust/storage/` during `configure`. If the
volume is not mounted at that path beforehand, data lands on the root disk and would need to
be migrated later.

This creates a problem when the deployment fails and must be restarted from scratch:

- **Floating IPs**: No problem — the IPs are already assigned and the DNS records already
  point to them. When a new server is created, you simply reassign the floating IPs to it.
  No DNS changes needed.
- **Volume**: The volume must be detached from the old server, then reattached and remounted
  on the new server. All the fstab and mount point setup must be repeated. This is not
  complex, but it is manual work.

### Alternative: defer volume setup until after `run` succeeds

An alternative approach would be to:

1. Run the full deployment (`provision` → `configure` → `release` → `run`) without the
   external volume — all data lands on the root disk at `/opt/torrust/storage/`.
2. Only after `run` succeeds and the tracker is confirmed working, attach the volume and
   migrate the data directory to it.

**Pros**: If `provision` fails and you need to start over, there is no volume to reattach
— just provision a new server and rerun the deployment commands.

**Cons**: Requires a data migration step (copy `/opt/torrust/storage/` from the root disk to
the volume, update the mount point) after the deployment is confirmed working. Slightly
more complex as a one-time operation.

For a long-running production server this migration cost is worth it. For a demo that may be
re-provisioned many times during development, the current approach (volume before `configure`)
is acceptable since it only requires rerunning the volume setup script.
