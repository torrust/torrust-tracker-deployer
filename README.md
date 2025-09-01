# Torrust Testing Infrastructure PoC

This repository contains configurations for testing VM provisioning and cloud-init execution using different virtualization approaches. The goal is to find the best solution for creating VMs that support cloud-init both locally (development) and in CI environments (GitHub Actions).

## 🎯 Project Goals

- ✅ **Create VMs supporting cloud-init** locally and in GitHub runners
- ✅ **Test cloud-init execution and verification**
- ✅ **Support Docker Compose** inside VMs (planned)
- ✅ **Fast, easy to install and use** solutions
- ❌ **No nested virtualization dependency** (CI compatibility)

## 🔧 Available Approaches

This repository tests two different virtualization technologies:

### 🖥️ **Multipass (`config/tofu/multipass/`)**

- **Technology**: Full VMs with nested virtualization
- **Status**: ⚠️ Works in GitHub Actions but undocumented
- **Best for**: Local development, full VM isolation
- **Requirements**: Nested virtualization support

**[📖 See detailed documentation →](config/tofu/multipass/README.md)**

### ☁️ **LXD Containers (`config/tofu/lxd/`)**

- **Technology**: System containers with cloud-init support
- **Status**: ✅ Guaranteed GitHub Actions compatibility
- **Best for**: CI/CD environments, fast provisioning
- **Requirements**: No special virtualization needed

**[📖 See detailed documentation →](config/tofu/lxd/README.md)**

## 🔄 **Quick Comparison**

| Feature                    | Multipass                      | LXD Containers              |
| -------------------------- | ------------------------------ | --------------------------- |
| **GitHub Actions Support** | 🔶 Discovered but undocumented | ✅ Guaranteed               |
| **Nested Virtualization**  | ✅ Required                    | ❌ Not needed               |
| **Cloud-init Support**     | ✅ Full VM boot                | ✅ Container boot           |
| **Resource Usage**         | ❌ Higher (full VMs)           | ✅ Lower (containers)       |
| **Isolation Level**        | ✅ Complete (separate kernel)  | 🔶 Process-level            |
| **Boot Time**              | ❌ Slower (full boot)          | ✅ Faster (container start) |
| **Docker Support**         | ✅ Full support                | ✅ Full support             |
| **Setup Complexity**       | ✅ Simple (snap install)       | 🔶 Requires LXD setup       |

## 🚀 **Getting Started**

Choose your preferred approach:

1. **For local development**: Start with [Multipass configuration](config/tofu/multipass/README.md)
2. **For CI/CD reliability**: Use [LXD configuration](config/tofu/lxd/README.md)
3. **For testing both**: Try both approaches to compare

## 🧪 **Testing in GitHub Actions**

Both configurations include GitHub Actions workflows for CI testing:

- **`.github/workflows/test-multipass-provision.yml`** - Tests Multipass VMs
- **`.github/workflows/test-lxd-provision.yml`** - Tests LXD containers

## 📊 **Current Status**

### ✅ **Completed**

- [x] Multipass VM provisioning (local + GitHub Actions)
- [x] LXD container provisioning (local + GitHub Actions)
- [x] Cloud-init support in both approaches
- [x] OpenTofu infrastructure as code
- [x] Automated testing workflows

### 🔄 **In Progress**

- [ ] Docker Compose integration testing
- [ ] Performance benchmarking
- [ ] Official GitHub Actions nested virtualization clarification

### 📋 **Planned**

- [ ] Additional VM providers evaluation
- [ ] Integration with Torrust application testing
- [ ] Multi-architecture support (ARM64)

## 📁 **Repository Structure**

```text
├── config/
│   └── tofu/
│       ├── multipass/
│       │   ├── main.tf           # OpenTofu configuration for Multipass VMs
│       │   ├── cloud-init.yml    # Cloud-init configuration
│       │   └── README.md         # Multipass-specific documentation
│       └── lxd/
│           ├── main.tf           # OpenTofu configuration for LXD containers
│           ├── cloud-init.yml    # Cloud-init configuration (same as multipass)
│           └── README.md         # LXD-specific documentation
├── .github/
│   └── workflows/
│       ├── test-multipass-provision.yml  # Tests Multipass VMs
│       └── test-lxd-provision.yml        # Tests LXD containers
├── README.md                 # This file - project overview
└── .gitignore                # Git ignore rules
```

The repository now properly documents this significant discovery and provides a clear path for others to follow the official GitHub Actions team response. The commit message follows conventional commit standards and clearly describes the documentation improvements.

## Next Steps

This is a basic setup. Future enhancements could include:

- Multiple VMs for different testing scenarios
- Custom images with pre-installed Torrust components
- Network configuration for multi-VM setups
- Enhanced CI/CD integration with nested virtualization
- Automated testing scripts
