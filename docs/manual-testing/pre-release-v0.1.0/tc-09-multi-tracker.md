# TC-09: Multi-Tracker (2 UDP + 2 HTTP)

**Status**: NOT STARTED

## Test Case Details

**Purpose**: Test deploying multiple tracker instances on different ports.
**Environment name**: `pre-release-test-09`
**Config file**: `envs/pre-release-test-09.json`

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

## Deployment Steps

| Step       | Result  | Duration | Notes |
| ---------- | ------- | -------- | ----- |
| create env | pending | --       |       |
| provision  | pending | --       |       |
| configure  | pending | --       |       |
| release    | pending | --       |       |
| run        | pending | --       |       |
| test       | pending | --       |       |

## Verification Results

| Check                  | Expected             | Result  | Notes |
| ---------------------- | -------------------- | ------- | ----- |
| docker ps (containers) | tracker, prom, gra   | pending |       |
| tracker health (7070)  | OK                   | pending |       |
| tracker health (7071)  | OK                   | pending |       |
| tracker API (1212)     | OK                   | pending |       |
| UDP port 6969          | open                 | pending |       |
| UDP port 6970          | open                 | pending |       |
| health check (1313)    | externally reachable | pending |       |
| Prometheus targets     | all up (incl. both)  | pending |       |
| Grafana health (3000)  | OK                   | pending |       |
| tracker logs (no err)  | clean                | pending |       |

## Findings

### Bugs

None yet.

### Documentation Issues

None yet.

### UX Improvements

None yet.

### Code Quality

None yet.

## Cleanup

| Step    | Result  | Notes |
| ------- | ------- | ----- |
| destroy | pending |       |
| purge   | pending |       |
| verify  | pending |       |
