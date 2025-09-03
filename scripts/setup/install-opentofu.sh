#!/bin/bash
# Install OpenTofu (Terraform alternative)

set -euo pipefail

# Check if tofu is already installed
if command -v tofu &> /dev/null; then
    echo "OpenTofu is already installed, skipping installation"
    exit 0
fi

# Download and install OpenTofu
curl --proto '=https' --tlsv1.2 -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
chmod +x install-opentofu.sh
./install-opentofu.sh --install-method deb
rm install-opentofu.sh
