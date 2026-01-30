# Phase 1: Environment Setup

**Status**: ✅ Complete
**Date**: 2026-01-29

## Goal

Create and deploy a working tracker environment with MySQL.

## Checklist

- [x] Create environment configuration file
- [x] Run deployer commands: create → provision → configure → release → run
- [x] Verify tracker is running and accessible
- [x] Verify MySQL has tracker tables

## Artifacts

- [environment-config.json](../artifacts/environment-config.json) - Environment
  configuration file

## Commands Executed

### 1. Create Environment

```bash
cargo run -- create environment --env-file envs/manual-test-sidecar-backup.json
```

**Output**:

```text
⏳ [1/3] Loading configuration...
⏳     → Loading configuration from 'envs/manual-test-sidecar-backup.json'...
⏳   ✓ Configuration loaded: manual-test-sidecar-backup (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Creating environment...
⏳     → Creating environment 'manual-test-sidecar-backup'...
⏳     → Validating configuration and creating environment...
⏳   ✓ Environment created: manual-test-sidecar-backup (took 1ms)
✅ Environment 'manual-test-sidecar-backup' created successfully

Environment Details:
1. Environment name: manual-test-sidecar-backup
2. Instance name: torrust-tracker-vm-manual-test-sidecar-backup
3. Data directory: ./data/manual-test-sidecar-backup
4. Build directory: ./build/manual-test-sidecar-backup
```

### 2. Provision Infrastructure

```bash
cargo run -- provision manual-test-sidecar-backup
```

**Output**:

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: manual-test-sidecar-backup (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
⏳   ✓ Infrastructure provisioned (took 28.0s)
✅ Environment 'manual-test-sidecar-backup' provisioned successfully

Instance Connection Details:
  IP Address:        10.140.190.35
  SSH Port:          22
  SSH Private Key:   fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i fixtures/testing_rsa torrust@10.140.190.35 -p 22
```

**Duration**: 28 seconds

### 3. Configure Environment

```bash
cargo run -- configure manual-test-sidecar-backup
```

**Output**:

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: manual-test-sidecar-backup (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
⏳   ✓ Infrastructure configured (took 45.4s)
✅ Environment 'manual-test-sidecar-backup' configured successfully
```

**Duration**: 45 seconds

### 4. Release Application

```bash
cargo run -- release manual-test-sidecar-backup
```

**Output**:

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: manual-test-sidecar-backup (took 0ms)
⏳ [2/2] Releasing application...
⏳   ✓ Application released successfully (took 15.8s)
✅ Release command completed successfully for 'manual-test-sidecar-backup'
```

**Duration**: 16 seconds

### 5. Run Services

```bash
cargo run -- run manual-test-sidecar-backup
```

**Output**:

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: manual-test-sidecar-backup (took 0ms)
⏳ [2/2] Running application services...
⏳   ✓ Services started (took 37.3s)
✅ Run command completed for 'manual-test-sidecar-backup'
```

**Duration**: 37 seconds

## Validation

### Docker Compose Services

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 "cd /opt/torrust && docker compose ps"
```

**Output**:

```text
NAME         IMAGE                     COMMAND                  SERVICE      CREATED         STATUS                   PORTS
grafana      grafana/grafana:12.3.1    "/run.sh"                grafana      3 minutes ago   Up 3 minutes (healthy)   0.0.0.0:3000->3000/tcp
mysql        mysql:8.4                 "docker-entrypoint.s…"   mysql        3 minutes ago   Up 3 minutes (healthy)   3306/tcp, 33060/tcp
prometheus   prom/prometheus:v3.5.0    "/bin/prometheus --c…"   prometheus   3 minutes ago   Up 3 minutes (healthy)   127.0.0.1:9090->9090/tcp
tracker      torrust/tracker:develop   "/usr/local/bin/entr…"   tracker      3 minutes ago   Up 3 minutes (healthy)   0.0.0.0:1212->1212/tcp, 0.0.0.0:7070->7070/tcp, 0.0.0.0:6969->6969/udp
```

✅ All 4 services running and healthy.

### MySQL Tables (InnoDB Verification)

```bash
docker compose exec mysql mysql -u tracker_user -ptracker_password torrust_tracker \
  -e "SELECT TABLE_NAME, ENGINE FROM information_schema.TABLES WHERE TABLE_SCHEMA = 'torrust_tracker';"
```

**Output**:

```text
TABLE_NAME                 ENGINE
keys                       InnoDB
torrent_aggregate_metrics  InnoDB
torrents                   InnoDB
whitelist                  InnoDB
```

✅ All 4 tables using InnoDB engine (supports `--single-transaction`).

### Tracker API

```bash
curl -s http://10.140.190.35:1212/api/v1/stats?token=MyAccessToken
```

**Output**:

```json
{"torrents":0,"seeders":0,"completed":0,"leechers":0,...}
```

✅ Tracker API responding.

## Instance Details

| Property    | Value                                               |
| ----------- | --------------------------------------------------- |
| IP Address  | `10.140.190.35`                                     |
| SSH Command | `ssh -i fixtures/testing_rsa torrust@10.140.190.35` |
| Tracker API | `http://10.140.190.35:1212`                         |
| Grafana     | `http://10.140.190.35:3000`                         |

## Issues Encountered

None.

## Next Steps

Proceed to [Phase 2: Minimal Backup Container](02-minimal-container.md).
