# Debugging Guide

This document outlines basic debugging techniques for the Torrust Tracker Deploy project.

## 🔍 E2E Test Debugging

### Running E2E Tests with Logging

The project uses tracing for logging. You can control the logging level using the `RUST_LOG` environment variable:

```bash
# Run with debug level logging (recommended for debugging)
RUST_LOG=debug cargo run --bin e2e-tests-full
```

## 📄 Capturing Output for Analysis

Since e2e test output can be very long, use `tee` to see output in real-time while saving it to a file:

```bash
# See output and save to file simultaneously
RUST_LOG=debug cargo run --bin e2e-tests-full 2>&1 | tee debug-output.log
```

You can then search the captured output for specific patterns:

```bash
# Search for errors in the captured output
grep -i "error" debug-output.log
```
