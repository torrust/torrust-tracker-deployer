# Dockerfile for Testing the Dependency Installer CLI
# 
# This Ubuntu 24.04 image is used for integration testing of the
# dependency-installer CLI binary. It includes sudo, curl, build tools,
# and Rust nightly to support testing cargo-machete installation.
# It intentionally does NOT include the tools we're testing for
# (cargo-machete, OpenTofu, Ansible, LXD) so we can verify that the CLI
# correctly identifies missing dependencies and can install them.

FROM ubuntu:24.04

# Metadata
LABEL description="Ubuntu 24.04 testing environment for dependency-installer CLI"
LABEL maintainer="Torrust Development Team"
LABEL version="2.0.0"
LABEL purpose="dependency-installer-integration-testing"

# Install system dependencies needed for testing
# - ca-certificates: Required for HTTPS connections
# - sudo: Required by some installers
# - curl: Required by rustup installer
# - build-essential: Required for compiling Rust projects (cargo-machete)
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    sudo \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly via rustup
# Using nightly-2025-10-15 to match local development environment
# This is required because cargo-machete v0.9.1 needs Rust edition2024
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain nightly-2025-10-15

# Add ~/.cargo/bin to PATH globally so installed binaries are found
# This allows `which cargo-machete` to work after installation
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# The binary will be copied by testcontainers at runtime
# No need to copy it here - testcontainers handles that dynamically

# Default command - keeps container running for test execution
CMD ["/bin/bash", "-c", "while true; do sleep 1000; done"]
