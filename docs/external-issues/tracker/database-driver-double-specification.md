# Tracker Database Driver Double Specification Issue

**Issue Date**: December 14, 2025  
**Affected Component**: Torrust Tracker Container (`torrust/tracker:develop`)  
**Status**: Documented - Issue to be filed in tracker repository

## Problem Description

The tracker container's entrypoint script requires the database driver to be specified **twice**:

1. **In the tracker configuration file** (`tracker.toml`): `[core.database] driver = "mysql"`
2. **As an environment variable**: `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=mysql`

Even when mounting a complete, valid `tracker.toml` configuration file with the correct driver setting, the container entrypoint will **overwrite** the mounted config file if the environment variable is not set.

## Root Cause

The tracker container's entrypoint script ([`entry_script_sh`](https://github.com/torrust/torrust-tracker/blob/develop/share/container/entry_script_sh)) **requires** the `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER` environment variable to be set, and uses it to select a default configuration template:

```bash
# Entrypoint exits with error if variable not set
if [ -n "$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER" ]; then
    if cmp_lc "$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER" "sqlite3"; then
        default_config="/usr/share/torrust/default/config/tracker.container.sqlite3.toml"
    elif cmp_lc "$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER" "mysql"; then
        default_config="/usr/share/torrust/default/config/tracker.container.mysql.toml"
    else
        echo "Error: Unsupported Database Type"
        exit 1
    fi
else
    echo "Error: \$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER was not set!"
    exit 1
fi

# Then installs the selected default config
inst "$default_config" "$install_config"  # Copies to /etc/torrust/tracker/tracker.toml
```

This behavior occurs **before** the tracker application starts and loads the configuration, meaning:

- Container starts
- Entrypoint script runs
- **Checks for `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER`**
- **If not set → exits with error (container fails to start)**
- If set → copies appropriate default template to `/etc/torrust/tracker/tracker.toml`
- **Overwrites any mounted configuration file at that path**
- Tracker application loads the (now overwritten) config file

## Impact

When the environment variable is missing:

1. **Container fails to start** with error message: `Error: $TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER was not set!`

When the environment variable is set but doesn't match the mounted config:

1. The entrypoint overwrites the mounted configuration with the default template for the specified driver
2. If driver value mismatches user's intent, tracker may fail to start or exhibit unexpected behavior

**Previous behavior** (legacy containers): Would default to `sqlite3` and overwrite mounted MySQL configs, causing:

- Configuration with driver/path mismatch: `driver: sqlite3` + `path: mysql://...`
- SQLite driver attempting to open MySQL connection string
- Container panic and crash loop:

  ```text
  thread 'main' panicked at 'unable to open database file: mysql://tracker_user:tracker_password@mysql:3306/torrust_tracker'
  ```

## Current Workaround

### Docker Compose

Set the database driver environment variable in `docker-compose.yml`:

```yaml
services:
  tracker:
    environment:
      - TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=${DATABASE_DRIVER}
```

And define it in `.env`:

```dotenv
# Database driver type - tells the container entrypoint which config template to use
# Must match the driver specified in tracker.toml
# Uses standardized TORRUST_TRACKER_CONFIG_OVERRIDE_* naming convention
TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER='mysql'
```

**Note**: The standardized variable name `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER` is used by the current `develop` branch. Legacy container versions may have used `DATABASE_DRIVER`.

### Direct Docker

```bash
docker run -e TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=mysql torrust/tracker:develop
```

## Environment Variable Naming

**Resolution**: The current `torrust/tracker:develop` image uses the **standardized** naming convention:

- **Current entrypoint** ([`develop` branch](https://github.com/torrust/torrust-tracker/blob/develop/share/container/entry_script_sh)): `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER`
- **Live demo** ([`torrust-demo`](https://github.com/torrust/torrust-demo/blob/main/compose.yaml)): `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER`
- **Our templates**: Now updated to use `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER`

The entrypoint script checks for this variable and **requires** it to be set, otherwise it exits with an error:

```bash
if [ -n "$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER" ]; then
    # Select appropriate config template
    if cmp_lc "$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER" "sqlite3"; then
        default_config="/usr/share/torrust/default/config/tracker.container.sqlite3.toml"
    elif cmp_lc "$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER" "mysql"; then
        default_config="/usr/share/torrust/default/config/tracker.container.mysql.toml"
    else
        echo "Error: Unsupported Database Type"
        exit 1
    fi
else
    echo "Error: \$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER was not set!"
    exit 1
fi
```

**Note**: Legacy container versions may have used `DATABASE_DRIVER`, but this is no longer supported in current images.

## Expected vs Actual Behavior

### Expected Behavior

When mounting a complete configuration file:

```bash
docker run \
  -v ./tracker.toml:/etc/torrust/tracker/tracker.toml:ro \
  torrust/tracker:develop
```

The tracker should:

- Load the mounted configuration file as-is
- Respect all settings from the file
- Not require environment variables for settings already in the config

### Actual Behavior

The entrypoint script requires environment variables **even when config file is complete**:

```bash
# Without TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER env var:
# → Entrypoint exits with error: "$TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER was not set!"
# → Container fails to start

# With TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER env var:
docker run \
  -e TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=mysql \
  -v ./tracker.toml:/etc/torrust/tracker/tracker.toml:ro \
  torrust/tracker:develop
# → Entrypoint selects MySQL default template
# → Overwrites mounted config with default MySQL template
# → Tracker starts with default MySQL configuration (not the mounted one!)
```

**Note**: The mounted configuration is **always overwritten**, regardless of whether the environment variable is set. The variable only controls which default template is used.

## Design Concern

This behavior creates a tight coupling between:

1. The entrypoint script's template selection logic
2. The application's configuration loading
3. The user's environment variable management

It requires users to specify the driver in **two places** that must remain synchronized, which violates the DRY (Don't Repeat Yourself) principle and creates potential for configuration drift.

## Recommended Solution

The tracker entrypoint script should be modified to:

1. **Check if config file already exists** before template copying
2. **Only copy default template if no config mounted**
3. **Trust mounted configurations** when provided
4. **Remove requirement** for environment variable when complete config is available

Example improved logic:

```bash
# Only generate config if one doesn't exist
if [ ! -f /etc/torrust/tracker/tracker.toml ]; then
    driver=${DATABASE_DRIVER:-sqlite3}
    cp /usr/share/torrust/default/config/tracker.container.${driver}.toml /etc/torrust/tracker/tracker.toml
fi
```

## References

- **Tracker Entrypoint**: <https://github.com/torrust/torrust-tracker/blob/develop/share/container/entry_script_sh>
- **Demo Compose**: <https://github.com/torrust/torrust-demo/blob/main/compose.yaml>
- **Configuration Override Pattern**: Uses `TORRUST_TRACKER_CONFIG_OVERRIDE_*` prefix for environment variable overrides
- **Related**: Environment variable naming decision - [`docs/decisions/environment-variable-prefix.md`](../../decisions/environment-variable-prefix.md)

## Action Items

- [ ] Open issue in `torrust/torrust-tracker` repository documenting this requirement
- [ ] Propose entrypoint script improvement to respect mounted configs
- [ ] Investigate which Docker image version we're using and why variable name differs
- [ ] Update tracker documentation to clarify environment variable requirements
- [ ] Consider adding health check that detects driver/path mismatches

## Related Documentation

- **Our Environment Variable Injection**: [`docs/decisions/environment-variable-injection-in-docker-compose.md`](../../decisions/environment-variable-injection-in-docker-compose.md)
- **Template System**: [`docs/technical/template-system-architecture.md`](../../technical/template-system-architecture.md)
- **Output Handling**: [`docs/contributing/output-handling.md`](../../contributing/output-handling.md)
