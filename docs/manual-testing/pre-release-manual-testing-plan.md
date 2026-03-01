# Pre-Release v0.1.0 Manual Testing Plan

## Goal

Systematically test the Torrust Tracker Deployer across **15 diverse use cases** before the v0.1.0 release. Each test case exercises a different combination of configuration options using the **LXD (local) provider**, covering the full deployment lifecycle.

All tests are designed to be **executed autonomously by AI agents**, one at a time, sequentially.

## Approach

```text
For each test case:
  1. Generate the environment config (envs/pre-release-test-NN.json)
  2. Deploy using the full workflow: create → provision → configure → release → run → test
  3. Verify services are working (HTTP, UDP, API, monitoring, backups, etc.)
  4. Record findings in a structured report
  5. Destroy and purge the environment
  6. Continue with the next case
```

## Feature Dimensions Under Test

Each test case is a specific point in this feature matrix:

| Dimension         | Options                                           |
| ----------------- | ------------------------------------------------- |
| **Database**      | `sqlite3`, `mysql`                                |
| **UDP Trackers**  | 0, 1, or 2 instances                              |
| **HTTP Trackers** | 0, 1, or 2 instances                              |
| **Private mode**  | `true`, `false`                                   |
| **Prometheus**    | enabled / disabled                                |
| **Grafana**       | enabled / disabled (requires Prometheus)          |
| **Backup**        | enabled / disabled (custom schedule, retention)   |
| **HTTPS (Caddy)** | enabled (staging certs) / disabled                |
| **Health Check**  | localhost-only (`127.0.0.1`) / externally exposed |

### Constraints to Respect

- Grafana → requires Prometheus
- HTTPS section → requires at least one service with `use_tls_proxy: true`
- `use_tls_proxy: true` → requires a `domain` on that service
- `use_tls_proxy: true` → incompatible with localhost bind addresses
- UDP trackers have no TLS support

## Test Cases (15)

### TC-01: Minimal SQLite (baseline)

**Purpose**: Simplest possible deployment. Baseline for comparison.

| Feature       | Setting        |
| ------------- | -------------- |
| Database      | SQLite         |
| UDP Trackers  | 1 (port 6969)  |
| HTTP Trackers | 1 (port 7070)  |
| Private       | false          |
| HTTP API      | port 1212      |
| Health Check  | 127.0.0.1:1313 |
| Prometheus    | --             |
| Grafana       | --             |
| Backup        | --             |
| HTTPS         | --             |

**Verification**: tracker health check, HTTP announce, UDP port open, API health.

---

### TC-02: SQLite + Monitoring (Prometheus + Grafana)

**Purpose**: Test monitoring stack with the simplest database.

| Feature       | Setting              |
| ------------- | -------------------- |
| Database      | SQLite               |
| UDP Trackers  | 1 (port 6969)        |
| HTTP Trackers | 1 (port 7070)        |
| Private       | false                |
| HTTP API      | port 1212            |
| Health Check  | 0.0.0.0:1313         |
| Prometheus    | scrape_interval: 15s |
| Grafana       | admin/admin          |
| Backup        | --                   |
| HTTPS         | --                   |

**Verification**: all TC-01 checks + Prometheus targets up + Grafana login + datasource configured.

---

### TC-03: MySQL Basic

**Purpose**: Test MySQL database backend without optional services.

| Feature       | Setting        |
| ------------- | -------------- |
| Database      | MySQL          |
| UDP Trackers  | 1 (port 6969)  |
| HTTP Trackers | 1 (port 7070)  |
| Private       | false          |
| HTTP API      | port 1212      |
| Health Check  | 127.0.0.1:1313 |
| Prometheus    | --             |
| Grafana       | --             |
| Backup        | --             |
| HTTPS         | --             |

**Verification**: TC-01 checks + MySQL container healthy + tracker tables created + DB connectivity.

---

### TC-04: MySQL + Full Monitoring

**Purpose**: Test MySQL with Prometheus and Grafana.

| Feature       | Setting              |
| ------------- | -------------------- |
| Database      | MySQL                |
| UDP Trackers  | 1 (port 6969)        |
| HTTP Trackers | 1 (port 7070)        |
| Private       | false                |
| HTTP API      | port 1212            |
| Health Check  | 0.0.0.0:1313         |
| Prometheus    | scrape_interval: 30s |
| Grafana       | admin/grafana-pass   |
| Backup        | --                   |
| HTTPS         | --                   |

**Verification**: TC-03 checks + Prometheus scraping metrics + Grafana dashboards rendering.

---

### TC-05: SQLite + Backup

**Purpose**: Test backup service with SQLite database.

| Feature       | Setting                  |
| ------------- | ------------------------ |
| Database      | SQLite                   |
| UDP Trackers  | 1 (port 6969)            |
| HTTP Trackers | 1 (port 7070)            |
| Private       | false                    |
| HTTP API      | port 1212                |
| Health Check  | 127.0.0.1:1313           |
| Prometheus    | scrape_interval: 15s     |
| Grafana       | admin/admin              |
| Backup        | every 4h, retain 14 days |
| HTTPS         | --                       |

**Verification**: TC-02 checks + backup cron configured + manual backup trigger + backup file created.

---

### TC-06: MySQL + Backup

**Purpose**: Test backup service with MySQL database.

| Feature       | Setting                 |
| ------------- | ----------------------- |
| Database      | MySQL                   |
| UDP Trackers  | 1 (port 6969)           |
| HTTP Trackers | 1 (port 7070)           |
| Private       | false                   |
| HTTP API      | port 1212               |
| Health Check  | 127.0.0.1:1313          |
| Prometheus    | scrape_interval: 15s    |
| Grafana       | admin/admin             |
| Backup        | daily at 3am, retain 7d |
| HTTPS         | --                      |

**Verification**: TC-04 checks + backup cron configured + manual backup trigger + MySQL dump file created.

---

### TC-07: UDP-Only Tracker

**Purpose**: Minimal tracker with only the UDP protocol — no HTTP trackers.

| Feature       | Setting         |
| ------------- | --------------- |
| Database      | SQLite          |
| UDP Trackers  | 1 (port 6969)   |
| HTTP Trackers | 0 (empty array) |
| Private       | false           |
| HTTP API      | port 1212       |
| Health Check  | 127.0.0.1:1313  |
| Prometheus    | --              |
| Grafana       | --              |
| Backup        | --              |
| HTTPS         | --              |

**Verification**: UDP port listening + API health + verify no HTTP tracker port is open.

---

### TC-08: HTTP-Only Tracker

**Purpose**: HTTP tracker with no UDP — isolates HTTP announce protocol.

| Feature       | Setting         |
| ------------- | --------------- |
| Database      | SQLite          |
| UDP Trackers  | 0 (empty array) |
| HTTP Trackers | 1 (port 7070)   |
| Private       | false           |
| HTTP API      | port 1212       |
| Health Check  | 127.0.0.1:1313  |
| Prometheus    | --              |
| Grafana       | --              |
| Backup        | --              |
| HTTPS         | --              |

**Verification**: HTTP tracker health + announce works + API health + verify no UDP port is open.

---

### TC-09: Multi-Tracker (2 UDP + 2 HTTP)

**Purpose**: Test deploying multiple tracker instances on different ports.

| Feature       | Setting              |
| ------------- | -------------------- |
| Database      | SQLite               |
| UDP Trackers  | 2 (ports 6969, 6970) |
| HTTP Trackers | 2 (ports 7070, 7071) |
| Private       | false                |
| HTTP API      | port 1212            |
| Health Check  | 0.0.0.0:1313         |
| Prometheus    | scrape_interval: 15s |
| Grafana       | admin/admin          |
| Backup        | --                   |
| HTTPS         | --                   |

**Verification**: All 4 tracker ports responding + API health + Prometheus showing all targets.

---

### TC-10: Private Tracker (SQLite)

**Purpose**: Test private mode (whitelisting required).

| Feature       | Setting        |
| ------------- | -------------- |
| Database      | SQLite         |
| UDP Trackers  | 1 (port 6969)  |
| HTTP Trackers | 1 (port 7070)  |
| Private       | **true**       |
| HTTP API      | port 1212      |
| Health Check  | 127.0.0.1:1313 |
| Prometheus    | --             |
| Grafana       | --             |
| Backup        | --             |
| HTTPS         | --             |

**Verification**: Tracker running + API health + verify announce fails without whitelisted key.

---

### TC-11: Private Tracker (MySQL + Monitoring)

**Purpose**: Test private mode with MySQL and full monitoring stack.

| Feature       | Setting              |
| ------------- | -------------------- |
| Database      | MySQL                |
| UDP Trackers  | 1 (port 6969)        |
| HTTP Trackers | 1 (port 7070)        |
| Private       | **true**             |
| HTTP API      | port 1212            |
| Health Check  | 0.0.0.0:1313         |
| Prometheus    | scrape_interval: 15s |
| Grafana       | admin/private-pass   |
| Backup        | --                   |
| HTTPS         | --                   |

**Verification**: TC-10 checks + MySQL DB + Prometheus + Grafana all working.

---

### TC-12: Full Stack (everything enabled, SQLite)

**Purpose**: Maximum feature coverage with SQLite.

| Feature       | Setting                  |
| ------------- | ------------------------ |
| Database      | SQLite                   |
| UDP Trackers  | 2 (ports 6969, 6970)     |
| HTTP Trackers | 1 (port 7070)            |
| Private       | false                    |
| HTTP API      | port 1212                |
| Health Check  | 0.0.0.0:1313             |
| Prometheus    | scrape_interval: 15s     |
| Grafana       | admin/fullstack-pass     |
| Backup        | every 6h, retain 30 days |
| HTTPS         | --                       |

**Verification**: All services running + backup works + monitoring complete + multi-tracker.

---

### TC-13: Full Stack (everything enabled, MySQL)

**Purpose**: Maximum feature coverage with MySQL.

| Feature       | Setting                     |
| ------------- | --------------------------- |
| Database      | MySQL                       |
| UDP Trackers  | 1 (port 6969)               |
| HTTP Trackers | 2 (ports 7070, 7071)        |
| Private       | false                       |
| HTTP API      | port 1212                   |
| Health Check  | 0.0.0.0:1313                |
| Prometheus    | scrape_interval: 15s        |
| Grafana       | admin/fullstack-mysql       |
| Backup        | daily at 2am, retain 7 days |
| HTTPS         | --                          |

**Verification**: All services running + MySQL dump backup + monitoring complete.

---

### TC-14: Prometheus Only (no Grafana)

**Purpose**: Verify Prometheus works without Grafana.

| Feature       | Setting              |
| ------------- | -------------------- |
| Database      | SQLite               |
| UDP Trackers  | 1 (port 6969)        |
| HTTP Trackers | 1 (port 7070)        |
| Private       | false                |
| HTTP API      | port 1212            |
| Health Check  | 0.0.0.0:1313         |
| Prometheus    | scrape_interval: 10s |
| Grafana       | --                   |
| Backup        | --                   |
| HTTPS         | --                   |

**Verification**: Tracker health + Prometheus web UI accessible + targets scraped + no Grafana container running.

---

### TC-15: Backup Only (no Monitoring)

**Purpose**: Verify backup works without the monitoring stack.

| Feature       | Setting                 |
| ------------- | ----------------------- |
| Database      | MySQL                   |
| UDP Trackers  | 1 (port 6969)           |
| HTTP Trackers | 1 (port 7070)           |
| Private       | false                   |
| HTTP API      | port 1212               |
| Health Check  | 127.0.0.1:1313          |
| Prometheus    | --                      |
| Grafana       | --                      |
| Backup        | daily at 3am, retain 7d |
| HTTPS         | --                      |

**Verification**: Tracker + MySQL running + backup cron configured + manual backup trigger + MySQL dump created + no Prometheus/Grafana containers.

---

## Feature Coverage Matrix

| TC  | DB     | UDP | HTTP | Private | Prom | Graf | Backup | HTTPS | Health Exposed |
| --- | ------ | --- | ---- | ------- | ---- | ---- | ------ | ----- | -------------- |
| 01  | SQLite | 1   | 1    | -       | -    | -    | -      | -     | localhost      |
| 02  | SQLite | 1   | 1    | -       | Y    | Y    | -      | -     | exposed        |
| 03  | MySQL  | 1   | 1    | -       | -    | -    | -      | -     | localhost      |
| 04  | MySQL  | 1   | 1    | -       | Y    | Y    | -      | -     | exposed        |
| 05  | SQLite | 1   | 1    | -       | Y    | Y    | Y      | -     | localhost      |
| 06  | MySQL  | 1   | 1    | -       | Y    | Y    | Y      | -     | localhost      |
| 07  | SQLite | 1   | 0    | -       | -    | -    | -      | -     | localhost      |
| 08  | SQLite | 0   | 1    | -       | -    | -    | -      | -     | localhost      |
| 09  | SQLite | 2   | 2    | -       | Y    | Y    | -      | -     | exposed        |
| 10  | SQLite | 1   | 1    | Y       | -    | -    | -      | -     | localhost      |
| 11  | MySQL  | 1   | 1    | Y       | Y    | Y    | -      | -     | exposed        |
| 12  | SQLite | 2   | 1    | -       | Y    | Y    | Y      | -     | exposed        |
| 13  | MySQL  | 1   | 2    | -       | Y    | Y    | Y      | -     | exposed        |
| 14  | SQLite | 1   | 1    | -       | Y    | -    | -      | -     | exposed        |
| 15  | MySQL  | 1   | 1    | -       | -    | -    | Y      | -     | localhost      |

### Coverage Summary

- **SQLite**: 9 cases (01, 02, 05, 07, 08, 09, 10, 12, 14)
- **MySQL**: 6 cases (03, 04, 06, 11, 13, 15)
- **Private mode**: 2 cases (10, 11)
- **UDP-only**: 1 case (07)
- **HTTP-only**: 1 case (08)
- **Multi-tracker**: 2 cases (09, 12+13)
- **Prometheus only**: 1 case (14)
- **Backup only (no monitoring)**: 1 case (15)
- **Full stack (all features)**: 2 cases (12, 13)
- **No optional services**: 3 cases (01, 03, 07/08/10)
- **Health exposed externally**: 6 cases (02, 04, 09, 11, 12, 13, 14)

### Why HTTPS Is Not Included

HTTPS (Caddy + Let's Encrypt) requires real domain names with DNS resolution pointing to the VM's IP. In a local LXD environment, the VMs get private IPs (e.g., `10.x.x.x`) that are not reachable from Let's Encrypt validation servers. Even staging certificates would fail validation. HTTPS testing requires either:

- A Hetzner deployment with a real domain (out of scope for local pre-release testing)
- A separate test approach with self-signed certificates (future work)

HTTPS is therefore **intentionally excluded** from this LXD-based test suite.

## Execution Protocol for AI Agent

Each test case must follow this exact protocol:

### Phase 1: Setup

```bash
# 1. Create the environment config file
#    (agent writes the JSON to envs/pre-release-test-NN.json)

# 2. Create environment
cargo run -- create environment --env-file envs/pre-release-test-NN.json

# 3. Provision (~30-90s)
cargo run -- provision pre-release-test-NN --log-output file-and-stderr

# 4. Configure (~40-60s)
cargo run -- configure pre-release-test-NN

# 5. Release (~7-10s)
cargo run -- release pre-release-test-NN

# 6. Run (~10-15s)
cargo run -- run pre-release-test-NN
```

### Phase 2: Automated Verification

```bash
# 7. Run built-in test command
cargo run -- test pre-release-test-NN

# 8. Show environment info (capture for report)
cargo run -- show pre-release-test-NN
cargo run -- show pre-release-test-NN --output-format json
```

### Phase 3: Manual Verification (by AI agent)

The agent should SSH into the VM and verify:

```bash
# Get IP
export INSTANCE_IP=$(cargo run -- show pre-release-test-NN --output-format json 2>/dev/null | jq -r '.infrastructure.instance_ip')

# SSH into VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null torrust@$INSTANCE_IP

# Inside the VM:
docker ps                                     # All expected containers running
docker logs tracker 2>&1 | tail -20           # No errors in tracker logs
cd /opt/torrust && cat docker-compose.yml     # Verify compose file correctness
cat /opt/torrust/storage/tracker/etc/tracker.toml  # Verify tracker config
```

**Service-specific checks** (depending on test case features):

- **Tracker HTTP**: `curl http://$INSTANCE_IP:7070/health_check`
- **Tracker API**: `curl http://$INSTANCE_IP:1212/api/health_check`
- **UDP port**: `ss -ulnp | grep 6969` (from inside VM)
- **MySQL**: `docker exec mysql mysql -u tracker_user -p'password' -e 'SHOW TABLES;' tracker`
- **Prometheus**: `curl http://$INSTANCE_IP:9090/api/v1/targets | jq '.data.activeTargets[].health'`
- **Grafana**: `curl -u admin:password http://$INSTANCE_IP:3000/api/health`
- **Backup**: Check cron schedule with `crontab -l` or `docker exec backup-service cat /etc/crontabs/root`, trigger manual backup, verify backup file exists

### Phase 4: Report & Cleanup

```bash
# 9. Destroy
cargo run -- destroy pre-release-test-NN

# 10. Purge
cargo run -- purge pre-release-test-NN --force

# 11. Verify cleanup
lxc list | grep pre-release-test-NN || echo "VM cleaned up"
lxc profile list | grep pre-release-test-NN || echo "Profile cleaned up"
```

## Report Storage and Progress Tracking

All reports are **pre-generated** under `docs/manual-testing/pre-release-v0.1.0/`:

```text
docs/manual-testing/pre-release-v0.1.0/
├── README.md                          # Progress tracker + summary (update throughout)
├── tc-01-minimal-sqlite.md            # Pre-generated report (fill in results)
├── tc-02-sqlite-monitoring.md
├── tc-03-mysql-basic.md
├── tc-04-mysql-monitoring.md
├── tc-05-sqlite-backup.md
├── tc-06-mysql-backup.md
├── tc-07-udp-only.md
├── tc-08-http-only.md
├── tc-09-multi-tracker.md
├── tc-10-private-sqlite.md
├── tc-11-private-mysql-monitoring.md
├── tc-12-full-stack-sqlite.md
├── tc-13-full-stack-mysql.md
├── tc-14-prometheus-only.md
└── tc-15-backup-only.md
```

### How the Agent Tracks Progress

Each report file is **pre-populated** with the test case details, verification checklist, and a `pending` status in every table cell. The agent's job is to **fill in the results** as it executes each test:

1. **Before starting a test**: Update the test case report status from `NOT STARTED` → `IN PROGRESS` at the top of the file, and update the progress table in `README.md`.
2. **During the test**: Replace each `pending` cell with the actual result (`OK`, `ERR`, duration, notes).
3. **After the test**: Update the status to `PASS`, `FAIL`, or `PARTIAL`, fill in the Findings sections, and update the progress table in `README.md`.

The **README.md** contains a progress tracker table showing all 15 tests with their current status — this gives an at-a-glance view of overall progress.

### Report File Lifecycle

| Agent Action            | What to Update                                                     |
| ----------------------- | ------------------------------------------------------------------ |
| Starting test TC-NN     | `tc-NN-*.md`: status → `IN PROGRESS`; `README.md`: table row       |
| Each deployment step    | `tc-NN-*.md`: fill deployment step row (result, duration, notes)   |
| Each verification check | `tc-NN-*.md`: fill verification row (result, notes)                |
| Recording findings      | `tc-NN-*.md`: fill bugs/docs/UX/code-quality sections              |
| Cleanup done            | `tc-NN-*.md`: fill cleanup table; status → `PASS`/`FAIL`/`PARTIAL` |
| Update summary          | `README.md`: update progress table row + totals at top             |
| All 15 tests done       | `README.md`: fill aggregated findings and recommendations          |

Environment config files used during testing are stored in `envs/` as `pre-release-test-NN.json` and should be **deleted during the purge phase** of each test case (they are ephemeral).

## Report Template

The pre-generated report files already contain this structure. For reference, here is the template used:

```markdown
## TC-NN: <Test Case Name>

**Config file**: `envs/pre-release-test-NN.json`
**Status**: PASS / FAIL / PARTIAL
**Duration**: Xm Ys (total deployment time)

### Deployment Steps

| Step       | Result | Duration | Notes |
| ---------- | ------ | -------- | ----- |
| create env | OK/ERR | Xs       |       |
| provision  | OK/ERR | Xs       |       |
| configure  | OK/ERR | Xs       |       |
| release    | OK/ERR | Xs       |       |
| run        | OK/ERR | Xs       |       |
| test       | OK/ERR | Xs       |       |

### Verification Results

| Check                  | Result | Notes                 |
| ---------------------- | ------ | --------------------- |
| docker ps (containers) | OK/ERR | Expected N, found N   |
| tracker health         | OK/ERR |                       |
| tracker API            | OK/ERR |                       |
| UDP port(s)            | OK/ERR | N/A if no UDP         |
| HTTP tracker(s)        | OK/ERR | N/A if no HTTP        |
| MySQL connectivity     | OK/ERR | N/A if SQLite         |
| Prometheus targets     | OK/ERR | N/A if not configured |
| Grafana health         | OK/ERR | N/A if not configured |
| Backup configured      | OK/ERR | N/A if not configured |
| Backup execution       | OK/ERR | N/A if not configured |

### Findings

#### Bugs

- (list any bugs found)

#### Documentation Issues

- (things that are not clear in docs)

#### UX Improvements

- (things that could be improved: error messages, output formatting, etc.)

#### Code Quality

- (things that could be improved in code)

### Cleanup

| Step    | Result |
| ------- | ------ |
| destroy | OK/ERR |
| purge   | OK/ERR |
| verify  | OK/ERR |
```

## Report Aggregation

After all 15 test cases are complete, the agent must produce a summary report:

```markdown
# Pre-Release v0.1.0 Test Report Summary

**Date**: YYYY-MM-DD
**Agent**: <agent identifier>
**Total Tests**: 15
**Passed**: N
**Failed**: N
**Partial**: N

## All Findings (Deduplicated)

### Bugs (prioritized)

1. [CRITICAL] ...
2. [HIGH] ...
3. [MEDIUM] ...
4. [LOW] ...

### Documentation Issues

1. ...

### UX Improvements

1. ...

### Code Quality

1. ...

## Recommendations for v0.1.0

- Must fix before release: ...
- Nice to have: ...
- Can defer to v0.2.0: ...
```

## Execution Order

Run test cases in this order to build confidence incrementally:

1. **TC-01** (minimal baseline) — if this fails, nothing else will work
2. **TC-03** (MySQL baseline) — validates second database driver
3. **TC-07** (UDP-only) — edge case: no HTTP trackers
4. **TC-08** (HTTP-only) — edge case: no UDP trackers
5. **TC-02** (SQLite + monitoring) — adds Prometheus + Grafana
6. **TC-14** (Prometheus only, no Grafana) — isolation test
7. **TC-04** (MySQL + monitoring) — MySQL + monitoring combination
8. **TC-10** (private SQLite) — private mode test
9. **TC-05** (SQLite + backup) — backup with SQLite
10. **TC-06** (MySQL + backup) — backup with MySQL
11. **TC-15** (backup only, no monitoring) — backup isolation test
12. **TC-09** (multi-tracker) — multiple instances
13. **TC-11** (private MySQL + monitoring) — complex private mode
14. **TC-12** (full stack SQLite) — maximum coverage
15. **TC-13** (full stack MySQL) — maximum coverage

This order ensures:

- Simple cases are validated first (fail-fast)
- Edge cases (0 UDP, 0 HTTP) are tested early
- Each new test adds only one new dimension over a previously passing test
- Full-stack tests come last (dependent on all components working)

## Prerequisites

Before starting the test suite:

```bash
# 1. Build the project
cargo build

# 2. Verify dependencies
cargo run -p torrust-dependency-installer --bin dependency-installer -- check

# 3. Verify LXD is working
lxc list

# 4. Ensure no leftover environments
lxc list | grep torrust
ls data/ | grep pre-release

# 5. Clean up any leftovers from previous runs
# (only if needed — skip if clean)
```

## Time Estimate

| Phase        | Per test case | Total (15) |
| ------------ | ------------- | ---------- |
| Setup        | ~3 min        | ~45 min    |
| Verification | ~3 min        | ~45 min    |
| Cleanup      | ~1 min        | ~15 min    |
| Reporting    | ~2 min        | ~30 min    |
| **Total**    | **~9 min**    | **~2.5h**  |

Plus buffer for failures, retries, and debugging: **estimate 3-4 hours total**.

## Notes

- Each test case uses a **unique environment name** (`pre-release-test-01` through `pre-release-test-15`) to avoid conflicts with existing environments
- Each test case uses `fixtures/testing_rsa` / `fixtures/testing_rsa.pub` as SSH keys (they are already in the repo for testing)
- Profile names follow the pattern `torrust-profile-pre-release-test-NN`
- Test cases are run **sequentially** (one at a time) to avoid LXD resource contention
- The agent should clean up completely after each test before starting the next
- If a test case fails during deployment, the agent should still record the failure and proceed to cleanup before moving to the next test
