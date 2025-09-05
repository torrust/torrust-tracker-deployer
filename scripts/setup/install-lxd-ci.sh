#!/bin/bash
# Install and configure LXD for CI environments
# 
# IMPORTANT: This script uses CI-specific approaches like 'sudo chmod 666' on the LXD socket
# and 'sudo' with LXD commands. These approaches are NOT recommended for local development.
# For local use, follow the proper group membership approach documented in templates/tofu/lxd/README.md

set -euo pipefail

# Check if LXD is already installed
if command -v lxd &> /dev/null; then
    echo "LXD is already installed, proceeding with configuration..."
else
    sudo snap install lxd
fi

# Wait for LXD to fully initialize
echo "Waiting for LXD daemon to start..."
sleep 15

# Initialize LXD with default settings
sudo lxd init --auto

# Add runner to lxd group
sudo usermod -a -G lxd runner

# IMPORTANT: This approach is ONLY for CI environments
# For local development, use proper group membership instead
# Fix socket permissions for CI environment (NOT recommended for local use)
sudo chmod 666 /var/snap/lxd/common/lxd/unix.socket

# Test basic LXD functionality
sudo lxc list
