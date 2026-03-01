# `exists` - Check Environment Existence

Check whether a named deployment environment exists in the workspace.

## Purpose

Provides a scripting-friendly way to test whether an environment has been created. Unlike `show`, this command:

- **Always exits 0** on success, regardless of whether the environment exists
- **Outputs bare `true` or `false`** — valid for both human reading and shell scripting
- **Never produces verbose output** — designed for use in conditionals and pipelines
- **Never loads the environment** — pure file-existence check, sub-millisecond

## Command Syntax

```bash
torrust-tracker-deployer exists <ENVIRONMENT> [OPTIONS]
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to check

## Options

- `-o, --output-format <FORMAT>` (optional) - Output format: `text` (default) or `json`

## Exit Code Contract

| Scenario                   | Exit Code | Stdout  | Stderr          |
| -------------------------- | --------- | ------- | --------------- |
| Environment exists         | **0**     | `true`  | —               |
| Environment does not exist | **0**     | `false` | —               |
| Invalid environment name   | **1**     | —       | Error with help |
| Repository/IO error        | **1**     | —       | Error with help |

## Basic Usage

Check if an environment exists:

```bash
torrust-tracker-deployer exists my-environment
# stdout: true  (if it exists)
# stdout: false (if it does not)
```

## Output Formats

Both output formats produce bare `true` or `false` — valid JSON boolean values.

### Text Output (default)

```bash
torrust-tracker-deployer exists my-environment
```

```text
true
```

### JSON Output

```bash
torrust-tracker-deployer exists my-environment --output-format json
```

```text
true
```

> **Note**: Both formats output the same bare value (`true` or `false`), which happens to be valid JSON. There is no JSON object wrapper — the entire stdout is a JSON boolean.

## Shell Scripting Examples

### Conditional creation

```bash
if [ "$(torrust-tracker-deployer exists my-env)" = "true" ]; then
    echo "Environment already exists, skipping creation"
else
    torrust-tracker-deployer create environment -f config.json
fi
```

### Guard pattern (exit early if missing)

```bash
ENV_EXISTS=$(torrust-tracker-deployer exists my-env)
if [ "$ENV_EXISTS" = "false" ]; then
    echo "Error: environment 'my-env' does not exist. Run create first." >&2
    exit 1
fi
```

### CI/CD pipeline example

```bash
# Check before potentially destructive operation
if [ "$(torrust-tracker-deployer exists staging)" = "true" ]; then
    torrust-tracker-deployer destroy staging
fi
```

## Differences from `show`

| Aspect                 | `exists`                  | `show`                                     |
| ---------------------- | ------------------------- | ------------------------------------------ |
| Exit code when missing | **0** (`false` on stdout) | **1** (error)                              |
| Output when found      | `true`                    | Full environment details                   |
| Output format          | Bare boolean              | Structured text or JSON                    |
| Use case               | Scripts, conditionals     | Human inspection                           |
| Performance            | Sub-millisecond           | Slightly slower (deserializes environment) |

## Related Commands

- [`create`](create.md) — Create a new environment
- [`show`](show.md) — Display detailed environment information
- [`list`](README.md) — List all environments
- [`destroy`](destroy.md) — Remove a deployed environment
