# TC-07: UDP-Only Tracker

**Status**: NOT STARTED

## Test Case Details

**Purpose**: Minimal tracker with only the UDP protocol — no HTTP trackers.
**Environment name**: `pre-release-test-07`
**Config file**: `envs/pre-release-test-07.json`

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

| Check                  | Expected      | Result  | Notes |
| ---------------------- | ------------- | ------- | ----- |
| docker ps (containers) | tracker       | pending |       |
| tracker API (1212)     | OK            | pending |       |
| UDP port 6969          | open          | pending |       |
| HTTP port 7070         | NOT listening | pending |       |
| tracker logs (no err)  | clean         | pending |       |

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
