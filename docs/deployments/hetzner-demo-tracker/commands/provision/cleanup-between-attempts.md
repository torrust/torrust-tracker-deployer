# Cleanup Between Provision Attempts

When `provision` fails, the environment is left in a `ProvisionFailed` state with a live
server still running on Hetzner. Before retrying, the failed environment must be fully
cleaned up — both on the provider and locally.

## Steps

### 1. Build the Docker image

If you have made code changes since the last attempt, rebuild the image first:

```bash
docker build --target release \
  --tag torrust/tracker-deployer:latest \
  --file docker/deployer/Dockerfile .
```

Skip this step if no code has changed since the last Docker build.

### 2. Destroy the failed environment on the provider

This runs `tofu destroy` to remove the Hetzner server:

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  destroy torrust-tracker-demo
```

### 3. Verify the server is gone on Hetzner

Log in to the [Hetzner Cloud Console](https://console.hetzner.cloud/) and confirm the
server no longer appears under the project. Also verify the floating IPs are unassigned
but still present (they are not destroyed by `destroy`).

### 4. Purge local environment data

This removes `build/torrust-tracker-demo/` and `data/torrust-tracker-demo/` and clears
the environment from the local registry:

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  purge torrust-tracker-demo --force
```

> **Note**: `--force` skips the interactive confirmation prompt. Omit it if you want to
> confirm interactively.

### 5. Clear the global log file

The global log at `data/logs/log.txt` accumulates entries across all environments and
sessions. Clearing it before a retry makes it much easier to isolate errors from the
new attempt:

```bash
> data/logs/log.txt
```

> **Note**: This truncates the file in-place (preserving the file itself). Git will not
> track the change since `data/logs/` is git-ignored.

---

After completing all steps, the environment is fully clean. You can now proceed to:

1. Re-create the environment: see [README.md](README.md)
2. Retry provision: see [README.md](README.md)
