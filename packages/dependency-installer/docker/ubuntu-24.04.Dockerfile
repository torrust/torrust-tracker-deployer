# Dockerfile for Testing the Dependency Installer CLI
# 
# This minimal Ubuntu 24.04 image is used for integration testing of the
# dependency-installer CLI binary. It intentionally does NOT include the
# tools we need to detect (cargo-machete, OpenTofu, Ansible, LXD) so we
# can verify that the CLI correctly identifies missing dependencies.

FROM ubuntu:24.04

# Metadata
LABEL description="Ubuntu 24.04 testing environment for dependency-installer CLI"
LABEL maintainer="Torrust Development Team"
LABEL version="1.0.0"
LABEL purpose="dependency-installer-integration-testing"

# Install minimal dependencies needed for running the binary
# Note: We intentionally do NOT install the tools we're testing for
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# The binary will be copied by testcontainers at runtime
# No need to copy it here - testcontainers handles that dynamically

# Default command - keeps container running for test execution
CMD ["/bin/bash", "-c", "while true; do sleep 1000; done"]
