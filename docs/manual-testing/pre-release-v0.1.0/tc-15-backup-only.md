# TC-15: Backup Only (no Monitoring)

**Status**: NOT STARTED

## Test Case Details

**Purpose**: Verify backup works without the monitoring stack.
**Environment name**: `pre-release-test-15`
**Config file**: `envs/pre-release-test-15.json`

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

| Check                  | Expected               | Result  | Notes |
| ---------------------- | ---------------------- | ------- | ----- |
| docker ps (containers) | tracker, mysql, backup | pending |       |
| tracker health (7070)  | OK                     | pending |       |
| tracker API (1212)     | OK                     | pending |       |
| UDP port 6969          | open                   | pending |       |
| MySQL container health | healthy                | pending |       |
| MySQL tables created   | torrust tables exist   | pending |       |
| Prometheus container   | NOT running            | pending |       |
| Grafana container      | NOT running            | pending |       |
| backup cron configured | daily at 3am           | pending |       |
| manual backup trigger  | succeeds               | pending |       |
| backup file created    | MySQL dump file        | pending |       |
| backup retention       | 7 days configured      | pending |       |
| tracker logs (no err)  | clean                  | pending |       |

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
