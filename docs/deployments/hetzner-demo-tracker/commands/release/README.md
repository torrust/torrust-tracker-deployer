# Command: release

> **Status**: 🔄 In progress (2026-03-04)

## What `release` does

The `release` command:

1. Pulls the latest Docker images for the tracker and monitoring stack on the server.
2. Stages the release artifacts.
3. Marks the environment as `Released` on success.

It does **not** start the services — that is done by the `run` command.

## Command

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  release torrust-tracker-demo 2>&1 | tee -a data/logs/log.txt
```

## Output

<!-- Populated after running the command -->

## Problems

<!-- Populated if issues are encountered -->
