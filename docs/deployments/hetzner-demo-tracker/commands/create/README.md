# Command: create

The `create` command has two sub-commands used before provisioning:

1. `create template` — generate a starter config file for a given provider
2. `create environment` — register the config with the deployer and create local state

See [problems.md](problems.md) for issues encountered when running these commands.

## create template

Generates a fully-featured JSON template for the chosen provider with placeholders for all
required fields.

```bash
docker run --rm \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  create template --provider hetzner /var/lib/torrust/deployer/envs/torrust-tracker-demo.json
```

The generated file is written to `envs/torrust-tracker-demo.json`. Edit it and replace:

| Placeholder                                  | Value                                                                       |
| -------------------------------------------- | --------------------------------------------------------------------------- |
| `REPLACE_WITH_ENVIRONMENT_NAME`              | `torrust-tracker-demo`                                                      |
| `REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH` | `/home/deployer/.ssh/torrust_tracker_deployer_ed25519` (container path)     |
| `REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH`  | `/home/deployer/.ssh/torrust_tracker_deployer_ed25519.pub` (container path) |
| `REPLACE_WITH_HETZNER_API_TOKEN`             | Your Hetzner API token (never commit this)                                  |

> **Important**: When running via Docker, all file paths in the config must be container-internal
> paths (e.g. `/home/deployer/.ssh/...`), not host paths. See [problems.md](problems.md).

See [deployment-spec.md](../../deployment-spec.md) for the full set of decisions and the sanitized
config used for this deployment.

## validate

After editing the config, validate it before registering the environment:

```bash
docker run --rm \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  validate --env-file /var/lib/torrust/deployer/envs/torrust-tracker-demo.json
```

Output confirming the config is valid:

```json
{
  "environment_name": "torrust-tracker-demo",
  "config_file": "envs/torrust-tracker-demo.json",
  "provider": "hetzner",
  "is_valid": true,
  "has_prometheus": true,
  "has_grafana": true,
  "has_https": true,
  "has_backup": true
}
```

> **Note**: In this deployment we used `cargo run --bin torrust-tracker-deployer validate ...`
> because we ran from source. For end-users the Docker command above is equivalent and simpler.

## create environment

Once the config is validated, register the environment with the deployer:

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  create environment --env-file /var/lib/torrust/deployer/envs/torrust-tracker-demo.json
```

Output:

```json
{
  "environment_name": "torrust-tracker-demo",
  "instance_name": "torrust-tracker-vm-torrust-tracker-demo",
  "data_dir": "./data/torrust-tracker-demo",
  "build_dir": "./build/torrust-tracker-demo",
  "created_at": "2026-03-03T13:59:22.635908188Z"
}
```

The deployer creates `data/torrust-tracker-demo/environment.json` with the internal state. This
file holds the environment's domain model — it is managed exclusively by the deployer and must
never be edited manually.

As shown in the output, `instance_name: null` in the config results in an auto-generated name:
`torrust-tracker-vm-torrust-tracker-demo`. This will be the name of the Hetzner VM.
