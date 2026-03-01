# TC-11: Private Tracker (MySQL + Monitoring)

**Status**: NOT STARTED

## Test Case Details

**Purpose**: Test private mode with MySQL and full monitoring stack.
**Environment name**: `pre-release-test-11`
**Config file**: `envs/pre-release-test-11.json`

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

| Check                       | Expected                  | Result  | Notes |
| --------------------------- | ------------------------- | ------- | ----- |
| docker ps (containers)      | tracker, mysql, prom, gra | pending |       |
| tracker health (7070)       | OK                        | pending |       |
| tracker API (1212)          | OK                        | pending |       |
| UDP port 6969               | open                      | pending |       |
| health check (1313)         | externally reachable      | pending |       |
| announce without key        | rejected / fails          | pending |       |
| tracker config private=true | confirmed in tracker.toml | pending |       |
| MySQL container health      | healthy                   | pending |       |
| MySQL tables created        | torrust tables exist      | pending |       |
| Prometheus targets          | all up                    | pending |       |
| Grafana health (3000)       | OK                        | pending |       |
| Grafana login               | admin/private-pass        | pending |       |
| tracker logs (no err)       | clean                     | pending |       |

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
