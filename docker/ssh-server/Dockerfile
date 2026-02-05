# Dockerfile for SSH Server Testing
# Minimal Alpine container with SSH server for testing SSH client functionality
# This container is used in integration tests to verify SSH connectivity and command execution

FROM alpine:3.23.3

# Metadata
LABEL description="Minimal SSH server for integration testing"
LABEL maintainer="Torrust Development Team" 
LABEL version="1.0.0"
LABEL purpose="ssh-integration-testing"

# Install essential packages
RUN apk add --no-cache \
    # SSH server for connectivity testing
    openssh-server \
    # Sudo for testing command execution
    sudo \
    # Basic utilities for command testing
    curl \
    wget \
    # Network tools for testing
    net-tools

# Generate SSH host keys
RUN ssh-keygen -A

# Create test user for SSH authentication
RUN adduser -D -s /bin/sh testuser && \
    echo 'testuser:testpass' | chpasswd && \
    addgroup testuser wheel && \
    # Enable passwordless sudo for wheel group (required for deployment testing)
    echo '%wheel ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

# Configure SSH daemon
RUN mkdir -p /run/sshd && \
    # Enable password authentication for testing
    sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config && \
    # Enable public key authentication
    sed -i 's/#PubkeyAuthentication yes/PubkeyAuthentication yes/' /etc/ssh/sshd_config && \
    # Allow root login for testing (if needed)
    sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config && \
    # Set root password for testing
    echo 'root:rootpass' | chpasswd

# Create SSH directory for testuser and set up key authentication
RUN mkdir -p /home/testuser/.ssh && \
    chmod 700 /home/testuser/.ssh && \
    chown testuser:testuser /home/testuser/.ssh

# IMPORTANT: Hardcoded test SSH public key for integration testing
# This key corresponds to the private key in fixtures/testing_rsa
# This setup allows SSH key authentication for automated testing
# DO NOT use this key in production - it's for testing only!
RUN echo 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQCw16sai+XVnawp/P/Q23kcXKekygZ6ALmQAyslREo6kbG8s5RScsmbQqOQEcIwnV2Vo88eeWVzX0N0H1dIczRa/ezijBEsGefthzmz9Ix/vM4lodzTPQFtW8c2eYw7ESy12/2x5//UQQ3mxawEWsz5Ri8XuyBEy/Xh7xH/KpoektaocIOt2/WdCe8CvZdMLd7AviGcTdHFWRiOVrmHM1Pd8znqeA3/1KQP/M4Ae5q21oPjchGjVfPkGh/e62Wt+Wo/2lT30AyMO7JHA1tB1W4xANRQkOd1Kb/TrDLXfg0PaHQ+Irmycjp/H4KkcdB06nzYawXMN5csd/5TWKwkb9/vofp6GQNP731U8+JR4cxRfD107KoHroDSJpG2Fanb2PVBkSXAiJl29YrtoP9vUtSIemQCD/aXFtTcpSv7Y16bdp7v+0adCEHwBmodm9GzLL808FpI2ZCzCi+Ae98P3z+yPCxbrnVAahU8AM2NSbrfyH1w2eb4hJ22oPjdd//tBYtkE1TZBw+i3n0vRn04s5BfPRwwj5GISxacTOZm/YWvoE4UU9axtFXOtMUniVKL3ycA+LEfK7C4velOKbluyL8fYYu4pUxHnYOOkYYeRoi2jf3oagbABOpznloPd93wYP3NoUpIdtMZW+iCF0NnZkVLC9lm1FbTcnmrfNzFtGVKCQ== testing@torrust-testing-infra' > /home/testuser/.ssh/authorized_keys && \
    chmod 600 /home/testuser/.ssh/authorized_keys && \
    chown testuser:testuser /home/testuser/.ssh/authorized_keys

# Generate SSH host keys
RUN ssh-keygen -A

# Create a simple entrypoint script
RUN echo '#!/bin/sh\n\
# Start SSH daemon in foreground\n\
exec /usr/sbin/sshd -D\n\
' > /entrypoint.sh && chmod +x /entrypoint.sh

# Expose SSH port
EXPOSE 22

# Set entrypoint
ENTRYPOINT ["/entrypoint.sh"]