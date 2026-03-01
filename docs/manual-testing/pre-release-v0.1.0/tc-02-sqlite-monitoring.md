# TC-02: SQLite + Monitoring (Prometheus + Grafana)

**Status**: NOT STARTED

## Test Case Details

**Purpose**: Test monitoring stack with the simplest database.
**Environment name**: `pre-release-test-02`
**Config file**: `envs/pre-release-test-02.json`

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

| Check                    | Expected              | Result  | Notes |
| ------------------------ | --------------------- | ------- | ----- |
| docker ps (containers)   | tracker, prom, gra    | pending |       |
| tracker health (7070)    | OK                    | pending |       |
| tracker API (1212)       | OK                    | pending |       |
| UDP port 6969            | open                  | pending |       |
| health check (1313)      | externally reachable  | pending |       |
| Prometheus targets       | all up                | pending |       |
| Prometheus web UI (9090) | accessible            | pending |       |
| Grafana health (3000)    | OK                    | pending |       |
| Grafana login            | admin/admin works     | pending |       |
| Grafana datasource       | Prometheus configured | pending |       |
| tracker logs (no err)    | clean                 | pending |       |

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
