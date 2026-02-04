# Dockerfile for Provisioned Instance
# Ubuntu 24.04 LTS representing the state after VM provisioning (before configuration)
# This container simulates a freshly provisioned VM ready for Ansible configuration
# Based on requirements from docs/research/e2e-docker-config-testing.md

FROM ubuntu:24.04

# Metadata
LABEL description="Ubuntu 24.04 provisioned instance - post-provision, pre-configuration state"
LABEL maintainer="Torrust Development Team" 
LABEL version="1.0.0"
LABEL deployment-phase="provisioned"

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Update package list and install essential packages
RUN apt-get update && apt-get install -y \
    # SSH server for Ansible connectivity
    openssh-server \
    # Sudo for privilege escalation
    sudo \
    # Supervisor for process management
    supervisor \
    # Python APT bindings for Ansible
    python3-apt \
    # Basic utilities
    curl \
    wget \
    ca-certificates \
    gnupg \
    lsb-release \
    # APT transport for HTTPS repositories
    apt-transport-https \
    # iptables for Docker networking
    iptables \
    # Clean up package cache
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Docker using official installation script
# This installs both Docker daemon and Docker Compose plugin
RUN curl -fsSL https://get.docker.com | sh \
    && rm -rf /var/lib/apt/lists/*

# Configure SSH server
RUN mkdir -p /var/run/sshd \
    # Configure SSH for password authentication initially (tests will set up keys later)
    && sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin no/' /etc/ssh/sshd_config \
    && sed -i 's/#PubkeyAuthentication yes/PubkeyAuthentication yes/' /etc/ssh/sshd_config \
    && sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config

# Create torrust user matching LXD VM configuration
RUN useradd -m -s /bin/bash -G sudo,docker torrust \
    # Set a simple password for initial SSH access (tests will set up keys later)
    && echo "torrust:torrust123" | chpasswd \
    # Configure passwordless sudo
    && echo "torrust ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers \
    # Create .ssh directory for authorized keys (to be populated by tests)
    && mkdir -p /home/torrust/.ssh \
    && chmod 700 /home/torrust/.ssh \
    && chown torrust:torrust /home/torrust/.ssh

# SSH public key will be copied by E2E tests during setup
# Create authorized_keys file with proper permissions
RUN touch /home/torrust/.ssh/authorized_keys \
    && chmod 600 /home/torrust/.ssh/authorized_keys \
    && chown torrust:torrust /home/torrust/.ssh/authorized_keys

# Create provision completion marker (matching cloud-init behavior)
RUN mkdir -p /tmp \
    && echo "Container provisioned successfully" > /tmp/provision_complete \
    && chown torrust:torrust /tmp/provision_complete

# Expose SSH port
EXPOSE 22

# Set working directory
WORKDIR /home/torrust

# Create supervisor configuration
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Create entrypoint script for initialization
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# Default command - run supervisor
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]