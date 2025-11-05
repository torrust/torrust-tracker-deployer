#!/bin/bash
# Install Ansible automation platform
#
# Pre-condition: apt package lists should be up-to-date
# (run 'sudo apt-get update' before using this installer)

set -euo pipefail

# Check if Ansible is already installed
if command -v ansible &> /dev/null; then
    echo "Ansible is already installed, skipping installation"
    exit 0
fi

sudo apt-get install -y ansible
