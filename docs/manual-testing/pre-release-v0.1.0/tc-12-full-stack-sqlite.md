# TC-12: Full Stack (everything enabled, SQLite)

**Status**: NOT STARTED

## Test Case Details

**Purpose**: Maximum feature coverage with SQLite.
**Environment name**: `pre-release-test-12`
**Config file**: `envs/pre-release-test-12.json`

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

| Check                    | Expected                   | Result  | Notes |
| ------------------------ | -------------------------- | ------- | ----- |
| docker ps (containers)   | tracker, prom, gra, backup | pending |       |
| tracker health (7070)    | OK                         | pending |       |
| tracker API (1212)       | OK                         | pending |       |
| UDP port 6969            | open                       | pending |       |
| UDP port 6970            | open                       | pending |       |
| health check (1313)      | externally reachable       | pending |       |
| Prometheus targets       | all up                     | pending |       |
| Prometheus web UI (9090) | accessible                 | pending |       |
| Grafana health (3000)    | OK                         | pending |       |
| Grafana login            | admin/fullstack-pass       | pending |       |
| Grafana datasource       | Prometheus configured      | pending |       |
| backup cron configured   | every 6h                   | pending |       |
| manual backup trigger    | succeeds                   | pending |       |
| backup file created      | SQLite .db copy            | pending |       |
| backup retention         | 30 days configured         | pending |       |
| tracker logs (no err)    | clean                      | pending |       |

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
