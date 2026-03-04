# Post-Provision Steps

These are manual steps performed **after** `provision` succeeds and **before** running `configure`.

They are not automated by the deployer CLI — they require actions in the Hetzner Console and
on the server via SSH.

## Steps

| Step            | Guide                              | Status     |
| --------------- | ---------------------------------- | ---------- |
| 1. DNS Setup    | [dns-setup.md](dns-setup.md)       | ⏳ Pending |
| 2. Volume Setup | [volume-setup.md](volume-setup.md) | ⏳ Pending |

## Why Before `configure`?

- **DNS**: The `configure` command installs Caddy as a TLS reverse proxy. Caddy uses
  Let's Encrypt to obtain TLS certificates for the domains in the environment config. This
  requires that all DNS records already resolve to the server's IP **before** `configure` runs,
  otherwise the ACME challenge will fail and TLS setup will break.

- **Volume**: The `configure` command creates `/opt/torrust/` and its subdirectories on the
  server. If the external volume is attached and mounted at `/opt/torrust/storage` **before**
  `configure` runs, Ansible writes all persistent data directly onto the volume — so nothing
  needs to be migrated afterwards.
