# syntax=docker/dockerfile:latest

# Torrust Tracker Deployer
#
# Multi-stage Dockerfile for building a containerized version of the
# Torrust Tracker Deployer with all required dependencies (OpenTofu, Ansible, SSH).
#
# IMPORTANT LIMITATION:
# This container only supports CLOUD PROVIDERS (Hetzner).
# LXD provider is NOT supported because LXD requires system-level access
# to local virtualization that cannot be provided inside a container.
#
# Build:
#   docker build --target release --tag torrust/tracker-deployer:release --file docker/deployer/Dockerfile .
#
# Run:
#   docker run --rm \
#     -v $(pwd)/data:/var/lib/torrust/deployer/data \
#     -v $(pwd)/build:/var/lib/torrust/deployer/build \
#     -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
#     -v ~/.ssh:/home/deployer/.ssh:ro \
#     torrust/tracker-deployer:release \
#     create environment --env-file /var/lib/torrust/deployer/envs/my-env.json

## =============================================================================
## Builder Image - Install cargo-chef for dependency caching
## =============================================================================
FROM docker.io/library/rust:trixie AS chef
WORKDIR /tmp
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm cargo-chef


## =============================================================================
## Chef Prepare - Analyze project dependencies
## =============================================================================
FROM chef AS recipe
WORKDIR /build/src
COPY . /build/src
RUN cargo chef prepare --recipe-path /build/recipe.json


## =============================================================================
## Cook - Build dependencies (cached layer)
## =============================================================================
FROM chef AS dependencies
WORKDIR /build/src
COPY --from=recipe /build/recipe.json /build/recipe.json
RUN cargo chef cook --release --recipe-path /build/recipe.json


## =============================================================================
## Build Binary
## =============================================================================
FROM dependencies AS build
WORKDIR /build/src
COPY . /build/src
RUN cargo build --release --bin torrust-tracker-deployer
RUN mkdir -p /app/bin/ && cp /build/src/target/release/torrust-tracker-deployer /app/bin/


## =============================================================================
## Runtime Image
## =============================================================================
FROM debian:trixie-slim AS runtime

# Metadata
LABEL org.opencontainers.image.title="Torrust Tracker Deployer"
LABEL org.opencontainers.image.description="Deployment tool for Torrust Tracker with OpenTofu and Ansible"
LABEL org.opencontainers.image.vendor="Torrust"
LABEL org.opencontainers.image.source="https://github.com/torrust/torrust-tracker-deployer"
LABEL org.opencontainers.image.licenses="MIT"

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    # SSH client for remote connections
    openssh-client \
    # Required for downloading tools
    curl \
    gnupg \
    # Python for Ansible
    python3 \
    python3-pip \
    pipx \
    # Git for version control operations
    git \
    # SSL certificates
    ca-certificates \
    # Additional utilities
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Install Ansible via pipx (isolated environment)
ENV PIPX_HOME=/opt/pipx
ENV PIPX_BIN_DIR=/usr/local/bin
RUN pipx install ansible-core

# Install required Ansible collections
# ansible.posix: Required for the 'debug' callback plugin used in ansible.cfg
# community.docker: Required for Docker management tasks
# community.general: Required for common utility modules
RUN pipx runpip ansible-core install ansible \
    && ansible-galaxy collection install ansible.posix community.docker community.general

# Install OpenTofu
# Using the official installation script with deb method for Debian
RUN curl -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh \
    && chmod +x install-opentofu.sh \
    && ./install-opentofu.sh --install-method deb \
    && rm install-opentofu.sh

# Build arguments for customization
ARG USER_ID=1000
ARG DEPLOYER_USER=deployer

# Environment variables
ENV USER_ID=${USER_ID}
ENV DEPLOYER_USER=${DEPLOYER_USER}
ENV TZ=Etc/UTC

# Data directories inside container
ENV TORRUST_TD_DATA_DIR=/var/lib/torrust/deployer/data
ENV TORRUST_TD_BUILD_DIR=/var/lib/torrust/deployer/build
ENV TORRUST_TD_ENVS_DIR=/var/lib/torrust/deployer/envs
ENV TORRUST_TD_LOG_DIR=/var/log/torrust/deployer

# Create deployer user (non-root for security)
RUN useradd -m -s /bin/bash -u ${USER_ID} ${DEPLOYER_USER} \
    && echo "${DEPLOYER_USER} ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Create application directories
RUN mkdir -p ${TORRUST_TD_DATA_DIR} \
             ${TORRUST_TD_BUILD_DIR} \
             ${TORRUST_TD_ENVS_DIR} \
             ${TORRUST_TD_LOG_DIR} \
    && chown -R ${DEPLOYER_USER}:${DEPLOYER_USER} /var/lib/torrust /var/log/torrust

# Copy binary from build stage
COPY --from=build /app/bin/torrust-tracker-deployer /usr/bin/torrust-tracker-deployer

# Copy entrypoint script
COPY --chmod=0555 ./docker/deployer/entry_script_sh /usr/local/bin/entry.sh

# Define volumes for persistent data
VOLUME ["${TORRUST_TD_DATA_DIR}", "${TORRUST_TD_BUILD_DIR}", "${TORRUST_TD_ENVS_DIR}"]

# Set working directory to deployer root (so ./data and ./build resolve correctly)
WORKDIR /var/lib/torrust/deployer

# Use non-root user
USER ${DEPLOYER_USER}

# Entrypoint and default command
ENTRYPOINT ["/usr/local/bin/entry.sh"]
CMD ["--help"]


## =============================================================================
## Release target (default)
## =============================================================================
FROM runtime AS release
# This is the default production image
# Inherits everything from runtime


## =============================================================================
## Debug target (for development/troubleshooting)
## =============================================================================
FROM runtime AS debug

USER root

# Install additional debug tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    vim \
    less \
    procps \
    net-tools \
    && rm -rf /var/lib/apt/lists/*

USER ${DEPLOYER_USER}

# Override entrypoint for debugging
CMD ["/bin/bash"]
