# Logging Guide

This guide explains how to configure and use logging in the Torrust Tracker Deployer.

## Overview

The application provides comprehensive structured logging for observability and troubleshooting. All operations are logged to persistent log files, allowing you to review what happened even after the application has finished running.

### Key Principles

- **Always persistent**: Logs are always written to files for post-mortem analysis
- **Configurable output**: Choose between file-only (production) or file+stderr (development)
- **Multiple formats**: Pretty, JSON, or compact formatting to suit your needs
- **Environment-based filtering**: Use `RUST_LOG` to control log verbosity

## Quick Start

### Default Behavior (Production)

By default, the application uses production-safe settings:

```bash
torrust-tracker-deployer
```

This configuration:

- Writes logs to `./data/logs/log.txt`
- Uses **compact** format (space-efficient, readable)
- **File-only** output (no stderr pollution)
- **Info** level logging (controlled by `RUST_LOG`)

### Development Mode

For development and troubleshooting, enable stderr output:

```bash
torrust-tracker-deployer --log-output file-and-stderr
```

This shows log events in real-time on your terminal while still writing to the log file.

### Pretty Format for Debugging

For maximum readability during development:

```bash
torrust-tracker-deployer --log-format pretty --log-output file-and-stderr
```

## Configuration Options

### Log Format (`--log-format`)

Controls how log entries are formatted:

#### Compact (Default)

Space-efficient single-line format, ideal for production:

```bash
torrust-tracker-deployer --log-format compact
```

Example output:

```text
2025-10-15T12:39:22.793955Z  INFO torrust_tracker_deployer::app: Application started app="torrust-tracker-deployer" version="0.1.0"
```

#### Pretty

Multi-line format with visual structure, ideal for development:

```bash
torrust-tracker-deployer --log-format pretty --log-output file-and-stderr
```

Example output:

```text
  2025-10-15T12:40:36.097921Z  INFO torrust_tracker_deployer::app: Application started, app: "torrust-tracker-deployer", version: "0.1.0"
    at src/app.rs:69
```

#### JSON

Machine-readable format for log aggregation systems:

```bash
torrust-tracker-deployer --log-format json
```

Example output:

```json
{"timestamp":"2025-10-15T12:42:34.178335Z","level":"INFO","fields":{"message":"Application started","app":"torrust-tracker-deployer","version":"0.1.0"},"target":"torrust_tracker_deployer::app"}
```

### Log Output (`--log-output`)

Controls where logs are written:

#### File Only (Default)

Production mode - logs written only to file:

```bash
torrust-tracker-deployer --log-output file-only
```

- ✅ Clean terminal output (no log noise)
- ✅ All logs captured in persistent file
- ✅ Suitable for production deployments

#### File and Stderr

Development mode - logs written to both file and stderr:

```bash
torrust-tracker-deployer --log-output file-and-stderr
```

- ✅ Real-time log visibility on terminal
- ✅ All logs still captured in persistent file
- ✅ Suitable for development and troubleshooting

### Log Directory (`--log-dir`)

Specifies where log files should be written:

```bash
# Use custom directory
torrust-tracker-deployer --log-dir /var/log/deployer

# Use relative path
torrust-tracker-deployer --log-dir ./custom-logs

# Use deeply nested directory (created automatically)
torrust-tracker-deployer --log-dir /tmp/app/logs/production
```

The log file is always named `log.txt` inside the specified directory. Parent directories are created automatically if they don't exist.

## Log Levels

Control log verbosity using the `RUST_LOG` environment variable:

### Info Level (Default)

Standard operational logging:

```bash
torrust-tracker-deployer
# or explicitly
RUST_LOG=info torrust-tracker-deployer
```

Shows:

- Application startup/shutdown
- Major operations and milestones
- Errors and warnings

### Debug Level

Detailed diagnostic information:

```bash
RUST_LOG=debug torrust-tracker-deployer --log-output file-and-stderr
```

Shows everything from Info level, plus:

- Internal operation details
- State transitions
- Configuration values

### Trace Level

Maximum verbosity for deep debugging:

```bash
RUST_LOG=trace torrust-tracker-deployer --log-output file-and-stderr
```

Shows everything from Debug level, plus:

- Function entry/exit
- Low-level details
- All internal operations

⚠️ **Warning**: Trace level generates significant log volume. Use only for specific debugging scenarios.

### Module-Specific Filtering

Filter logs by module or crate:

```bash
# Only logs from torrust_tracker_deployer
RUST_LOG=torrust_tracker_deployer=debug torrust-tracker-deployer

# Multiple modules with different levels
RUST_LOG=torrust_tracker_deployer=debug,ansible=trace torrust-tracker-deployer

# Exclude specific modules
RUST_LOG=debug,tokio=warn torrust-tracker-deployer
```

## Common Scenarios

### Scenario 1: Production Deployment

Production-safe defaults with minimal configuration:

```bash
torrust-tracker-deployer
```

- Logs to `./data/logs/log.txt`
- Compact format
- File-only output
- Info level

### Scenario 2: Development Work

Real-time log visibility with readable format:

```bash
torrust-tracker-deployer --log-format pretty --log-output file-and-stderr
```

- Logs to `./data/logs/log.txt` and stderr
- Pretty format
- Info level (increase with RUST_LOG if needed)

### Scenario 3: Troubleshooting Issues

Maximum verbosity for debugging:

```bash
RUST_LOG=debug torrust-tracker-deployer --log-format pretty --log-output file-and-stderr
```

- Debug level logging
- Pretty format for readability
- Real-time visibility on terminal
- Persistent file for later analysis

### Scenario 4: Log Aggregation

JSON format for external monitoring systems:

```bash
torrust-tracker-deployer --log-format json --log-dir /var/log/deployer
```

- JSON format for machine parsing
- Custom log directory
- File-only output (monitor the file externally)

### Scenario 5: CI/CD Pipeline

Visible logs for automated testing:

```bash
torrust-tracker-deployer --log-output file-and-stderr
```

- Compact format (space-efficient)
- Stderr output (captured by CI system)
- Persistent file (artifact for later review)

## Log File Management

### Log File Location

The log file is created at:

```text
<log-dir>/log.txt
```

Default: `./data/logs/log.txt` (relative to working directory)

### Append Mode

Logs are **appended** to existing log files, not overwritten:

```bash
# First run
torrust-tracker-deployer
# Creates ./data/logs/log.txt with entries

# Second run
torrust-tracker-deployer
# Appends new entries to ./data/logs/log.txt
```

This allows you to:

- ✅ Track multiple runs in a single file
- ✅ Preserve historical logs
- ✅ Analyze trends over time

### Log Rotation

⚠️ **Note**: Automatic log rotation is not currently implemented.

For production use, consider:

- External log rotation tools (logrotate)
- Regular manual cleanup
- Monitoring log file size

## Error Handling

### Log Directory Creation

The application automatically creates the log directory if it doesn't exist:

```bash
# Non-existent directory is created automatically
torrust-tracker-deployer --log-dir /tmp/new/nested/logs
```

### Permission Issues

If the log directory cannot be created due to permission issues, the application will exit with an error:

```bash
torrust-tracker-deployer --log-dir /root/logs

# Output:
# thread 'main' panicked at src/logging.rs:260:9:
# Failed to create log directory: /root/logs - check filesystem permissions
```

**This behavior is intentional** - logging is critical for observability, and the application cannot function properly without it.

**Solutions:**

- Use a writable directory
- Adjust filesystem permissions
- Run with appropriate user privileges

## Best Practices

### Development

1. **Use stderr output** for real-time visibility:

   ```bash
   torrust-tracker-deployer --log-output file-and-stderr
   ```

2. **Use pretty format** for readability:

   ```bash
   torrust-tracker-deployer --log-format pretty --log-output file-and-stderr
   ```

3. **Increase verbosity** when debugging:

   ```bash
   RUST_LOG=debug torrust-tracker-deployer --log-output file-and-stderr
   ```

### Production

1. **Use default settings** for production:

   ```bash
   torrust-tracker-deployer
   ```

2. **Consider JSON format** for log aggregation:

   ```bash
   torrust-tracker-deployer --log-format json
   ```

3. **Use absolute paths** for log directories:

   ```bash
   torrust-tracker-deployer --log-dir /var/log/torrust-tracker-deployer
   ```

4. **Monitor log file size** and implement rotation

### CI/CD

1. **Enable stderr output** for CI system capture:

   ```bash
   torrust-tracker-deployer --log-output file-and-stderr
   ```

2. **Use compact format** for space efficiency:

   ```bash
   torrust-tracker-deployer --log-format compact --log-output file-and-stderr
   ```

3. **Archive log files** as build artifacts

## Additional Resources

- [Contributing: Logging Guide](../contributing/logging-guide.md) - How to add logging to code
- [Development Principles](../development-principles.md) - Observability principles
- [User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md) - Design rationale
