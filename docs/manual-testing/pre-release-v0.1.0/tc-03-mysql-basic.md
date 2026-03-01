# TC-03: MySQL Basic

**Status**: NOT STARTED

## Test Case Details

**Purpose**: Test MySQL database backend without optional services.
**Environment name**: `pre-release-test-03`
**Config file**: `envs/pre-release-test-03.json`

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
| docker ps (containers) | tracker, mysql       | pending |       |
| tracker health (7070)  | OK                   | pending |       |
| tracker API (1212)     | OK                   | pending |       |
| UDP port 6969          | open                 | pending |       |
| MySQL container health | healthy              | pending |       |
| MySQL tables created   | torrust tables exist | pending |       |
| MySQL connectivity     | tracker connects     | pending |       |
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
