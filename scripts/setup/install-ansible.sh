#!/bin/bash
# Install Ansible automation platform

set -euo pipefail

# Check if Ansible is already installed
if command -v ansible &> /dev/null; then
    echo "Ansible is already installed, skipping installation"
    exit 0
fi

sudo apt-get update
sudo apt-get install -y ansible
