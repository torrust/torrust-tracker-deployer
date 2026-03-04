# Command: run

> **Status**: ⏳ Not yet run

## What `run` does

The `run` command:

1. Starts all Docker Compose services on the server (tracker, Prometheus, Grafana).
2. Waits for the services to become healthy.
3. Marks the environment as `Running` on success.

It requires the environment to already be in a `Released` state (i.e., `release` must have been
run first).

## Command

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  run torrust-tracker-demo 2>&1 | tee -a data/logs/log.txt
```

## Output

<!-- Populated after running the command -->
