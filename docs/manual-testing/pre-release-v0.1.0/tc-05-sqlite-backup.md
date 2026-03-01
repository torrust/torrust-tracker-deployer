# TC-05: SQLite + Backup

**Status**: NOT STARTED

## Test Case Details

**Purpose**: Test backup service with SQLite database.
**Environment name**: `pre-release-test-05`
**Config file**: `envs/pre-release-test-05.json`

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

| Check                  | Expected                   | Result  | Notes |
| ---------------------- | -------------------------- | ------- | ----- |
| docker ps (containers) | tracker, prom, gra, backup | pending |       |
| tracker health (7070)  | OK                         | pending |       |
| tracker API (1212)     | OK                         | pending |       |
| UDP port 6969          | open                       | pending |       |
| Prometheus targets     | all up                     | pending |       |
| Grafana health (3000)  | OK                         | pending |       |
| backup cron configured | every 4h                   | pending |       |
| manual backup trigger  | succeeds                   | pending |       |
| backup file created    | SQLite .db copy            | pending |       |
| backup retention       | 14 days configured         | pending |       |
| tracker logs (no err)  | clean                      | pending |       |

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
