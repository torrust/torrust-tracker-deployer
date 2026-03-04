# Secrets Rotation

**Date**: 2026-03-04
**Reason**: Full deployment was performed with an AI coding agent (Claude Sonnet 4.6
via GitHub Copilot in Visual Studio Code). All secrets that appeared in terminal output,
configuration files, or SSH sessions were potentially sent to Microsoft (GitHub
Copilot), Anthropic (Claude), and processed by cloud infrastructure operated by
those companies. All live secrets must be rotated.

## What to Rotate vs Delete

| Secret                        | Action  | Reason                                                  |
| ----------------------------- | ------- | ------------------------------------------------------- |
| Tracker admin token           | ✅ Done | Rotated 2026-03-04 — tracker and Prometheus scraping OK |
| MySQL `torrust` user password | Rotate  | In `.env`, `tracker.toml`, `backup.conf`                |
| MySQL `root` user password    | Rotate  | In `.env`, terminal session                             |
| Grafana admin password        | ✅ Done | Rotated 2026-03-04                                      |
| SSH deployer key              | Rotate  | Agent ran `ssh`/`scp` commands using this key           |
| Hetzner Cloud API token       | ✅ Done | Deleted 2026-03-04 — no longer needed after deployment  |
| Hetzner DNS API token         | ✅ Done | Deleted 2026-03-04 — no longer needed after DNS setup   |
| Local sensitive files         | Archive | `build/`, `data/`, `envs/` dirs contain live secrets    |

> **Nothing to delete**: all tokens and keys are still needed for ongoing
> administration of the running instance, **except** the Hetzner Cloud and DNS
> API tokens which are only needed during deployment and can be deleted.

## Secret-to-Files Relationship Map

The same secret can appear in multiple configuration files. Every location must
be updated when rotating — missing one will cause service failures.

| Secret                   | Server file path                                     | Variable / field                                                              |
| ------------------------ | ---------------------------------------------------- | ----------------------------------------------------------------------------- |
| Tracker admin token      | `/opt/torrust/.env`                                  | `TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN`              |
| Tracker admin token      | `/opt/torrust/storage/prometheus/etc/prometheus.yml` | `params.token` in `tracker_stats` scrape job                                  |
| Tracker admin token      | `/opt/torrust/storage/prometheus/etc/prometheus.yml` | `params.token` in `tracker_metrics` scrape job                                |
| MySQL `torrust` password | `/opt/torrust/.env`                                  | `MYSQL_PASSWORD`                                                              |
| MySQL `torrust` password | `/opt/torrust/storage/tracker/etc/tracker.toml`      | `core.database.path` connection string (password is URL-encoded: `%2F` → `/`) |
| MySQL `torrust` password | `/opt/torrust/storage/backup/etc/backup.conf`        | `DB_PASSWORD`                                                                 |
| MySQL `root` password    | `/opt/torrust/.env`                                  | `MYSQL_ROOT_PASSWORD`                                                         |
| Grafana admin password   | `/opt/torrust/.env`                                  | `GF_SECURITY_ADMIN_PASSWORD`                                                  |

> **Rule**: whenever Step 1 updates the tracker admin token in `.env`, you must
> **also** update `prometheus.yml` (step 1b below) — otherwise Prometheus can no
> longer scrape tracker metrics.

## Suggested Password Generation Commands

Run these locally to generate new secrets before starting rotation. Use the
output as your `<NEW_...>` values below.

```bash
# New tracker admin token (URL-safe base64)
openssl rand -base64 32 | tr -d '\n='

# New MySQL torrust password (hex — no special chars, no URL-encoding needed)
openssl rand -hex 24

# New MySQL root password (hex)
openssl rand -hex 24

# New Grafana admin password
openssl rand -base64 24 | tr -d '\n='
```

Keep these values in a local password manager — do not share them with any AI
agent or paste them into a chat window.

---

## Step 1: Rotate the Tracker Admin Token ✅ Done (2026-03-04)

The admin token appears in **three places**: `.env` (used by the tracker
container at startup) and **two scrape jobs** in `prometheus.yml` (used by
Prometheus to poll `/api/v1/stats` and `/api/v1/metrics`). All three must be
updated together.

**On the server:**

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201
```

### 1a. Update `.env`

```bash
sudo vim /opt/torrust/.env
```

Change:

```text
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN='<OLD_TOKEN>'
```

To:

```text
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN='<NEW_TOKEN>'
```

### 1b. Update `prometheus.yml`

```bash
sudo vim /opt/torrust/storage/prometheus/etc/prometheus.yml
```

Find and replace the token in **both** scrape jobs (`tracker_stats` and
`tracker_metrics`):

```yaml
params:
  token: ["<OLD_TOKEN>"]
```

Change to:

```yaml
params:
  token: ["<NEW_TOKEN>"]
```

There are two occurrences — update both.

### 1c. Recreate tracker and Prometheus containers

> **Important**: `docker compose restart` is **not enough** here. The admin
> token is injected as an environment variable when the container is first
> created. `restart` only restarts the process inside the existing container —
> it does not re-read `.env`. You must recreate the containers to pick up the
> new value.

```bash
cd /opt/torrust && sudo docker compose up -d --force-recreate tracker prometheus
```

Verify the new token works:

```bash
curl -s "https://api.torrust-tracker-demo.com/api/v1/stats?token=<NEW_TOKEN>" | head -c 200
```

Verify Prometheus is scraping again (wait ~30 s then check targets):

```bash
curl -s http://localhost:9090/api/v1/targets | python3 -m json.tool | grep -A3 'scrapePool'
```

---

## Step 2: Rotate the MySQL Passwords

### 2a. Change passwords in MySQL

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201
cd /opt/torrust
sudo docker compose exec mysql mysql -u root -p
```

Enter the current root password when prompted. Then inside MySQL:

```sql
ALTER USER 'torrust'@'%' IDENTIFIED BY '<NEW_MYSQL_PASSWORD>';
ALTER USER 'root'@'%' IDENTIFIED BY '<NEW_MYSQL_ROOT_PASSWORD>';
ALTER USER 'root'@'localhost' IDENTIFIED BY '<NEW_MYSQL_ROOT_PASSWORD>';
FLUSH PRIVILEGES;
EXIT;
```

Verify the new torrust user password works:

```bash
sudo docker compose exec -e MYSQL_PWD="<NEW_MYSQL_PASSWORD>" mysql \
  mysql -u torrust torrust_tracker -e "SELECT COUNT(*) FROM torrents;"
```

### 2b. Update `/opt/torrust/.env`

```bash
sudo vim /opt/torrust/.env
```

Change:

```text
MYSQL_ROOT_PASSWORD='<OLD_MYSQL_ROOT_PASSWORD>'
MYSQL_PASSWORD='<OLD_MYSQL_PASSWORD>'
```

To:

```text
MYSQL_ROOT_PASSWORD='<NEW_MYSQL_ROOT_PASSWORD>'
MYSQL_PASSWORD='<NEW_MYSQL_PASSWORD>'
```

> **Note**: These vars only take effect when the MySQL container is first created.
> Since MySQL passwords are now changed directly in the database (step 2a), the
> `.env` update is for documentation and future container rebuilds only.

### 2c. Update `tracker.toml`

```bash
sudo vim /opt/torrust/storage/tracker/etc/tracker.toml
```

Find the line like:

```toml
path = "mysql://torrust:<OLD_PASSWORD_URL_ENCODED>@mysql:3306/torrust_tracker"
```

Change to (if your new password contains no special characters, no URL-encoding
needed):

```toml
path = "mysql://torrust:<NEW_MYSQL_PASSWORD>@mysql:3306/torrust_tracker"
```

> **Note**: If the new password contains `/`, encode it as `%2F`. If it contains
> `@`, encode it as `%40`. Using a hex password avoids this entirely.

### 2d. Update `backup.conf`

```bash
sudo vim /opt/torrust/storage/backup/etc/backup.conf
```

Change:

```text
DB_PASSWORD=<OLD_MYSQL_PASSWORD>
```

To:

```text
DB_PASSWORD=<NEW_MYSQL_PASSWORD>
```

### 2e. Restart tracker and backup containers

> **Note**: `restart` is sufficient here. Unlike the admin token (an env var),
> `tracker.toml` and `backup.conf` are bind-mounted files — the process re-reads
> them on each startup, so no container recreation is needed.

```bash
cd /opt/torrust
sudo docker compose restart tracker backup
```

Verify tracker is up and connected:

```bash
sudo docker compose ps tracker
sudo docker compose logs --tail=20 tracker
```

---

## Step 3: Rotate the Grafana Admin Password ✅ Done (2026-03-04)

### Option A: Change via the Grafana UI (simplest)

1. Log in at `https://grafana.torrust-tracker-demo.com` with `admin` / current
   password
2. Click your profile avatar (bottom-left) → **Profile**
3. Scroll to **Change Password**
4. Enter the current password and the new one, then click **Change Password**

Then update `.env` to keep it in sync:

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201
sudo vim /opt/torrust/.env
```

Change:

```text
GF_SECURITY_ADMIN_PASSWORD='<OLD_GRAFANA_PASSWORD>'
```

To:

```text
GF_SECURITY_ADMIN_PASSWORD='<NEW_GRAFANA_PASSWORD>'
```

### Option B: Change via the Grafana CLI (headless)

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201
cd /opt/torrust
sudo docker compose exec grafana grafana-cli admin reset-admin-password '<NEW_GRAFANA_PASSWORD>'
```

Then update `.env` as shown in Option A.

Verify login at `https://grafana.torrust-tracker-demo.com` with credentials
`admin` / `<NEW_GRAFANA_PASSWORD>`.

---

## Step 4: Rotate the SSH Deployer Key

The `torrust_tracker_deployer_ed25519` key was used by the AI agent to SSH into
the server and run `scp` commands. Generate a new key pair and replace it.

### 4a. Generate the new key pair (on your local machine)

```bash
ssh-keygen -t ed25519 -C "torrust-tracker-deployer" \
  -f ~/.ssh/torrust_tracker_deployer_ed25519_new
```

Leave the passphrase empty (or add one if you prefer).

### 4b. Add the new public key to the server

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201 \
  "echo '$(cat ~/.ssh/torrust_tracker_deployer_ed25519_new.pub)' >> ~/.ssh/authorized_keys"
```

### 4c. Test the new key

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519_new torrust@46.225.234.201 "echo OK"
```

### 4d. Remove the old public key from the server

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519_new torrust@46.225.234.201
```

Then inside the server:

```bash
vim ~/.ssh/authorized_keys
```

Delete the line with `torrust-tracker-deployer` that contains the **old** key
fingerprint (the first line — there should now be two entries, old and new).

### 4e. Replace the local key files

```bash
mv ~/.ssh/torrust_tracker_deployer_ed25519_new ~/.ssh/torrust_tracker_deployer_ed25519
mv ~/.ssh/torrust_tracker_deployer_ed25519_new.pub ~/.ssh/torrust_tracker_deployer_ed25519.pub
```

### 4f. Update the deployer env file

Edit `envs/torrust-tracker-demo.json` and update the `ssh_private_key_path`
field (or equivalent) to point to the same path — no change needed if the path
did not change.

---

## Step 5: Delete the Hetzner Cloud API Token ✅ Done (2026-03-04)

The token was only needed during provisioning (`provision` command). The server
is running and no further OpenTofu operations are planned, so it can be deleted.

1. Open [Hetzner Cloud Console](https://console.hetzner.cloud/)
2. Go to **Security → API Tokens**
3. Find the token used by the deployer (e.g. `torrust-tracker-deployer`)
4. Click **Delete**

If you need to run deployer commands again in the future, create a new token at
that point.

---

## Step 6: Delete the Hetzner DNS API Token ✅ Done (2026-03-04)

The token was only needed during DNS setup. All DNS records are in place and no
changes are planned, so it can be deleted.

1. Open [Hetzner DNS Console](https://dns.hetzner.com/)
2. Go to **API Tokens**
3. Delete the token used during setup

If you need to manage DNS records again in the future, create a new token at
that point.

---

## Step 7: Archive and Remove Local Sensitive Files

The following local directories and files were generated by the deployer and
contain real secrets (API tokens, passwords, SSH key paths). They are
git-ignored and must be archived in a safe place before being removed from
the local machine.

| Path                             | Sensitive contents                                          |
| -------------------------------- | ----------------------------------------------------------- |
| `build/torrust-tracker-demo/`    | Generated configs with real passwords and tokens            |
| `data/torrust-tracker-demo/`     | Deployment state including any cached credentials           |
| `envs/torrust-tracker-demo.json` | Full environment config: Hetzner tokens, DB passwords, etc. |

### 7a. Archive to a safe location

Move the files to an encrypted vault or password manager attachment before
deleting them. At minimum, store `envs/torrust-tracker-demo.json` — it is the
source of truth for recreating the deployment.

Example using a local encrypted directory (adjust path to your vault):

```bash
cp -r build/torrust-tracker-demo ~/vault/torrust-tracker-demo-build-2026-03-04
cp -r data/torrust-tracker-demo ~/vault/torrust-tracker-demo-data-2026-03-04
cp envs/torrust-tracker-demo.json ~/vault/torrust-tracker-demo-env-2026-03-04.json
```

> **Note**: Do not commit these files to git or store them in any cloud service
> accessible without encryption.

### 7b. Remove the local copies

Once archived safely:

```bash
rm -rf build/torrust-tracker-demo
rm -rf data/torrust-tracker-demo
rm envs/torrust-tracker-demo.json
```

> **When to do this**: After all secrets are rotated on the server (steps 1–4),
> so the archived files reflect the **old** credentials only. If you archive
> before rotation, you archive the same compromised secrets — which is still
> useful as a record, but make sure the archived copy is clearly labelled as
> pre-rotation.

---

## Verification Checklist

After completing all steps, run through this checklist:

- [ ] HTTP tracker responds: `curl -s https://http1.torrust-tracker-demo.com/announce?...`
- [ ] UDP tracker responds: BEP 15 handshake or `udp_tracker_client`
- [ ] API responds with new token: `curl .../api/v1/stats?token=<NEW_TOKEN>`
- [ ] Grafana login works with new password
- [ ] SSH access works with new key
- [ ] Application backup runs successfully — SSH in and run:
      `cd /opt/torrust && sudo docker compose run --rm backup`
- [ ] MySQL dumps are not empty (check last backup file size)
- [ ] Local sensitive files archived to safe storage (step 7)
- [ ] `build/torrust-tracker-demo/`, `data/torrust-tracker-demo/`, `envs/torrust-tracker-demo.json` removed from local machine

---

## Local Files to Update

| File                                      | What to update                                         |
| ----------------------------------------- | ------------------------------------------------------ |
| `envs/torrust-tracker-demo.json`          | Tracker admin token, MySQL passwords (if stored there) |
| `~/.ssh/torrust_tracker_deployer_ed25519` | New private key (step 4e)                              |
